/// アプリケーション全体で使用する定数定義
/// 文字列リテラルをハードコードせず、すべてここで管理する

// DB
pub const DB_FILE_NAME: &str = "data.db";
pub const APP_DIR_NAME: &str = "minutter";

// ファイルサイズ上限
pub const MAX_AUDIO_BYTES: u64 = 100 * 1024 * 1024; // 100MB

// Vosk モデルパス
pub const VOSK_MODEL_DIR: &str = "models/vosk-model-ja";

// FFmpeg バイナリ名（Tauri サイドカー）
pub const FFMPEG_SIDECAR_NAME: &str = "ffmpeg";

// ステータス値
pub const STATUS_RECORDING: &str = "recording";
pub const STATUS_PROCESSING: &str = "processing";
pub const STATUS_DONE: &str = "done";
pub const STATUS_ERROR: &str = "error";

// セクションタイプ
pub const SECTION_DECISIONS: &str = "decisions";
pub const SECTION_NEXT: &str = "next";
pub const SECTION_BODY: &str = "body";

// エラーコード
pub const ERR_MODEL_NOT_FOUND: &str = "MODEL_NOT_FOUND";
pub const ERR_DB_CORRUPTED: &str = "DB_CORRUPTED";
pub const ERR_AUDIO_TOO_LARGE: &str = "AUDIO_TOO_LARGE";
pub const ERR_UNSUPPORTED_FORMAT: &str = "UNSUPPORTED_FORMAT";
pub const ERR_RECORDING_IN_PROGRESS: &str = "RECORDING_IN_PROGRESS";
pub const ERR_NO_RECORDING: &str = "NO_RECORDING";
pub const ERR_FFMPEG_FAILED: &str = "FFMPEG_FAILED";
pub const ERR_TRANSCRIBE_FAILED: &str = "TRANSCRIBE_FAILED";
pub const ERR_DB_INIT_FAILED: &str = "DB_INIT_FAILED";

// 対応音声フォーマット
pub const SUPPORTED_FORMATS: &[&str] = &["wav", "mp3", "m4a", "webm"];

// WAV 変換パラメータ
pub const WAV_SAMPLE_RATE: &str = "16000";
pub const WAV_CHANNELS: &str = "1";
pub const WAV_FORMAT: &str = "wav";

// TextRank パラメータ
pub const SUMMARY_RATIO_MIN: f64 = 0.15;
pub const SUMMARY_RATIO_MAX: f64 = 0.20;
pub const MIN_WORD_LENGTH: usize = 2;

// 期限キーワード
pub const KEYWORDS_DUE: &[&str] = &[
    "来週", "来月", "月曜", "火曜", "水曜", "木曜", "金曜", "今週",
];

// キーワード（TextProcessor）
pub const KEYWORDS_DECISION: &[&str] = &[
    "決定", "合意", "承認", "採択", "確定", "決まり", "決める",
];
pub const KEYWORDS_TODO: &[&str] = &[
    "する", "やる", "対応", "実施", "確認", "作成", "準備",
    "検討", "調査", "報告", "連絡", "提出", "送付", "対処",
];
pub const KEYWORDS_NEXT: &[&str] = &[
    "次回", "来週", "来月", "議題", "アジェンダ", "テーマ",
];
pub const KEYWORDS_PAST: &[&str] = &[
    "した", "だった", "でした", "ました", "ていた",
];
pub const KEYWORDS_NEGATION: &[&str] = &[
    "しない", "できない", "やらない", "しません", "できません",
];

// cpal 録音パラメータ
pub const RECORDING_SAMPLE_RATE: u32 = 16000;
pub const RECORDING_CHANNELS: u16 = 1;

// WAV ヘッダ定数
pub const WAV_BITS_PER_SAMPLE: u16 = 16;

// ファイル拡張子
pub const EXT_WAV: &str = "wav";
pub const EXT_MP3: &str = "mp3";
pub const EXT_M4A: &str = "m4a";
pub const EXT_WEBM: &str = "webm";

// DB 初期化 SQL
pub const SQL_CREATE_MEETINGS: &str = "
CREATE TABLE IF NOT EXISTS meetings (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    audio_path TEXT NOT NULL,
    duration_sec INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
)";

pub const SQL_CREATE_TRANSCRIPTS: &str = "
CREATE TABLE IF NOT EXISTS transcripts (
    id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL REFERENCES meetings(id),
    raw_text TEXT NOT NULL DEFAULT '',
    edited_text TEXT NOT NULL DEFAULT '',
    vosk_confidence REAL NOT NULL DEFAULT 0.0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)";

pub const SQL_CREATE_MINUTES: &str = "
CREATE TABLE IF NOT EXISTS minutes (
    id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL REFERENCES meetings(id),
    section_type TEXT NOT NULL,
    content TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
)";

pub const SQL_CREATE_TODOS: &str = "
CREATE TABLE IF NOT EXISTS todos (
    id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL REFERENCES meetings(id),
    todo_text TEXT NOT NULL,
    due_keyword TEXT NOT NULL DEFAULT '',
    is_checked INTEGER NOT NULL DEFAULT 0,
    is_manual INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)";

