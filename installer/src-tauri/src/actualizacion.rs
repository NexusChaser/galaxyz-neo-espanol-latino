//! Actualización automática del parche y del propio instalador (§6.6 del plan).

use crate::candado::Candado;
use crate::deteccion;
use crate::error::{io, Error, Resultado};
use crate::estado::{self, Estado};
use crate::hash::sha256_archivo;
use crate::instalacion::{self, Entorno, Progreso, ResultadoInstalacion};
use crate::paquete::{Manifest, PaqueteActual, PaqueteDescargado};
use crate::version::es_mayor;
use crate::{notificacion, remoto};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

pub const VERSION_INSTALADOR: &str = env!("CARGO_PKG_VERSION");
/// Margen para que una tarea que se dispara justo a la hora no se salte la comprobación.
const TOLERANCIA_MINUTOS: i64 = 15;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoActualizacion {
    pub version_instalada: Option<String>,
    pub version_disponible: String,
    pub hay_parche_nuevo: bool,
    pub version_juego: Option<String>,
    /// Si el parche que quedaría instalado soporta la versión del juego (`None`: no se sabe).
    pub compatible_con_juego: Option<bool>,
    pub notas: Option<String>,
    pub tamano_descarga: u64,
    pub version_instalador: String,
    pub instalador_disponible: Option<String>,
    pub hay_instalador_nuevo: bool,
}

/// `versiones_instaladas`: versiones del juego que soporta el parche que ya está puesto.
pub fn comparar(
    remoto: &Manifest,
    estado: Option<&Estado>,
    version_juego: Option<&str>,
    versiones_instaladas: &[String],
) -> InfoActualizacion {
    let instalada = estado.map(|e| e.patch_version.clone());
    let hay_parche_nuevo = instalada.as_deref().is_some_and(|v| es_mayor(&remoto.version, v));
    let versiones_compatibles: Vec<String> = if hay_parche_nuevo || instalada.is_none() {
        remoto.game_versions.clone()
    } else {
        // Sin versión nueva, lo que cuenta es lo que ya está instalado (y lo publicado si es la misma versión).
        let mut v = versiones_instaladas.to_vec();
        if instalada.as_deref() == Some(remoto.version.as_str()) {
            v.extend(remoto.game_versions.iter().cloned());
        }
        v
    };
    let instalador_disponible = remoto.installer.as_ref().map(|i| i.version.clone());
    InfoActualizacion {
        hay_instalador_nuevo: instalador_disponible.as_deref().is_some_and(|v| es_mayor(v, VERSION_INSTALADOR)),
        version_instalada: instalada,
        version_disponible: remoto.version.clone(),
        hay_parche_nuevo,
        compatible_con_juego: version_juego.map(|v| versiones_compatibles.iter().any(|c| c == v)),
        version_juego: version_juego.map(str::to_string),
        notas: remoto.notes.clone(),
        tamano_descarga: remoto.zip.as_ref().map(|z| z.size).unwrap_or(0),
        version_instalador: VERSION_INSTALADOR.to_string(),
        instalador_disponible,
    }
}

