use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

/// 音频管理器
pub struct AudioManager {
    _stream: OutputStream,
    sink: Sink,
}

impl AudioManager {
    /// 创建新的音频管理器
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        Ok(AudioManager {
            _stream,
            sink,
        })
    }

    /// 播放黑棋落子音效
    pub fn play_black_move(&self) {
        // 生成一个较低频率的音效（黑棋）
        let frequency = 220.0; // A3音符
        let duration = 0.2; // 200ms
        self.play_tone(frequency, duration, 0.3);
    }

    /// 播放白棋落子音效
    pub fn play_white_move(&self) {
        // 生成一个较高频率的音效（白棋）
        let frequency = 440.0; // A4音符
        let duration = 0.2; // 200ms
        self.play_tone(frequency, duration, 0.3);
    }

    /// 播放指定频率的音调
    fn play_tone(&self, frequency: f32, duration: f32, volume: f32) {
        // 生成正弦波音频数据
        let sample_rate = 44100;
        let samples = (sample_rate as f32 * duration) as usize;
        let mut audio_data = Vec::new();
        
        for i in 0..samples {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * volume;
            // 转换为16位PCM
            let pcm_sample = (sample * 32767.0) as i16;
            audio_data.extend_from_slice(&pcm_sample.to_le_bytes());
        }
        
        // 创建WAV格式的音频数据
        let wav_data = self.create_wav_data(&audio_data, sample_rate);
        
        // 播放音频
        let cursor = Cursor::new(wav_data);
        if let Ok(source) = Decoder::new(cursor) {
            self.sink.append(source);
        }
    }

    /// 创建WAV格式的音频数据
    fn create_wav_data(&self, pcm_data: &[u8], sample_rate: u32) -> Vec<u8> {
        let mut wav_data = Vec::new();
        
        // WAV文件头
        // RIFF header
        wav_data.extend_from_slice(b"RIFF");
        let file_size = 36 + pcm_data.len() as u32;
        wav_data.extend_from_slice(&file_size.to_le_bytes());
        wav_data.extend_from_slice(b"WAVE");
        
        // fmt chunk
        wav_data.extend_from_slice(b"fmt ");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
        wav_data.extend_from_slice(&1u16.to_le_bytes());  // audio format (PCM)
        wav_data.extend_from_slice(&1u16.to_le_bytes());  // number of channels
        wav_data.extend_from_slice(&sample_rate.to_le_bytes()); // sample rate
        let byte_rate = sample_rate * 2; // 16 bits = 2 bytes
        wav_data.extend_from_slice(&byte_rate.to_le_bytes());
        wav_data.extend_from_slice(&2u16.to_le_bytes());  // block align
        wav_data.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        
        // data chunk
        wav_data.extend_from_slice(b"data");
        wav_data.extend_from_slice(&(pcm_data.len() as u32).to_le_bytes());
        wav_data.extend_from_slice(pcm_data);
        
        wav_data
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // 如果音频初始化失败，创建一个空的实现
            // 这确保了即使在没有音频设备的情况下程序也能正常运行
            panic!("Failed to initialize audio system");
        })
    }
}

