//! El parche que se instala (§8 del plan).
//!
//! Hay dos fuentes: el paquete **incluido** en el ejecutable y el **descargado** por la
//! actualización automática, que se guarda en `%LOCALAPPDATA%\GalaxyzNeoES\paquete\`. Se usa
//! siempre la versión más nueva de las dos, así «Reparar» no vuelve a una versión antigua.
//!
//! Dos clases de archivo:
//! - **Sin ruta** (los `.epk` de `epk/`): se copian a cada carpeta de `targets`.
//! - **Con ruta** (lo de `extra/`, p. ej. imágenes traducidas): van a esa ruta exacta dentro de `data\`.

use crate::error::{io, Error, Resultado};
use crate::hash::sha256_bytes;
use crate::version::es_mayor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct ArchivoPayload {
    pub nombre: &'static str,
    pub ruta: Option<&'static str>,
    pub sha256: &'static str,
    pub datos: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/payload.rs"));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivoManifest {
    pub name: String,
    /// Ruta dentro de `data\` (con `\`). Si falta, el archivo va a cada carpeta de `targets`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub size: u64,
    pub sha256: String,
}

impl ArchivoManifest {
    pub fn id(&self) -> String {
        id_archivo(&self.name, self.path.as_deref())
    }
}

/// Identificador único de un archivo del parche: su ruta si la tiene, si no su nombre.
pub fn id_archivo(nombre: &str, ruta: Option<&str>) -> String {
    ruta.map(normalizar_ruta).unwrap_or_else(|| nombre.to_string())
}

/// `res/gui\x.webp` → `res\gui\x.webp`
pub fn normalizar_ruta(r: &str) -> String {
    r.replace('/', "\\").trim_matches('\\').to_string()
}

/// Rutas relativas seguras: sin `..`, sin unidad y sin raíz.
pub fn ruta_segura(r: &str) -> bool {
    let p = Path::new(r);
    !r.is_empty() && p.components().all(|c| matches!(c, std::path::Component::Normal(_)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Descarga {
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstaladorRemoto {
    pub version: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: String,
    #[serde(default)]
    pub released: Option<String>,
    pub game_versions: Vec<String>,
    /// Subcarpetas de `data\` donde se copian los archivos.
    pub targets: Vec<String>,
    pub files: Vec<ArchivoManifest>,
    /// Hashes de versiones anteriores del parche: versión → (archivo → sha256).
    #[serde(default)]
    pub known_patch_hashes: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    pub zip: Option<Descarga>,
    #[serde(default)]
    pub installer: Option<InstaladorRemoto>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Un archivo listo para escribir.
pub struct ArchivoPaquete<'a> {
    pub nombre: String,
    /// Ruta fija dentro de `data\`; `None` para los que van a cada carpeta de `targets`.
    pub ruta: Option<String>,
    pub sha256: String,
    pub datos: &'a [u8],
}

impl ArchivoPaquete<'_> {
    pub fn id(&self) -> String {
        id_archivo(&self.nombre, self.ruta.as_deref())
    }
}

/// Todo lo que el instalador necesita saber del parche. Se separa de dónde vienen los bytes
/// para poder instalar desde el ejecutable, desde la caché o desde paquetes de prueba.
pub struct Paquete<'a> {
    pub version: String,
    pub game_versions: Vec<String>,
    pub targets: Vec<String>,
    pub archivos: Vec<ArchivoPaquete<'a>>,
    pub conocidos: BTreeMap<String, BTreeMap<String, String>>,
}

pub fn manifest_incluido() -> &'static Manifest {
    static M: OnceLock<Manifest> = OnceLock::new();
    M.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/manifest.json")))
            .expect("manifest.json incluido no válido")
    })
}

impl Paquete<'static> {
    /// El parche incluido en este ejecutable.
    pub fn incluido() -> Self {
        let m = manifest_incluido();
        Paquete {
            version: m.version.clone(),
            game_versions: m.game_versions.clone(),
            targets: m.targets.clone(),
            archivos: ARCHIVOS
                .iter()
                .map(|a| ArchivoPaquete {
                    nombre: a.nombre.to_string(),
                    ruta: a.ruta.map(str::to_string),
                    sha256: a.sha256.to_string(),
                    datos: a.datos,
                })
                .collect(),
            conocidos: m.known_patch_hashes.clone(),
        }
    }
}

