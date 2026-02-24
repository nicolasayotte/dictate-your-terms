# Architecture

> **AI Context Summary**: DictateYourTerms is a two-process voice pipeline — a persistent `dyt-daemon` (axum + whisper-rs) holds the ML model hot in RAM while a transient `dyt` CLI (cpal + ringbuf) captures audio and POSTs WAV bytes over localhost HTTP. The key constraint is real-time safety in the cpal audio callback: zero locks, zero allocations. `ModelProvider` is the sole extension point for adding STT backends.

## Overview

DictateYourTerms uses a deliberate two-process split to eliminate model-loading latency. Loading a whisper model takes seconds; the CLI is cheap to start and kill on every hotkey press. The daemon pays the startup cost once and amortizes it across every invocation.

```
Hotkey
  │  ▼
┌─────────────────────────────┐    POST /transcribe (WAV bytes)    ┌──────────────────────────────┐
│         dyt (CLI)           │ ─────────────────────────────────> │        dyt-daemon            │
│                             │                                     │                              │
│  cpal callback              │ <──────── plain text ────────────  │  axum + tokio                │
│    └─> ringbuf Producer     │                                     │  Arc<Mutex<ModelProvider>>   │
│                             │                                     │  whisper-rs                  │
│  drain thread               │                                     │                              │
│    └─> ringbuf Consumer     │                                     │  handler:                    │
│    └─> mono conversion      │                                     │    decode WAV → f32          │
│    └─> Vec<f32> accumulate  │                                     │    spawn_blocking            │
│    lock + run inference      │
│                             │                                     │    return plain text         │
│  encode.rs                  │                                     │                              │
│    └─> resample to 16kHz    │                                     │                              │
│    └─> hound WAV encode     │                                     │                              │
│                             │                                     │                              │
│  transport.rs               │                                     │                              │
│    └─> reqwest POST         │                                     │                              │
│                             │                                     │                              │
│  arboard clipboard inject   │                                     │                              │
│  stdout print               │                                     │                              │
└─────────────────────────────┘                                     └──────────────────────────────┘
```

## Data Flow

1. **Capture** — cpal registers an audio callback on the OS audio thread. The callback holds only the `Producer` half of a ringbuf SPSC buffer and calls `push_slice()` — a pure atomic operation with no locks or allocations.

2. **Drain** — A dedicated drain thread holds the `Consumer` half. It polls the ring buffer, converts multi-channel interleaved samples to mono, and accumulates into a `Vec<f32>`.

3. **Stop signal** — The main CLI thread waits for Enter key (`record_until_enter()`) or a timeout (`record_for_duration()`), then signals the drain thread to stop.

4. **Encode** — `encode::to_wav()` resamples `Vec<f32>` from the device sample rate to 16 kHz using linear interpolation, then hound-encodes to 16-bit PCM WAV in a `Vec<u8>`.

5. **Transport** — `transport::post_wav()` POSTs the WAV bytes as the request body to `POST http://127.0.0.1:3030/transcribe` via reqwest.

6. **Server decode** — The axum handler reads the body, decodes WAV via hound (supports F32 and I16), converts stereo→mono, and resamples to 16 kHz if needed.

7. **Inference** — Handler clones the `Arc`, dispatches to `tokio::task::spawn_blocking`, acquires the mutex on a dedicated blocking thread, and calls `transcribe(&[f32])`. Inference duration is logged at INFO level.

8. **Response** — Returns plain-text transcription as `text/plain`.

9. **Clipboard** — CLI writes transcript to system clipboard via arboard and prints to stdout.

## Component Map

