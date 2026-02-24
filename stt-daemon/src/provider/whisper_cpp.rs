use crate::config::EngineConfig;
use anyhow::{Context, Result};
use tracing::info;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperCppProvider {
    ctx: WhisperContext,
    threads: u16,
}

impl WhisperCppProvider {
    pub fn new(engine: &EngineConfig) -> Result<Self> {
        let model_path = shellexpand_path(&engine.model_path);
        info!("Loading whisper model from {model_path}");

        let ctx = WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())
            .context("Failed to load whisper model")?;

        Ok(Self {
            ctx,
            threads: engine.threads,
        })
    }
}

impl super::ModelProvider for WhisperCppProvider {
    fn transcribe(&self, audio_data: &[f32]) -> Result<String> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(self.threads as i32);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);

        let mut state = self.ctx.create_state().context("creating whisper state")?;
        state
            .full(params, audio_data)
            .context("whisper inference failed")?;

        let num_segments = state.full_n_segments().context("getting segment count")?;
        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        Ok(text.trim().to_string())
    }
}

/// Expand ~ to the user's home directory.
fn shellexpand_path(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped).to_string_lossy().to_string();
        }
    }
    path.to_string()
}
