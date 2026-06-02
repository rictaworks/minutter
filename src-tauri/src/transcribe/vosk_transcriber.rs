use log::{debug, error, info};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::config;
use crate::error::AppError;

/// Vosk 音声認識の結果
#[derive(Debug, Clone)]
pub struct TranscriptResult {
    pub text: String,
    pub confidence: f64,
}

/// Vosk オフライン音声認識を担当するクラス
pub struct VoskTranscriber {
    model: vosk::Model,
    model_path: PathBuf,
}

impl VoskTranscriber {
    /// モデルパスから VoskTranscriber を作成する
    /// モデルが存在しない場合は Err を返す（フォールバックなし）
    pub fn new(model_path: PathBuf) -> Result<Self, AppError> {
        info!("Vosk モデル読み込み開始: {:?}", model_path);

        if !model_path.exists() {
            error!("Vosk モデルが見つかりません: {:?}", model_path);
            return Err(AppError::ModelNotFound(
                model_path.to_string_lossy().to_string(),
            ));
        }

        let model_path_str = model_path.to_str().ok_or_else(|| {
            AppError::Internal("モデルパスの文字列変換失敗".to_string())
        })?;

        let model = vosk::Model::new(model_path_str).ok_or_else(|| {
            error!("Vosk モデル読み込み失敗: {}", model_path_str);
            AppError::ModelNotFound(model_path_str.to_string())
        })?;

        info!("Vosk モデル読み込み完了");
        Ok(VoskTranscriber { model, model_path })
    }

    /// モデルが存在するかどうかを確認する
    pub fn check_model(model_path: &Path) -> bool {
        let exists = model_path.exists();
        debug!("モデル存在確認: {:?} -> {}", model_path, exists);
        exists
    }

    /// WAV ファイルを文字起こしする
    /// Vosk 推奨: 16kHz モノラル WAV
    pub fn transcribe(&self, wav_path: &PathBuf) -> Result<TranscriptResult, AppError> {
        info!("文字起こし開始: {:?}", wav_path);

        if !wav_path.exists() {
            error!("WAV ファイルが見つかりません: {:?}", wav_path);
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("WAV ファイルが見つかりません: {:?}", wav_path),
            )));
        }

        let file = std::fs::File::open(wav_path).map_err(|e| {
            error!("WAV ファイルオープン失敗: {}", e);
            AppError::Io(e)
        })?;

        let mut reader = BufReader::new(file);
        let (header, samples) = parse_wav(&mut reader).map_err(|e| {
            error!("WAV パース失敗: {}", e);
            AppError::TranscribeFailed(e.to_string())
        })?;

        debug!("WAV ヘッダ: sample_rate={}, channels={}", header.sample_rate, header.channels);

        let mut recognizer = vosk::Recognizer::new(&self.model, header.sample_rate as f32)
            .ok_or_else(|| {
                error!("Vosk Recognizer 作成失敗");
                AppError::TranscribeFailed("Recognizer 作成失敗".to_string())
            })?;

        recognizer.set_words(true);

        // サンプルを 4096 フレームずつ処理する
        const CHUNK_SIZE: usize = 4096;
        let mut all_words: Vec<String> = Vec::new();
        let mut word_confidences: Vec<f32> = Vec::new();

        for chunk in samples.chunks(CHUNK_SIZE) {
            let state = recognizer.accept_waveform(chunk);
            match state {
                vosk::DecodingState::Running => {
                    debug!("デコード中...");
                }
                vosk::DecodingState::Finalized => {
                    debug!("フレーム確定");
                    let result = recognizer.result();
                    if let Some(single) = result.single() {
                        all_words.push(single.text.to_string());
                        for word in &single.result {
                            word_confidences.push(word.conf);
                        }
                    }
                }
                vosk::DecodingState::Failed => {
                    error!("Vosk デコード失敗");
                    return Err(AppError::TranscribeFailed("デコード失敗".to_string()));
                }
            }
        }

        let final_result = recognizer.final_result();
        let final_text = match &final_result {
            vosk::CompleteResult::Single(r) => {
                for word in &r.result {
                    word_confidences.push(word.conf);
                }
                r.text.to_string()
            }
            vosk::CompleteResult::Multiple(r) => {
                r.alternatives
                    .first()
                    .map(|a| a.text.to_string())
                    .unwrap_or_default()
            }
        };
        all_words.push(final_text);

        let text = all_words.join(" ");

        let avg_confidence = if word_confidences.is_empty() {
            0.0
        } else {
            word_confidences.iter().map(|c| *c as f64).sum::<f64>() / word_confidences.len() as f64
        };

        info!(
            "文字起こし完了: {} 文字, confidence={:.3}",
            text.len(),
            avg_confidence
        );

        Ok(TranscriptResult {
            text,
            confidence: avg_confidence,
        })
    }
}

