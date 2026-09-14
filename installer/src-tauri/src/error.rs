use std::path::Path;

/// Errores que llegan a la interfaz. El texto ya está en español y se muestra tal cual.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("El juego está abierto. Ciérralo para continuar.")]
    JuegoAbierto,

    #[error("{0}")]
    Validacion(String),

    #[error("No se pudo {accion} «{ruta}»: {fuente}")]
    Io {
        accion: &'static str,
        ruta: String,
        #[source]
        fuente: std::io::Error,
    },

    #[error("El archivo «{0}» del parche está dañado: su hash no coincide con el esperado.")]
    PaqueteCorrupto(String),

    #[error("La verificación falló: «{0}» no quedó igual que el archivo del parche.")]
    Verificacion(String),

    #[error("No hay ninguna instalación del parche registrada.")]
    SinInstalacion,

    #[error("No se pudo conectar con GitHub: {0}")]
    Red(String),

    #[error("Hay otra operación del parche en curso. Espera a que termine.")]
    Ocupado,
}

impl serde::Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Resultado<T> = Result<T, Error>;

/// Convierte un error de E/S en `Error::Io` con la acción y la ruta, para usar con `map_err`.
pub fn io<'a>(accion: &'static str, ruta: &'a Path) -> impl FnOnce(std::io::Error) -> Error + 'a {
    move |fuente| Error::Io {
        accion,
        ruta: ruta.display().to_string(),
        fuente,
    }
}
