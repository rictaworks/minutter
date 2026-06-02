use log::{debug, error, info};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::audio::{AudioImporter, AudioRecorder};
use crate::config;
use crate::db::models::{MinuteItem, NewMeeting, NewMinute, NewSummary, NewTodo, NewTranscript, TodoItem};
use crate::db::MeetingRepository;
use crate::processor::{MinutesProcessor, SummaryProcessor, TodoProcessor};
use crate::transcribe::VoskTranscriber;

// ---- フロントエンド向け型定義 ----

/// generate_all コマンドの戻り値
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerateResult {
    pub minutes: Vec<MinuteItem>,
    pub todos: Vec<TodoItem>,
    pub summary: String,
}

// ---- アプリ状態 ----

/// 録音状態管理（Arc<Mutex> でスレッド安全に共有）
pub struct RecordingState {
    pub recorder: Option<AudioRecorder>,
}

impl RecordingState {
    pub fn new() -> Self {
        RecordingState { recorder: None }
    }
}

/// Tauri のマネージドステート型
pub type SharedRecordingState = Arc<Mutex<RecordingState>>;

/// DB リポジトリのマネージドステート型
pub type SharedRepository = Arc<Mutex<MeetingRepository>>;

// ---- アプリ起動系コマンド ----

/// Vosk モデルが存在するかチェックする
#[tauri::command]
pub fn check_model(app: AppHandle) -> Result<bool, String> {
    debug!("コマンド: check_model");
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| {
            error!("リソースディレクトリ取得失敗: {}", e);
            e.to_string()
        })?;
    let model_path = resource_path.join(config::VOSK_MODEL_DIR);
    let exists = VoskTranscriber::check_model(&model_path);
    info!("モデル存在確認: {:?} -> {}", model_path, exists);
    Ok(exists)
}

/// アプリを初期化する（DB セットアップ）
#[tauri::command]
pub fn init_app(
    app: AppHandle,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<(), String> {
    debug!("コマンド: init_app");
    let db_path = get_db_path(&app)?;
    debug!("DB パス: {:?}", db_path);

    let repo = MeetingRepository::new(
        db_path.to_str().ok_or_else(|| "DB パスの文字列変換失敗".to_string())?
    ).map_err(|e| {
        error!("DB 初期化失敗: {}", e);
        e.to_string()
    })?;

    let mut state = repo_state.lock().map_err(|e| {
        error!("DB ステートロック失敗: {}", e);
        e.to_string()
    })?;
    *state = repo;

    info!("アプリ初期化完了");
    Ok(())
}

// ---- 録音コマンド ----

/// 利用可能な入力デバイス一覧を返す
#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>, String> {
    debug!("コマンド: list_audio_devices");
    let devices = AudioRecorder::list_devices();
    info!("デバイス一覧: {} 件", devices.len());
    Ok(devices)
}

/// 録音を開始する
#[tauri::command]
pub fn start_recording(
    app: AppHandle,
    device_index: usize,
    meeting_id: String,
    recording_state: tauri::State<'_, SharedRecordingState>,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<(), String> {
    info!("コマンド: start_recording, meeting_id={}", meeting_id);

    let mut rec_state = recording_state.lock().map_err(|e| {
        error!("録音ステートロック失敗: {}", e);
        e.to_string()
    })?;

    if rec_state.recorder.is_some() {
        error!("録音中のため開始できません");
        return Err(config::ERR_RECORDING_IN_PROGRESS.to_string());
    }

    let audio_dir = get_audio_dir(&app)?;
    std::fs::create_dir_all(&audio_dir).map_err(|e| {
        error!("音声ディレクトリ作成失敗: {}", e);
        e.to_string()
    })?;

    let audio_path = audio_dir.join(format!("{}.wav", meeting_id));
    let mut recorder = AudioRecorder::new(audio_path);
    recorder.select_device(device_index);
    recorder.start().map_err(|e| {
        error!("録音開始失敗: {}", e);
        e.to_string()
    })?;

    rec_state.recorder = Some(recorder);

    // ステータスを recording に更新
    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.update_meeting_status(&meeting_id, config::STATUS_RECORDING)
        .map_err(|e| e.to_string())?;

    info!("録音開始完了: meeting_id={}", meeting_id);
    Ok(())
}

