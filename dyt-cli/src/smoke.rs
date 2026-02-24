use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "dyt-smoke", about = "Audio pipeline smoke test for DictateYourTerms")]
struct Args {
    /// Recording duration in seconds
    #[arg(long, default_value = "3")]
    duration: u64,

    /// Output WAV file path
    #[arg(long, default_value = "dyt-smoke-output.wav")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let duration = Duration::from_secs(args.duration);

    eprintln!("Recording for {} seconds...", args.duration);
    let (samples, sample_rate) = dyt_cli::capture::record_for_duration(duration)?;

    // Report capture stats
    let actual_duration = samples.len() as f64 / sample_rate as f64;
    eprintln!(
        "Captured {} samples at {} Hz ({:.2}s)",
        samples.len(),
        sample_rate,
        actual_duration,
    );

    // Compute and report RMS level
    if samples.is_empty() {
        eprintln!("Warning: no samples captured");
    } else {
        let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
        let rms = (sum_sq / samples.len() as f64).sqrt();

        if rms == 0.0 {
            eprintln!("Warning: all samples are silent — mic may be muted or wrong device selected");
        } else {
            let dbfs = 20.0 * rms.log10();
            eprintln!("RMS level: {:.1} dBFS", dbfs);
            if dbfs < -40.0 {
                eprintln!(
                    "Warning: signal is very quiet — check mic selection or mute status"
                );
            }
        }
    }

    // Encode and save
    let wav_bytes = dyt_cli::encode::to_wav(&samples, sample_rate)?;
    std::fs::write(&args.output, &wav_bytes)?;
    eprintln!("Saved {} bytes to {}", wav_bytes.len(), args.output.display());
    eprintln!("Play back with: aplay {}", args.output.display());

    Ok(())
}
