# API Reference

> **AI Context Summary**: The daemon exposes a single HTTP endpoint — `POST /transcribe` on `127.0.0.1:3030` — accepting raw WAV bytes as the request body and returning plain-text transcription. The server internally decodes WAV (F32 or I16, any sample rate, mono or stereo), resamples to 16 kHz mono f32, and dispatches to the configured ModelProvider. No authentication; localhost only by design.

## Base URL

```
http://127.0.0.1:3030
```

Configurable via `[server]` in `config.toml`. Localhost-only by design — no authentication, no TLS.

## Endpoints

### POST /transcribe

Transcribe audio from a WAV payload.

**Request**

| Property | Value |
|----------|-------|
| Method | `POST` |
| Path | `/transcribe` |
| Content-Type | `application/octet-stream` (or omit) |
| Body | Raw WAV bytes |

The WAV must be a valid RIFF WAV file. Supported input formats:

| Property | Supported values |
|----------|-----------------|
| Sample format | `F32` or `I16` |
| Channels | Mono or stereo (stereo averaged to mono) |
| Sample rate | Any (server resamples to 16 kHz internally) |

The CLI sends 16 kHz mono 16-bit PCM WAV (pre-encoded by `encode.rs`), so server-side resampling is a no-op in the common path.

**Response**

| Status | Body | Meaning |
|--------|------|---------|
| `200 OK` | Plain UTF-8 text | Successful transcription |
| `400 Bad Request` | Error message | Invalid or undecodable WAV |
| `500 Internal Server Error` | Error message | Inference failure |

Response body is `text/plain`, UTF-8. An empty string is a valid response (silence or no speech detected).

**Example — curl**

```bash
# Record 3 seconds with sox, then transcribe
sox -d -r 16000 -c 1 -b 16 -e signed-integer /tmp/test.wav trim 0 3

curl -s -X POST http://127.0.0.1:3030/transcribe \
  --data-binary @/tmp/test.wav
```

**Example — reqwest (Rust)**

```rust
// dyt-cli/src/transport.rs
let response = client
    .post(&url)
    .body(wav_bytes)
    .send()
    .await?;
let text = response.text().await?;
```

## Configuration

The daemon reads `~/.config/dyt/config.toml` at startup:

```toml
[server]
host = "127.0.0.1"  # bind address
port = 3030         # bind port

[engine]
provider = "whisper_cpp"                    # STT backend
model_path = "/path/to/ggml-model.bin"     # required
threads = 4                                 # inference threads
```

Config is loaded once at startup. Changes require a daemon restart.

**Defaults** (when fields are omitted):

| Key | Default |
|-----|---------|
| `server.host` | `"127.0.0.1"` |
| `server.port` | `3030` |
| `engine.provider` | `"whisper_cpp"` |
| `engine.threads` | `4` |

## Server Internals

The handler at `dyt-daemon/src/server.rs` processes each request as follows:

1. Read full request body as bytes
2. Decode WAV via hound — supports F32 and I16 sample formats
3. Convert stereo interleaved → mono by averaging channel pairs
4. Resample from source rate to 16 kHz (linear interpolation) if needed
5. Acquire `Arc<Mutex<Box<dyn ModelProvider>>>`
6. Call `provider.transcribe(&samples_16khz_mono_f32)`
7. Release lock, return transcript as `text/plain`

The mutex ensures only one inference runs at a time (whisper-rs is not re-entrant). Concurrent requests will queue at the mutex.

## CLI URL Override

The CLI defaults to `http://127.0.0.1:3030/transcribe`. Override with:

```bash
dyt --record --url http://127.0.0.1:3031/transcribe
```

(See `dyt-cli/src/main.rs` for clap argument definitions.)

## Health Check

There is no `/health` endpoint. To verify the daemon is running:

```bash
curl -s -o /dev/null -w "%{http_code}" -X POST \
  http://127.0.0.1:3030/transcribe --data-binary ""
# Returns 400 (empty WAV → decode error) if daemon is up
# Returns "Failed to connect" if daemon is not running
```

## Cross-References

- Architecture & data flow: `docs/architecture.md`
- ModelProvider trait: `dyt-daemon/src/provider.rs`
- Server implementation: `dyt-daemon/src/server.rs`
- Behavioral contracts (Gherkin): `specs/behavioral-contracts.md`