/// 録音を停止し、文字起こし結果（raw_text）を返す
#[tauri::command]
pub fn stop_recording(
    app: AppHandle,
    meeting_id: String,
    recording_state: tauri::State<'_, SharedRecordingState>,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<String, String> {
    info!("コマンド: stop_recording, meeting_id={}", meeting_id);

    let wav_path = {
        let mut rec_state = recording_state.lock().map_err(|e| {
            error!("録音ステートロック失敗: {}", e);
            e.to_string()
        })?;

        let recorder = rec_state.recorder.as_mut().ok_or_else(|| {
            error!("録音中でない");
            config::ERR_NO_RECORDING.to_string()
        })?;

        let path = recorder.stop().map_err(|e| {
            error!("録音停止失敗: {}", e);
            e.to_string()
        })?;

        rec_state.recorder = None;
        path
    };

    // ステータスを processing に更新
    {
        let repo = repo_state.lock().map_err(|e| e.to_string())?;
        repo.update_meeting_status(&meeting_id, config::STATUS_PROCESSING)
            .map_err(|e| e.to_string())?;
    }

    // 文字起こし
    let raw_text = transcribe_file(&app, &wav_path)?;

    // トランスクリプトを保存
    {
        let repo = repo_state.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let transcript = NewTranscript {
            id: Uuid::new_v4().to_string(),
            meeting_id: meeting_id.clone(),
            raw_text: raw_text.clone(),
            edited_text: raw_text.clone(),
            vosk_confidence: 0.0, // 後続で更新可能
            created_at: now.clone(),
            updated_at: now,
        };
        repo.save_transcript(&transcript).map_err(|e| e.to_string())?;
    }

    info!("録音停止・文字起こし完了: meeting_id={}", meeting_id);
    Ok(raw_text)
}

// ---- ファイルインポートコマンド ----

