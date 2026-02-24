use crate::config::ServerConfig;
use crate::provider::ModelProvider;
use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::post, Router};
use std::sync::{Arc, Mutex};
use tracing::info;

type SharedProvider = Arc<Mutex<Box<dyn ModelProvider>>>;

pub async fn run(config: ServerConfig, provider: Box<dyn ModelProvider>) -> Result<()> {
    let state: SharedProvider = Arc::new(Mutex::new(provider));

    let app = Router::new()
        .route("/transcribe", post(transcribe))
        .with_state(state);

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

    let provider = provider.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    provider.transcribe(&audio_data).map_err(|e| {
        tracing::error!("Transcription failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
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