/// WAV ヘッダ情報
#[derive(Debug)]
struct WavHeader {
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
}

/// WAV ファイルを解析して PCM サンプルを返す
fn parse_wav(reader: &mut BufReader<std::fs::File>) -> Result<(WavHeader, Vec<i16>), String> {
    use std::io::Read;

    let mut buf4 = [0u8; 4];
    let mut buf2 = [0u8; 2];

    // RIFF チャンク確認
    reader.read_exact(&mut buf4).map_err(|e| e.to_string())?;
    if &buf4 != b"RIFF" {
        return Err("RIFF ヘッダが見つかりません".to_string());
    }
    reader.read_exact(&mut buf4).map_err(|e| e.to_string())?; // ファイルサイズ（スキップ）
    reader.read_exact(&mut buf4).map_err(|e| e.to_string())?;
    if &buf4 != b"WAVE" {
        return Err("WAVE フォーマットではありません".to_string());
    }

    // fmt チャンク
    reader.read_exact(&mut buf4).map_err(|e| e.to_string())?;
    if &buf4 != b"fmt " {
        return Err("fmt チャンクが見つかりません".to_string());
    }
    reader.read_exact(&mut buf4).map_err(|e| e.to_string())?;
    let fmt_size = u32::from_le_bytes(buf4);
    reader.read_exact(&mut buf2).map_err(|e| e.to_string())?; // audio format
    reader.read_exact(&mut buf2).map_err(|e| e.to_string())?;
    let channels = u16::from_le_bytes(buf2);
    reader.read_exact(&mut buf4).map_err(|e| e.to_string())?;
    let sample_rate = u32::from_le_bytes(buf4);
    reader.read_exact(&mut buf4).map_err(|e| e.to_string())?; // byte rate
    reader.read_exact(&mut buf2).map_err(|e| e.to_string())?; // block align
    reader.read_exact(&mut buf2).map_err(|e| e.to_string())?;
    let bits_per_sample = u16::from_le_bytes(buf2);

    // 拡張 fmt データをスキップ
    if fmt_size > 16 {
        let extra = fmt_size - 16;
        let mut skip_buf = vec![0u8; extra as usize];
        reader.read_exact(&mut skip_buf).map_err(|e| e.to_string())?;
    }

    // data チャンクを探す
    loop {
        let mut chunk_id = [0u8; 4];
        let mut chunk_size_buf = [0u8; 4];
        reader.read_exact(&mut chunk_id).map_err(|e| e.to_string())?;
        reader.read_exact(&mut chunk_size_buf).map_err(|e| e.to_string())?;
        let chunk_size = u32::from_le_bytes(chunk_size_buf);

        if &chunk_id == b"data" {
            // data チャンク見つかった
            let mut data = vec![0u8; chunk_size as usize];
            reader.read_exact(&mut data).map_err(|e| e.to_string())?;
            let samples: Vec<i16> = data
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();
            return Ok((WavHeader { sample_rate, channels, bits_per_sample }, samples));
        } else {
            // 他のチャンクをスキップ
            let mut skip_buf = vec![0u8; chunk_size as usize];
            reader.read_exact(&mut skip_buf).map_err(|e| e.to_string())?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_check_model_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/to/model");
        assert!(!VoskTranscriber::check_model(&path));
    }

    #[test]
    fn test_check_model_existing_dir() {
        // /tmp は必ず存在する
        let path = PathBuf::from("/tmp");
        assert!(VoskTranscriber::check_model(&path));
    }

    #[test]
    fn test_new_model_not_found() {
        let path = PathBuf::from("/nonexistent/vosk-model");
        let result = VoskTranscriber::new(path);
        assert!(result.is_err());
        match result {
            Err(AppError::ModelNotFound(_)) => {}
            Err(e) => panic!("期待外エラー: {:?}", e),
            Ok(_) => panic!("エラーが返るべき"),
        }
    }

    #[test]
    fn test_model_not_found_error_code() {
        let path = PathBuf::from("/nonexistent/model");
        let result = VoskTranscriber::new(path);
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains(config::ERR_MODEL_NOT_FOUND));
    }

    #[test]
    fn test_transcript_result_fields() {
        let result = TranscriptResult {
            text: "テストテキスト".to_string(),
            confidence: 0.95,
        };
        assert_eq!(result.text, "テストテキスト");
        assert!((result.confidence - 0.95).abs() < f64::EPSILON);
    }
}