pub const SQL_CREATE_SUMMARIES: &str = "
CREATE TABLE IF NOT EXISTS summaries (
    id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL REFERENCES meetings(id),
    summary_text TEXT NOT NULL,
    created_at TEXT NOT NULL
)";

// プラグマ
pub const SQL_PRAGMA_FOREIGN_KEYS: &str = "PRAGMA foreign_keys = ON";
pub const SQL_PRAGMA_INTEGRITY_CHECK: &str = "PRAGMA integrity_check";
pub const SQL_INTEGRITY_OK: &str = "ok";

// 文章分割デリミタ
pub const SENTENCE_DELIMITERS: &[char] = &['。', '！', '？', '\n', '.', '!', '?'];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_constants_not_empty() {
        assert!(!DB_FILE_NAME.is_empty(), "DB_FILE_NAME が空文字");
        assert!(!APP_DIR_NAME.is_empty(), "APP_DIR_NAME が空文字");
    }

    #[test]
    fn test_status_constants_not_empty() {
        assert!(!STATUS_RECORDING.is_empty());
        assert!(!STATUS_PROCESSING.is_empty());
        assert!(!STATUS_DONE.is_empty());
        assert!(!STATUS_ERROR.is_empty());
    }

    #[test]
    fn test_section_constants_not_empty() {
        assert!(!SECTION_DECISIONS.is_empty());
        assert!(!SECTION_NEXT.is_empty());
        assert!(!SECTION_BODY.is_empty());
    }

    #[test]
    fn test_error_codes_not_empty() {
        assert!(!ERR_MODEL_NOT_FOUND.is_empty());
        assert!(!ERR_DB_CORRUPTED.is_empty());
        assert!(!ERR_AUDIO_TOO_LARGE.is_empty());
        assert!(!ERR_UNSUPPORTED_FORMAT.is_empty());
        assert!(!ERR_RECORDING_IN_PROGRESS.is_empty());
        assert!(!ERR_NO_RECORDING.is_empty());
        assert!(!ERR_FFMPEG_FAILED.is_empty());
        assert!(!ERR_TRANSCRIBE_FAILED.is_empty());
        assert!(!ERR_DB_INIT_FAILED.is_empty());
    }

    #[test]
    fn test_supported_formats_not_empty() {
        assert!(!SUPPORTED_FORMATS.is_empty());
        for fmt in SUPPORTED_FORMATS {
            assert!(!fmt.is_empty(), "SUPPORTED_FORMATS に空文字が含まれている");
        }
    }

    #[test]
    fn test_keywords_not_empty() {
        assert!(!KEYWORDS_DECISION.is_empty());
        assert!(!KEYWORDS_TODO.is_empty());
        assert!(!KEYWORDS_NEXT.is_empty());
        assert!(!KEYWORDS_PAST.is_empty());
        assert!(!KEYWORDS_NEGATION.is_empty());
        assert!(!KEYWORDS_DUE.is_empty());
    }

    #[test]
    fn test_keywords_no_empty_strings() {
        for kw in KEYWORDS_DECISION {
            assert!(!kw.is_empty(), "KEYWORDS_DECISION に空文字が含まれている");
        }
        for kw in KEYWORDS_TODO {
            assert!(!kw.is_empty(), "KEYWORDS_TODO に空文字が含まれている");
        }
        for kw in KEYWORDS_NEXT {
            assert!(!kw.is_empty(), "KEYWORDS_NEXT に空文字が含まれている");
        }
        for kw in KEYWORDS_PAST {
            assert!(!kw.is_empty(), "KEYWORDS_PAST に空文字が含まれている");
        }
        for kw in KEYWORDS_NEGATION {
            assert!(!kw.is_empty(), "KEYWORDS_NEGATION に空文字が含まれている");
        }
    }

    #[test]
    fn test_max_audio_bytes_is_100mb() {
        assert_eq!(MAX_AUDIO_BYTES, 100 * 1024 * 1024);
    }

    #[test]
    fn test_summary_ratio_range() {
        assert!(SUMMARY_RATIO_MIN > 0.0);
        assert!(SUMMARY_RATIO_MAX > SUMMARY_RATIO_MIN);
        assert!(SUMMARY_RATIO_MAX <= 1.0);
    }

    #[test]
    fn test_vosk_model_dir_not_empty() {
        assert!(!VOSK_MODEL_DIR.is_empty());
    }

    #[test]
    fn test_ffmpeg_sidecar_name_not_empty() {
        assert!(!FFMPEG_SIDECAR_NAME.is_empty());
    }

    #[test]
    fn test_sql_create_tables_not_empty() {
        assert!(!SQL_CREATE_MEETINGS.is_empty());
        assert!(!SQL_CREATE_TRANSCRIPTS.is_empty());
        assert!(!SQL_CREATE_MINUTES.is_empty());
        assert!(!SQL_CREATE_TODOS.is_empty());
        assert!(!SQL_CREATE_SUMMARIES.is_empty());
    }

    #[test]
    fn test_wav_params_not_empty() {
        assert!(!WAV_SAMPLE_RATE.is_empty());
        assert!(!WAV_CHANNELS.is_empty());
        assert!(!WAV_FORMAT.is_empty());
    }
}
