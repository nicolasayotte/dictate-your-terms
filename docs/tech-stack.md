# Tech Stack

> **AI Context Summary**: Rust workspace (edition 2021), two crates. Daemon: axum + tokio (async HTTP), whisper-rs (FFI to whisper.cpp), hound (WAV decode), tracing (logging), dirs (config path). CLI: cpal (cross-platform audio), ringbuf (lock-free SPSC — mandatory for real-time callback safety), reqwest (HTTP client), arboard (clipboard), hound (WAV encode), clap (args). All choices prioritize real-time safety and cross-platform correctness.

## Workspace

```toml
# Cargo.toml
[workspace]
members = ["dyt-daemon", "dyt-cli"]
resolver = "2"

[workspace.package]
version = "1.0.0"
edition = "2021"
```

## dyt-daemon Dependencies

| Crate | Version | Role |
|-------|---------|------|
| `axum` | 0.8 | HTTP framework — routes, extractors, response types |
| `tokio` | 1 (full) | Async runtime underlying axum |
| `whisper-rs` | 0.14 | Safe Rust FFI bindings to whisper.cpp (C++ STT library) |
| `hound` | 3.5 | WAV decode from `Cursor<&[u8]>` in the request handler |
| `serde` | 1 | Derive macros for config deserialization |
| `toml` | 0.8 | TOML config file parsing |
| `anyhow` | 1 | Ergonomic error propagation with context chaining |
| `tracing` | 0.1 | Structured logging macros (`info!`, `error!`, etc.) |
| `tracing-subscriber` | 0.3 | Log formatting and `RUST_LOG` env filter |
| `dirs` | 6 | XDG / platform config path resolution |

Dev dependencies: `tower` 0.5 (axum test helpers), `reqwest` 0.12 (integration test HTTP client).

## dyt-cli Dependencies

| Crate | Version | Role |
|-------|---------|------|
| `cpal` | 0.15 | Cross-platform audio capture — ALSA/PipeWire (Linux), WASAPI (Windows) |
| `ringbuf` | 0.4 | Lock-free SPSC ring buffer for the cpal audio callback |
| `reqwest` | 0.12 | Async HTTP client for POSTing WAV bytes to the daemon |
| `arboard` | 3 | Cross-platform clipboard write — X11/Wayland (Linux), Win32 (Windows) |
| `hound` | 3.5 | WAV encode to `Cursor<&mut Vec<u8>>` |
| `clap` | 4 | CLI argument parsing with derive macros |
| `anyhow` | 1 | Ergonomic error propagation |

## Key Design Choices

### ringbuf over `Arc<Mutex<Vec>>`

The cpal callback runs on a real-time OS audio thread. Mutexes can block (priority inversion when a non-RT thread holds the lock). The SPSC ring buffer's `push_slice()` is a pure atomic operation — two index reads/writes — safe under hard-real-time constraints. See `specs/audio-capture-design.md` for the full analysis.

### whisper-rs (whisper.cpp FFI) over pure-Rust models

whisper.cpp is the reference implementation with the best quantization support, SIMD acceleration, and ggml model compatibility. `whisper-rs` provides safe Rust bindings. The `ModelProvider` trait abstracts the backend, keeping it swappable without affecting the HTTP API.

### reqwest (async) in the CLI

Despite the CLI being short-lived, the reqwest async API is used because axum uses async body framing. A minimal tokio current-thread runtime runs the single POST call. This keeps the CLI dependency tree consistent with the daemon.

### hound for WAV

Pure-Rust WAV library with zero unsafe code. The in-memory encode pattern (`Cursor<&mut Vec<u8>>`) avoids temporary files. The critical gotcha: call `writer.finalize()` before reading `buf`, and never call `writer.into_inner()` when the writer was constructed with `&mut cursor`. See CLAUDE.md Notes section.

### arboard for clipboard

Handles X11, Wayland, and Win32 clipboard APIs behind a single interface without requiring external utilities (`xclip`, `xsel`) on Linux.

### dirs for config paths

Correctly resolves `~/.config/dyt/` on Linux (XDG-compliant) and `%APPDATA%\dyt\` on Windows without manual `$HOME` or `%APPDATA%` parsing.

## Rust Edition & MSRV

Edition 2021. No explicit MSRV pinned — use the latest stable Rust from rustup. The CI pipeline uses the latest stable toolchain.

## Cross-References

- Build & release: `docs/deployment.md`
- System library setup: `docs/setup.md`
- Architecture overview: `docs/architecture.md`
