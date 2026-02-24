use criterion::{Criterion, criterion_group, criterion_main};
use std::path::PathBuf;

fn load_fixture() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../benches/fixtures/sample.wav");
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to read WAV fixture at {}: {e}\n\
             Record a sample first: cargo run --bin dyt-record-fixture",
            path.display()
        )
    })
}

fn bench_transcribe_round_trip(c: &mut Criterion) {
    let wav_bytes = load_fixture();

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .expect("failed to build HTTP client");

    // Verify daemon is reachable before entering the benchmark loop
    let probe = client
        .post("http://127.0.0.1:3030/transcribe")
        .header("Content-Type", "audio/wav")
        .body(wav_bytes.clone())
        .send();

    if let Err(e) = probe {
        panic!(
            "Cannot reach dyt-daemon at 127.0.0.1:3030: {e}\n\
             Start the daemon first: cargo run -p dyt-daemon"
        );
    }

    let mut group = c.benchmark_group("transcribe");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(20));

    group.bench_function("http_round_trip", |b| {
        b.iter(|| {
            let response = client
                .post("http://127.0.0.1:3030/transcribe")
                .header("Content-Type", "audio/wav")
                .body(wav_bytes.clone())
                .send()
                .expect("HTTP request to daemon failed");

            assert!(
                response.status().is_success(),
                "daemon returned HTTP {}",
                response.status()
            );

            response.text().expect("failed to read response body")
        })
    });

    group.finish();
}

criterion_group!(benches, bench_transcribe_round_trip);
criterion_main!(benches);
