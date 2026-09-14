//! Detección del juego y de la carpeta del parche (§6.1 y §6.2 del plan).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const EXE_JUEGO: &str = "galaxyz-win64vc14-release.exe";
const PREFIJO_NOMBRE: &str = "GALAXYZ neo";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Origen {
    Estado,
    Steam,
    Registro,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JuegoDetectado {
    pub nombre: String,
    pub app_id: Option<String>,
    pub ruta: PathBuf,
    pub version: Option<String>,
    pub origen: Origen,
}

/// `%LOCALAPPDATA%`, leído de la variable de entorno igual que hace el juego (§2.1).
pub fn local_app_data() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// `%LOCALAPPDATA%\fuzz\galaxyz\data`. Puede no existir todavía: es normal.
pub fn carpeta_datos_por_defecto() -> Option<PathBuf> {
    local_app_data().map(|l| l.join("fuzz").join("galaxyz").join("data"))
}

pub fn es_carpeta_juego(ruta: &Path) -> bool {
    ruta.join(EXE_JUEGO).is_file()
}

pub fn leer_version(ruta_juego: &Path) -> Option<String> {
    std::fs::read_to_string(ruta_juego.join("ver.dat"))
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Busca el juego: ruta guardada → Steam → registro de desinstalación.
pub fn detectar_juego(ruta_guardada: Option<&Path>) -> Option<JuegoDetectado> {
    if let Some(r) = ruta_guardada.filter(|r| es_carpeta_juego(r)) {
        return Some(JuegoDetectado {
            nombre: nombre_desde_ruta(r),
            app_id: None,
            ruta: r.to_path_buf(),
            version: leer_version(r),
            origen: Origen::Estado,
        });
    }
    let mut candidatos = Vec::new();
    if let Some(steam) = carpeta_steam() {
        for biblioteca in bibliotecas_steam(&steam) {
            candidatos.extend(juegos_en_biblioteca(&biblioteca));
        }
    }
    if candidatos.is_empty() {
        candidatos.extend(desde_registro());
    }
    elegir_mejor(candidatos)
}

/// Si hay versión completa y Demo, se prefiere la completa.
fn elegir_mejor(mut candidatos: Vec<JuegoDetectado>) -> Option<JuegoDetectado> {
    candidatos.retain(|j| es_carpeta_juego(&j.ruta));
    candidatos.sort_by_key(|j| j.nombre.to_lowercase().contains("demo"));
    candidatos.into_iter().next()
}

fn nombre_desde_ruta(ruta: &Path) -> String {
    ruta.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .filter(|n| n.to_lowercase().starts_with(&PREFIJO_NOMBRE.to_lowercase()))
        .unwrap_or_else(|| PREFIJO_NOMBRE.to_string())
}

#[cfg(windows)]
fn carpeta_steam() -> Option<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(k) = hkcu.open_subkey(r"Software\Valve\Steam") {
        if let Ok(p) = k.get_value::<String, _>("SteamPath") {
            let p = PathBuf::from(p.replace('/', "\\"));
            if p.is_dir() {
                return Some(p);
            }
        }
    }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    for sub in [r"SOFTWARE\WOW6432Node\Valve\Steam", r"SOFTWARE\Valve\Steam"] {
        if let Ok(k) = hklm.open_subkey(sub) {
            if let Ok(p) = k.get_value::<String, _>("InstallPath") {
                let p = PathBuf::from(p);
                if p.is_dir() {
                    return Some(p);
                }
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn carpeta_steam() -> Option<PathBuf> {
    None
}

/// Todas las bibliotecas de Steam (el juego puede estar en otro disco).
pub fn bibliotecas_steam(steam: &Path) -> Vec<PathBuf> {
    let mut libs = vec![steam.to_path_buf()];
    for vdf in [
        steam.join("steamapps").join("libraryfolders.vdf"),
        steam.join("config").join("libraryfolders.vdf"),
    ] {
        if let Ok(texto) = std::fs::read_to_string(&vdf) {
            for (clave, valor) in pares_vdf(&texto) {
                if clave.eq_ignore_ascii_case("path") {
                    let p = PathBuf::from(valor);
                    if !libs.iter().any(|l| misma_ruta(l, &p)) {
                        libs.push(p);
                    }
                }
            }
        }
    }
    libs
}

pub fn juegos_en_biblioteca(biblioteca: &Path) -> Vec<JuegoDetectado> {
    let steamapps = biblioteca.join("steamapps");
    let Ok(entradas) = std::fs::read_dir(&steamapps) else {
        return Vec::new();
    };
    let mut juegos = Vec::new();
    for entrada in entradas.flatten() {
        let nombre_archivo = entrada.file_name().to_string_lossy().to_lowercase();
        if !(nombre_archivo.starts_with("appmanifest_") && nombre_archivo.ends_with(".acf")) {
            continue;
        }
        let Ok(texto) = std::fs::read_to_string(entrada.path()) else {
            continue;
        };
        let pares = pares_vdf(&texto);
        let valor = |k: &str| {
            pares
                .iter()
                .find(|(c, _)| c.eq_ignore_ascii_case(k))
                .map(|(_, v)| v.clone())
        };
        let (Some(nombre), Some(carpeta)) = (valor("name"), valor("installdir")) else {
            continue;
        };
        if !nombre
            .to_lowercase()
            .starts_with(&PREFIJO_NOMBRE.to_lowercase())
        {
            continue;
        }
        let ruta = steamapps.join("common").join(carpeta);
        juegos.push(JuegoDetectado {
            version: leer_version(&ruta),
            nombre,
            app_id: valor("appid"),
            ruta,
            origen: Origen::Steam,
        });
    }
    juegos
}

#[cfg(windows)]
fn desde_registro() -> Vec<JuegoDetectado> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;
    let mut juegos = Vec::new();
    let raices = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];
    for (raiz, sub) in raices {
        let Ok(clave) = RegKey::predef(raiz).open_subkey(sub) else {
            continue;
        };
        for nombre_sub in clave.enum_keys().flatten() {
            let Ok(k) = clave.open_subkey(&nombre_sub) else {
                continue;
            };
            let nombre: String = k.get_value("DisplayName").unwrap_or_default();
            let editor: String = k.get_value("Publisher").unwrap_or_default();
            let es_juego = nombre
                .to_lowercase()
                .starts_with(&PREFIJO_NOMBRE.to_lowercase())
                || (editor.eq_ignore_ascii_case("fuzz, Inc.")
                    && nombre.to_lowercase().contains("galaxyz"));
            if !es_juego {
                continue;
            }
            let Ok(ruta) = k.get_value::<String, _>("InstallLocation") else {
                continue;
            };
            let ruta = PathBuf::from(ruta);
            juegos.push(JuegoDetectado {
                version: leer_version(&ruta),
                app_id: nombre_sub
                    .strip_prefix("Steam App ")
                    .map(|s| s.to_string()),
                nombre,
                ruta,
                origen: Origen::Registro,
            });
        }
    }
    juegos
}

#[cfg(not(windows))]
fn desde_registro() -> Vec<JuegoDetectado> {
    Vec::new()
}

/// Lee los pares `"clave" "valor"` de un archivo `.vdf`/`.acf` de Steam. Las llaves de
/// sección se ignoran; basta con los pares, porque las claves que usamos son únicas.
pub fn pares_vdf(texto: &str) -> Vec<(String, String)> {
    let mut pares = Vec::new();
    for linea in texto.lines() {
        let mut cadenas = Vec::new();
        let mut chars = linea.chars();
        while let Some(c) = chars.next() {
            if c != '"' {
                continue;
            }
            let mut s = String::new();
            while let Some(c) = chars.next() {
                match c {
                    '\\' => {
                        if let Some(sig) = chars.next() {
                            s.push(match sig {
                                'n' => '\n',
                                't' => '\t',
                                otro => otro,
                            });
                        }
                    }
                    '"' => break,
                    otro => s.push(otro),
                }
            }
            cadenas.push(s);
        }
        if cadenas.len() >= 2 {
            let mut it = cadenas.into_iter();
            pares.push((it.next().unwrap(), it.next().unwrap()));
        }
    }
    pares
}

/// Comparación de rutas sin distinguir mayúsculas ni separadores finales.
pub fn misma_ruta(a: &Path, b: &Path) -> bool {
    normalizar_texto(a) == normalizar_texto(b)
}

pub fn normalizar_texto(p: &Path) -> String {
    p.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

/// ¿Está abierto el juego?
pub fn juego_abierto() -> bool {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
    let mut sistema =
        System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()));
    sistema.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    sistema
        .processes()
        .values()
        .any(|p| p.name().eq_ignore_ascii_case(EXE_JUEGO))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_libraryfolders_y_appmanifest() {
        let vdf = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"apps" { "4306970" "1014011063" }
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
	}
}"#;
        let rutas: Vec<_> = pares_vdf(vdf)
            .into_iter()
            .filter(|(k, _)| k == "path")
            .map(|(_, v)| v)
            .collect();
        assert_eq!(rutas, vec![r"C:\Program Files (x86)\Steam", r"D:\SteamLibrary"]);

        let dir = tempfile::tempdir().unwrap();
        let steamapps = dir.path().join("steamapps");
        std::fs::create_dir_all(steamapps.join("common").join("GALAXYZ neo Demo")).unwrap();
        std::fs::write(
            steamapps.join("appmanifest_4306970.acf"),
            "\"AppState\"\n{\n\t\"appid\"\t\t\"4306970\"\n\t\"name\"\t\t\"GALAXYZ neo Demo\"\n\t\"installdir\"\t\t\"GALAXYZ neo Demo\"\n}\n",
        )
        .unwrap();
        std::fs::write(steamapps.join("appmanifest_1.acf"), "\"name\" \"Otro juego\"\n\"installdir\" \"x\"").unwrap();
        let juegos = juegos_en_biblioteca(dir.path());
        assert_eq!(juegos.len(), 1);
        assert_eq!(juegos[0].app_id.as_deref(), Some("4306970"));
        assert!(juegos[0].ruta.ends_with(r"common\GALAXYZ neo Demo"));
    }

    #[test]
    fn prefiere_la_version_completa() {
        let dir = tempfile::tempdir().unwrap();
        let mk = |n: &str| {
            let r = dir.path().join(n);
            std::fs::create_dir_all(&r).unwrap();
            std::fs::write(r.join(EXE_JUEGO), b"").unwrap();
            JuegoDetectado { nombre: n.into(), app_id: None, ruta: r, version: None, origen: Origen::Steam }
        };
        let elegido = elegir_mejor(vec![mk("GALAXYZ neo Demo"), mk("GALAXYZ neo")]).unwrap();
        assert_eq!(elegido.nombre, "GALAXYZ neo");
    }
}
