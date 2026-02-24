use crate::config::ServerConfig;
use crate::provider::ModelProvider;
use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::post, Router};
use std::sync::{Arc, Mutex};
use tracing::info;

type SharedProvider = Arc<Mutex<Box<dyn ModelProvider>>>;

pub fn build_router(provider: Box<dyn ModelProvider>) -> Router {
    let state: SharedProvider = Arc::new(Mutex::new(provider));
    Router::new()
        .route("/transcribe", post(transcribe))
        .with_state(state)
}

pub async fn run(config: ServerConfig, provider: Box<dyn ModelProvider>) -> Result<()> {
    let app = build_router(provider);
    let addr = format!("{}:{}", config.host, config.port);
    info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn transcribe(
    State(provider): State<SharedProvider>,
    body: axum::body::Bytes,
) -> Result<String, StatusCode> {
    let audio_data = decode_wav(&body).map_err(|e| {
        tracing::warn!("Bad audio payload: {e}");
        StatusCode::BAD_REQUEST
    })?;

    let provider_clone = Arc::clone(&provider);
    let result = tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        let guard = provider_clone
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let transcription = guard.transcribe(&audio_data).map_err(|e| {
            tracing::error!("Transcription failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let elapsed = start.elapsed();
        tracing::info!("Inference completed in {:.1}ms", elapsed.as_secs_f64() * 1000.0);
        Ok::<String, StatusCode>(transcription)
    })
    .await
    .map_err(|e| {
        tracing::error!("spawn_blocking task panicked: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    result
}

/// Decode WAV bytes into 16kHz mono f32 samples.
fn decode_wav(bytes: &[u8]) -> Result<Vec<f32>> {
    let cursor = std::io::Cursor::new(bytes);
    let mut reader = hound::WavReader::new(cursor)?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().filter_map(|s| s.ok()).collect(),
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / i16::MAX as f32)
            .collect(),
    };

    // Convert to mono if stereo
    let mono = if spec.channels > 1 {
        samples
            .chunks(spec.channels as usize)
            .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
            .collect()
    } else {
        samples
    };

    // Resample to 16kHz if needed (simple linear interpolation)
    let target_rate = 16000u32;
    if spec.sample_rate == target_rate {
        return Ok(mono);
    }

    let ratio = spec.sample_rate as f64 / target_rate as f64;
    let output_len = (mono.len() as f64 / ratio) as usize;
    let mut resampled = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let sample = if idx + 1 < mono.len() {
            mono[idx] * (1.0 - frac as f32) + mono[idx + 1] * frac as f32
        } else {
            mono[idx.min(mono.len() - 1)]
        };
        resampled.push(sample);
    }

    Ok(resampled)
}

#[cfg(test)]
mod tests {
    use super::decode_wav;

    fn make_wav_f32(sample_rate: u32, channels: u16, samples: &[f32]) -> Vec<u8> {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
            for &s in samples {
                writer.write_sample(s).unwrap();
            }
            writer.finalize().unwrap();
        }
        cursor.into_inner()
    }

    fn make_wav_i16(sample_rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
            for &s in samples {
                writer.write_sample(s).unwrap();
            }
            writer.finalize().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn decode_f32_16k_mono() {
        let input = vec![0.1_f32, 0.5, -0.3, 0.0, 1.0];
        let wav = make_wav_f32(16000, 1, &input);
        let output = decode_wav(&wav).unwrap();
        assert_eq!(output.len(), input.len());
        for (a, b) in output.iter().zip(input.iter()) {
            assert!((a - b).abs() < 1e-5, "sample mismatch: got {a}, want {b}");
        }
    }

    #[test]
    fn decode_i16_16k_mono() {
        let input_i16 = vec![i16::MAX, 0, i16::MIN / 2];
        let wav = make_wav_i16(16000, 1, &input_i16);
        let output = decode_wav(&wav).unwrap();
        assert_eq!(output.len(), 3);
        assert!((output[0] - 1.0).abs() < 1e-4, "expected ~1.0, got {}", output[0]);
        assert!(output[1].abs() < 1e-4, "expected ~0.0, got {}", output[1]);
        assert!((output[2] - (-0.5)).abs() < 1e-3, "expected ~-0.5, got {}", output[2]);
    }

    #[test]
    fn decode_stereo_to_mono() {
        let stereo: Vec<f32> = std::iter::repeat([1.0_f32, -1.0_f32])
            .take(10)
            .flatten()
            .collect();
        let wav = make_wav_f32(16000, 2, &stereo);
        let output = decode_wav(&wav).unwrap();
        assert_eq!(output.len(), 10, "expected 10 mono frames from 10 stereo frames");
        for (i, &s) in output.iter().enumerate() {
            assert!(s.abs() < 1e-5, "frame {i}: expected 0.0, got {s}");
        }
    }

    #[test]
    fn decode_resamples_48k() {
        let input: Vec<f32> = (0..480).map(|i| (i as f32 / 480.0).sin()).collect();
        let wav = make_wav_f32(48000, 1, &input);
        let output = decode_wav(&wav).unwrap();
        let expected = 160usize;
        assert!(
            (output.len() as isize - expected as isize).abs() <= 2,
            "expected ~{expected} samples, got {}",
            output.len()
        );
    }

    #[test]
    fn decode_invalid_bytes() {
        let garbage = b"not a wav file at all xxxx".to_vec();
        let result = decode_wav(&garbage);
        assert!(result.is_err(), "expected Err for invalid WAV bytes, got Ok");
    }
}
