mod actualizacion;
mod candado;
mod cli;
mod comandos;
mod deteccion;
mod error;
mod estado;
mod hash;
mod instalacion;
mod log;
mod notificacion;
mod paquete;
mod programador;
mod remoto;
mod sistema;
mod validacion;
mod version;

use tauri::Manager;

pub fn run() {
    let args = match cli::analizar(std::env::args()) {
        Ok(a) => a,
        Err(mensaje) => {
            eprintln!("{mensaje}");
            std::process::exit(1);
        }
    };
    if args.sin_ventana() {
        std::process::exit(cli::ejecutar(&args));
    }
    actualizacion::limpiar_restos();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(ventana) = app.get_webview_window("main") {
                let _ = ventana.unminimize();
                let _ = ventana.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(comandos::ArgumentosInicio(args))
        .manage(comandos::UltimoManifest::default())
        .invoke_handler(tauri::generate_handler![
            comandos::argumentos_inicio,
            comandos::info_paquete,
            comandos::detectar,
            comandos::validar_juego,
            comandos::validar_datos,
            comandos::juego_abierto,
            comandos::abrir_juego,
            comandos::instalar,
            comandos::reparar,
            comandos::desinstalar,
            comandos::buscar_actualizacion,
            comandos::aplicar_actualizacion,
            comandos::actualizar_instalador,
            comandos::cambiar_opciones,
            comandos::abrir_url,
        ])
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar la ventana");
}
