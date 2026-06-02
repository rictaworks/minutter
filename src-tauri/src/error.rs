use thiserror::Error;
use crate::config;

/// アプリケーション共通エラー型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("{}: {0}", config::ERR_MODEL_NOT_FOUND)]
    ModelNotFound(String),

    #[error("{}: {0}", config::ERR_DB_CORRUPTED)]
    DbCorrupted(String),

    #[error("{}: {0}", config::ERR_DB_INIT_FAILED)]
    DbInitFailed(String),

    #[error("{}: {0}", config::ERR_AUDIO_TOO_LARGE)]
    AudioTooLarge(u64),

    #[error("{}: {0}", config::ERR_UNSUPPORTED_FORMAT)]
    UnsupportedFormat(String),

    #[error("{}: {0}", config::ERR_RECORDING_IN_PROGRESS)]
    RecordingInProgress(String),

    #[error("{}: {0}", config::ERR_NO_RECORDING)]
    NoRecording(String),

    #[error("{}: {0}", config::ERR_FFMPEG_FAILED)]
    FfmpegFailed(String),

    #[error("{}: {0}", config::ERR_TRANSCRIBE_FAILED)]
    TranscribeFailed(String),

    #[error("IO エラー: {0}")]
    Io(#[from] std::io::Error),

    #[error("DB エラー: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("内部エラー: {0}")]
    Internal(String),
}

impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_not_found_error_message() {
        let err = AppError::ModelNotFound("テストパス".to_string());
        let msg = err.to_string();
        assert!(msg.contains(config::ERR_MODEL_NOT_FOUND));
    }

    #[test]
    fn test_db_corrupted_error_message() {
        let err = AppError::DbCorrupted("破損".to_string());
        let msg = err.to_string();
        assert!(msg.contains(config::ERR_DB_CORRUPTED));
    }

    #[test]
    fn test_audio_too_large_error_message() {
        let err = AppError::AudioTooLarge(200 * 1024 * 1024);
        let msg = err.to_string();
        assert!(msg.contains(config::ERR_AUDIO_TOO_LARGE));
    }

    #[test]
    fn test_unsupported_format_error_message() {
        let err = AppError::UnsupportedFormat("flac".to_string());
        let msg = err.to_string();
        assert!(msg.contains(config::ERR_UNSUPPORTED_FORMAT));
    }

    #[test]
    fn test_error_to_string_conversion() {
        let err = AppError::Internal("予期しないエラー".to_string());
        let s: String = err.into();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_ffmpeg_failed_error_message() {
        let err = AppError::FfmpegFailed("exit code 1".to_string());
        let msg = err.to_string();
        assert!(msg.contains(config::ERR_FFMPEG_FAILED));
    }
}
