//! Descargas desde GitHub Releases (§6.6 y §8 del plan).

use crate::error::{Error, Resultado};
use crate::hash::sha256_bytes;
use crate::paquete::Manifest;
use std::io::Read;
use std::time::Duration;

pub const REPO: &str = "NexusChaser/galaxyz-neo-espanol-latino";
const LIMITE_DESCARGA: u64 = 512 * 1024 * 1024;

/// `…/releases/latest/download/manifest.json`: esta URL no tiene el límite de la API de GitHub.
/// En desarrollo se puede apuntar a un servidor local con `GALAXYZ_ES_URL_MANIFEST`.
pub fn url_manifest() -> String {
    if cfg!(debug_assertions) {
        if let Ok(url) = std::env::var("GALAXYZ_ES_URL_MANIFEST") {
            return url;
        }
    }
    format!("https://github.com/{REPO}/releases/latest/download/manifest.json")
}

pub fn url_releases() -> String {
    format!("https://github.com/{REPO}/releases/latest")
}

/// Las URL del manifiesto pueden ser relativas a él (`GALAXYZ-neo-Espanol-Latino.zip`).
pub fn resolver(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    let base = url_manifest();
    match base.rsplit_once('/') {
        Some((carpeta, _)) => format!("{carpeta}/{}", url.trim_start_matches('/')),
        None => url.to_string(),
    }
}

fn agente() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_global(Some(Duration::from_secs(30 * 60)))
        .user_agent(format!("GalaxyzNeoES/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

fn error_red(e: ureq::Error) -> Error {
    match e {
        ureq::Error::StatusCode(404) => Error::Red("todavía no hay ninguna versión publicada con manifiesto (404).".into()),
        ureq::Error::StatusCode(c) => Error::Red(format!("el servidor respondió {c}.")),
        otro => Error::Red(otro.to_string()),
    }
}

pub fn obtener_manifest() -> Resultado<Manifest> {
    let url = url_manifest();
    let mut resp = agente().get(&url).call().map_err(error_red)?;
    let texto = resp
        .body_mut()
        .with_config()
        .limit(4 * 1024 * 1024)
        .read_to_string()
        .map_err(error_red)?;
    serde_json::from_str(&texto).map_err(|e| Error::Red(format!("el manifiesto no es válido: {e}")))
}

/// Descarga `url` y comprueba su SHA-256. `progreso(recibido, total)`.
pub fn descargar(url: &str, sha256: &str, progreso: &mut dyn FnMut(u64, Option<u64>)) -> Resultado<Vec<u8>> {
    let url = resolver(url);
    let mut resp = agente().get(&url).call().map_err(error_red)?;
    let total = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    let mut lector = resp.body_mut().with_config().limit(LIMITE_DESCARGA).reader();
    let mut datos = Vec::with_capacity(total.unwrap_or(0) as usize);
    let mut buf = vec![0u8; 1 << 16];
    let mut ultimo = 0u64;
    loop {
        let n = lector.read(&mut buf).map_err(|e| Error::Red(format!("se cortó la descarga: {e}")))?;
        if n == 0 {
            break;
        }
        datos.extend_from_slice(&buf[..n]);
        let recibido = datos.len() as u64;
        if recibido - ultimo >= 256 * 1024 {
            ultimo = recibido;
            progreso(recibido, total);
        }
    }
    progreso(datos.len() as u64, total);
    if !sha256.is_empty() && sha256_bytes(&datos) != sha256.to_lowercase() {
        return Err(Error::PaqueteCorrupto(format!("{url} (la descarga está dañada)")));
    }
    Ok(datos)
}
