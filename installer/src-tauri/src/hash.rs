use crate::error::{io, Resultado};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn sha256_bytes(datos: &[u8]) -> String {
    hex::encode(Sha256::digest(datos))
}

pub fn sha256_archivo(ruta: &Path) -> Resultado<String> {
    let mut archivo = std::fs::File::open(ruta).map_err(io("leer", ruta))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = archivo.read(&mut buf).map_err(io("leer", ruta))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}