impl Paquete<'_> {
    /// Cada copia a escribir: (ruta relativa dentro de `data\`, archivo).
    pub fn destinos(&self) -> Vec<(PathBuf, &ArchivoPaquete<'_>)> {
        let mut v = Vec::new();
        for a in &self.archivos {
            match &a.ruta {
                Some(r) => v.push((PathBuf::from(r), a)),
                None => {
                    for t in &self.targets {
                        v.push((Path::new(t).join(&a.nombre), a));
                    }
                }
            }
        }
        v
    }

    /// Bytes que ocupa el parche instalado (con las copias en cada carpeta).
    pub fn tamano_instalado(&self) -> u64 {
        self.destinos().iter().map(|(_, a)| a.datos.len() as u64).sum()
    }

    pub fn tamano_total(&self) -> u64 {
        self.archivos.iter().map(|a| a.datos.len() as u64).sum()
    }

    /// Identificador del archivo que corresponde a una ruta instalada (`epk\x.epk` → `x.epk`).
    pub fn id_de_ruta(&self, relativa: &Path) -> String {
        let texto = normalizar_ruta(&relativa.to_string_lossy());
        if let Some((_, a)) = self
            .destinos()
            .into_iter()
            .find(|(r, _)| r.to_string_lossy().eq_ignore_ascii_case(&texto))
        {
            return a.id();
        }
        relativa
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or(texto)
    }

    /// Versión del parche a la que pertenece un archivo con este hash, si es del parche.
    pub fn version_del_hash(&self, id: &str, sha: &str) -> Option<String> {
        if self.archivos.iter().any(|a| a.id().eq_ignore_ascii_case(id) && a.sha256 == sha) {
            return Some(self.version.clone());
        }
        self.conocidos
            .iter()
            .rev()
            .find(|(_, archivos)| archivos.iter().any(|(n, h)| n.eq_ignore_ascii_case(id) && h == sha))
            .map(|(v, _)| v.clone())
    }

    /// Comprueba que los bytes coinciden con los hashes declarados.
    pub fn verificar(&self) -> Resultado<()> {
        for a in &self.archivos {
            if sha256_bytes(a.datos) != a.sha256 {
                return Err(Error::PaqueteCorrupto(a.nombre.clone()));
            }
        }
        Ok(())
    }

    pub fn es_compatible(&self, version_juego: &str) -> bool {
        self.game_versions.iter().any(|v| v == version_juego)
    }
}

/// Un paquete con los bytes en memoria (descargado o leído de la caché).
pub struct PaqueteDescargado {
    pub manifest: Manifest,
    /// (entrada del manifiesto, bytes)
    pub archivos: Vec<(ArchivoManifest, Vec<u8>)>,
}

impl PaqueteDescargado {
    /// Construye el paquete comprobando que están todos los archivos del manifiesto con su hash.
    /// `bytes` va indexado por `id` en minúsculas.
    pub fn nuevo(manifest: Manifest, mut bytes: BTreeMap<String, Vec<u8>>) -> Resultado<Self> {
        let mut archivos = Vec::new();
        for f in &manifest.files {
            if let Some(r) = &f.path {
                if !ruta_segura(&normalizar_ruta(r)) {
                    return Err(Error::PaqueteCorrupto(format!("{r} (ruta no permitida)")));
                }
            }
            let datos = bytes
                .remove(&f.id().to_lowercase())
                .ok_or_else(|| Error::PaqueteCorrupto(format!("{} (falta en la descarga)", f.id())))?;
            if sha256_bytes(&datos) != f.sha256 {
                return Err(Error::PaqueteCorrupto(f.id()));
            }
            archivos.push((f.clone(), datos));
        }
        Ok(Self { manifest, archivos })
    }

    pub fn paquete(&self) -> Paquete<'_> {
        let mut conocidos = self.manifest.known_patch_hashes.clone();
        // Que se reconozcan también los archivos de la versión incluida en el ejecutable.
        let incluido = manifest_incluido();
        conocidos
            .entry(incluido.version.clone())
            .or_insert_with(|| incluido.files.iter().map(|f| (f.id(), f.sha256.clone())).collect());
        Paquete {
            version: self.manifest.version.clone(),
            game_versions: self.manifest.game_versions.clone(),
            targets: self.manifest.targets.clone(),
            archivos: self
                .archivos
                .iter()
                .map(|(f, d)| ArchivoPaquete {
                    nombre: f.name.clone(),
                    ruta: f.path.as_deref().map(normalizar_ruta),
                    sha256: f.sha256.clone(),
                    datos: d,
                })
                .collect(),
            conocidos,
        }
    }

    pub fn guardar_cache(&self, carpeta_app: &Path) -> Resultado<()> {
        let final_dir = carpeta_cache(carpeta_app);
        let tmp = carpeta_app.join("paquete.nuevo");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).map_err(io("crear", &tmp))?;
        for (f, datos) in &self.archivos {
            let r = tmp.join(f.id());
            if let Some(padre) = r.parent() {
                std::fs::create_dir_all(padre).map_err(io("crear", padre))?;
            }
            std::fs::write(&r, datos).map_err(io("guardar", &r))?;
        }
        let m = tmp.join("manifest.json");
        std::fs::write(&m, serde_json::to_vec_pretty(&self.manifest).expect("serializa"))
            .map_err(io("guardar", &m))?;
        let _ = std::fs::remove_dir_all(&final_dir);
        std::fs::rename(&tmp, &final_dir).map_err(io("guardar", &final_dir))
    }

    pub fn leer_cache(carpeta_app: &Path) -> Option<Self> {
        let dir = carpeta_cache(carpeta_app);
        let manifest: Manifest =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).ok()?).ok()?;
        let mut bytes = BTreeMap::new();
        for f in &manifest.files {
            let id = f.id();
            if !ruta_segura(&id) {
                return None;
            }
            bytes.insert(id.to_lowercase(), std::fs::read(dir.join(&id)).ok()?);
        }
        Self::nuevo(manifest, bytes).ok()
    }
}

