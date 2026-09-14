//! El `Entorno` real: procesos, `%LOCALAPPDATA%`, registro de Windows y acceso directo (§7 del plan).

use crate::deteccion;
use crate::estado::Estado;
use crate::instalacion::Entorno;
use std::path::PathBuf;

pub const NOMBRE_APP: &str = "GalaxyzNeoES";
pub const NOMBRE_VISIBLE: &str = "Parche GALAXYZ neo (Español Latino)";
const NOMBRE_ACCESO: &str = "Parche GALAXYZ neo";

pub struct EntornoSistema;

impl EntornoSistema {
    fn lad() -> PathBuf {
        deteccion::local_app_data().unwrap_or_else(std::env::temp_dir)
    }

    pub fn carpeta_programa() -> PathBuf {
        Self::lad().join("Programs").join(NOMBRE_APP)
    }

    /// En desarrollo no se registra el ejecutable de pruebas, salvo que se pida.
    fn registro_activo() -> bool {
        !cfg!(debug_assertions) || std::env::var_os("GALAXYZ_ES_REGISTRAR").is_some()
    }
}

impl Entorno for EntornoSistema {
    fn juego_abierto(&self) -> bool {
        // Solo en desarrollo: permite probar contra una LOCALAPPDATA falsa con el juego abierto.
        if cfg!(debug_assertions) && std::env::var_os("GALAXYZ_ES_IGNORAR_JUEGO").is_some() {
            return false;
        }
        deteccion::juego_abierto()
    }

    fn carpeta_app(&self) -> PathBuf {
        Self::lad().join(NOMBRE_APP)
    }

    fn local_app_data(&self) -> Option<PathBuf> {
        deteccion::local_app_data()
    }

    fn registrar_aplicacion(&self, estado: &Estado) -> Result<(), String> {
        if !Self::registro_activo() {
            crate::log::escribir("Desarrollo: se omite el registro de la aplicación.");
            return Ok(());
        }
        let destino_dir = Self::carpeta_programa();
        let destino = destino_dir.join(format!("{NOMBRE_APP}.exe"));
        let actual = std::env::current_exe().map_err(|e| e.to_string())?;
        if !deteccion::misma_ruta(&actual, &destino) {
            std::fs::create_dir_all(&destino_dir).map_err(|e| e.to_string())?;
            std::fs::copy(&actual, &destino).map_err(|e| format!("copiar el programa: {e}"))?;
        }
        // Cada paso por separado: que falle el acceso directo no debe dejar sin actualizaciones.
        let mut problemas = Vec::new();
        if let Err(e) = registrar_desinstalacion(&destino, &estado.patch_version) {
            problemas.push(e);
        }
        if let Err(e) = self.configurar_actualizacion(estado.auto_update, estado.update_interval_hours) {
            problemas.push(e);
        }
        if let Some(menu) = carpeta_menu_inicio() {
            let lnk = menu.join(format!("{NOMBRE_ACCESO}.lnk"));
            if estado.shortcut {
                let creado = std::fs::create_dir_all(&menu)
                    .map_err(|e| format!("acceso directo: {e}"))
                    .and_then(|_| crear_acceso_directo(&lnk, &destino));
                if let Err(e) = creado {
                    problemas.push(e);
                }
            } else {
                let _ = std::fs::remove_file(lnk);
            }
        }
        if problemas.is_empty() {
            Ok(())
        } else {
            Err(problemas.join("; "))
        }
    }

    fn configurar_actualizacion(&self, activar: bool, horas: u32) -> Result<(), String> {
        if !Self::registro_activo() {
            crate::log::escribir(&format!(
                "Desarrollo: se omite configurar la actualización automática ({activar}, cada {horas} h)."
            ));
            return Ok(());
        }
        if activar {
            let exe = Self::carpeta_programa().join(format!("{NOMBRE_APP}.exe"));
            crate::programador::activar(&exe, horas)
        } else {
            crate::programador::desactivar();
            Ok(())
        }
    }

    fn quitar_aplicacion(&self) -> Result<(), String> {
        if !Self::registro_activo() {
            return Ok(());
        }
        crate::programador::desactivar();
        quitar_desinstalacion();
        if let Some(d) = carpeta_menu_inicio() {
            let _ = std::fs::remove_file(d.join(format!("{NOMBRE_ACCESO}.lnk")));
        }
        crate::log::desactivar();
        let _ = std::fs::remove_dir_all(self.carpeta_app());
        borrar_programa_al_salir(&Self::carpeta_programa());
        Ok(())
    }
}

fn carpeta_menu_inicio() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|a| a.join(r"Microsoft\Windows\Start Menu\Programs"))
}

#[cfg(windows)]
const CLAVE_DESINSTALACION: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\GalaxyzNeoES";

#[cfg(windows)]
fn registrar_desinstalacion(exe: &std::path::Path, version: &str) -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let (k, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(CLAVE_DESINSTALACION)
        .map_err(|e| format!("registro: {e}"))?;
    let exe_txt = exe.display().to_string();
    let valores: [(&str, String); 7] = [
        ("DisplayName", NOMBRE_VISIBLE.to_string()),
        ("DisplayVersion", version.to_string()),
        ("Publisher", "Proyecto de fans".to_string()),
        ("DisplayIcon", exe_txt.clone()),
        ("InstallLocation", exe.parent().map(|p| p.display().to_string()).unwrap_or_default()),
        ("UninstallString", format!("\"{exe_txt}\" --uninstall")),
        ("URLInfoAbout", "https://github.com/NexusChaser/galaxyz-neo-espanol-latino".to_string()),
    ];
    for (nombre, valor) in valores {
        k.set_value(nombre, &valor).map_err(|e| format!("registro: {e}"))?;
    }
    k.set_value("NoModify", &1u32).map_err(|e| e.to_string())?;
    k.set_value("NoRepair", &1u32).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(windows)]
fn quitar_desinstalacion() {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(CLAVE_DESINSTALACION);
}

#[cfg(not(windows))]
fn registrar_desinstalacion(_: &std::path::Path, _: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
fn quitar_desinstalacion() {}

/// Crea el `.lnk` con WScript.Shell a través de PowerShell (sin ventana).
fn crear_acceso_directo(lnk: &std::path::Path, destino: &std::path::Path) -> Result<(), String> {
    let comillas = |p: &std::path::Path| p.display().to_string().replace('\'', "''");
    let script = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.WorkingDirectory='{}';$s.Save()",
        comillas(lnk),
        comillas(destino),
        destino.parent().map(comillas).unwrap_or_default()
    );
    let estado = comando_oculto("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("acceso directo: {e}"))?;
    estado
        .success()
        .then_some(())
        .ok_or_else(|| "no se pudo crear el acceso directo".to_string())
}

/// El programa no puede borrarse a sí mismo mientras corre: se encarga un `cmd` que espera.
fn borrar_programa_al_salir(carpeta: &std::path::Path) {
    let Ok(actual) = std::env::current_exe() else { return };
    if !actual.starts_with(carpeta) {
        let _ = std::fs::remove_dir_all(carpeta);
        return;
    }
    let orden = format!("ping -n 3 127.0.0.1 >nul & rmdir /s /q \"{}\"", carpeta.display());
    let _ = comando_oculto("cmd.exe").args(["/C", &orden]).spawn();
}

pub fn comando_oculto(programa: &str) -> std::process::Command {
    #[allow(unused_mut)]
    let mut c = std::process::Command::new(programa);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c
}
