use axum::body::Body;
use axum::http::{Request, StatusCode};
use stt_daemon::provider::ModelProvider;
use tower::ServiceExt;

struct MockProvider {
    reply: &'static str,
}

impl ModelProvider for MockProvider {
    fn transcribe(&self, _: &[f32]) -> anyhow::Result<String> {
        Ok(self.reply.to_string())
    }
}

struct FailProvider;

impl ModelProvider for FailProvider {
    fn transcribe(&self, _: &[f32]) -> anyhow::Result<String> {
        anyhow::bail!("model error")
    }
}

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
    for _ in 0..1600 {
        writer.write_sample(0i16).unwrap();
    }
    writer.finalize().unwrap();
    buf
}

#[tokio::test]
async fn transcribe_ok() {
    let mock = MockProvider { reply: "hello" };
    let app = stt_daemon::server::build_router(Box::new(mock));

    let wav = make_wav_bytes();
    let request = Request::builder()
        .method("POST")
        .uri("/transcribe")
        .body(Body::from(wav))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"hello");
}

#[tokio::test]
async fn transcribe_bad_body() {
    let mock = MockProvider { reply: "irrelevant" };
    let app = stt_daemon::server::build_router(Box::new(mock));

    let request = Request::builder()
        .method("POST")
        .uri("/transcribe")
        .body(Body::from(&b"not a wav"[..]))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn transcribe_provider_error() {
    let app = stt_daemon::server::build_router(Box::new(FailProvider));

    let wav = make_wav_bytes();
    let request = Request::builder()
        .method("POST")
        .uri("/transcribe")
        .body(Body::from(wav))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
