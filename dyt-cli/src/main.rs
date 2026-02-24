use stt_cli::{capture, encode, transport};

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "dyt", about = "DictateYourTerms — voice to clipboard")]
struct Cli {
    /// Start recording from the default microphone
    #[arg(long)]
    record: bool,

    /// Daemon address
    #[arg(long, default_value = "http://127.0.0.1:3030")]
    daemon: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if !cli.record {
        eprintln!("Usage: dyt --record");
        std::process::exit(1);
    }

    eprintln!("Recording... press Enter to stop.");
    let (samples, sample_rate) = capture::record_until_enter()?;
    eprintln!("Captured {} samples at {}Hz", samples.len(), sample_rate);

    let wav_bytes = encode::to_wav(&samples, sample_rate)?;
    eprintln!("Encoded {} bytes of WAV", wav_bytes.len());

    let text = transport::transcribe(&cli.daemon, &wav_bytes)?;
    eprintln!("Transcribed: {text}");

    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(&text)?;
    eprintln!("Copied to clipboard.");

    // Also print to stdout for piping
    print!("{text}");

    Ok(())
}
