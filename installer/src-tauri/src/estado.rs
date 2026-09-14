//! `state.json`: lo que se instaló y con qué opciones (§7 del plan).

use crate::error::{io, Resultado};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModoRuta {
    #[default]
    Auto,
    Manual,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estado {
    pub patch_version: String,
    pub data_path: PathBuf,
    pub path_mode: ModoRuta,
    pub game_path: Option<PathBuf>,
    pub game_version: Option<String>,
    pub backup: bool,
    pub backup_path: Option<PathBuf>,
    pub auto_update: bool,
    /// Cada cuántas horas se buscan actualizaciones (§6.6). Por defecto, una vez al día.
    #[serde(default = "intervalo_por_defecto")]
    pub update_interval_hours: u32,
    pub shortcut: bool,
    pub installed_at: String,
    pub last_check: Option<String>,
    /// Resultado de la última comprobación, para mostrarlo en Mantenimiento.
    #[serde(default)]
    pub last_check_result: Option<String>,
    /// Versión del juego que no soporta el parche instalado (se detectó al comprobar).
    #[serde(default)]
    pub unsupported_game_version: Option<String>,
    pub targets: Vec<String>,
    /// Archivo → sha256 instalado.
    pub files: BTreeMap<String, String>,
    /// Carpetas que creó el instalador (para quitarlas al desinstalar si quedan vacías).
    #[serde(default)]
    pub created_dirs: Vec<PathBuf>,
}

pub const INTERVALO_POR_DEFECTO: u32 = 24;
/// Frecuencias que ofrece la interfaz, en horas.
pub const INTERVALOS_PERMITIDOS: [u32; 4] = [6, 12, 24, 168];

fn intervalo_por_defecto() -> u32 {
    INTERVALO_POR_DEFECTO
}

pub fn intervalo_valido(horas: u32) -> u32 {
    if INTERVALOS_PERMITIDOS.contains(&horas) {
        horas
    } else {
        INTERVALO_POR_DEFECTO
    }
}

pub fn ruta_estado(carpeta_app: &Path) -> PathBuf {
    carpeta_app.join("state.json")
}

pub fn leer(carpeta_app: &Path) -> Option<Estado> {
    let texto = std::fs::read_to_string(ruta_estado(carpeta_app)).ok()?;
    serde_json::from_str(&texto).ok()
}

/// Escribe en un temporal y renombra, para no dejar nunca un `state.json` a medias.
pub fn guardar(carpeta_app: &Path, estado: &Estado) -> Resultado<()> {
    std::fs::create_dir_all(carpeta_app).map_err(io("crear", carpeta_app))?;
    let ruta = ruta_estado(carpeta_app);
    let tmp = ruta.with_extension("json.tmp");
    let texto = serde_json::to_string_pretty(estado).expect("Estado siempre serializa");
    std::fs::write(&tmp, texto).map_err(io("escribir", &tmp))?;
    std::fs::rename(&tmp, &ruta).map_err(io("guardar", &ruta))
}

pub fn borrar(carpeta_app: &Path) -> Resultado<()> {
    let ruta = ruta_estado(carpeta_app);
    match std::fs::remove_file(&ruta) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io("borrar", &ruta)(e)),
        _ => Ok(()),
    }
}
