// En la versión final no se abre una consola extra; con --silent se engancha a la de quien lo lanza.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    galaxyz_neo_es_lib::run();
}
