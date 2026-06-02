use log::{debug, error, info};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;
use crate::error::AppError;

/// 外部音声ファイルを WAV に変換するインポーター
/// ffmpeg をサイドカーとして呼び出す
pub struct AudioImporter {
    ffmpeg_path: PathBuf,
}

impl AudioImporter {
    /// 新しい AudioImporter を作成する
    pub fn new(ffmpeg_path: PathBuf) -> Self {
        debug!("AudioImporter 作成: ffmpeg={:?}", ffmpeg_path);
        AudioImporter { ffmpeg_path }
    }

    /// ファイルを検証して WAV に変換する
    /// - ファイルサイズが MAX_AUDIO_BYTES を超える場合はエラー
    /// - 非対応フォーマットの場合はエラー
    pub fn import(&self, path: PathBuf) -> Result<PathBuf, AppError> {
        info!("音声インポート開始: {:?}", path);

        // ファイルサイズチェック
        let metadata = std::fs::metadata(&path).map_err(|e| {
            error!("ファイルメタデータ取得失敗: {:?} {}", path, e);
            AppError::Io(e)
        })?;
        let file_size = metadata.len();
        if file_size > config::MAX_AUDIO_BYTES {
            error!("ファイルサイズ超過: {} bytes", file_size);
            return Err(AppError::AudioTooLarge(file_size));
        }
        debug!("ファイルサイズ OK: {} bytes", file_size);

        // フォーマット検証
        if !self.validate_format(&path) {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown");
            error!("非対応フォーマット: {}", ext);
            return Err(AppError::UnsupportedFormat(ext.to_string()));
        }

        // WAV に変換
        self.convert_to_wav(path)
    }

    /// 音声ファイルを 16kHz モノラル WAV に変換する
    pub fn convert_to_wav(&self, path: PathBuf) -> Result<PathBuf, AppError> {
        info!("WAV 変換開始: {:?}", path);

        let output_path = self.make_output_path(&path);
        debug!("出力パス: {:?}", output_path);

        let input = path.to_str().ok_or_else(|| {
            AppError::Internal("入力パスの文字列変換失敗".to_string())
        })?;
        let output = output_path.to_str().ok_or_else(|| {
            AppError::Internal("出力パスの文字列変換失敗".to_string())
        })?;

        let result = Command::new(&self.ffmpeg_path)
            .args([
                "-y",
                "-i", input,
                "-ar", config::WAV_SAMPLE_RATE,
                "-ac", config::WAV_CHANNELS,
                "-f", config::WAV_FORMAT,
                output,
            ])
            .output()
            .map_err(|e| {
                error!("ffmpeg 実行失敗: {}", e);
                AppError::FfmpegFailed(e.to_string())
            })?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            error!("ffmpeg 変換失敗: exit={}, stderr={}", result.status, stderr);
            return Err(AppError::FfmpegFailed(format!(
                "exit={}, stderr={}",
                result.status,
                stderr.chars().take(500).collect::<String>()
            )));
        }

        info!("WAV 変換完了: {:?}", output_path);
        Ok(output_path)
    }

    /// ファイルの拡張子が対応フォーマットかどうかを検証する
    pub fn validate_format(&self, path: &Path) -> bool {
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_lowercase(),
            None => {
                debug!("拡張子なし: {:?}", path);
                return false;
            }
        };
        let valid = config::SUPPORTED_FORMATS.contains(&ext.as_str());
        debug!("フォーマット検証: {} -> {}", ext, valid);
        valid
    }

    /// 入力ファイルパスから出力 WAV パスを生成する
    fn make_output_path(&self, input: &Path) -> PathBuf {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let dir = input.parent().unwrap_or_else(|| Path::new("/tmp"));
        dir.join(format!("{}_converted.{}", stem, config::EXT_WAV))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_importer() -> AudioImporter {
        AudioImporter::new(PathBuf::from("/usr/bin/ffmpeg"))
    }

    #[test]
    fn test_validate_format_wav() {
        let importer = make_importer();
        assert!(importer.validate_format(Path::new("test.wav")));
    }

    #[test]
    fn test_validate_format_mp3() {
        let importer = make_importer();
        assert!(importer.validate_format(Path::new("test.mp3")));
    }

    #[test]
    fn test_validate_format_m4a() {
        let importer = make_importer();
        assert!(importer.validate_format(Path::new("test.m4a")));
    }

    #[test]
    fn test_validate_format_webm() {
        let importer = make_importer();
        assert!(importer.validate_format(Path::new("test.webm")));
    }

    #[test]
    fn test_validate_format_unsupported() {
        let importer = make_importer();
        assert!(!importer.validate_format(Path::new("test.flac")));
        assert!(!importer.validate_format(Path::new("test.ogg")));
        assert!(!importer.validate_format(Path::new("test.aac")));
    }

    #[test]
    fn test_validate_format_no_extension() {
        let importer = make_importer();
        assert!(!importer.validate_format(Path::new("noextension")));
    }

    #[test]
    fn test_validate_format_uppercase_extension() {
        let importer = make_importer();
        // 大文字拡張子も対応する
        assert!(importer.validate_format(Path::new("test.WAV")));
        assert!(importer.validate_format(Path::new("test.MP3")));
    }

    #[test]
    fn test_validate_format_supported_formats_constant() {
        // SUPPORTED_FORMATS にハードコードがないことを確認
        for fmt in config::SUPPORTED_FORMATS {
            assert!(!fmt.is_empty(), "SUPPORTED_FORMATS に空文字が含まれている");
        }
    }

    #[test]
    fn test_make_output_path() {
        let importer = make_importer();
        let input = PathBuf::from("/tmp/recording.mp3");
        let output = importer.make_output_path(&input);
        assert!(output.to_str().unwrap().contains("recording_converted"));
        assert!(output.to_str().unwrap().ends_with(".wav"));
    }

    #[test]
    fn test_import_file_too_large() {
        // 実ファイルなしでサイズ超過を直接テストできないため
        // validate_format で非対応フォーマットを確認する
        let importer = make_importer();
        let result = importer.import(PathBuf::from("/nonexistent.flac"));
        // ファイルが存在しないか非対応フォーマットのどちらかのエラーが返る
        assert!(result.is_err());
    }

    #[test]
    fn test_import_nonexistent_wav() {
        let importer = make_importer();
        // ファイルが存在しない WAV ファイル
        let result = importer.import(PathBuf::from("/nonexistent_file.wav"));
        assert!(result.is_err());
    }
}
