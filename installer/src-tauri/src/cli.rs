//! Argumentos de línea de comandos (§6.8 del plan).
//!
//! Con `--silent` (o `--dry-run`) la acción se ejecuta sin ventana y el resultado sale por la
//! consola desde la que se lanzó. Sin `--silent`, `--uninstall`, `--repair` y `--update` abren la
//! interfaz en la pantalla de Mantenimiento.

use crate::candado::Candado;
use crate::estado::ModoRuta;
use crate::instalacion::{self, Entorno, Opciones, Progreso};
use crate::paquete::PaqueteActual;
use crate::sistema::EntornoSistema;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Accion {
    Asistente,
    Instalar,
    Desinstalar,
    Reparar,
    Actualizar,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Argumentos {
    pub accion: Accion,
    pub silencioso: bool,
    pub dry_run: bool,
    pub ruta_datos: Option<PathBuf>,
    pub ruta_juego: Option<PathBuf>,
    pub sin_copia: bool,
    pub auto_actualizar: bool,
    pub intervalo_horas: u32,
    pub acceso_directo: bool,
    pub aceptar_version: bool,
    pub forzar: bool,
    pub borrar_copia: bool,
    pub al_iniciar_sesion: bool,
}

pub const AYUDA: &str = "\
GalaxyzNeoES.exe [acción] [opciones]

Acciones:
  (ninguna)             Abre el asistente
  --install             Instala el parche (sin ventana)
  --uninstall           Desinstala el parche
  --repair              Vuelve a instalar con las opciones guardadas
  --update              Busca e instala actualizaciones

Opciones:
  --silent              Sin ventana; el resultado sale por la consola
  --dry-run             Simula: no escribe nada (implica --silent)
  --data-path <ruta>    Carpeta del parche (por defecto %LOCALAPPDATA%\\fuzz\\galaxyz\\data)
  --game-path <ruta>    Carpeta del juego
  --no-backup           No hacer copia de seguridad
  --auto-update         Activar la actualización automática
  --update-every <h>    Frecuencia en horas: 6, 12, 24 (por defecto) o 168
  --shortcut            Crear acceso directo en el menú Inicio
  --accept-untested     Instalar aunque la versión del juego no esté probada
  --force               --uninstall: borrar también archivos modificados
                        --update: comprobar aunque no toque o esté desactivada
  --delete-backup       Al desinstalar, borrar la copia de seguridad
";

pub fn analizar(args: impl IntoIterator<Item = String>) -> Result<Argumentos, String> {
    let mut a = Argumentos {
        accion: Accion::Asistente,
        silencioso: false,
        dry_run: false,
        ruta_datos: None,
        ruta_juego: None,
        sin_copia: false,
        auto_actualizar: false,
        intervalo_horas: crate::estado::INTERVALO_POR_DEFECTO,
        acceso_directo: false,
        aceptar_version: false,
        forzar: false,
        borrar_copia: false,
        al_iniciar_sesion: false,
    };
    let mut it = args.into_iter().skip(1);
    while let Some(arg) = it.next() {
        let mut valor = |nombre: &str| it.next().ok_or_else(|| format!("Falta el valor después de {nombre}"));
        match arg.to_lowercase().as_str() {
            "--install" => a.accion = Accion::Instalar,
            "--uninstall" => a.accion = Accion::Desinstalar,
            "--repair" => a.accion = Accion::Reparar,
            "--update" => a.accion = Accion::Actualizar,
            "--silent" => a.silencioso = true,
            "--dry-run" => {
                a.dry_run = true;
                a.silencioso = true;
            }
            "--data-path" => a.ruta_datos = Some(PathBuf::from(valor("--data-path")?)),
            "--game-path" => a.ruta_juego = Some(PathBuf::from(valor("--game-path")?)),
            "--no-backup" => a.sin_copia = true,
            "--auto-update" => a.auto_actualizar = true,
            "--update-every" => {
                let v = valor("--update-every")?;
                let horas: u32 = v.parse().map_err(|_| format!("Frecuencia no válida: {v}"))?;
                if !crate::estado::INTERVALOS_PERMITIDOS.contains(&horas) {
                    return Err(format!("La frecuencia tiene que ser 6, 12, 24 o 168 horas (no {horas})."));
                }
                a.intervalo_horas = horas;
            }
            "--shortcut" => a.acceso_directo = true,
            "--accept-untested" => a.aceptar_version = true,
            "--force" => a.forzar = true,
            "--delete-backup" => a.borrar_copia = true,
            "--at-login" => a.al_iniciar_sesion = true,
            "--help" | "-h" | "/?" => return Err(AYUDA.to_string()),
            // Argumentos que añade Tauri o Windows y no nos afectan.
            otro if !otro.starts_with("--") => {}
            otro => return Err(format!("Argumento desconocido: {otro}\n\n{AYUDA}")),
        }
    }
    // `--install` siempre es sin ventana: el asistente ya es la instalación con ventana.
    if a.accion == Accion::Instalar {
        a.silencioso = true;
    }
    Ok(a)
}

impl Argumentos {
    pub fn sin_ventana(&self) -> bool {
        self.silencioso && self.accion != Accion::Asistente
    }
}

/// Ejecuta la acción sin ventana. Devuelve el código de salida.
pub fn ejecutar(args: &Argumentos) -> i32 {
    let entorno = EntornoSistema;
    crate::log::escribir(&format!("CLI: {args:?}"));

    if args.accion == Accion::Actualizar {
        // Lo lanza el inicio de sesión o la tarea programada: no se engancha a ninguna consola.
        return crate::actualizacion::ejecutar_silencioso(&entorno, args.forzar, args.al_iniciar_sesion);
    }

    conectar_consola();
    let mut mostrar = |p: Progreso| {
        println!("[{:>3.0}%] {}", p.porcentaje, p.mensaje);
    };
    let carpeta = entorno.carpeta_app();
    let _candado = if args.dry_run {
        None
    } else {
        match Candado::tomar(&carpeta) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 1;
            }
        }
    };
    let actual = PaqueteActual::cargar(&carpeta);
    let paquete = actual.paquete();

    let resultado: Result<serde_json::Value, crate::error::Error> = match args.accion {
        Accion::Instalar => {
            let ruta_datos = args
                .ruta_datos
                .clone()
                .or_else(crate::deteccion::carpeta_datos_por_defecto);
            let Some(ruta_datos) = ruta_datos else {
                eprintln!("No se pudo leer LOCALAPPDATA. Indica la carpeta con --data-path.");
                return 1;
            };
            let ruta_juego = args
                .ruta_juego
                .clone()
                .or_else(|| crate::deteccion::detectar_juego(None).map(|j| j.ruta));
            let opciones = Opciones {
                modo_ruta: if args.ruta_datos.is_some() { ModoRuta::Manual } else { ModoRuta::Auto },
                ruta_datos,
                ruta_juego,
                copia_seguridad: !args.sin_copia,
                auto_actualizar: args.auto_actualizar,
                intervalo_actualizacion_horas: args.intervalo_horas,
                acceso_directo: args.acceso_directo,
                dry_run: args.dry_run,
                aceptar_version_no_probada: args.aceptar_version,
            };
            instalacion::instalar(&paquete, &opciones, &entorno, &mut mostrar).map(|r| json(&r))
        }
        Accion::Reparar => instalacion::reparar(&paquete, &entorno, args.dry_run, &mut mostrar).map(|r| json(&r)),
        Accion::Desinstalar => instalacion::desinstalar(
            &paquete,
            &entorno,
            args.ruta_datos.as_deref(),
            args.borrar_copia,
            args.forzar,
            args.dry_run,
            &mut mostrar,
        )
        .map(|r| json(&r)),
        Accion::Actualizar | Accion::Asistente => return 0,
    };

    match resultado {
        Ok(valor) => {
            println!("{}", serde_json::to_string_pretty(&valor).unwrap_or_default());
            crate::log::escribir("CLI: terminado correctamente");
            0
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            crate::log::escribir(&format!("CLI: error: {e}"));
            if matches!(e, crate::error::Error::JuegoAbierto) { 2 } else { 1 }
        }
    }
}

