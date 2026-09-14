//! Registro en `%LOCALAPPDATA%\GalaxyzNeoES\logs\AAAA-MM-DD.log`; se guardan los 10 más recientes.

use std::io::Write;
use std::path::PathBuf;

const MAXIMO_ARCHIVOS: usize = 10;

static DESACTIVADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Tras desinstalar no se vuelve a crear la carpeta de registros.
pub fn desactivar() {
    DESACTIVADO.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn carpeta() -> Option<PathBuf> {
    crate::deteccion::local_app_data().map(|l| l.join(crate::sistema::NOMBRE_APP).join("logs"))
}

pub fn escribir(mensaje: &str) {
    let ahora = chrono::Local::now();
    let linea = format!("{} {mensaje}", ahora.format("%H:%M:%S"));
    if cfg!(debug_assertions) {
        eprintln!("{linea}");
    }
    if DESACTIVADO.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let Some(dir) = carpeta() else { return };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let ruta = dir.join(format!("{}.log", ahora.format("%Y-%m-%d")));
    let nuevo = !ruta.exists();
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&ruta) {
        let _ = writeln!(f, "{linea}");
    }
    if nuevo {
        rotar(&dir);
    }
}

fn rotar(dir: &std::path::Path) {
    let Ok(it) = std::fs::read_dir(dir) else { return };
    let mut logs: Vec<PathBuf> = it
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "log"))
        .collect();
    logs.sort();
    let sobran = logs.len().saturating_sub(MAXIMO_ARCHIVOS);
    for viejo in &logs[..sobran] {
        let _ = std::fs::remove_file(viejo);
    }
}
