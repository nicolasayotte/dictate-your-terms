# stt-daemon (Inference Engine)

## Responsibility

A persistent background process. Its sole responsibility is to hold the machine learning model hot in memory and process incoming audio arrays as fast as possible.

## Tech Stack

| Dependency   | Purpose |
|--------------|---------|
| `axum`       | Web framework. Purely functional routing paradigm for managing shared model state safely across async threads. |
| `tokio`      | Async runtime powering `axum`. |
| `whisper-rs` | Rust FFI bindings for `whisper.cpp`. Runs quantized `ggml` models on CPU or GPU without Python/PyTorch bloat. |
| `hound`      | Decodes incoming WAV bytes into 16kHz, 32-bit float (`f32`) arrays required by Whisper. |

## Architecture

On startup the daemon:
1. Reads a local configuration file (`config.toml`).
2. Initializes the configured `ModelProvider` implementation.
3. Wraps the model context in `Arc<Mutex<>>` within Axum's application state.
4. Binds to `localhost` on the configured port.
5. Exposes a single `POST /transcribe` endpoint.

Because the model remains loaded in RAM/VRAM, the cold-start penalty is eliminated; inference begins the exact millisecond the audio payload is received.

## Configuration

```toml
[server]
port = 3030
host = "127.0.0.1"

[engine]
# Can be changed to "onnx_runtime", "remote_api", or new models as they evolve
provider = "whisper_cpp"
model_path = "~/.models/ggml-base.en.bin"
threads = 4
```

## Extensibility (Trait-Based Model Provider)

To ensure the system isn't tightly coupled to Whisper, the daemon relies on a trait-based architecture.

```rust
trait ModelProvider {
    fn transcribe(&self, audio_data: &[f32]) -> Result<String, Error>;
}
```

The daemon uses `config.toml` to determine which implementation to instantiate. If a faster local model drops in the future, you write a new struct that implements `ModelProvider`, point the config to it, and restart the daemon. The API contract and the CLI remain completely untouched.

## API Contract

### `POST /transcribe`

- **Request**: Body contains a valid 16kHz mono WAV file.
- **Response (200)**: Plain text transcription string.
- **Response (400)**: Invalid or corrupt audio payload; daemon remains alive and ready.