| Component | Location | Responsibility |
|-----------|----------|----------------|
| CLI entry | `dyt-cli/src/main.rs` | clap args, orchestrate capture→encode→transport→clipboard |
| Audio capture | `dyt-cli/src/capture.rs` | cpal setup, ringbuf SPSC, drain thread, mono conversion |
| WAV encode | `dyt-cli/src/encode.rs` | Resample to 16kHz, hound 16-bit PCM WAV encode |
| HTTP transport | `dyt-cli/src/transport.rs` | reqwest POST to daemon |
| Daemon entry | `dyt-daemon/src/main.rs` | Load config, create ModelProvider, start server |
| HTTP server | `dyt-daemon/src/server.rs` | axum routes, WAV decode, dispatch to ModelProvider |
| Config | `dyt-daemon/src/config.rs` | TOML deserialization, defaults, path resolution |
| Provider registry | `dyt-daemon/src/provider.rs` | ModelProvider trait + factory `from_config()` |
| whisper.cpp backend | `dyt-daemon/src/provider/whisper_cpp.rs` | whisper-rs FFI, default inference backend |

## ModelProvider Extension Point

`ModelProvider` is the **only** sanctioned extension point for new STT backends (`dyt-daemon/src/provider.rs:8`):

```rust
pub trait ModelProvider: Send + Sync {
    fn transcribe(&self, audio_data: &[f32]) -> Result<String>;
}
```

Input is always 16 kHz mono f32 — the server normalizes before dispatch. To add a backend:

1. Create `dyt-daemon/src/provider/my_engine.rs` implementing `ModelProvider`
2. Add `mod my_engine;` to `provider.rs`
3. Match `"my_engine"` in `from_config()` and construct your provider
4. Add any new `[engine]` config fields to `config.rs`

## Audio Callback Constraint

The cpal callback runs on the OS audio thread under real-time scheduling. Violations cause audible glitches or callback termination. **The callback must never**:

- Acquire any mutex (priority inversion risk)
- Allocate or free heap memory
- Perform system calls (I/O, sleep)

The ringbuf SPSC design enforces this: the callback holds only the `Producer` and calls `push_slice()` — two atomic index reads/writes, nothing else. See `specs/audio-capture-design.md` for the design rationale.

## Configuration

| Platform | Config path |
|----------|-------------|
| Linux (XDG) | `~/.config/dyt/config.toml` |
| Windows | `%APPDATA%\dyt\config.toml` |

```toml
[server]
host = "127.0.0.1"  # default
port = 3030          # default

[engine]
provider = "whisper_cpp"               # default
model_path = "/path/to/ggml-model.bin" # required
threads = 4                            # optional — defaults to available CPU core count
```

## Process Lifecycle

```
system boot
  └─> dyt-daemon start (once, manually or via systemd/Task Scheduler)
        └─> load config
        └─> initialize ModelProvider (load model into RAM — takes seconds)
        └─> bind 127.0.0.1:3030
        └─> await requests indefinitely

hotkey press
  └─> dyt --record (transient, ~100ms startup)
        └─> open mic via cpal
        └─> record until Enter key
        └─> encode WAV
        └─> POST to daemon
        └─> receive transcript
        └─> inject clipboard
        └─> exit
```

## Cross-Platform Design

| Concern | Linux | Windows |
|---------|-------|---------|
| Audio | ALSA / PipeWire via cpal | WASAPI via cpal |
| Clipboard | X11 / Wayland via arboard | Win32 clipboard via arboard |
| Config path | `~/.config/dyt/` via `dirs` crate | `%APPDATA%\dyt\` via `dirs` crate |
| Release binary | ubuntu-22.04 runner, dynamic glibc | windows-latest, MSVC toolchain |
| BLAS acceleration | auto-detected via pkg-config | OpenBLAS via vcpkg (`BLAS_INCLUDE_DIRS` required) |

## Cross-References

- HTTP API reference: `docs/api.md`
- Setup & system dependencies: `docs/setup.md`
- Tech stack details: `docs/tech-stack.md`
- Audio capture design rationale: `specs/audio-capture-design.md`
- Behavioral contracts (Gherkin): `specs/behavioral-contracts.md`
