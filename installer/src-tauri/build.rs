//! Genera en tiempo de compilación el paquete del parche que va dentro del ejecutable:
//! - `payload.rs`: la lista de archivos con su ruta, SHA-256 y bytes (`include_bytes!`).
//! - `manifest.json`: `payload/manifest.base.json` + la lista de archivos con sus hashes.
//!
//! Fuentes (rutas relativas a la raíz del repositorio):
//! - `epk/*.epk` (o `GALAXYZ_PAYLOAD_DIR`): se copian a cada carpeta de `targets`.
//! - `extra/**` (o `GALAXYZ_EXTRA_DIR`), opcional: cada archivo va a la misma ruta dentro de
//!   `data\`. Ahí van, por ejemplo, las imágenes traducidas: `extra/res/gui/textures/TEST/logo.webp`.

use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn listar(dir: &Path, salida: &mut Vec<PathBuf>) {
    let Ok(it) = std::fs::read_dir(dir) else { return };
    for e in it.flatten() {
        let p = e.path();
        if p.is_dir() {
            listar(&p, salida);
        } else if p.file_name().is_some_and(|n| n != ".gitkeep" && n != "LEEME.md") {
            salida.push(p);
        }
    }
}

fn main() {
    let raiz = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let salida = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let repo = raiz.join("..").join("..");

    let carpeta = std::env::var("GALAXYZ_PAYLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo.join("epk"));
    let carpeta = carpeta
        .canonicalize()
        .unwrap_or_else(|_| panic!("No existe la carpeta del parche: {}", carpeta.display()));
    let extra = std::env::var("GALAXYZ_EXTRA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo.join("extra"));
    let base_ruta = raiz.join("payload").join("manifest.base.json");

    println!("cargo:rerun-if-env-changed=GALAXYZ_PAYLOAD_DIR");
    println!("cargo:rerun-if-env-changed=GALAXYZ_EXTRA_DIR");
    println!("cargo:rerun-if-changed={}", carpeta.display());
    println!("cargo:rerun-if-changed={}", extra.display());
    println!("cargo:rerun-if-changed={}", base_ruta.display());

    let mut epk: Vec<PathBuf> = std::fs::read_dir(&carpeta)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("epk")))
        .collect();
    epk.sort();
    assert!(!epk.is_empty(), "No hay .epk en {}", carpeta.display());

    let mut extras = Vec::new();
    if extra.is_dir() {
        listar(&extra, &mut extras);
        extras.sort();
    }
    let extra = extra.canonicalize().unwrap_or(extra);

    let mut codigo = String::from("pub static ARCHIVOS: &[ArchivoPayload] = &[\n");
    let mut lista = Vec::new();
    let entradas = epk
        .iter()
        .map(|p| (p.clone(), None))
        .chain(extras.iter().map(|p| {
            let p = p.canonicalize().unwrap();
            let rel = p.strip_prefix(&extra).unwrap().to_string_lossy().replace('/', "\\");
            (p, Some(rel))
        }));
    for (ruta, relativa) in entradas {
        println!("cargo:rerun-if-changed={}", ruta.display());
        let datos = std::fs::read(&ruta).unwrap();
        let sha = hex::encode(Sha256::digest(&datos));
        let nombre = ruta.file_name().unwrap().to_string_lossy().to_string();
        let ruta_str = ruta.to_string_lossy().to_string();
        writeln!(
            codigo,
            "    ArchivoPayload {{ nombre: {nombre:?}, ruta: {relativa:?}, sha256: {sha:?}, datos: include_bytes!({ruta_str:?}) }},"
        )
        .unwrap();
        let mut entrada = serde_json::json!({ "name": nombre, "size": datos.len(), "sha256": sha });
        if let Some(r) = relativa {
            entrada["path"] = serde_json::Value::String(r);
        }
        lista.push(entrada);
    }
    codigo.push_str("];\n");
    std::fs::write(salida.join("payload.rs"), codigo).unwrap();

    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&base_ruta).unwrap()).unwrap();
    manifest["files"] = serde_json::Value::Array(lista);
    std::fs::write(salida.join("manifest.json"), serde_json::to_string_pretty(&manifest).unwrap()).unwrap();

    tauri_build::build()
}
