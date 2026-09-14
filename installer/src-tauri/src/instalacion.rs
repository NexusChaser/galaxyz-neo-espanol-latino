//! Copia de seguridad, instalación atómica con vuelta atrás, reparación y desinstalación
//! (§6.4, §6.5 y §6.7 del plan).

use crate::deteccion;
use crate::error::{io, Error, Resultado};
use crate::estado::{self, Estado, ModoRuta};
use crate::hash::sha256_archivo;
use crate::paquete::Paquete;
use crate::validacion::{self, EstadoCarpeta, CARPETA_BACKUP};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Lo que depende del sistema real. En las pruebas se sustituye por uno falso.
pub trait Entorno {
    fn juego_abierto(&self) -> bool;
    /// `%LOCALAPPDATA%\GalaxyzNeoES`.
    fn carpeta_app(&self) -> PathBuf;
    fn local_app_data(&self) -> Option<PathBuf>;
    /// Copia el ejecutable, crea la entrada de «Aplicaciones instaladas» y el acceso directo.
    fn registrar_aplicacion(&self, estado: &Estado) -> Result<(), String>;
    /// Crea o quita el arranque de la búsqueda de actualizaciones.
    fn configurar_actualizacion(&self, activar: bool, horas: u32) -> Result<(), String>;
    fn quitar_aplicacion(&self) -> Result<(), String>;
}

