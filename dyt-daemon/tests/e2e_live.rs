fn make_wav_bytes() -> Vec<u8> {
    let mut buf = Vec::new();
    let cursor = std::io::Cursor::new(&mut buf);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::new(cursor, spec).unwrap();
    for _ in 0..16000 {
        writer.write_sample(0i16).unwrap();
    }
    writer.finalize().unwrap();
    buf
}

#[tokio::test]
#[ignore]
async fn test_live_transcription() {
    let wav = make_wav_bytes();

    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:3030/transcribe")
        .body(wav)
        .send()
        .await
        .expect("failed to connect to dyt-daemon — is it running on 127.0.0.1:3030?");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "daemon returned non-200: {}",
        response.status()
    );
}
