//! Evita que la actualización en segundo plano y la ventana escriban a la vez en el juego.

use crate::error::{io, Error, Resultado};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Candado(PathBuf);

impl Candado {
    pub fn tomar(carpeta_app: &Path) -> Resultado<Self> {
        std::fs::create_dir_all(carpeta_app).map_err(io("crear", carpeta_app))?;
        let ruta = carpeta_app.join("operacion.lock");
        for _ in 0..2 {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&ruta) {
                Ok(mut f) => {
                    let _ = write!(f, "{}", std::process::id());
                    return Ok(Self(ruta));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let pid = std::fs::read_to_string(&ruta).ok().and_then(|t| t.trim().parse::<u32>().ok());
                    if pid.is_some_and(proceso_vivo) {
                        return Err(Error::Ocupado);
                    }
                    // Candado huérfano (el proceso ya no existe): se quita y se reintenta.
                    let _ = std::fs::remove_file(&ruta);
                }
                Err(e) => return Err(io("crear", &ruta)(e)),
            }
        }
        Err(Error::Ocupado)
    }
}

impl Drop for Candado {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn proceso_vivo(pid: u32) -> bool {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
    let mut s = System::new();
    let pid = Pid::from_u32(pid);
    s.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, ProcessRefreshKind::nothing());
    s.process(pid).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_deja_dos_operaciones_y_limpia_huerfanos() {
        let dir = tempfile::tempdir().unwrap();
        let c = Candado::tomar(dir.path()).unwrap();
        assert!(matches!(Candado::tomar(dir.path()), Err(Error::Ocupado)));
        drop(c);
        let _c2 = Candado::tomar(dir.path()).unwrap();
        drop(_c2);
        std::fs::write(dir.path().join("operacion.lock"), "4294967290").unwrap();
        assert!(Candado::tomar(dir.path()).is_ok());
    }
}
