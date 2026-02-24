
# Project Context

**Mission:** Zero-latency voice pipeline that translates spoken architectural intent into terminal-native text — keeping developer hands on the keyboard during agentic programming sessions.

DictateYourTerms (dyt) — a Rust Cargo workspace with two crates:
- `stt-daemon`: Persistent inference server (axum + whisper-rs). Keeps the STT model hot in RAM.
- `stt-cli`: Transient capture client (cpal + ringbuf). Records mic, sends to daemon, copies result to clipboard.

Binary names: `stt-daemon`, `dyt`

## About This Project

Rust workspace (edition 2021). Daemon exposes `POST /transcribe` on `127.0.0.1:3030`, accepting raw WAV bytes and returning plain-text transcription. CLI captures audio into a lock-free ring buffer, WAV-encodes it, POSTs to daemon, and injects the transcript into the clipboard. All system dependencies (whisper.cpp, ALSA, etc.) are pinned in `flake.nix`.

## Key Directories

- `stt-daemon/src/` — Daemon: config loading, axum server, ModelProvider trait
- `stt-daemon/src/provider/` — Backend implementations (`whisper_cpp.rs`); add new engines here
- `stt-cli/src/` — CLI: `capture.rs` (cpal), `encode.rs` (hound), `transport.rs` (reqwest), `main.rs` (clap)
- `config/` — Example `config.toml` for the daemon
- `specs/` — Architecture specs and behavioral contracts (authoritative design decisions)
- `docs/` — Integration guides and design notes

## Commands

All `cargo` commands **must** run inside the Nix dev shell. Either enter it
interactively or prefix each command with `nix develop --command`:

```bash
nix develop                                          # Enter dev shell (interactive)

nix develop --command cargo check                    # Type-check without building
nix develop --command cargo build --release          # Build both crates
nix develop --command cargo test                     # Run tests
nix develop --command cargo run -p stt-daemon        # Run daemon (needs ~/.config/dyt/config.toml)
nix develop --command cargo run -p stt-cli -- --record  # Record and transcribe (needs daemon running)
nix develop --command cargo run --bin dyt-smoke      # Smoke-test mic → WAV pipeline (no daemon needed)
```

### Nix dev shell — what it provides

| Category | Packages |
|----------|----------|
| Rust toolchain | `rustc`, `cargo`, `clippy`, `rustfmt` |
| whisper.cpp build | `cmake`, `pkg-config`, `libclang`, `gcc` |
| Audio (cpal) | `alsa-lib`, `pipewire` |
| Clipboard (arboard) | `libx11`, `libxcursor`, `libxrandr`, `libxi` |

Environment variables set by the shell:
- `LIBCLANG_PATH` — points `bindgen` at `libclang.lib`
- `ALSA_PLUGIN_DIR` — routes ALSA through PipeWire on Ubuntu
- `LD_LIBRARY_PATH` — exposes `stdenv.cc.cc.lib`, `alsa-lib`, `pipewire`

## Standards

- **Cross-platform**: code must compile on both Ubuntu and Windows
- **Audio callback** (cpal): no locks, no allocations — real-time safe by constraint
- **ModelProvider trait** is the only extension point for new STT backends; register in `provider.rs`
- **Config path**: `~/.config/dyt/config.toml` on Linux (XDG), `%APPDATA%\dyt\config.toml` on Windows

## Notes

- Daemon resamples all incoming audio to 16 kHz mono f32 before inference (linear interpolation)
- Default bind: `127.0.0.1:3030` — localhost only, no auth
- `nix develop` is required; do not attempt manual dependency installs
- **hound in-memory WAV** (`hound = "3.5"`): use the pattern where the writer owns the `Cursor` — construct with `Cursor::new(&mut buf)`, call `finalize()`, then return `buf`. Do NOT specify `writer.into_inner()` when `WavWriter` was given `&mut cursor` — that requires an explicit scope block. Preferred pattern (matches `encode.rs`):
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
- Daemon internals: `specs/stt-daemon.md`
- CLI internals: `specs/stt-cli.md`
- Audio capture design: `specs/audio-capture-design.md`
- Behavioral contracts: `specs/behavioral-contracts.md`

Generate structured docs: `/initializing-project-docs`
