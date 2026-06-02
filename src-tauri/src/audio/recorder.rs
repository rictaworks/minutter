use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use log::{debug, error, info};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config;
use crate::error::AppError;

/// 録音状態を保持する内部構造体
struct RecordingState {
    /// 収集した PCM サンプル（i16）
    samples: Vec<i16>,
    /// 書き込み済みバイト数
    bytes_written: u64,
    /// 最大バイト数超過フラグ
    overflow: bool,
}

impl RecordingState {
    fn new() -> Self {
        RecordingState {
            samples: Vec::new(),
            bytes_written: 0,
            overflow: false,
        }
    }
}

/// マイク録音を管理するクラス
/// `cpal::Stream` は Send でないため録音状態は Arc<Mutex<...>> で管理する
pub struct AudioRecorder {
    device_index: usize,
    output_path: PathBuf,
    max_bytes: u64,
    state: Arc<Mutex<RecordingState>>,
    stream: Option<Stream>,
}

impl AudioRecorder {
    /// 新しい AudioRecorder を作成する
    pub fn new(output_path: PathBuf) -> Self {
        debug!("AudioRecorder 作成: {:?}", output_path);
        AudioRecorder {
            device_index: 0,
            output_path,
            max_bytes: config::MAX_AUDIO_BYTES,
            state: Arc::new(Mutex::new(RecordingState::new())),
            stream: None,
        }
    }

    /// デバイスインデックスを選択する
    pub fn select_device(&mut self, index: usize) {
        debug!("録音デバイス選択: index={}", index);
        self.device_index = index;
    }

    /// 利用可能な入力デバイス名の一覧を返す
    pub fn list_devices() -> Vec<String> {
        debug!("録音デバイス一覧取得");
        let host = cpal::default_host();
        match host.input_devices() {
            Ok(devices) => {
                let names: Vec<String> = devices
                    .filter_map(|d| d.name().ok())
                    .collect();
                info!("録音デバイス {} 件取得", names.len());
                names
            }
            Err(e) => {
                error!("録音デバイス取得失敗: {}", e);
                Vec::new()
            }
        }
    }

    /// 録音を開始する
    pub fn start(&mut self) -> Result<(), AppError> {
        info!("録音開始: device_index={}", self.device_index);

        let host = cpal::default_host();
        let device = self.select_input_device(&host)?;

        let config = self.build_stream_config(&device)?;
        debug!("ストリーム設定: {:?}", config);

        let state = Arc::clone(&self.state);
        let max_bytes = self.max_bytes;

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mut st = match state.lock() {
                        Ok(s) => s,
                        Err(e) => {
                            error!("録音状態ロック失敗: {}", e);
                            return;
                        }
                    };
                    if st.overflow {
                        return;
                    }
                    let byte_size = (data.len() * 2) as u64;
                    if st.bytes_written + byte_size > max_bytes {
                        error!("録音ファイルサイズ上限超過: {} bytes", max_bytes);
                        st.overflow = true;
                        return;
                    }
                    st.samples.extend_from_slice(data);
                    st.bytes_written += byte_size;
                },
                move |err| {
                    error!("録音ストリームエラー: {}", err);
                },
                None,
            )
            .map_err(|e| {
                error!("ストリーム構築失敗: {}", e);
                AppError::Internal(e.to_string())
            })?;

        stream.play().map_err(|e| {
            error!("ストリーム再生開始失敗: {}", e);
            AppError::Internal(e.to_string())
        })?;

        self.stream = Some(stream);
        info!("録音ストリーム開始完了");
        Ok(())
    }

    /// 録音を停止し WAV ファイルを書き出す
    pub fn stop(&mut self) -> Result<PathBuf, AppError> {
        info!("録音停止");

        // ストリームをドロップして停止
        self.stream = None;

        let state = self.state.lock().map_err(|e| {
            error!("録音状態ロック失敗: {}", e);
            AppError::Internal(e.to_string())
        })?;

        if state.overflow {
            return Err(AppError::AudioTooLarge(state.bytes_written));
        }

        let samples = &state.samples;
        debug!("サンプル数: {}", samples.len());

        self.write_wav(samples)?;

        info!("録音停止完了: {:?}", self.output_path);
        Ok(self.output_path.clone())
    }

    /// 入力デバイスを選択する
    fn select_input_device(&self, host: &cpal::Host) -> Result<Device, AppError> {
        let devices: Vec<Device> = host
            .input_devices()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .collect();

        if devices.is_empty() {
            return Err(AppError::Internal("入力デバイスが見つかりません".to_string()));
        }

        if self.device_index < devices.len() {
            debug!("デバイス選択: index={}", self.device_index);
            // index でアクセスするために再度列挙する（consume されているため）
            let device = host
                .input_devices()
                .map_err(|e| AppError::Internal(e.to_string()))?
                .nth(self.device_index)
                .ok_or_else(|| AppError::Internal(format!("デバイスインデックス {} が範囲外", self.device_index)))?;
            Ok(device)
        } else {
            // デフォルトデバイスにフォールバックしない → エラーを返す
            Err(AppError::Internal(format!(
                "デバイスインデックス {} が範囲外（デバイス数 {}）",
                self.device_index,
                devices.len()
            )))
        }
    }

    /// ストリーム設定を構築する（16kHz モノラル i16 固定）
    fn build_stream_config(&self, device: &Device) -> Result<StreamConfig, AppError> {
        let supported = device
            .default_input_config()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        debug!("デフォルト設定: {:?}", supported);

        // Vosk 推奨: 16kHz モノラル
        // cpal が 16kHz をサポートしていない場合はデフォルトレートを使用
        let sample_rate = if supported.sample_rate().0 >= config::RECORDING_SAMPLE_RATE {
            cpal::SampleRate(config::RECORDING_SAMPLE_RATE)
        } else {
            supported.sample_rate()
        };

        Ok(StreamConfig {
            channels: config::RECORDING_CHANNELS,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        })
    }

    /// PCM サンプルを WAV ファイルに書き込む
    fn write_wav(&self, samples: &[i16]) -> Result<(), AppError> {
        debug!("WAV 書き込み開始: {:?}", self.output_path);

        let file = File::create(&self.output_path).map_err(|e| {
            error!("WAV ファイル作成失敗: {}", e);
            AppError::Io(e)
        })?;
        let mut writer = BufWriter::new(file);

        let num_samples = samples.len() as u32;
        let num_channels = config::RECORDING_CHANNELS as u32;
        let sample_rate = config::RECORDING_SAMPLE_RATE;
        let bits_per_sample = config::WAV_BITS_PER_SAMPLE as u32;
        let byte_rate = sample_rate * num_channels * (bits_per_sample / 8);
        let block_align = num_channels * (bits_per_sample / 8);
        let data_size = num_samples * (bits_per_sample / 8);

        write_wav_header(
            &mut writer,
            sample_rate,
            config::RECORDING_CHANNELS,
            config::WAV_BITS_PER_SAMPLE,
            byte_rate as u16,
            block_align as u16,
            data_size,
        )?;

        use std::io::Write;
        for sample in samples {
            writer.write_all(&sample.to_le_bytes()).map_err(|e| {
                error!("WAV データ書き込み失敗: {}", e);
                AppError::Io(e)
            })?;
        }
        writer.flush().map_err(|e| {
            error!("WAV バッファフラッシュ失敗: {}", e);
            AppError::Io(e)
        })?;

        debug!("WAV 書き込み完了: {} サンプル", num_samples);
        Ok(())
    }
}

