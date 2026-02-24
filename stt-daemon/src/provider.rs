mod whisper_cpp;

use crate::config::EngineConfig;
use anyhow::{bail, Result};

/// Trait contract for swappable inference backends.
/// Implement this for any new STT engine and register it in `from_config`.
pub trait ModelProvider: Send + Sync {
    fn transcribe(&self, audio_data: &[f32]) -> Result<String>;
}

/// Instantiate the configured ModelProvider implementation.
pub fn from_config(engine: &EngineConfig) -> Result<Box<dyn ModelProvider>> {
    match engine.provider.as_str() {
        "whisper_cpp" => {
            let provider = whisper_cpp::WhisperCppProvider::new(engine)?;
            Ok(Box::new(provider))
        }
        other => bail!("Unknown engine provider: {other}"),
    }
}
