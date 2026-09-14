//! Arranque de la comprobación de actualizaciones (§6.6 del plan), sin permisos de administrador:
//!
//! - Valor `Run` de `HKCU`: comprueba al iniciar sesión.
//! - Tarea programada del usuario con un disparador que se repite cada N horas y
//!   «ejecutar lo antes posible si se perdió», para quien no reinicia nunca la PC.
//!
//! Las dos lanzan `--update --silent`; el propio programa respeta la frecuencia elegida,
//! así que no importa que coincidan.

use std::path::Path;

pub const VALOR_RUN: &str = "GalaxyzNeoES";
pub const NOMBRE_TAREA: &str = "GalaxyzNeoES - Buscar actualizaciones del parche";

pub fn duracion_iso(horas: u32) -> String {
    if horas % 24 == 0 {
        format!("P{}D", horas / 24)
    } else {
        format!("PT{horas}H")
    }
}

pub fn xml_tarea(exe: &Path, horas: u32, inicio: &str) -> String {
    let exe = escapar_xml(&exe.display().to_string());
    let intervalo = duracion_iso(horas);
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>Parche GALAXYZ neo (proyecto de fans)</Author>
    <Description>Busca versiones nuevas de la traducción al Español Latino de GALAXYZ neo y las instala cuando el juego está cerrado.</Description>
  </RegistrationInfo>
  <Triggers>
    <TimeTrigger>
      <Repetition>
        <Interval>{intervalo}</Interval>
        <StopAtDurationEnd>false</StopAtDurationEnd>
      </Repetition>
      <StartBoundary>{inicio}</StartBoundary>
      <Enabled>true</Enabled>
    </TimeTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>true</RunOnlyIfNetworkAvailable>
    <ExecutionTimeLimit>PT8H</ExecutionTimeLimit>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{exe}</Command>
      <Arguments>--update --silent</Arguments>
    </Exec>
  </Actions>
</Task>
"#
    )
}

fn escapar_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(windows)]
pub fn activar(exe: &Path, horas: u32) -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let (run, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .map_err(|e| format!("inicio de Windows: {e}"))?;
    run.set_value(VALOR_RUN, &format!("\"{}\" --update --silent --at-login", exe.display()))
        .map_err(|e| format!("inicio de Windows: {e}"))?;

    // schtasks exige el XML en UTF-16 con BOM.
    let inicio = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let xml = xml_tarea(exe, horas, &inicio);
    let archivo = std::env::temp_dir().join(format!("galaxyz-es-tarea-{}.xml", std::process::id()));
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(xml.encode_utf16().flat_map(|u| u.to_le_bytes()));
    std::fs::write(&archivo, bytes).map_err(|e| format!("tarea programada: {e}"))?;
    let salida = crate::sistema::comando_oculto("schtasks.exe")
        .args(["/Create", "/TN", NOMBRE_TAREA, "/XML"])
        .arg(&archivo)
        .arg("/F")
        .output();
    let _ = std::fs::remove_file(&archivo);
    let salida = salida.map_err(|e| format!("tarea programada: {e}"))?;
    if !salida.status.success() {
        return Err(format!(
            "tarea programada: {}",
            String::from_utf8_lossy(&salida.stderr).trim()
        ));
    }
    Ok(())
}

#[cfg(windows)]
pub fn desactivar() {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;
    if let Ok(run) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_SET_VALUE)
    {
        let _ = run.delete_value(VALOR_RUN);
    }
    let _ = crate::sistema::comando_oculto("schtasks.exe")
        .args(["/Delete", "/TN", NOMBRE_TAREA, "/F"])
        .output();
}

/// (valor `Run` presente, tarea presente)
#[cfg(windows)]
pub fn comprobar() -> (bool, bool) {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let run = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .and_then(|k| k.get_value::<String, _>(VALOR_RUN))
        .is_ok();
    let tarea = crate::sistema::comando_oculto("schtasks.exe")
        .args(["/Query", "/TN", NOMBRE_TAREA])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    (run, tarea)
}

#[cfg(not(windows))]
pub fn activar(_: &Path, _: u32) -> Result<(), String> {
    Ok(())
}
#[cfg(not(windows))]
pub fn desactivar() {}
#[cfg(not(windows))]
pub fn comprobar() -> (bool, bool) {
    (false, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genera_la_tarea() {
        assert_eq!(duracion_iso(6), "PT6H");
        assert_eq!(duracion_iso(24), "P1D");
        assert_eq!(duracion_iso(168), "P7D");
        let xml = xml_tarea(Path::new(r"C:\Users\A&B\GalaxyzNeoES.exe"), 12, "2026-09-13T10:00:00");
        assert!(xml.contains("<Interval>PT12H</Interval>"));
        assert!(xml.contains(r"C:\Users\A&amp;B\GalaxyzNeoES.exe"));
        assert!(xml.contains("<StartWhenAvailable>true</StartWhenAvailable>"));
    }
}