fn intervalo_por_defecto() -> u32 {
    estado::INTERVALO_POR_DEFECTO
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opciones {
    pub ruta_datos: PathBuf,
    #[serde(default)]
    pub modo_ruta: ModoRuta,
    pub ruta_juego: Option<PathBuf>,
    pub copia_seguridad: bool,
    pub auto_actualizar: bool,
    /// Cada cuántas horas buscar actualizaciones (6, 12, 24 o 168).
    #[serde(default = "intervalo_por_defecto")]
    pub intervalo_actualizacion_horas: u32,
    pub acceso_directo: bool,
    #[serde(default)]
    pub dry_run: bool,
    /// Instalar aunque `ver.dat` no esté entre las versiones probadas.
    #[serde(default)]
    pub aceptar_version_no_probada: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Paso {
    Descargando,
    Comprobando,
    CopiaSeguridad,
    Copiando,
    Verificando,
    Guardando,
    Borrando,
    Restaurando,
    Revirtiendo,
    Terminado,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progreso {
    pub paso: Paso,
    pub porcentaje: f32,
    pub mensaje: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoInstalacion {
    pub dry_run: bool,
    pub version: String,
    pub ruta_datos: PathBuf,
    pub archivos_escritos: usize,
    pub archivos_sin_cambios: usize,
    pub carpetas_creadas: Vec<PathBuf>,
    pub copia_seguridad: Option<PathBuf>,
    pub archivos_respaldados: Vec<String>,
    /// Qué se hizo (o se haría, en `dry_run`), en orden.
    pub acciones: Vec<String>,
    pub avisos: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoDesinstalacion {
    pub dry_run: bool,
    pub borrados: Vec<String>,
    /// Archivos con nombre del parche pero modificados después: no se borran sin `forzar`.
    pub conservados: Vec<String>,
    pub restaurados: Vec<String>,
    pub carpetas_quitadas: Vec<PathBuf>,
    pub acciones: Vec<String>,
    pub avisos: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoBackup {
    pub creado: String,
    pub version_juego: Option<String>,
    pub archivos: Vec<EntradaBackup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntradaBackup {
    /// Ruta relativa a `data\` (la misma dentro de la carpeta de la copia).
    pub ruta: String,
    pub sha256: String,
}

/// Registro de cambios para poder deshacer la instalación si algo falla.
#[derive(Default)]
struct Diario {
    temporal: Option<PathBuf>,
    carpetas_creadas: Vec<PathBuf>,
    archivos_nuevos: Vec<PathBuf>,
    /// (destino, copia del original en la carpeta temporal)
    reemplazados: Vec<(PathBuf, PathBuf)>,
    /// `backup.json` previo, si existía y se reescribió.
    backup_json_previo: Option<(PathBuf, Vec<u8>)>,
}

impl Diario {
    fn temporal(&mut self) -> Resultado<PathBuf> {
        if let Some(t) = &self.temporal {
            return Ok(t.clone());
        }
        let t = std::env::temp_dir().join(format!(
            "galaxyz-es-{}-{}",
            std::process::id(),
            chrono::Local::now().format("%Y%m%d%H%M%S%f")
        ));
        std::fs::create_dir_all(&t).map_err(io("crear", &t))?;
        self.temporal = Some(t.clone());
        Ok(t)
    }

    fn crear_carpeta(&mut self, ruta: &Path) -> Resultado<()> {
        let nuevas = validacion::crear_con_registro(ruta).map_err(io("crear la carpeta", ruta))?;
        self.carpetas_creadas.extend(nuevas);
        Ok(())
    }

    /// Escribe `datos` en `destino` pasando por `destino.tmp` y un renombrado.
    fn escribir(&mut self, destino: &Path, datos: &[u8]) -> Resultado<()> {
        if destino.exists() {
            let t = self.temporal()?;
            let copia = t.join(format!("{}.orig", self.reemplazados.len()));
            std::fs::copy(destino, &copia).map_err(io("guardar una copia de", destino))?;
            self.reemplazados.push((destino.to_path_buf(), copia));
        } else {
            self.archivos_nuevos.push(destino.to_path_buf());
        }
        let tmp = con_sufijo(destino, ".tmp");
        escribir_sincronizado(&tmp, datos)?;
        std::fs::rename(&tmp, destino).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            io("reemplazar", destino)(e)
        })
    }

    fn copiar_nuevo(&mut self, origen: &Path, destino: &Path) -> Resultado<()> {
        std::fs::copy(origen, destino).map_err(io("copiar", origen))?;
        self.archivos_nuevos.push(destino.to_path_buf());
        Ok(())
    }

    /// Deshace todo lo registrado, en orden inverso. Nunca falla: hace lo que puede.
    fn revertir(&mut self) -> Vec<String> {
        let mut problemas = Vec::new();
        for (destino, copia) in self.reemplazados.drain(..).rev() {
            if let Err(e) = std::fs::copy(&copia, &destino) {
                problemas.push(format!("No se pudo restaurar «{}»: {e}", destino.display()));
            }
        }
        for archivo in self.archivos_nuevos.drain(..).rev() {
            let _ = std::fs::remove_file(con_sufijo(&archivo, ".tmp"));
            if let Err(e) = std::fs::remove_file(&archivo) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    problemas.push(format!("No se pudo borrar «{}»: {e}", archivo.display()));
                }
            }
        }
        if let Some((ruta, contenido)) = self.backup_json_previo.take() {
            let _ = std::fs::write(ruta, contenido);
        }
        for carpeta in self.carpetas_creadas.drain(..).rev() {
            let _ = std::fs::remove_dir(carpeta);
        }
        self.limpiar();
        problemas
    }

    fn limpiar(&mut self) {
        if let Some(t) = self.temporal.take() {
            let _ = std::fs::remove_dir_all(t);
        }
    }
}

fn con_sufijo(ruta: &Path, sufijo: &str) -> PathBuf {
    let mut s = ruta.as_os_str().to_owned();
    s.push(sufijo);
    PathBuf::from(s)
}

fn escribir_sincronizado(ruta: &Path, datos: &[u8]) -> Resultado<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(ruta).map_err(io("crear", ruta))?;
    f.write_all(datos).map_err(io("escribir", ruta))?;
    f.sync_all().map_err(io("escribir", ruta))
}

fn ahora() -> String {
    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
}

pub fn instalar(
    paquete: &Paquete,
    opciones: &Opciones,
    entorno: &dyn Entorno,
    progreso: &mut dyn FnMut(Progreso),
) -> Resultado<ResultadoInstalacion> {
    let mut informar = |paso, porcentaje: f32, mensaje: String| {
        progreso(Progreso { paso, porcentaje: porcentaje.clamp(0.0, 100.0), mensaje })
    };

    // 0. Validar rutas y versión.
    informar(Paso::Comprobando, 0.0, "Comprobando la carpeta del parche…".into());
    let lad = entorno.local_app_data();
    let validacion = validacion::validar_datos(&opciones.ruta_datos, paquete, lad.as_deref());
    if !validacion.ok {
        return Err(Error::Validacion(validacion.errores.join("\n")));
    }
    let datos = validacion.ruta_normalizada.clone();
    let mut resultado = ResultadoInstalacion {
        dry_run: opciones.dry_run,
        version: paquete.version.clone(),
        ruta_datos: datos.clone(),
        avisos: validacion.avisos.clone(),
        ..Default::default()
    };

    let (ruta_juego, version_juego) = match &opciones.ruta_juego {
        Some(r) => {
            let v = validacion::validar_juego(r, paquete);
            if !v.ok {
                return Err(Error::Validacion(v.errores.join("\n")));
            }
            if v.version_compatible == Some(false) && !opciones.aceptar_version_no_probada {
                return Err(Error::Validacion(v.avisos.join("\n")));
            }
            resultado.avisos.extend(v.avisos);
            (v.ruta, v.version)
        }
        None => (None, None),
    };

    // 1. Juego cerrado.
    informar(Paso::Comprobando, 1.0, "Comprobando que el juego esté cerrado…".into());
    if entorno.juego_abierto() {
        return Err(Error::JuegoAbierto);
    }

    // 2. Paquete íntegro.
    informar(Paso::Comprobando, 2.0, "Verificando los archivos del parche…".into());
    paquete.verificar()?;

    let ajenos = match validacion.estado {
        Some(EstadoCarpeta::Ajenos { archivos }) => archivos,
        _ => Vec::new(),
    };
    let carpeta_backup = datos.join(CARPETA_BACKUP);
    let destinos = paquete.destinos();
    let total = destinos.len().max(1) as f32;

    if opciones.dry_run {
        if opciones.copia_seguridad {
            for a in &ajenos {
                resultado.acciones.push(format!(
                    "Copia de seguridad: {} → {}",
                    a.ruta,
                    carpeta_backup.join(&a.ruta).display()
                ));
            }
        }
        for c in validacion::carpetas_de_destino(paquete) {
            let destino = datos.join(&c);
            if !destino.exists() {
                resultado.acciones.push(format!("Crear carpeta {}", destino.display()));
            }
            let n = destinos.iter().filter(|(r, _)| r.parent() == Some(c.as_path())).count();
            resultado.acciones.push(format!("Copiar {n} archivos a {}", destino.display()));
        }
        resultado.acciones.push(format!(
            "Guardar el estado en {}",
            estado::ruta_estado(&entorno.carpeta_app()).display()
        ));
        informar(Paso::Terminado, 100.0, "Simulación terminada: no se escribió nada.".into());
        return Ok(resultado);
    }

    let mut diario = Diario::default();
    let paso_a_paso = (|| -> Resultado<Estado> {
        // 3. Copia de seguridad de los archivos ajenos (§6.4).
        if opciones.copia_seguridad && !ajenos.is_empty() {
            informar(Paso::CopiaSeguridad, 3.0, "Haciendo copia de seguridad…".into());
            let json = carpeta_backup.join("backup.json");
            let mut info: InfoBackup = match std::fs::read(&json) {
                Ok(previo) => {
                    let info = serde_json::from_slice(&previo).unwrap_or_default();
                    diario.backup_json_previo = Some((json.clone(), previo));
                    info
                }
                Err(_) => InfoBackup {
                    creado: ahora(),
                    version_juego: version_juego.clone(),
                    archivos: Vec::new(),
                },
            };
            for a in &ajenos {
                let origen = datos.join(&a.ruta);
                let destino = carpeta_backup.join(&a.ruta);
                if destino.exists() {
                    // Una copia que ya existe no se sobrescribe nunca.
                    continue;
                }
                if let Some(padre) = destino.parent() {
                    diario.crear_carpeta(padre)?;
                }
                diario.copiar_nuevo(&origen, &destino)?;
                info.archivos.push(EntradaBackup { ruta: a.ruta.clone(), sha256: sha256_archivo(&destino)? });
                resultado.archivos_respaldados.push(a.ruta.clone());
            }
            if !resultado.archivos_respaldados.is_empty() {
                if diario.backup_json_previo.is_none() {
                    diario.archivos_nuevos.push(json.clone());
                }
                let texto = serde_json::to_vec_pretty(&info).expect("InfoBackup siempre serializa");
                std::fs::write(&json, texto).map_err(io("escribir", &json))?;
                resultado.copia_seguridad = Some(carpeta_backup.clone());
                resultado.acciones.push(format!(
                    "Copia de seguridad de {} archivos en {}",
                    resultado.archivos_respaldados.len(),
                    carpeta_backup.display()
                ));
            }
        }

        // 4 y 5. Crear carpetas y copiar. Se comprueba el juego cada vez que cambia la carpeta.
        let mut carpeta_actual: Option<PathBuf> = None;
        for (i, (relativa, a)) in destinos.iter().enumerate() {
            let padre = relativa.parent().map(Path::to_path_buf).unwrap_or_default();
            if carpeta_actual.as_ref() != Some(&padre) {
                if entorno.juego_abierto() {
                    return Err(Error::JuegoAbierto);
                }
                let destino_dir = datos.join(&padre);
                let antes = diario.carpetas_creadas.len();
                diario.crear_carpeta(&destino_dir)?;
                if diario.carpetas_creadas.len() > antes {
                    resultado.acciones.push(format!("Carpeta creada: {}", destino_dir.display()));
                }
                carpeta_actual = Some(padre.clone());
            }
            let destino = datos.join(relativa);
            informar(
                Paso::Copiando,
                5.0 + 70.0 * (i + 1) as f32 / total,
                format!("Instalando en {} ({}/{})…", padre.display(), i + 1, total as usize),
            );
            let igual = destino.is_file() && sha256_archivo(&destino).ok().as_deref() == Some(&a.sha256);
            if igual {
                resultado.archivos_sin_cambios += 1;
                continue;
            }
            diario.escribir(&destino, a.datos)?;
            resultado.archivos_escritos += 1;
        }
        resultado.acciones.push(format!(
            "{} archivos escritos, {} ya estaban al día",
            resultado.archivos_escritos, resultado.archivos_sin_cambios
        ));

        // 6. Verificar lo instalado.
        for (i, (relativa, a)) in destinos.iter().enumerate() {
            informar(
                Paso::Verificando,
                75.0 + 20.0 * (i + 1) as f32 / total,
                format!("Verificando ({}/{})…", i + 1, total as usize),
            );
            let destino = datos.join(relativa);
            if sha256_archivo(&destino)? != a.sha256 {
                return Err(Error::Verificacion(destino.display().to_string()));
            }
        }

        // 7. Guardar el estado.
        informar(Paso::Guardando, 96.0, "Guardando…".into());
        let carpeta_app = entorno.carpeta_app();
        let previo = estado::leer(&carpeta_app);
        let mut creadas = previo.as_ref().map(|e| e.created_dirs.clone()).unwrap_or_default();
        for c in &diario.carpetas_creadas {
            if !creadas.iter().any(|x| deteccion::misma_ruta(x, c)) {
                creadas.push(c.clone());
            }
        }
        let estado_nuevo = Estado {
            patch_version: paquete.version.clone(),
            data_path: datos.clone(),
            path_mode: opciones.modo_ruta,
            game_path: ruta_juego.clone(),
            game_version: version_juego.clone(),
            backup: opciones.copia_seguridad,
            backup_path: resultado
                .copia_seguridad
                .clone()
                .or_else(|| previo.as_ref().and_then(|e| e.backup_path.clone())),
            auto_update: opciones.auto_actualizar,
            update_interval_hours: estado::intervalo_valido(opciones.intervalo_actualizacion_horas),
            shortcut: opciones.acceso_directo,
            installed_at: ahora(),
            last_check_result: previo.as_ref().and_then(|e| e.last_check_result.clone()),
            last_check: previo.and_then(|e| e.last_check),
            unsupported_game_version: None,
            targets: paquete.targets.clone(),
            files: destinos
                .iter()
                .map(|(r, a)| (r.display().to_string(), a.sha256.clone()))
                .collect::<BTreeMap<_, _>>(),
            created_dirs: creadas,
        };
        estado::guardar(&carpeta_app, &estado_nuevo)?;
        Ok(estado_nuevo)
    })();

    match paso_a_paso {
        Ok(estado_nuevo) => {
            resultado.carpetas_creadas = diario.carpetas_creadas.clone();
            diario.limpiar();
            if let Err(e) = entorno.registrar_aplicacion(&estado_nuevo) {
                resultado
                    .avisos
                    .push(format!("El parche se instaló, pero no se pudo registrar la aplicación: {e}"));
            }
            informar(Paso::Terminado, 100.0, "¡Parche instalado!".into());
            Ok(resultado)
        }
        Err(e) => {
            informar(Paso::Revirtiendo, 100.0, "Algo falló. Deshaciendo los cambios…".into());
            let problemas = diario.revertir();
            if problemas.is_empty() {
                Err(e)
            } else {
                Err(Error::Validacion(format!(
                    "{e}\n\nAdemás, al deshacer los cambios:\n{}",
                    problemas.join("\n")
                )))
            }
        }
    }
}

/// Vuelve a instalar con las opciones guardadas.
pub fn reparar(
    paquete: &Paquete,
    entorno: &dyn Entorno,
    dry_run: bool,
    progreso: &mut dyn FnMut(Progreso),
) -> Resultado<ResultadoInstalacion> {
    let e = estado::leer(&entorno.carpeta_app()).ok_or(Error::SinInstalacion)?;
    let opciones = Opciones {
        ruta_datos: e.data_path,
        modo_ruta: e.path_mode,
        ruta_juego: e.game_path.filter(|r| deteccion::es_carpeta_juego(r)),
        copia_seguridad: e.backup,
        auto_actualizar: e.auto_update,
        intervalo_actualizacion_horas: e.update_interval_hours,
        acceso_directo: e.shortcut,
        dry_run,
        aceptar_version_no_probada: true,
    };
    instalar(paquete, &opciones, entorno, progreso)
}

/// Desinstala (§6.7). Si no hay `state.json`, usa `ruta_datos` y los nombres del paquete.
pub fn desinstalar(
    paquete: &Paquete,
    entorno: &dyn Entorno,
    ruta_datos: Option<&Path>,
    borrar_backup: bool,
    forzar: bool,
    dry_run: bool,
    progreso: &mut dyn FnMut(Progreso),
) -> Resultado<ResultadoDesinstalacion> {
    let mut informar = |paso, porcentaje: f32, mensaje: String| {
        progreso(Progreso { paso, porcentaje, mensaje })
    };
    let mut r = ResultadoDesinstalacion { dry_run, ..Default::default() };

    informar(Paso::Comprobando, 0.0, "Comprobando que el juego esté cerrado…".into());
    if entorno.juego_abierto() {
        return Err(Error::JuegoAbierto);
    }
    let carpeta_app = entorno.carpeta_app();
    let guardado = estado::leer(&carpeta_app);
    let datos = match (ruta_datos, &guardado) {
        (Some(p), _) => validacion::normalizar_datos(p),
        (None, Some(e)) => e.data_path.clone(),
        (None, None) => entorno
            .local_app_data()
            .map(|l| l.join("fuzz").join("galaxyz").join("data"))
            .ok_or(Error::SinInstalacion)?,
    };
    // Rutas del parche: las guardadas al instalar o, sin estado, las del paquete.
    let rutas: Vec<String> = guardado
        .as_ref()
        .map(|e| e.files.keys().cloned().collect::<Vec<_>>())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| paquete.destinos().iter().map(|(r, _)| r.display().to_string()).collect());

    // 1. Borrar los archivos del parche.
    let total = rutas.len().max(1) as f32;
    for (i, relativa) in rutas.iter().enumerate() {
        informar(Paso::Borrando, 5.0 + 70.0 * (i + 1) as f32 / total, "Quitando archivos del parche…".into());
        if !crate::paquete::ruta_segura(relativa) {
            continue;
        }
        let ruta = datos.join(relativa);
        if !ruta.is_file() {
            continue;
        }
        let hash = sha256_archivo(&ruta)?;
        let id = paquete.id_de_ruta(Path::new(relativa));
        let es_del_parche = paquete.version_del_hash(&id, &hash).is_some()
            || guardado.as_ref().and_then(|e| e.files.get(relativa)) == Some(&hash);
        if !es_del_parche && !forzar {
            r.conservados.push(relativa.clone());
            continue;
        }
        if !dry_run {
            std::fs::remove_file(&ruta).map_err(io("borrar", &ruta))?;
        }
        r.borrados.push(relativa.clone());
    }
    r.acciones.push(format!("{} archivos del parche quitados", r.borrados.len()));
    if !r.conservados.is_empty() {
        r.avisos.push(format!(
            "No se borraron {} archivos porque fueron modificados después de instalar el parche.",
            r.conservados.len()
        ));
    }

    // 2. Devolver la copia de seguridad.
    let carpeta_backup = datos.join(CARPETA_BACKUP);
    let json = carpeta_backup.join("backup.json");
    if let Some(info) = std::fs::read(&json)
        .ok()
        .and_then(|b| serde_json::from_slice::<InfoBackup>(&b).ok())
    {
        informar(Paso::Restaurando, 80.0, "Restaurando la copia de seguridad…".into());
        for entrada in &info.archivos {
            if !crate::paquete::ruta_segura(&entrada.ruta) {
                continue;
            }
            let origen = carpeta_backup.join(&entrada.ruta);
            let destino = datos.join(&entrada.ruta);
            if !origen.is_file() || destino.exists() {
                continue;
            }
            if !dry_run {
                if let Some(padre) = destino.parent() {
                    std::fs::create_dir_all(padre).map_err(io("crear", padre))?;
                }
                std::fs::copy(&origen, &destino).map_err(io("restaurar", &destino))?;
            }
            r.restaurados.push(entrada.ruta.clone());
        }
        if !r.restaurados.is_empty() {
            r.acciones.push(format!("{} archivos restaurados de la copia de seguridad", r.restaurados.len()));
        }
    }
    if borrar_backup && carpeta_backup.is_dir() {
        if !dry_run {
            std::fs::remove_dir_all(&carpeta_backup).map_err(io("borrar", &carpeta_backup))?;
        }
        r.acciones.push(format!("Copia de seguridad borrada: {}", carpeta_backup.display()));
    }

    // 3. Quitar carpetas vacías. Con estado: solo las que creó el instalador (si creó `data`
    // porque el juego nunca se abrió, también esa). Sin estado: las subcarpetas del parche,
    // nunca `data` ni nada por encima, que son del juego.
    let mut candidatas: Vec<PathBuf> = match &guardado {
        Some(e) if !e.created_dirs.is_empty() => e.created_dirs.clone(),
        _ => rutas
            .iter()
            .flat_map(|t| {
                Path::new(t)
                    .ancestors()
                    .skip(1)
                    .filter(|a| !a.as_os_str().is_empty())
                    .map(|a| datos.join(a))
                    .collect::<Vec<_>>()
            })
            .filter(|c| c.starts_with(&datos) && !deteccion::misma_ruta(c, &datos))
            .collect(),
    };
    candidatas.sort_by_key(|c| std::cmp::Reverse(c.components().count()));
    candidatas.dedup();
    if !dry_run {
        for c in candidatas {
            if std::fs::remove_dir(&c).is_ok() {
                r.carpetas_quitadas.push(c);
            }
        }
    }

    // 4. Estado y registro de la aplicación.
    informar(Paso::Guardando, 95.0, "Terminando…".into());
    if !dry_run {
        estado::borrar(&carpeta_app)?;
        if let Err(e) = entorno.quitar_aplicacion() {
            r.avisos.push(format!("No se pudo quitar el registro de la aplicación: {e}"));
        }
    }
    informar(Paso::Terminado, 100.0, "Parche desinstalado.".into());
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paquete::tests_util::paquete_prueba;
    use std::cell::Cell;

    struct EntornoFalso {
        lad: PathBuf,
        /// Cuántas veces responder «cerrado» antes de decir que el juego se abrió.
        cerrado_hasta: Cell<i32>,
        registrado: Cell<bool>,
    }

    impl EntornoFalso {
        fn nuevo(lad: &Path) -> Self {
            Self { lad: lad.to_path_buf(), cerrado_hasta: Cell::new(i32::MAX), registrado: Cell::new(false) }
        }
        fn datos(&self) -> PathBuf {
            self.lad.join(r"fuzz\galaxyz\data")
        }
    }

    impl Entorno for EntornoFalso {
        fn juego_abierto(&self) -> bool {
            let n = self.cerrado_hasta.get();
            self.cerrado_hasta.set(n - 1);
            n <= 0
        }
        fn carpeta_app(&self) -> PathBuf {
            self.lad.join("GalaxyzNeoES")
        }
        fn local_app_data(&self) -> Option<PathBuf> {
            Some(self.lad.clone())
        }
        fn registrar_aplicacion(&self, _: &Estado) -> Result<(), String> {
            self.registrado.set(true);
            Ok(())
        }
        fn configurar_actualizacion(&self, _: bool, _: u32) -> Result<(), String> {
            Ok(())
        }
        fn quitar_aplicacion(&self) -> Result<(), String> {
            self.registrado.set(false);
            Ok(())
        }
    }

    fn opciones(datos: &Path) -> Opciones {
        Opciones {
            ruta_datos: datos.to_path_buf(),
            modo_ruta: ModoRuta::Auto,
            ruta_juego: None,
            copia_seguridad: true,
            auto_actualizar: false,
            intervalo_actualizacion_horas: 24,
            acceso_directo: false,
            dry_run: false,
            aceptar_version_no_probada: false,
        }
    }

    fn archivos_en(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = walk(dir)
            .into_iter()
            .map(|p| p.strip_prefix(dir).unwrap().display().to_string())
            .collect();
        v.sort();
        v
    }

    fn walk(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Ok(it) = std::fs::read_dir(dir) {
            for e in it.flatten() {
                let p = e.path();
                if p.is_dir() {
                    out.extend(walk(&p));
                } else {
                    out.push(p);
                }
            }
        }
        out
    }

    #[test]
    fn instala_en_carpeta_inexistente_y_desinstala_sin_dejar_nada() {
        let lad = tempfile::tempdir().unwrap();
        let ent = EntornoFalso::nuevo(lad.path());
        let p = paquete_prueba();
        let mut eventos = 0;
        let r = instalar(&p, &opciones(&ent.datos()), &ent, &mut |_| eventos += 1).unwrap();
        assert_eq!(r.archivos_escritos, 6);
        assert!(r.copia_seguridad.is_none());
        assert!(eventos > 6);
        assert!(ent.registrado.get());
        assert_eq!(archivos_en(&ent.datos()).len(), 6);
        let e = estado::leer(&ent.carpeta_app()).unwrap();
        assert_eq!(e.files.len(), 6, "una entrada por cada copia instalada");
        assert!(e.files.contains_key(r"locale\us\epk\scene_data.epk"));
        assert!(e.created_dirs.iter().any(|c| c.ends_with("fuzz")));

        // Reinstalar no reescribe lo que ya está igual.
        let r2 = instalar(&p, &opciones(&ent.datos()), &ent, &mut |_| {}).unwrap();
        assert_eq!((r2.archivos_escritos, r2.archivos_sin_cambios), (0, 6));

        let d = desinstalar(&p, &ent, None, false, false, false, &mut |_| {}).unwrap();
        assert_eq!(d.borrados.len(), 6);
        assert!(!lad.path().join("fuzz").exists(), "quedaron carpetas: {:?}", archivos_en(lad.path()));
        assert!(estado::leer(&ent.carpeta_app()).is_none());
        assert!(!ent.registrado.get());
    }

    #[test]
    fn respalda_ajenos_y_los_devuelve_sin_tocar_la_partida() {
        let lad = tempfile::tempdir().unwrap();
        let ent = EntornoFalso::nuevo(lad.path());
        let p = paquete_prueba();
        let datos = ent.datos();
        std::fs::create_dir_all(datos.join("epk")).unwrap();
        std::fs::create_dir_all(datos.join("user")).unwrap();
        std::fs::write(datos.join(r"epk\scene_data.epk"), b"MOD").unwrap();
        std::fs::write(datos.join(r"epk\otro_mod.epk"), b"MOD2").unwrap();
        std::fs::write(datos.join(r"user\save.dat"), b"PARTIDA").unwrap();

        let r = instalar(&p, &opciones(&datos), &ent, &mut |_| {}).unwrap();
        assert_eq!(r.archivos_respaldados, vec![r"epk\scene_data.epk".to_string()]);
        assert_eq!(std::fs::read(datos.join(r"backup_antes_del_parche\epk\scene_data.epk")).unwrap(), b"MOD");

        let d = desinstalar(&p, &ent, None, false, false, false, &mut |_| {}).unwrap();
        assert_eq!(d.restaurados, vec![r"epk\scene_data.epk".to_string()]);
        assert_eq!(std::fs::read(datos.join(r"epk\scene_data.epk")).unwrap(), b"MOD");
        assert_eq!(std::fs::read(datos.join(r"epk\otro_mod.epk")).unwrap(), b"MOD2");
        assert_eq!(std::fs::read(datos.join(r"user\save.dat")).unwrap(), b"PARTIDA");
        assert!(!datos.join("locale").exists());
        assert!(datos.join("backup_antes_del_parche").exists(), "la copia no se borra por defecto");
    }

    #[test]
    fn si_el_juego_se_abre_a_mitad_deshace_todo() {
        let lad = tempfile::tempdir().unwrap();
        let ent = EntornoFalso::nuevo(lad.path());
        let p = paquete_prueba();
        let datos = ent.datos();
        std::fs::create_dir_all(datos.join(r"locale\us\epk")).unwrap();
        std::fs::write(datos.join(r"locale\us\epk\commontext.epk"), b"ORIGINAL AJENO").unwrap();
        // Primera comprobación y primera carpeta: cerrado. Al pasar a la segunda carpeta: abierto.
        ent.cerrado_hasta.set(2);

        let err = instalar(&p, &opciones(&datos), &ent, &mut |_| {}).unwrap_err();
        assert!(matches!(err, Error::JuegoAbierto), "{err}");
        assert_eq!(
            archivos_en(&datos),
            vec![r"locale\us\epk\commontext.epk".to_string()],
            "debe quedar exactamente como estaba"
        );
        assert_eq!(std::fs::read(datos.join(r"locale\us\epk\commontext.epk")).unwrap(), b"ORIGINAL AJENO");
        assert!(estado::leer(&ent.carpeta_app()).is_none());
    }

    #[test]
    fn no_instala_con_el_juego_abierto_ni_en_simulacion_escribe() {
        let lad = tempfile::tempdir().unwrap();
        let ent = EntornoFalso::nuevo(lad.path());
        let p = paquete_prueba();
        ent.cerrado_hasta.set(0);
        assert!(matches!(instalar(&p, &opciones(&ent.datos()), &ent, &mut |_| {}), Err(Error::JuegoAbierto)));

        let ent = EntornoFalso::nuevo(lad.path());
        let mut o = opciones(&ent.datos());
        o.dry_run = true;
        let r = instalar(&p, &o, &ent, &mut |_| {}).unwrap();
        assert!(r.dry_run && !r.acciones.is_empty());
        assert!(!lad.path().join("fuzz").exists());
        assert!(!ent.carpeta_app().exists());
    }

    #[test]
    fn no_borra_archivos_modificados_despues() {
        let lad = tempfile::tempdir().unwrap();
        let ent = EntornoFalso::nuevo(lad.path());
        let p = paquete_prueba();
        instalar(&p, &opciones(&ent.datos()), &ent, &mut |_| {}).unwrap();
        let tocado = ent.datos().join(r"epk\commontext.epk");
        std::fs::write(&tocado, b"editado a mano").unwrap();
        let d = desinstalar(&p, &ent, None, false, false, false, &mut |_| {}).unwrap();
        assert_eq!(d.conservados, vec![r"epk\commontext.epk".to_string()]);
        assert!(tocado.exists());
        assert_eq!(d.borrados.len(), 5);
    }

    #[test]
    fn instala_y_quita_archivos_con_ruta_propia() {
        let lad = tempfile::tempdir().unwrap();
        let ent = EntornoFalso::nuevo(lad.path());
        let p = crate::paquete::tests_util::paquete_con_imagen();
        let r = instalar(&p, &opciones(&ent.datos()), &ent, &mut |_| {}).unwrap();
        assert_eq!(r.archivos_escritos, 7, "6 .epk en 3 carpetas + 1 imagen en su ruta");
        let imagen = ent.datos().join(r"res\gui\textures\TEST\logo.webp");
        assert_eq!(std::fs::read(&imagen).unwrap(), b"imagen traducida");
        assert!(!ent.datos().join(r"epk\logo.webp").exists(), "la imagen no se copia a las carpetas epk");
        let e = estado::leer(&ent.carpeta_app()).unwrap();
        assert!(e.files.contains_key(r"res\gui\textures\TEST\logo.webp"));

        let d = desinstalar(&p, &ent, None, false, false, false, &mut |_| {}).unwrap();
        assert_eq!(d.borrados.len(), 7);
        assert!(!lad.path().join("fuzz").exists(), "quedaron carpetas: {:?}", archivos_en(lad.path()));
    }

    #[test]
    fn exige_confirmar_version_no_probada() {
        let lad = tempfile::tempdir().unwrap();
        let ent = EntornoFalso::nuevo(lad.path());
        let p = paquete_prueba();
        let juego = lad.path().join("juego");
        std::fs::create_dir_all(&juego).unwrap();
        std::fs::write(juego.join(deteccion::EXE_JUEGO), b"").unwrap();
        std::fs::write(juego.join("ver.dat"), "20990101_000000").unwrap();
        let mut o = opciones(&ent.datos());
        o.ruta_juego = Some(juego);
        assert!(matches!(instalar(&p, &o, &ent, &mut |_| {}), Err(Error::Validacion(_))));
        o.aceptar_version_no_probada = true;
        let r = instalar(&p, &o, &ent, &mut |_| {}).unwrap();
        assert_eq!(r.avisos.len(), 1);
    }
}
