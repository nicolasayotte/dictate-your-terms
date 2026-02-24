use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use ringbuf::{traits::*, HeapRb};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Owns a live audio recording session. Call `stop()` to halt recording
/// and retrieve the captured samples.
pub struct RecordingSession {
    stream: cpal::Stream,
    running: Arc<AtomicBool>,
    drain_handle: std::thread::JoinHandle<Vec<f32>>,
    dropped: Arc<AtomicUsize>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Initialize audio capture: open device, create ring buffer, start cpal stream
/// and drain thread. Returns a RecordingSession that can be stopped later.
pub fn start_recording() -> Result<RecordingSession> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No input device available")?;

    let config = device
        .default_input_config()
        .context("No default input config")?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    // Pre-allocate ring buffer: 60 seconds at native sample rate
    let buf_size = sample_rate as usize * channels as usize * 60;
    let ring = HeapRb::<f32>::new(buf_size);
    let (mut producer, mut consumer) = ring.split();

    let running = Arc::new(AtomicBool::new(true));
    let dropped = Arc::new(AtomicUsize::new(0));

    let running_stream = running.clone();
    let dropped_cb = dropped.clone();
    let running_drain = running.clone();
    let channels_usize = channels as usize;

    // Drain thread: pull from ring buffer into a local Vec (Bug 1 fix: no Mutex)
    let drain_handle = std::thread::spawn(move || {
        let mut collected = Vec::new();
        let mut local_buf = vec![0.0f32; 4096];
        while running_drain.load(Ordering::Relaxed) {
            let count = consumer.pop_slice(&mut local_buf);
            if count > 0 {
                drain_to_collected(&local_buf[..count], channels_usize, &mut collected);
            } else {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        // Final drain after stop signal
        loop {
            let count = consumer.pop_slice(&mut local_buf);
            if count == 0 {
                break;
            }
            drain_to_collected(&local_buf[..count], channels_usize, &mut collected);
        }
        collected
    });

    // Build cpal input stream (Bug 2: track dropped samples, Bug 3: handle I16)
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if running_stream.load(Ordering::Relaxed) {
                    let pushed = producer.push_slice(data);
                    let missed = data.len() - pushed;
                    if missed > 0 {
                        dropped_cb.fetch_add(missed, Ordering::Relaxed);
                    }
                }
            },
            |err| eprintln!("Audio stream error: {err}"),
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if running_stream.load(Ordering::Relaxed) {
                    let pushed = producer.push_iter(
                        data.iter().map(|&s| f32::from_sample(s)),
                    );
                    let missed = data.len() - pushed;
                    if missed > 0 {
                        dropped_cb.fetch_add(missed, Ordering::Relaxed);
                    }
                }
            },
            |err| eprintln!("Audio stream error: {err}"),
            None,
        )?,
        format => anyhow::bail!("Unsupported sample format: {format:?}"),
    };

    stream.play().context("Failed to start audio stream")?;

    Ok(RecordingSession {
        stream,
        running,
        drain_handle,
        dropped,
        sample_rate,
        channels,
    })
}

impl RecordingSession {
    /// Stop recording, join the drain thread, and return (samples, sample_rate).
    /// Reports dropped samples to stderr if any occurred.
    pub fn stop(self) -> Result<(Vec<f32>, u32)> {
        self.running.store(false, Ordering::Relaxed);
        drop(self.stream);

        let samples = self
            .drain_handle
            .join()
            .map_err(|_| anyhow::anyhow!("Drain thread panicked"))?;

        let dropped_count = self.dropped.load(Ordering::Relaxed);
        if dropped_count > 0 {
            eprintln!(
                "Warning: {dropped_count} audio samples were dropped due to ring buffer overrun"
            );
        }

        Ok((samples, self.sample_rate))
    }
}

/// Record audio from the default input device until Enter is pressed.
/// Returns the raw f32 samples (mono, native rate) and the device's native sample rate.
pub fn record_until_enter() -> Result<(Vec<f32>, u32)> {
    let session = start_recording()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    session.stop()
}

/// Record audio from the default input device for a fixed duration.
/// Returns the raw f32 samples (mono, native rate) and the device's native sample rate.
pub fn record_for_duration(duration: Duration) -> Result<(Vec<f32>, u32)> {
    let session = start_recording()?;
    std::thread::sleep(duration);
    session.stop()
}

/// Drain interleaved audio samples into a mono collected buffer.
/// Runs in the drain thread only — never in the cpal callback.
fn drain_to_collected(buf: &[f32], channels: usize, collected: &mut Vec<f32>) {
    if channels > 1 {
        for frame in buf.chunks(channels) {
            let mono: f32 = frame.iter().copied().sum::<f32>() / channels as f32;
            collected.push(mono);
        }
    } else {
        collected.extend_from_slice(buf);
    }
}
