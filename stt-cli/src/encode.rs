use anyhow::Result;

/// Encode raw f32 mono samples as a WAV byte buffer (16kHz, 16-bit PCM).
/// Resamples from `source_rate` to 16kHz if needed.
pub fn to_wav(samples: &[f32], source_rate: u32) -> Result<Vec<u8>> {
    let target_rate = 16000u32;

    let resampled = if source_rate == target_rate {
        samples.to_vec()
    } else {
        resample(samples, source_rate, target_rate)
    };

    let mut buf = Vec::new();
    let cursor = std::io::Cursor::new(&mut buf);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: target_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::new(cursor, spec)?;
    for &sample in &resampled {
        let clamped = sample.clamp(-1.0, 1.0);
        writer.write_sample((clamped * i16::MAX as f32) as i16)?;
    }
    writer.finalize()?;

    Ok(buf)
}

/// Simple linear-interpolation resampler.
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = (src_idx - idx as f64) as f32;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac) + samples[idx + 1] * frac
        } else {
            samples[idx.min(samples.len() - 1)]
        };
        output.push(sample);
    }

    output
}
