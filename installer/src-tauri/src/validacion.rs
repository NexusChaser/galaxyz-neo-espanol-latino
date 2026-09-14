//! Validación de las rutas del juego y de la carpeta del parche (§6.3 del plan).

use crate::deteccion::{self, EXE_JUEGO};
use crate::hash::sha256_archivo;
use crate::paquete::Paquete;
use serde::Serialize;
use std::path::{Component, Path, PathBuf};

pub const CARPETA_BACKUP: &str = "backup_antes_del_parche";
const MARGEN_ESPACIO: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidacionJuego {
    pub ok: bool,
    pub ruta: Option<PathBuf>,
    pub version: Option<String>,
    /// `None` si no hay `ver.dat`.
    pub version_compatible: Option<bool>,
    pub errores: Vec<String>,
    pub avisos: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivoAjeno {
    /// Ruta relativa a `data\`, p. ej. `epk\scene_data.epk`.
    pub ruta: String,
    pub tamano: u64,
}

/// Qué hay ahora mismo en la carpeta del parche.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "tipo", rename_all = "camelCase")]
pub enum EstadoCarpeta {
    /// Sin `.epk` del parche: lo normal antes de instalar.
    Vacia,
    /// Solo hay archivos del parche.
    Parche {
        versiones: Vec<String>,
        /// Están todos los archivos de esta versión en todas las carpetas.
        completo: bool,
    },
    /// Hay `.epk` con el mismo nombre que no son del parche (otro mod, otra traducción).
    Ajenos { archivos: Vec<ArchivoAjeno> },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidacionDatos {
    pub ok: bool,
    pub ruta_normalizada: PathBuf,
    pub existe: bool,
    pub errores: Vec<String>,
    pub avisos: Vec<String>,
    pub estado: Option<EstadoCarpeta>,
    pub espacio_libre: Option<u64>,
    pub espacio_necesario: u64,
}

pub fn validar_juego(ruta: &Path, paquete: &Paquete) -> ValidacionJuego {
    let mut v = ValidacionJuego {
        ok: false,
        ruta: None,
        version: None,
        version_compatible: None,
        errores: Vec::new(),
        avisos: Vec::new(),
    };
    let encontrada = if deteccion::es_carpeta_juego(ruta) {
        Some(ruta.to_path_buf())
    } else {
        // Si eligió una carpeta de más arriba (p. ej. steamapps\common), mirar un nivel abajo.
        let mut hijos: Vec<PathBuf> = std::fs::read_dir(ruta)
            .map(|it| {
                it.flatten()
                    .map(|e| e.path())
                    .filter(|p| deteccion::es_carpeta_juego(p))
                    .collect()
            })
            .unwrap_or_default();
        hijos.sort_by_key(|p| p.to_string_lossy().to_lowercase().contains("demo"));
        if hijos.len() > 1 {
            v.avisos.push(format!(
                "Hay varias instalaciones del juego en esta carpeta; se usará «{}».",
                hijos[0].display()
            ));
        }
        hijos.into_iter().next()
    };
    match encontrada {
        Some(r) => {
            v.ok = true;
            v.version = deteccion::leer_version(&r);
            v.version_compatible = v.version.as_deref().map(|ver| paquete.es_compatible(ver));
            if v.version_compatible == Some(false) {
                v.avisos.push(format!(
                    "La versión del juego ({}) no se probó con este parche. Puede haber textos raros o cierres.",
                    v.version.as_deref().unwrap_or_default()
                ));
            }
            v.ruta = Some(r);
        }
        None if !ruta.is_dir() => v.errores.push("Esa carpeta no existe.".into()),
        None => v
            .errores
            .push(format!("En esta carpeta no está «{EXE_JUEGO}».")),
    }
    v
}

/// Lleva lo que haya elegido el usuario a la carpeta `…\fuzz\galaxyz\data` correspondiente.
pub fn normalizar_datos(ruta: &Path) -> PathBuf {
    let partes: Vec<Component> = ruta.components().collect();
    let nombres: Vec<String> = partes
        .iter()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();
    // …\fuzz\galaxyz\data[\lo que sea]
    if let Some(i) = (2..nombres.len())
        .find(|&i| nombres[i] == "data" && nombres[i - 1] == "galaxyz" && nombres[i - 2] == "fuzz")
    {
        return partes[..=i].iter().collect();
    }
    match nombres.last().map(String::as_str) {
        Some("galaxyz") if nombres.len() >= 2 && nombres[nombres.len() - 2] == "fuzz" => ruta.join("data"),
        Some("fuzz") => ruta.join("galaxyz").join("data"),
        // Una carpeta cualquiera se toma como LOCALAPPDATA.
        _ => ruta.join("fuzz").join("galaxyz").join("data"),
    }
}

pub fn validar_datos(ruta: &Path, paquete: &Paquete, local_app_data: Option<&Path>) -> ValidacionDatos {
    let datos = normalizar_datos(ruta);
    let mut v = ValidacionDatos {
        ok: false,
        existe: datos.is_dir(),
        ruta_normalizada: datos.clone(),
        errores: Vec::new(),
        avisos: Vec::new(),
        estado: None,
        espacio_libre: None,
        espacio_necesario: 0,
    };

    if !datos.is_absolute() {
        v.errores.push("La ruta tiene que ser completa (por ejemplo C:\\…).".into());
        return v;
    }

    match local_app_data {
        Some(lad) => {
            let esperada = lad.join("fuzz").join("galaxyz").join("data");
            if !deteccion::misma_ruta(&datos, &esperada) {
                v.avisos.push(format!(
                    "El juego solo leerá esta carpeta si lo arrancas con LOCALAPPDATA apuntando a «{}».",
                    datos.parent().and_then(Path::parent).and_then(Path::parent).unwrap_or(&datos).display()
                ));
            }
        }
        None => v.avisos.push("No se pudo leer LOCALAPPDATA para comprobar la ruta.".into()),
    }

    // Un archivo donde debería ir una carpeta.
    let mut a_revisar = vec![datos.clone()];
    for c in carpetas_de_destino(paquete) {
        let mut r = datos.clone();
        for parte in c.components() {
            r.push(parte);
            if !a_revisar.contains(&r) {
                a_revisar.push(r.clone());
            }
        }
    }
    for r in a_revisar.iter().chain(datos.ancestors().skip(1).map(Path::to_path_buf).collect::<Vec<_>>().iter()) {
        if r.is_file() {
            v.errores.push(format!(
                "Hay un archivo llamado «{}» donde debería ir una carpeta: {}",
                r.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
                r.display()
            ));
        }
    }
    v.errores.dedup();
    if !v.errores.is_empty() {
        return v;
    }

    // Qué hay ya en la carpeta.
    let estado = clasificar(&datos, paquete);
    let bytes_ajenos: u64 = match &estado {
        EstadoCarpeta::Ajenos { archivos } => archivos.iter().map(|a| a.tamano).sum(),
        _ => 0,
    };
    v.estado = Some(estado);

    // Permiso de escritura.
    if let Err(e) = probar_escritura(&datos, paquete) {
        v.errores.push(e);
    }

    // Espacio libre.
    v.espacio_necesario = paquete.tamano_instalado() + bytes_ajenos + MARGEN_ESPACIO;
    v.espacio_libre = ancestro_existente(&datos).and_then(|a| espacio_libre(&a));
    if let Some(libre) = v.espacio_libre {
        if libre < v.espacio_necesario {
            v.errores.push(format!(
                "No hay espacio suficiente: hacen falta {} y quedan {}.",
                formato_mb(v.espacio_necesario),
                formato_mb(libre)
            ));
        }
    }

    v.ok = v.errores.is_empty();
    v
}

/// Carpetas (relativas a `data\`) donde el parche escribe algo, sin repetir.
pub fn carpetas_de_destino(paquete: &Paquete) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    for (r, _) in paquete.destinos() {
        if let Some(padre) = r.parent().filter(|p| !p.as_os_str().is_empty()) {
            if !v.iter().any(|x| x == padre) {
                v.push(padre.to_path_buf());
            }
        }
    }
    v
}

/// Clasifica los archivos que ya existen en las rutas donde va a escribir el parche.
pub fn clasificar(datos: &Path, paquete: &Paquete) -> EstadoCarpeta {
    let mut ajenos = Vec::new();
    let mut versiones: Vec<String> = Vec::new();
    let mut del_parche_actual = 0usize;
    let mut existentes = 0usize;
    let destinos = paquete.destinos();
    for (relativa, a) in &destinos {
        let ruta = datos.join(relativa);
        let Ok(meta) = std::fs::metadata(&ruta) else { continue };
        if !meta.is_file() {
            continue;
        }
        existentes += 1;
        let version = sha256_archivo(&ruta).ok().and_then(|h| paquete.version_del_hash(&a.id(), &h));
        match version {
            Some(ver) => {
                if ver == paquete.version {
                    del_parche_actual += 1;
                }
                if !versiones.contains(&ver) {
                    versiones.push(ver);
                }
            }
            None => ajenos.push(ArchivoAjeno {
                ruta: relativa.display().to_string(),
                tamano: meta.len(),
            }),
        }
    }
    if !ajenos.is_empty() {
        EstadoCarpeta::Ajenos { archivos: ajenos }
    } else if existentes == 0 {
        EstadoCarpeta::Vacia
    } else {
        EstadoCarpeta::Parche { versiones, completo: del_parche_actual == destinos.len() }
    }
}

/// Crea la ruta si falta, escribe y borra un archivo de prueba y deshace lo creado.
fn probar_escritura(datos: &Path, paquete: &Paquete) -> Result<(), String> {
    let creadas = crear_con_registro(datos).map_err(|e| mensaje_escritura(datos, &e))?;
    let mut carpetas = vec![datos.to_path_buf()];
    carpetas.extend(carpetas_de_destino(paquete).iter().map(|c| datos.join(c)).filter(|p| p.is_dir()));
    let resultado = carpetas.iter().try_for_each(|c| {
        let prueba = c.join(format!(".galaxyz-es-prueba-{}", std::process::id()));
        std::fs::write(&prueba, b"ok")
            .and_then(|_| std::fs::remove_file(&prueba))
            .map_err(|e| mensaje_escritura(c, &e))
    });
    for c in creadas.iter().rev() {
        let _ = std::fs::remove_dir(c);
    }
    resultado
}

fn mensaje_escritura(ruta: &Path, e: &std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        format!("No hay permiso para escribir en «{}».", ruta.display())
    } else {
        format!("No se puede escribir en «{}»: {e}", ruta.display())
    }
}