fn json<T: Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or_default()
}

/// En la versión final el programa no tiene consola propia; se engancha a la de quien lo lanzó.
#[cfg(windows)]
fn conectar_consola() {
    use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
fn conectar_consola() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Result<Argumentos, String> {
        analizar(std::iter::once("app").chain(v.iter().copied()).map(String::from))
    }

    #[test]
    fn analiza_argumentos() {
        assert_eq!(a(&[]).unwrap().accion, Accion::Asistente);
        let x = a(&["--install", "--data-path", r"D:\x", "--no-backup", "--update-every", "12"]).unwrap();
        assert!(x.sin_ventana() && x.sin_copia);
        assert_eq!(x.intervalo_horas, 12);
        assert_eq!(x.ruta_datos.as_deref(), Some(std::path::Path::new(r"D:\x")));
        assert!(!a(&["--uninstall"]).unwrap().sin_ventana());
        assert!(!a(&["--update"]).unwrap().sin_ventana(), "sin --silent abre Mantenimiento");
        assert!(a(&["--update", "--silent", "--at-login"]).unwrap().al_iniciar_sesion);
        assert!(a(&["--uninstall", "--dry-run"]).unwrap().sin_ventana());
        assert!(a(&["--data-path"]).is_err());
        assert!(a(&["--update-every", "5"]).is_err());
        assert!(a(&["--nada"]).is_err());
    }
}
