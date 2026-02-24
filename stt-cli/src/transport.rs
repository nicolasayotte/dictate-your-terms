use anyhow::{Context, Result};

/// Send WAV bytes to the daemon and return the transcribed text.
pub fn transcribe(daemon_url: &str, wav_bytes: &[u8]) -> Result<String> {
    let url = format!("{daemon_url}/transcribe");

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "audio/wav")
        .body(wav_bytes.to_vec())
        .send()
        .with_context(|| format!("Failed to connect to daemon at {url}"))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Daemon returned HTTP {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        );
    }

    response.text().context("Reading response body")
}