/// `create_dir_all` que devuelve las carpetas que creó, de la más alta a la más profunda.
pub fn crear_con_registro(ruta: &Path) -> std::io::Result<Vec<PathBuf>> {
    let faltan: Vec<PathBuf> = ruta
        .ancestors()
        .take_while(|p| !p.exists())
        .map(Path::to_path_buf)
        .collect();
    std::fs::create_dir_all(ruta)?;
    Ok(faltan.into_iter().rev().collect())
}

pub fn ancestro_existente(ruta: &Path) -> Option<PathBuf> {
    ruta.ancestors().find(|p| p.is_dir()).map(Path::to_path_buf)
}

#[cfg(windows)]
pub fn espacio_libre(ruta: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let ancha: Vec<u16> = ruta.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut libre = 0u64;
    let ok = unsafe { GetDiskFreeSpaceExW(ancha.as_ptr(), &mut libre, std::ptr::null_mut(), std::ptr::null_mut()) };
    (ok != 0).then_some(libre)
}

#[cfg(not(windows))]
pub fn espacio_libre(_ruta: &Path) -> Option<u64> {
    None
}

pub fn formato_mb(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paquete::tests_util::paquete_prueba;

    #[test]
    fn normaliza_las_rutas_elegidas() {
        let base = Path::new(r"C:\Users\x\AppData\Local");
        let data = base.join(r"fuzz\galaxyz\data");
        assert_eq!(normalizar_datos(&data), data);
        assert_eq!(normalizar_datos(&base.join(r"fuzz\galaxyz")), data);
        assert_eq!(normalizar_datos(&base.join("fuzz")), data);
        assert_eq!(normalizar_datos(&data.join(r"locale\us\epk")), data);
        assert_eq!(normalizar_datos(base), data);
        assert_eq!(normalizar_datos(Path::new(r"C:\Users\x\AppData\Local\FUZZ\GalaxyZ\Data")), PathBuf::from(r"C:\Users\x\AppData\Local\FUZZ\GalaxyZ\Data"));
    }

    #[test]
    fn carpeta_inexistente_es_valida_y_no_deja_rastro() {
        let lad = tempfile::tempdir().unwrap();
        let p = paquete_prueba();
        let v = validar_datos(lad.path(), &p, Some(lad.path()));
        assert!(v.ok, "{:?}", v.errores);
        assert!(!v.existe);
        assert!(v.avisos.is_empty(), "{:?}", v.avisos);
        assert!(matches!(v.estado, Some(EstadoCarpeta::Vacia)));
        assert!(!lad.path().join("fuzz").exists(), "la prueba de escritura dejó carpetas");
    }

    #[test]
    fn avisa_fuera_de_localappdata_y_detecta_archivo_en_lugar_de_carpeta() {
        let lad = tempfile::tempdir().unwrap();
        let otra = tempfile::tempdir().unwrap();
        let p = paquete_prueba();
        let v = validar_datos(otra.path(), &p, Some(lad.path()));
        assert!(v.ok);
        assert_eq!(v.avisos.len(), 1);

        let data = lad.path().join(r"fuzz\galaxyz\data");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("epk"), b"soy un archivo").unwrap();
        let v = validar_datos(&data, &p, Some(lad.path()));
        assert!(!v.ok);
        assert!(v.errores[0].contains("«epk»"), "{:?}", v.errores);
    }

    #[test]
    fn clasifica_parche_y_ajenos() {
        let lad = tempfile::tempdir().unwrap();
        let p = paquete_prueba();
        let data = lad.path().join(r"fuzz\galaxyz\data");
        for t in &p.targets {
            std::fs::create_dir_all(data.join(t)).unwrap();
            for a in &p.archivos {
                std::fs::write(data.join(t).join(&a.nombre), a.datos).unwrap();
            }
        }
        assert!(matches!(clasificar(&data, &p), EstadoCarpeta::Parche { completo: true, .. }));
        std::fs::write(data.join(&p.targets[0]).join(&p.archivos[0].nombre), b"mod").unwrap();
        match clasificar(&data, &p) {
            EstadoCarpeta::Ajenos { archivos } => assert_eq!(archivos.len(), 1),
            otro => panic!("{otro:?}"),
        }
    }

    #[test]
    fn valida_carpeta_de_juego() {
        let dir = tempfile::tempdir().unwrap();
        let p = paquete_prueba();
        assert!(!validar_juego(dir.path(), &p).ok);
        let juego = dir.path().join("GALAXYZ neo Demo");
        std::fs::create_dir_all(&juego).unwrap();
        std::fs::write(juego.join(EXE_JUEGO), b"").unwrap();
        std::fs::write(juego.join("ver.dat"), "20990101_000000").unwrap();
        let v = validar_juego(dir.path(), &p);
        assert!(v.ok);
        assert_eq!(v.ruta.as_deref(), Some(juego.as_path()));
        assert_eq!(v.version_compatible, Some(false));
        assert_eq!(v.avisos.len(), 1);
    }
}
