use hound::WavReader;
use dyt_cli::encode::to_wav;
use std::f32::consts::PI;

#[test]
fn roundtrip_16k() {
    let source_rate = 16_000u32;
    let frequency = 440.0_f32;
    let duration_secs = 0.1_f32;
    let num_samples = (source_rate as f32 * duration_secs) as usize;

    let original: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / source_rate as f32;
            (2.0 * PI * frequency * t).sin() * 0.5
        })
        .collect();

    let wav_bytes = to_wav(&original, source_rate).unwrap();

    let cursor = std::io::Cursor::new(wav_bytes);
    let mut reader = WavReader::new(cursor).unwrap();

    let decoded: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    assert_eq!(
        decoded.len(),
        original.len(),
        "sample count must be preserved for native-rate input"
    );

    let tolerance = 1.0_f32 / i16::MAX as f32;
    for (orig, dec) in original.iter().zip(decoded.iter()) {
        assert!(
            (orig - dec).abs() <= tolerance,
            "sample out of tolerance: orig={orig}, dec={dec}"
        );
    }
}

#[test]
fn roundtrip_48k_resampled() {
    let source_rate = 48_000u32;
    let frequency = 440.0_f32;
    let duration_secs = 0.5_f32;
    let num_samples = (source_rate as f32 * duration_secs) as usize;

    let original: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / source_rate as f32;
            (2.0 * PI * frequency * t).sin() * 0.5
        })
        .collect();

    let input_duration = num_samples as f32 / source_rate as f32;
    let wav_bytes = to_wav(&original, source_rate).unwrap();

    let cursor = std::io::Cursor::new(wav_bytes);
    let mut reader = WavReader::new(cursor).unwrap();

    let decoded: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    let output_duration = decoded.len() as f32 / 16_000_f32;
    let tolerance = 0.01_f32;
    assert!(
        (output_duration - input_duration).abs() / input_duration <= tolerance,
        "duration mismatch after resampling: input={input_duration}s, output={output_duration}s"
    );
}
