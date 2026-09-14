//! Notificaciones de Windows para la actualización en segundo plano (no hay ventana abierta).

pub fn mostrar(titulo: &str, texto: &str) {
    crate::log::escribir(&format!("Notificación: {titulo} — {texto}"));
    #[cfg(windows)]
    {
        use tauri_winrt_notification::Toast;
        let _ = Toast::new(Toast::POWERSHELL_APP_ID).title(titulo).text1(texto).show();
    }
}
