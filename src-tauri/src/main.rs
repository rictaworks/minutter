// Tauri が Windows で余分なコンソールウィンドウを開かないようにする
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    minutter_lib::run();
}
