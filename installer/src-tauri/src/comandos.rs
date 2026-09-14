//! Comandos que la interfaz llama con `invoke(...)`.

use crate::actualizacion::{self, InfoActualizacion};
use crate::candado::Candado;
use crate::cli::Argumentos;
use crate::deteccion::{self, JuegoDetectado};
use crate::error::{Error, Resultado};
use crate::estado::{self, Estado};
use crate::instalacion::{self, Entorno, Opciones, Progreso, ResultadoDesinstalacion, ResultadoInstalacion};
use crate::paquete::{Manifest, PaqueteActual};
use crate::sistema::EntornoSistema;
use crate::validacion::{self, ValidacionDatos, ValidacionJuego};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

pub struct ArgumentosInicio(pub Argumentos);

/// Último manifiesto descargado, para no volver a pedirlo al pulsar «Actualizar».
#[derive(Default)]
pub struct UltimoManifest(pub Mutex<Option<Manifest>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoPaquete {
    version: String,
    version_instalador: String,
    archivos: usize,
    copias: usize,
    tamano_total: u64,
    tamano_instalado: u64,
    targets: Vec<String>,
    game_versions: Vec<String>,
    url_releases: String,
    url_reportar: String,
    intervalos: Vec<u32>,
    intervalo_por_defecto: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deteccion {
    juego: Option<JuegoDetectado>,
    validacion_juego: Option<ValidacionJuego>,
    ruta_datos: Option<PathBuf>,
    validacion_datos: Option<ValidacionDatos>,
    estado: Option<Estado>,
    archivos_danados: Vec<String>,
    actualizacion_programada: bool,
    juego_abierto: bool,
    local_app_data: Option<PathBuf>,
}

fn carpeta_app() -> PathBuf {
    EntornoSistema.carpeta_app()
}

#[tauri::command]
pub fn argumentos_inicio(args: State<'_, ArgumentosInicio>) -> Argumentos {
    args.0.clone()
}

#[tauri::command]
pub async fn info_paquete() -> Resultado<InfoPaquete> {
    en_segundo_plano(|| {
        let actual = PaqueteActual::cargar(&carpeta_app());
        let p = actual.paquete();
        Ok(InfoPaquete {
            version: p.version.clone(),
            version_instalador: actualizacion::VERSION_INSTALADOR.to_string(),
            archivos: p.archivos.len(),
            copias: p.destinos().len(),
            tamano_total: p.tamano_total(),
            tamano_instalado: p.tamano_instalado(),
            targets: p.targets.clone(),
            game_versions: p.game_versions.clone(),
            url_releases: crate::remoto::url_releases(),
            url_reportar: format!("https://github.com/{}/issues/new/choose", crate::remoto::REPO),
            intervalos: estado::INTERVALOS_PERMITIDOS.to_vec(),
            intervalo_por_defecto: estado::INTERVALO_POR_DEFECTO,
        })
    })
    .await
}

#[tauri::command]
pub async fn detectar() -> Resultado<Deteccion> {
    en_segundo_plano(|| {
        let entorno = EntornoSistema;
        let actual = PaqueteActual::cargar(&entorno.carpeta_app());
        let paquete = actual.paquete();
        let estado = estado::leer(&entorno.carpeta_app());
        let juego = deteccion::detectar_juego(estado.as_ref().and_then(|e| e.game_path.as_deref()));
        let validacion_juego = juego.as_ref().map(|j| validacion::validar_juego(&j.ruta, &paquete));
        let ruta_datos = estado
            .as_ref()
            .map(|e| e.data_path.clone())
            .or_else(deteccion::carpeta_datos_por_defecto);
        let lad = entorno.local_app_data();
        let validacion_datos = ruta_datos
            .as_ref()
            .map(|r| validacion::validar_datos(r, &paquete, lad.as_deref()));
        let archivos_danados = estado.as_ref().map(actualizacion::archivos_danados).unwrap_or_default();
        let (run, tarea) = crate::programador::comprobar();
        Ok(Deteccion {
            juego,
            validacion_juego,
            ruta_datos,
            validacion_datos,
            archivos_danados,
            actualizacion_programada: run || tarea,
            estado,
            juego_abierto: entorno.juego_abierto(),
            local_app_data: lad,
        })
    })
    .await
}

#[tauri::command]
pub async fn validar_juego(ruta: PathBuf) -> Resultado<ValidacionJuego> {
    en_segundo_plano(move || {
        let actual = PaqueteActual::cargar(&carpeta_app());
        Ok(validacion::validar_juego(&ruta, &actual.paquete()))
    })
    .await
}

#[tauri::command]
pub async fn validar_datos(ruta: PathBuf) -> Resultado<ValidacionDatos> {
    en_segundo_plano(move || {
        let actual = PaqueteActual::cargar(&carpeta_app());
        Ok(validacion::validar_datos(&ruta, &actual.paquete(), deteccion::local_app_data().as_deref()))
    })
    .await
}

#[tauri::command]
pub fn juego_abierto() -> bool {
    EntornoSistema.juego_abierto()
}

#[tauri::command]
pub fn abrir_juego(app: AppHandle, app_id: Option<String>, ruta: Option<PathBuf>) -> Resultado<()> {
    if let Some(id) = app_id.filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit())) {
        return app
            .opener()
            .open_url(format!("steam://rungameid/{id}"), None::<&str>)
            .map_err(|e| Error::Validacion(format!("No se pudo abrir Steam: {e}")));
    }
    let ruta = ruta
        .filter(|r| deteccion::es_carpeta_juego(r))
        .ok_or_else(|| Error::Validacion("No se encontró el juego para abrirlo.".into()))?;
    std::process::Command::new(ruta.join(deteccion::EXE_JUEGO))
        .current_dir(&ruta)
        .spawn()
        .map(|_| ())
        .map_err(|e| Error::Validacion(format!("No se pudo abrir el juego: {e}")))
}

