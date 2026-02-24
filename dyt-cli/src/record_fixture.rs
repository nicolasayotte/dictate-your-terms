use anyhow::Result;
use clap::Parser;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "dyt-record-fixture", about = "Record a WAV fixture for benchmarks")]
struct Args {
    /// Recording duration in seconds
    #[arg(long, default_value = "5")]
    duration: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Locate workspace root
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| std::path::PathBuf::from(d).parent().unwrap().to_path_buf())
        .unwrap_or_else(|_| std::env::current_dir().expect("cannot determine current directory"));
    let output_path = workspace_root.join("benches").join("fixtures").join("sample.wav");

    // Create output directory
    std::fs::create_dir_all(output_path.parent().unwrap())?;

    eprintln!("Recording for {} seconds...", args.duration);
    let (samples, sample_rate) = dyt_cli::capture::record_for_duration(Duration::from_secs(args.duration))?;

    // Report capture stats
    let actual_duration = samples.len() as f64 / sample_rate as f64;
    eprintln!(
        "Captured {} samples at {} Hz ({:.2}s)",
        samples.len(),
        sample_rate,
        actual_duration,
    );

    // Encode and save
    let wav_bytes = dyt_cli::encode::to_wav(&samples, sample_rate)?;
    std::fs::write(&output_path, &wav_bytes)?;
    eprintln!(
        "Saved to {} ({} bytes, {:.2}s)",
        output_path.display(),
        wav_bytes.len(),
        actual_duration,
    );

    Ok(())
}