/// WAV ヘッダを書き込む
fn write_wav_header(
    writer: &mut BufWriter<File>,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    byte_rate: u16,
    block_align: u16,
    data_size: u32,
) -> Result<(), AppError> {
    use std::io::Write;

    let riff_size = 36 + data_size;

    // RIFF チャンク
    writer.write_all(b"RIFF").map_err(AppError::Io)?;
    writer.write_all(&riff_size.to_le_bytes()).map_err(AppError::Io)?;
    writer.write_all(b"WAVE").map_err(AppError::Io)?;

    // fmt チャンク
    writer.write_all(b"fmt ").map_err(AppError::Io)?;
    writer.write_all(&16u32.to_le_bytes()).map_err(AppError::Io)?; // チャンクサイズ
    writer.write_all(&1u16.to_le_bytes()).map_err(AppError::Io)?;  // PCM
    writer.write_all(&channels.to_le_bytes()).map_err(AppError::Io)?;
    writer.write_all(&sample_rate.to_le_bytes()).map_err(AppError::Io)?;
    writer.write_all(&(byte_rate as u32).to_le_bytes()).map_err(AppError::Io)?;
    writer.write_all(&block_align.to_le_bytes()).map_err(AppError::Io)?;
    writer.write_all(&bits_per_sample.to_le_bytes()).map_err(AppError::Io)?;

    // data チャンク
    writer.write_all(b"data").map_err(AppError::Io)?;
    writer.write_all(&data_size.to_le_bytes()).map_err(AppError::Io)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_list_devices_returns_vec() {
        // デバイスが 0 件でもパニックしないこと
        let devices = AudioRecorder::list_devices();
        // Vec<String> が返れば OK（デバイス数は環境依存）
        let _ = devices.len();
    }

    #[test]
    fn test_new_recorder_default_max_bytes() {
        let recorder = AudioRecorder::new(PathBuf::from("/tmp/test.wav"));
        assert_eq!(recorder.max_bytes, config::MAX_AUDIO_BYTES);
    }

    #[test]
    fn test_select_device_sets_index() {
        let mut recorder = AudioRecorder::new(PathBuf::from("/tmp/test.wav"));
        recorder.select_device(2);
        assert_eq!(recorder.device_index, 2);
    }

    #[test]
    fn test_recording_state_initial() {
        let state = RecordingState::new();
        assert!(state.samples.is_empty());
        assert_eq!(state.bytes_written, 0);
        assert!(!state.overflow);
    }
}