#[tauri::command]
pub async fn instalar(app: AppHandle, opciones: Opciones) -> Resultado<ResultadoInstalacion> {
    en_segundo_plano(move || {
        crate::log::escribir(&format!("Instalar: {opciones:?}"));
        let _c = candado(&opciones)?;
        let actual = PaqueteActual::cargar(&carpeta_app());
        let r = instalacion::instalar(&actual.paquete(), &opciones, &EntornoSistema, &mut emisor(&app));
        registrar_resultado("Instalar", &r);
        r
    })
    .await
}

fn candado(opciones: &Opciones) -> Resultado<Option<Candado>> {
    if opciones.dry_run {
        Ok(None)
    } else {
        Candado::tomar(&carpeta_app()).map(Some)
    }
}

#[tauri::command]
pub async fn reparar(app: AppHandle, dry_run: bool) -> Resultado<ResultadoInstalacion> {
    en_segundo_plano(move || {
        let _c = Candado::tomar(&carpeta_app())?;
        let actual = PaqueteActual::cargar(&carpeta_app());
        let r = instalacion::reparar(&actual.paquete(), &EntornoSistema, dry_run, &mut emisor(&app));
        registrar_resultado("Reparar", &r);
        r
    })
    .await
}

#[tauri::command]
pub async fn desinstalar(
    app: AppHandle,
    borrar_backup: bool,
    forzar: bool,
    dry_run: bool,
) -> Resultado<ResultadoDesinstalacion> {
    en_segundo_plano(move || {
        let _c = Candado::tomar(&carpeta_app())?;
        let actual = PaqueteActual::cargar(&carpeta_app());
        let r = instalacion::desinstalar(
            &actual.paquete(),
            &EntornoSistema,
            None,
            borrar_backup,
            forzar,
            dry_run,
            &mut emisor(&app),
        );
        if r.is_ok() && !dry_run {
            let _ = std::fs::remove_dir_all(crate::paquete::carpeta_cache(&carpeta_app()));
        }
        registrar_resultado("Desinstalar", &r);
        r
    })
    .await
}

#[tauri::command]
pub async fn buscar_actualizacion(ultimo: State<'_, UltimoManifest>) -> Resultado<InfoActualizacion> {
    let (info, manifest) = en_segundo_plano(|| actualizacion::buscar(&EntornoSistema)).await?;
    *ultimo.0.lock().unwrap() = Some(manifest);
    Ok(info)
}

#[tauri::command]
pub async fn aplicar_actualizacion(
    app: AppHandle,
    ultimo: State<'_, UltimoManifest>,
) -> Resultado<ResultadoInstalacion> {
    let manifest = ultimo.0.lock().unwrap().clone();
    en_segundo_plano(move || {
        let manifest = match manifest {
            Some(m) => m,
            None => crate::remoto::obtener_manifest()?,
        };
        let r = actualizacion::aplicar(&EntornoSistema, manifest, &mut emisor(&app));
        registrar_resultado("Actualizar parche", &r);
        r
    })
    .await
}

#[tauri::command]
pub async fn actualizar_instalador(app: AppHandle, ultimo: State<'_, UltimoManifest>) -> Resultado<String> {
    let manifest = ultimo.0.lock().unwrap().clone();
    let version = en_segundo_plano({
        let app = app.clone();
        move || {
            let manifest = match manifest {
                Some(m) => m,
                None => crate::remoto::obtener_manifest()?,
            };
            actualizacion::actualizar_instalador(&manifest, &mut emisor(&app))
        }
    })
    .await?;
    // Reinicia desde el ejecutable nuevo.
    let exe = crate::sistema::EntornoSistema::carpeta_programa().join(format!("{}.exe", crate::sistema::NOMBRE_APP));
    let _ = std::process::Command::new(exe).spawn();
    app.exit(0);
    Ok(version)
}

#[tauri::command]
pub async fn cambiar_opciones(auto_update: bool, intervalo_horas: u32) -> Resultado<Estado> {
    en_segundo_plano(move || actualizacion::cambiar_opciones(&EntornoSistema, auto_update, intervalo_horas)).await
}

#[tauri::command]
pub fn abrir_url(app: AppHandle, url: String) -> Resultado<()> {
    if !url.starts_with("https://github.com/") {
        return Err(Error::Validacion("Solo se abren enlaces del repositorio.".into()));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| Error::Validacion(e.to_string()))
}

fn emisor(app: &AppHandle) -> impl FnMut(Progreso) + '_ {
    move |p| {
        let _ = app.emit("progreso", p);
    }
}

fn registrar_resultado<T>(accion: &str, r: &Resultado<T>) {
    match r {
        Ok(_) => crate::log::escribir(&format!("{accion}: correcto")),
        Err(e) => crate::log::escribir(&format!("{accion}: error: {e}")),
    }
}

/// Las operaciones de disco y red no deben bloquear la ventana.
async fn en_segundo_plano<T: Send + 'static>(
    f: impl FnOnce() -> Resultado<T> + Send + 'static,
) -> Resultado<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| Error::Validacion(format!("Error interno: {e}")))?
}
