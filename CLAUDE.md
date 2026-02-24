
# Project Context

**Mission:** Zero-latency voice pipeline that translates spoken architectural intent into terminal-native text — keeping developer hands on the keyboard during agentic programming sessions.

DictateYourTerms (dyt) — a Rust Cargo workspace with two crates:
- `dyt-daemon`: Persistent inference server (axum + whisper-rs). Keeps the STT model hot in RAM.
- `dyt-cli`: Transient capture client (cpal + ringbuf). Records mic, sends to daemon, copies result to clipboard.

Binary names: `dyt-daemon`, `dyt`

## About This Project

Rust workspace (edition 2021). Daemon exposes `POST /transcribe` on `127.0.0.1:3030`, accepting raw WAV bytes and returning plain-text transcription. CLI captures audio into a lock-free ring buffer, WAV-encodes it, POSTs to daemon, and injects the transcript into the clipboard.

## Key Directories

- `dyt-daemon/src/` — Daemon: config loading, axum server, ModelProvider trait
- `dyt-daemon/src/provider/` — Backend implementations (`whisper_cpp.rs`); add new engines here
- `dyt-cli/src/` — CLI: `capture.rs` (cpal), `encode.rs` (hound), `transport.rs` (reqwest), `main.rs` (clap)
- `config/` — Example `config.toml` for the daemon
- `scripts/` — Interactive setup scripts (`setup.sh`, `setup.ps1`)
- `specs/` — Architecture specs and behavioral contracts (authoritative design decisions)
- `docs/` — Integration guides and design notes

## Commands

```bash
cargo check                          # Type-check without building
cargo build --release                # Build both crates
cargo test                           # Run tests
cargo run -p dyt-daemon              # Run daemon (needs ~/.config/dyt/config.toml)
cargo run -p dyt-cli -- --record     # Record and transcribe (needs daemon running)
cargo run --bin dyt-smoke            # Smoke-test mic → WAV pipeline (no daemon needed)
```

Run `./scripts/setup.sh` (Linux/macOS) or `.\scripts\setup.ps1` (Windows) for interactive first-time setup (deps, model, config, binary install). See `docs/setup.md` for manual steps.

## Standards

- **Cross-platform**: code must compile on both Ubuntu and Windows
- **Audio callback** (cpal): no locks, no allocations — real-time safe by constraint
- **ModelProvider trait** is the only extension point for new STT backends; register in `provider.rs`
- **Config path**: `~/.config/dyt/config.toml` on Linux, `%APPDATA%\dyt\config.toml` on Windows

## Notes

- **Windows OpenBLAS build**: The `whisper-rs` dependency uses `features = ["openblas"]` on Windows via `[target.'cfg(windows)'.dependencies]`. Building requires vcpkg + three things:
  1. `BLAS_INCLUDE_DIRS` env var must point to `%VCPKG_ROOT%\installed\x64-windows\include\openblas` (the `openblas` subdirectory — vcpkg puts `cblas.h` there, not in `include\` directly).
  2. `libopenblas.lib` must exist in `%VCPKG_ROOT%\installed\x64-windows\lib\` — vcpkg installs `openblas.lib` (no `lib` prefix) but `whisper-rs-sys` hardcodes `cargo:rustc-link-lib=libopenblas`, so copy: `Copy-Item openblas.lib libopenblas.lib`.
  3. `VCPKG_ROOT` env var must be set — `dyt-daemon/build.rs` emits `cargo:rustc-link-search=native=` pointing to the vcpkg lib dir so the Rust linker can find `libopenblas.lib`.
  Run `.\scripts\setup.ps1` to set up automatically.

- **hound in-memory WAV**: construct with `Cursor::new(&mut buf)`, call `finalize()`, return `buf`. Do NOT call `writer.into_inner()` when `WavWriter` was given `&mut cursor`. Preferred pattern (matches `encode.rs`):
  ```rust
  let mut buf = Vec::new();
  let mut writer = hound::WavWriter::new(std::io::Cursor::new(&mut buf), spec)?;
  writer.write_sample(s)?;
  writer.finalize()?;
  // buf now contains complete WAV bytes
  ```

## Workflow

When implementing multi-part features:
1. Decompose into independent subproblems
2. Spawn parallel planner agents via Task tool
3. Collect plans, resolve file-overlap conflicts
4. Spawn parallel builder agents for independent tasks
5. Sequence tasks that touch shared files; parallelize the rest

## Additional Documentation

Read relevant specs before modifying architecture:
- System overview: `specs/architecture.md`
- Daemon internals: `specs/dyt-daemon.md`
- CLI internals: `specs/dyt-cli.md`
- Audio capture design: `specs/audio-capture-design.md`
- Behavioral contracts: `specs/behavioral-contracts.md`
- Distribution & CI/CD: `specs/distribution.md`

Generate structured docs: `/initializing-project-docs`