/// Lee solo el `manifest.json` de la caché (sin cargar los `.epk`) para saber qué juego soporta lo instalado.
fn versiones_juego_instaladas(carpeta_app: &Path, estado: Option<&Estado>) -> Vec<String> {
    let cache = std::fs::read(crate::paquete::carpeta_cache(carpeta_app).join("manifest.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Manifest>(&b).ok())
        .filter(|m| estado.is_some_and(|e| e.patch_version == m.version));
    cache
        .map(|m| m.game_versions)
        .unwrap_or_else(|| crate::paquete::manifest_incluido().game_versions.clone())
}

pub fn version_juego(estado: Option<&Estado>) -> Option<String> {
    estado
        .and_then(|e| e.game_path.as_deref())
        .filter(|r| deteccion::es_carpeta_juego(r))
        .map(Path::to_path_buf)
        .or_else(|| deteccion::detectar_juego(None).map(|j| j.ruta))
        .and_then(|r| deteccion::leer_version(&r))
}

/// Descarga el manifiesto y guarda la hora y el resultado de la comprobación.
pub fn buscar(entorno: &dyn Entorno) -> Resultado<(InfoActualizacion, Manifest)> {
    let carpeta = entorno.carpeta_app();
    let mut estado = estado::leer(&carpeta);
    let resultado = remoto::obtener_manifest();
    let ahora = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
    let resultado = resultado.map(|m| {
        let juego = version_juego(estado.as_ref());
        let instaladas = versiones_juego_instaladas(&carpeta, estado.as_ref());
        (comparar(&m, estado.as_ref(), juego.as_deref(), &instaladas), m)
    });
    if let Some(e) = estado.as_mut() {
        e.last_check = Some(ahora);
        e.last_check_result = Some(match &resultado {
            Ok((i, _)) if i.hay_parche_nuevo => format!("Hay una versión nueva: {}", i.version_disponible),
            Ok((i, _)) if i.compatible_con_juego == Some(false) => {
                format!("El juego ({}) no está soportado todavía", i.version_juego.clone().unwrap_or_default())
            }
            Ok(_) => "El parche está al día".into(),
            Err(err) => format!("No se pudo comprobar: {err}"),
        });
        e.unsupported_game_version = match &resultado {
            Ok((i, _)) if i.compatible_con_juego == Some(false) => i.version_juego.clone(),
            Ok(_) => None,
            Err(_) => e.unsupported_game_version.clone(),
        };
        let _ = estado::guardar(&carpeta, e);
    }
    resultado
}

/// Extrae los `.epk` de un ZIP del parche y los comprueba contra el manifiesto.
pub fn paquete_desde_zip(manifest: Manifest, zip: &[u8]) -> Resultado<PaqueteDescargado> {
    let mut archivo = zip::ZipArchive::new(std::io::Cursor::new(zip))
        .map_err(|e| Error::PaqueteCorrupto(format!("ZIP del parche ({e})")))?;
    let mut bytes = BTreeMap::new();
    for i in 0..archivo.len() {
        let mut entrada = archivo
            .by_index(i)
            .map_err(|e| Error::PaqueteCorrupto(format!("ZIP del parche ({e})")))?;
        if !entrada.is_file() {
            continue;
        }
        let nombre = entrada.name().replace('\\', "/");
        let minusculas = nombre.to_lowercase();
        // Como en el ZIP publicado: `…/epk/<x>.epk` (id = nombre) y `…/extra/<ruta>` (id = ruta).
        let id = if let Some(pos) = minusculas.find("/extra/").map(|p| p + 7).or(minusculas.starts_with("extra/").then_some(6)) {
            crate::paquete::normalizar_ruta(&nombre[pos..]).to_lowercase()
        } else if minusculas.ends_with(".epk") && (minusculas.contains("/epk/") || minusculas.starts_with("epk/")) {
            minusculas.rsplit('/').next().unwrap_or_default().to_string()
        } else {
            continue;
        };
        let mut datos = Vec::with_capacity(entrada.size() as usize);
        entrada
            .read_to_end(&mut datos)
            .map_err(|e| Error::PaqueteCorrupto(format!("{id} ({e})")))?;
        bytes.insert(id, datos);
    }
    PaqueteDescargado::nuevo(manifest, bytes)
}

/// Descarga e instala la versión del manifiesto con las opciones guardadas.
pub fn aplicar(
    entorno: &dyn Entorno,
    manifest: Manifest,
    progreso: &mut dyn FnMut(Progreso),
) -> Resultado<ResultadoInstalacion> {
    let _candado = Candado::tomar(&entorno.carpeta_app())?;
    let zip = manifest
        .zip
        .clone()
        .ok_or_else(|| Error::Red("el manifiesto no indica dónde descargar el parche.".into()))?;
    let version = manifest.version.clone();
    let bytes = remoto::descargar(&zip.url, &zip.sha256, &mut |recibido, total| {
        let total = total.or((zip.size > 0).then_some(zip.size));
        let pct = total.map(|t| 40.0 * recibido as f32 / t.max(1) as f32).unwrap_or(0.0);
        progreso(Progreso {
            paso: instalacion::Paso::Descargando,
            porcentaje: pct,
            mensaje: format!("Descargando la versión {version} ({:.1} MB)…", recibido as f64 / 1_048_576.0),
        });
    })?;
    let descargado = paquete_desde_zip(manifest, &bytes)?;
    descargado.guardar_cache(&entorno.carpeta_app())?;
    let paquete = descargado.paquete();
    let resultado = instalacion::reparar(&paquete, entorno, false, &mut |p| {
        progreso(Progreso { porcentaje: 40.0 + p.porcentaje * 0.6, ..p })
    })?;
    if let Some(mut e) = estado::leer(&entorno.carpeta_app()) {
        e.last_check_result = Some(format!("Actualizado a la versión {version}"));
        let _ = estado::guardar(&entorno.carpeta_app(), &e);
    }
    Ok(resultado)
}

/// Archivos del parche que faltan o cambiaron en la carpeta del juego.
pub fn archivos_danados(estado: &Estado) -> Vec<String> {
    estado
        .files
        .iter()
        .filter(|(relativa, sha)| {
            !crate::paquete::ruta_segura(relativa)
                || sha256_archivo(&estado.data_path.join(relativa)).ok().as_ref() != Some(*sha)
        })
        .map(|(relativa, _)| relativa.clone())
        .collect()
}

/// Activa o desactiva la búsqueda automática y cambia la frecuencia («Cambiar opciones», §5.8).
pub fn cambiar_opciones(entorno: &dyn Entorno, activar: bool, horas: u32) -> Resultado<Estado> {
    let carpeta = entorno.carpeta_app();
    let mut e = estado::leer(&carpeta).ok_or(Error::SinInstalacion)?;
    let horas = estado::intervalo_valido(horas);
    entorno
        .configurar_actualizacion(activar, horas)
        .map_err(Error::Validacion)?;
    e.auto_update = activar;
    e.update_interval_hours = horas;
    estado::guardar(&carpeta, &e)?;
    Ok(e)
}

pub fn toca_comprobar(estado: &Estado, ahora: chrono::DateTime<chrono::Local>) -> bool {
    let Some(ultima) = estado
        .last_check
        .as_deref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
    else {
        return true;
    };
    let siguiente = ultima + chrono::Duration::hours(estado.update_interval_hours as i64)
        - chrono::Duration::minutes(TOLERANCIA_MINUTOS);
    ahora >= siguiente
}

/// Descarga el instalador nuevo y sustituye el de `%LOCALAPPDATA%\Programs\GalaxyzNeoES`.
/// Windows deja renombrar un ejecutable en uso: el actual pasa a `.old` y se borra en el próximo arranque.
pub fn actualizar_instalador(manifest: &Manifest, progreso: &mut dyn FnMut(Progreso)) -> Resultado<String> {
    let inst = manifest
        .installer
        .as_ref()
        .filter(|i| es_mayor(&i.version, VERSION_INSTALADOR))
        .ok_or_else(|| Error::Validacion("No hay una versión nueva del instalador.".into()))?;
    let dir = crate::sistema::EntornoSistema::carpeta_programa();
    let exe = dir.join(format!("{}.exe", crate::sistema::NOMBRE_APP));
    if !exe.is_file() {
        return Err(Error::Validacion(format!(
            "El instalador no está instalado en esta PC. Descarga la versión nueva desde {}",
            remoto::url_releases()
        )));
    }
    let bytes = remoto::descargar(&inst.url, &inst.sha256, &mut |recibido, total| {
        let total = total.unwrap_or(inst.size.max(1));
        progreso(Progreso {
            paso: instalacion::Paso::Descargando,
            porcentaje: 95.0 * recibido as f32 / total as f32,
            mensaje: format!("Descargando el instalador {}…", inst.version),
        });
    })?;
    let nuevo = dir.join(format!("{}.new.exe", crate::sistema::NOMBRE_APP));
    let viejo = dir.join(format!("{}.old.exe", crate::sistema::NOMBRE_APP));
    std::fs::write(&nuevo, &bytes).map_err(io("guardar", &nuevo))?;
    let _ = std::fs::remove_file(&viejo);
    std::fs::rename(&exe, &viejo).map_err(io("reemplazar", &exe))?;
    if let Err(e) = std::fs::rename(&nuevo, &exe) {
        let _ = std::fs::rename(&viejo, &exe);
        return Err(io("reemplazar", &exe)(e));
    }
    crate::log::escribir(&format!("Instalador actualizado a {}", inst.version));
    Ok(inst.version.clone())
}

pub fn limpiar_restos() {
    let dir = crate::sistema::EntornoSistema::carpeta_programa();
    let _ = std::fs::remove_file(dir.join(format!("{}.old.exe", crate::sistema::NOMBRE_APP)));
    let _ = std::fs::remove_file(dir.join(format!("{}.new.exe", crate::sistema::NOMBRE_APP)));
}

/// `--update --silent`: lo que lanzan el inicio de sesión y la tarea programada. Devuelve el código de salida.
pub fn ejecutar_silencioso(entorno: &dyn Entorno, forzar: bool, al_iniciar_sesion: bool) -> i32 {
    let carpeta = entorno.carpeta_app();
    let Some(estado_actual) = estado::leer(&carpeta) else {
        crate::log::escribir("Actualización: el parche no está instalado; nada que hacer.");
        return 0;
    };
    if !estado_actual.auto_update && !forzar {
        crate::log::escribir("Actualización: desactivada por el usuario.");
        return 0;
    }
    if !forzar && !toca_comprobar(&estado_actual, chrono::Local::now()) {
        crate::log::escribir("Actualización: todavía no toca comprobar.");
        return 0;
    }
    if al_iniciar_sesion {
        // No frenar el arranque de Windows.
        std::thread::sleep(Duration::from_secs(60));
    }
    limpiar_restos();

    let (info, manifest) = match buscar(entorno) {
        Ok(r) => r,
        Err(e) => {
            crate::log::escribir(&format!("Actualización: {e}"));
            return 1;
        }
    };
    crate::log::escribir(&format!("Actualización: {info:?}"));

    if info.compatible_con_juego == Some(false) {
        if estado_actual.unsupported_game_version != info.version_juego {
            notificacion::mostrar(
                "GALAXYZ neo se actualizó",
                "El parche en español todavía no soporta esta versión del juego. Abre «Parche GALAXYZ neo» para decidir si lo quitas por ahora.",
            );
        }
        return 0;
    }

    let mut codigo = 0;
    if info.hay_parche_nuevo {
        if !esperar_juego_cerrado(entorno, Duration::from_secs(6 * 3600)) {
            crate::log::escribir("Actualización: el juego siguió abierto 6 horas; se intentará la próxima vez.");
            return 0;
        }
        match aplicar(entorno, manifest.clone(), &mut |_| {}) {
            Ok(_) => notificacion::mostrar(
                "Parche en español actualizado",
                &format!("GALAXYZ neo ya tiene la versión {} de la traducción.", info.version_disponible),
            ),
            Err(e) => {
                crate::log::escribir(&format!("Actualización: falló al aplicar: {e}"));
                codigo = 1;
            }
        }
    } else {
        let danados = archivos_danados(&estado_actual);
        if !danados.is_empty() && esperar_juego_cerrado(entorno, Duration::from_secs(6 * 3600)) {
            crate::log::escribir(&format!("Auto-reparación: {danados:?}"));
            let resultado = Candado::tomar(&carpeta).and_then(|_c| {
                let actual = PaqueteActual::cargar(&carpeta);
                instalacion::reparar(&actual.paquete(), entorno, false, &mut |_| {})
            });
            match resultado {
                Ok(_) => notificacion::mostrar(
                    "Parche en español reparado",
                    &format!("Se volvieron a poner {} archivos de la traducción que faltaban o estaban dañados.", danados.len()),
                ),
                Err(e) => {
                    crate::log::escribir(&format!("Auto-reparación: falló: {e}"));
                    codigo = 1;
                }
            }
        }
    }

    if info.hay_instalador_nuevo {
        if let Err(e) = actualizar_instalador(&manifest, &mut |_| {}) {
            crate::log::escribir(&format!("Actualización del instalador: {e}"));
        }
    }
    codigo
}

fn esperar_juego_cerrado(entorno: &dyn Entorno, maximo: Duration) -> bool {
    let inicio = std::time::Instant::now();
    while entorno.juego_abierto() {
        if inicio.elapsed() >= maximo {
            return false;
        }
        std::thread::sleep(Duration::from_secs(30));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::sha256_bytes;
    use crate::paquete::{ArchivoManifest, Descarga};
    use std::io::Write;

    fn manifest(version: &str, juegos: &[&str], archivos: &[(&str, &[u8])]) -> Manifest {
        Manifest {
            version: version.into(),
            released: None,
            game_versions: juegos.iter().map(|s| s.to_string()).collect(),
            targets: vec!["epk".into()],
            files: archivos
                .iter()
                .map(|(n, d)| ArchivoManifest {
                    name: n.rsplit('\\').next().unwrap().to_string(),
                    path: n.contains('\\').then(|| n.to_string()),
                    size: d.len() as u64,
                    sha256: sha256_bytes(d),
                })
                .collect(),
            known_patch_hashes: Default::default(),
            zip: Some(Descarga { url: "x.zip".into(), sha256: String::new(), size: 0 }),
            installer: None,
            notes: None,
        }
    }

    fn estado_con(version: &str) -> Estado {
        Estado { patch_version: version.into(), update_interval_hours: 24, ..Default::default() }
    }

    #[test]
    fn compara_versiones_y_compatibilidad() {
        let instaladas = vec!["A".to_string()];
        let m = manifest("1.1.0", &["B"], &[]);
        let i = comparar(&m, Some(&estado_con("1.0.0")), Some("B"), &instaladas);
        assert!(i.hay_parche_nuevo);
        assert_eq!(i.compatible_con_juego, Some(true));

        // Juego actualizado a una versión que no soporta ni lo instalado ni lo publicado.
        let m = manifest("1.0.0", &["A"], &[]);
        assert_eq!(comparar(&m, Some(&estado_con("1.0.0")), Some("A"), &instaladas).compatible_con_juego, Some(true));
        let i = comparar(&m, Some(&estado_con("1.0.0")), Some("Z"), &instaladas);
        assert!(!i.hay_parche_nuevo);
        assert_eq!(i.compatible_con_juego, Some(false));
    }

    #[test]
    fn extrae_el_zip_publicado() {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let o = zip::write::SimpleFileOptions::default();
            z.start_file("GALAXYZ-neo-Espanol-Latino/epk/scene_data.epk", o).unwrap();
            z.write_all(b"hola").unwrap();
            z.start_file("GALAXYZ-neo-Espanol-Latino/extra/res/gui/textures/TEST/logo.webp", o).unwrap();
            z.write_all(b"imagen").unwrap();
            z.start_file("GALAXYZ-neo-Espanol-Latino/README.md", o).unwrap();
            z.write_all(b"readme").unwrap();
            z.start_file("GALAXYZ-neo-Espanol-Latino/instalar.bat", o).unwrap();
            z.finish().unwrap();
        }
        let zip = buf.into_inner();
        let p = paquete_desde_zip(
            manifest("2.0.0", &[], &[("scene_data.epk", b"hola"), (r"res\gui\textures\TEST\logo.webp", b"imagen")]),
            &zip,
        )
        .unwrap();
        assert_eq!(p.archivos.len(), 2);
        let paquete = p.paquete();
        assert!(paquete.destinos().iter().any(|(r, _)| r == Path::new(r"res\gui\textures\TEST\logo.webp")));
        assert!(paquete_desde_zip(manifest("2.0.0", &[], &[("scene_data.epk", b"otra")]), &zip).is_err());
        assert!(paquete_desde_zip(manifest("2.0.0", &[], &[("falta.epk", b"x")]), &zip).is_err());
    }

    #[test]
    fn respeta_la_frecuencia() {
        let ahora = chrono::Local::now();
        let mut e = estado_con("1.0.0");
        assert!(toca_comprobar(&e, ahora));
        e.last_check = Some((ahora - chrono::Duration::hours(3)).to_rfc3339());
        e.update_interval_hours = 6;
        assert!(!toca_comprobar(&e, ahora));
        e.last_check = Some((ahora - chrono::Duration::minutes(6 * 60 - 5)).to_rfc3339());
        assert!(toca_comprobar(&e, ahora), "la tolerancia evita saltarse una tarea puntual");
    }

    #[test]
    fn detecta_archivos_danados() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("epk")).unwrap();
        std::fs::write(dir.path().join(r"epk\a.epk"), b"bien").unwrap();
        let mut e = estado_con("1.0.0");
        e.data_path = dir.path().to_path_buf();
        e.targets = vec!["epk".into()];
        e.files = BTreeMap::from([(r"epk\a.epk".into(), sha256_bytes(b"bien")), (r"epk\b.epk".into(), "00".into())]);
        assert_eq!(archivos_danados(&e), vec![r"epk\b.epk".to_string()]);
    }
}
