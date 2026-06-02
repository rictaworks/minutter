pub mod audio;
pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod processor;
pub mod transcribe;

use std::sync::{Arc, Mutex};
use commands::{RecordingState, SharedRecordingState, SharedRepository};
use db::MeetingRepository;
use log::info;

/// Tauri アプリのエントリポイント（lib から呼ばれる）
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    info!("minutter 起動");

    // 初期 DB（ダミー）。init_app コマンドで実際の DB に差し替える
    let initial_repo = MeetingRepository::new_in_memory()
        .expect("インメモリ DB 初期化失敗");

    let recording_state: SharedRecordingState = Arc::new(Mutex::new(RecordingState::new()));
    let repo_state: SharedRepository = Arc::new(Mutex::new(initial_repo));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(recording_state)
        .manage(repo_state)
        .invoke_handler(tauri::generate_handler![
            commands::check_model,
            commands::init_app,
            commands::list_audio_devices,
            commands::start_recording,
            commands::stop_recording,
            commands::import_audio,
            commands::generate_all,
            commands::create_meeting,
            commands::list_meetings,
            commands::get_meeting,
            commands::delete_meeting,
            commands::update_transcript,
            commands::update_todo_check,
            commands::delete_todo,
            commands::add_todo,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri アプリ実行失敗");
}