pub fn carpeta_cache(carpeta_app: &Path) -> PathBuf {
    carpeta_app.join("paquete")
}

/// El paquete más nuevo disponible: la caché si es más nueva que el incluido.
pub enum PaqueteActual {
    Incluido(Paquete<'static>),
    Descargado(PaqueteDescargado),
}

impl PaqueteActual {
    pub fn cargar(carpeta_app: &Path) -> Self {
        match PaqueteDescargado::leer_cache(carpeta_app) {
            Some(c) if es_mayor(&c.manifest.version, &manifest_incluido().version) => Self::Descargado(c),
            _ => Self::Incluido(Paquete::incluido()),
        }
    }

    pub fn paquete(&self) -> Paquete<'_> {
        match self {
            Self::Incluido(p) => Paquete {
                version: p.version.clone(),
                game_versions: p.game_versions.clone(),
                targets: p.targets.clone(),
                archivos: p
                    .archivos
                    .iter()
                    .map(|a| ArchivoPaquete {
                        nombre: a.nombre.clone(),
                        ruta: a.ruta.clone(),
                        sha256: a.sha256.clone(),
                        datos: a.datos,
                    })
                    .collect(),
                conocidos: p.conocidos.clone(),
            },
            Self::Descargado(d) => d.paquete(),
        }
    }
}

#[cfg(test)]
pub mod tests_util {
    use super::*;

    /// Paquete pequeño con dos archivos y las mismas carpetas de destino que el real.
    pub fn paquete_prueba() -> Paquete<'static> {
        let archivo = |nombre: &str, contenido: &'static [u8]| ArchivoPaquete {
            nombre: nombre.to_string(),
            ruta: None,
            sha256: sha256_bytes(contenido),
            datos: contenido,
        };
        Paquete {
            version: "1.0.0".into(),
            game_versions: vec!["20260210_102029".into()],
            targets: vec![r"locale\us\epk".into(), "epk".into(), r"root\epk".into()],
            archivos: vec![
                archivo("scene_data.epk", b"escenas en espanol"),
                archivo("commontext.epk", b"textos comunes"),
            ],
            conocidos: BTreeMap::new(),
        }
    }

    /// Como `paquete_prueba`, más una imagen traducida con ruta propia.
    pub fn paquete_con_imagen() -> Paquete<'static> {
        let mut p = paquete_prueba();
        let datos: &'static [u8] = b"imagen traducida";
        p.archivos.push(ArchivoPaquete {
            nombre: "logo.webp".into(),
            ruta: Some(r"res\gui\textures\TEST\logo.webp".into()),
            sha256: sha256_bytes(datos),
            datos,
        });
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_paquete_incluido_es_coherente() {
        let p = Paquete::incluido();
        assert_eq!(p.archivos.len(), manifest_incluido().files.len());
        assert!(p.archivos.iter().any(|a| a.nombre == "scene_data.epk"));
        p.verificar().unwrap();
        let a = &p.archivos[0];
        assert_eq!(p.version_del_hash(&a.nombre, &a.sha256), Some(p.version.clone()));
        assert_eq!(p.version_del_hash(&a.nombre, "00"), None);
    }

    #[test]
    fn la_cache_mas_nueva_sustituye_al_incluido() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(PaqueteActual::cargar(dir.path()), PaqueteActual::Incluido(_)));

        let datos = b"traduccion nueva".to_vec();
        let manifest = Manifest {
            version: "99.0.0".into(),
            released: None,
            game_versions: vec!["x".into()],
            targets: vec!["epk".into()],
            files: vec![ArchivoManifest { name: "scene_data.epk".into(), path: None, size: datos.len() as u64, sha256: sha256_bytes(&datos) }],
            known_patch_hashes: BTreeMap::new(),
            zip: None,
            installer: None,
            notes: None,
        };
        let d = PaqueteDescargado::nuevo(manifest, BTreeMap::from([("scene_data.epk".to_string(), datos)])).unwrap();
        d.guardar_cache(dir.path()).unwrap();
        let actual = PaqueteActual::cargar(dir.path());
        let p = actual.paquete();
        assert_eq!(p.version, "99.0.0");
        // Reconoce los archivos de la versión incluida como del parche.
        let incluido = Paquete::incluido();
        let a = &incluido.archivos[0];
        assert_eq!(p.version_del_hash(&a.nombre, &a.sha256), Some(incluido.version.clone()));

        // Una caché dañada se ignora.
        std::fs::write(carpeta_cache(dir.path()).join("scene_data.epk"), b"roto").unwrap();
        assert!(matches!(PaqueteActual::cargar(dir.path()), PaqueteActual::Incluido(_)));
    }
}