/// 音声ファイルをインポートし、文字起こし結果（raw_text）を返す
#[tauri::command]
pub fn import_audio(
    app: AppHandle,
    path: String,
    meeting_id: String,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<String, String> {
    info!("コマンド: import_audio, path={}", path);

    let ffmpeg_path = get_ffmpeg_path(&app)?;
    let importer = AudioImporter::new(ffmpeg_path);

    let wav_path = importer.import(PathBuf::from(&path)).map_err(|e| {
        error!("音声インポート失敗: {}", e);
        e.to_string()
    })?;

    // ステータスを processing に更新
    {
        let repo = repo_state.lock().map_err(|e| e.to_string())?;
        repo.update_meeting_status(&meeting_id, config::STATUS_PROCESSING)
            .map_err(|e| e.to_string())?;
        // 音声パスを更新（audio_path は録音時に設定されるが、インポート時は後から更新）
        let wav_str = wav_path.to_str().unwrap_or("").to_string();
        repo.update_meeting_audio_path(&meeting_id, &wav_str)
            .map_err(|e| e.to_string())?;
    }

    // 文字起こし
    let raw_text = transcribe_file(&app, &wav_path)?;

    // トランスクリプトを保存
    {
        let repo = repo_state.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let transcript = NewTranscript {
            id: Uuid::new_v4().to_string(),
            meeting_id: meeting_id.clone(),
            raw_text: raw_text.clone(),
            edited_text: raw_text.clone(),
            vosk_confidence: 0.0,
            created_at: now.clone(),
            updated_at: now,
        };
        repo.save_transcript(&transcript).map_err(|e| e.to_string())?;
    }

    info!("インポート・文字起こし完了: meeting_id={}", meeting_id);
    Ok(raw_text)
}

// ---- テキスト処理コマンド ----

/// テキストから議事録・ToDo・要約を一括生成する
#[tauri::command]
pub fn generate_all(
    meeting_id: String,
    text: String,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<GenerateResult, String> {
    info!("コマンド: generate_all, meeting_id={}", meeting_id);

    let minute_items = MinutesProcessor::generate_minutes(&text);
    let todo_items = TodoProcessor::extract_todos(&text);
    let summary_text = SummaryProcessor::summarize(&text);

    // DB に保存
    {
        let repo = repo_state.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        let new_minutes: Vec<NewMinute> = minute_items
            .iter()
            .map(|m| NewMinute {
                id: m.id.clone(),
                meeting_id: meeting_id.clone(),
                section_type: m.section_type.clone(),
                content: m.content.clone(),
                sort_order: m.sort_order,
                created_at: now.clone(),
            })
            .collect();
        repo.save_minutes(&new_minutes).map_err(|e| e.to_string())?;

        let new_todos: Vec<NewTodo> = todo_items
            .iter()
            .map(|t| NewTodo {
                id: t.id.clone(),
                meeting_id: meeting_id.clone(),
                todo_text: t.todo_text.clone(),
                due_keyword: t.due_keyword.clone(),
                is_manual: false,
                created_at: now.clone(),
                updated_at: now.clone(),
            })
            .collect();
        repo.save_todos(&new_todos).map_err(|e| e.to_string())?;

        let new_summary = NewSummary {
            id: Uuid::new_v4().to_string(),
            meeting_id: meeting_id.clone(),
            summary_text: summary_text.clone(),
            created_at: now,
        };
        repo.save_summary(&new_summary).map_err(|e| e.to_string())?;

        // ステータスを done に更新
        repo.update_meeting_status(&meeting_id, config::STATUS_DONE)
            .map_err(|e| e.to_string())?;
    }

    info!("generate_all 完了: meeting_id={}", meeting_id);
    Ok(GenerateResult {
        minutes: minute_items,
        todos: todo_items,
        summary: summary_text,
    })
}

// ---- CRUD コマンド ----

/// 会議を新規作成し、meeting_id を返す
#[tauri::command]
pub fn create_meeting(
    app: AppHandle,
    title: String,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<String, String> {
    debug!("コマンド: create_meeting, title={}", title);

    let meeting_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let audio_dir = get_audio_dir(&app)?;
    let audio_path = audio_dir.join(format!("{}.wav", meeting_id));
    let audio_path_str = audio_path.to_str().unwrap_or("").to_string();

    let new_meeting = NewMeeting {
        id: meeting_id.clone(),
        title,
        recorded_at: now.clone(),
        audio_path: audio_path_str,
        duration_sec: 0,
        status: config::STATUS_RECORDING.to_string(),
        created_at: now,
    };

    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.save_meeting(&new_meeting).map_err(|e| {
        error!("会議作成失敗: {}", e);
        e.to_string()
    })?;

    info!("会議作成完了: id={}", meeting_id);
    Ok(meeting_id)
}

/// 会議一覧を返す
#[tauri::command]
pub fn list_meetings(
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<Vec<crate::db::models::Meeting>, String> {
    debug!("コマンド: list_meetings");
    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.list_meetings().map_err(|e| {
        error!("会議一覧取得失敗: {}", e);
        e.to_string()
    })
}

/// 会議詳細を返す
#[tauri::command]
pub fn get_meeting(
    id: String,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<Option<crate::db::models::MeetingDetail>, String> {
    debug!("コマンド: get_meeting, id={}", id);
    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.get_meeting(&id).map_err(|e| {
        error!("会議詳細取得失敗: {}", e);
        e.to_string()
    })
}

/// 会議を削除する
#[tauri::command]
pub fn delete_meeting(
    id: String,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<(), String> {
    info!("コマンド: delete_meeting, id={}", id);
    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.delete_meeting(&id).map_err(|e| {
        error!("会議削除失敗: {}", e);
        e.to_string()
    })
}

/// トランスクリプトの編集テキストを更新する
#[tauri::command]
pub fn update_transcript(
    meeting_id: String,
    edited_text: String,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<(), String> {
    debug!("コマンド: update_transcript, meeting_id={}", meeting_id);
    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.update_transcript(&meeting_id, &edited_text).map_err(|e| {
        error!("トランスクリプト更新失敗: {}", e);
        e.to_string()
    })
}

/// ToDo のチェック状態を更新する
#[tauri::command]
pub fn update_todo_check(
    id: String,
    is_checked: bool,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<(), String> {
    debug!("コマンド: update_todo_check, id={}", id);
    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.update_todo(&id, is_checked).map_err(|e| {
        error!("ToDo チェック更新失敗: {}", e);
        e.to_string()
    })
}

/// ToDo を論理削除する
#[tauri::command]
pub fn delete_todo(
    id: String,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<(), String> {
    info!("コマンド: delete_todo, id={}", id);
    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.delete_todo(&id).map_err(|e| {
        error!("ToDo 削除失敗: {}", e);
        e.to_string()
    })
}

/// ToDo を手動追加する
#[tauri::command]
pub fn add_todo(
    meeting_id: String,
    todo_text: String,
    repo_state: tauri::State<'_, SharedRepository>,
) -> Result<(), String> {
    debug!("コマンド: add_todo, meeting_id={}", meeting_id);
    let now = chrono::Utc::now().to_rfc3339();
    let new_todo = NewTodo {
        id: Uuid::new_v4().to_string(),
        meeting_id,
        todo_text,
        due_keyword: String::new(),
        is_manual: true,
        created_at: now.clone(),
        updated_at: now,
    };
    let repo = repo_state.lock().map_err(|e| e.to_string())?;
    repo.save_todos(&[new_todo]).map_err(|e| {
        error!("ToDo 追加失敗: {}", e);
        e.to_string()
    })
}

// ---- ヘルパー関数 ----

/// DB パスを取得する
fn get_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| {
            error!("AppDataDir 取得失敗: {}", e);
            e.to_string()
        })?;
    let db_dir = data_dir.join(config::APP_DIR_NAME);
    std::fs::create_dir_all(&db_dir).map_err(|e| {
        error!("DB ディレクトリ作成失敗: {}", e);
        e.to_string()
    })?;
    Ok(db_dir.join(config::DB_FILE_NAME))
}

/// 音声ファイルの保存ディレクトリを取得する
fn get_audio_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    Ok(data_dir.join(config::APP_DIR_NAME).join("audio"))
}

/// ffmpeg バイナリのパスを取得する
fn get_ffmpeg_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?;
    Ok(resource_path
        .join("binaries")
        .join(config::FFMPEG_SIDECAR_NAME))
}

/// WAV ファイルを文字起こしする
fn transcribe_file(app: &AppHandle, wav_path: &PathBuf) -> Result<String, String> {
    debug!("文字起こし開始: {:?}", wav_path);
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?;
    let model_path = resource_path.join(config::VOSK_MODEL_DIR);

    let transcriber = VoskTranscriber::new(model_path).map_err(|e| {
        error!("Vosk 初期化失敗: {}", e);
        e.to_string()
    })?;

    let result = transcriber.transcribe(wav_path).map_err(|e| {
        error!("文字起こし失敗: {}", e);
        e.to_string()
    })?;

    debug!("文字起こし完了: {} 文字", result.text.len());
    Ok(result.text)
}
