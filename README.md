# DictateYourTerms

Zero-latency voice pipeline that translates speech into terminal-native text — keeping developer hands on the keyboard during agentic programming sessions.

Speak into your mic, get text on your clipboard. That's it.

## Architecture

Two Rust binaries, split for latency:

| Crate | Binary | Role |
|-------|--------|------|
| `stt-daemon` | `stt-daemon` | Persistent inference server (axum + whisper-rs). Keeps the STT model hot in RAM. |
| `stt-cli` | `dyt` | Transient capture client (cpal + ringbuf). Records mic, sends WAV to daemon, copies transcript to clipboard. |

```
  mic ──► dyt (capture + WAV encode)
              │
              │ POST /transcribe (raw WAV bytes)
              ▼
          stt-daemon (whisper.cpp inference)
              │
              ▼
          plain text ──► clipboard + stdout
```

## Prerequisites

- [Nix](https://nixos.org/download/) with flakes enabled
- A whisper.cpp GGML model file (e.g. `ggml-base.en.bin`)

All build dependencies (Rust toolchain, whisper.cpp, ALSA, PipeWire, X11 libs) are pinned in `flake.nix` — no manual installs needed.

## Quickstart

```bash
# 1. Enter the Nix dev shell
nix develop

# 2. Build both crates
cargo build --release

# 3. Set up daemon config
mkdir -p ~/.config/dyt
cp config/default.toml ~/.config/dyt/config.toml
# Edit model_path to point at your GGML model file

# 4. Start the daemon
./bin/dyt-daemon

# 5. In another terminal — record and transcribe
./bin/dyt-record
# Speak, then press Enter. Transcript lands on your clipboard.
```

## Usage

### Daemon

```bash
# Start (or restart) the daemon
./bin/dyt-daemon

# Or run directly
cargo run -p stt-daemon
```

The daemon binds to `127.0.0.1:3030` and exposes a single endpoint:

```
POST /transcribe
Content-Type: application/octet-stream
Body: raw WAV bytes

Response: plain text transcription
```

### CLI

```bash
# Record from default mic, transcribe, copy to clipboard
dyt --record

# Point at a different daemon address
dyt --record --daemon http://192.168.1.10:3030
```

Output goes to both clipboard and stdout, so you can pipe it:

```bash
dyt --record | wc -w
```

### Smoke test

Verify your mic works without needing the daemon:

```bash
cargo run --bin dyt-smoke
```

## Configuration

Daemon config lives at `~/.config/dyt/config.toml` (Linux/XDG) or `%APPDATA%\dyt\config.toml` (Windows).

```toml
[server]
port = 3030
host = "127.0.0.1"

[engine]
provider = "whisper_cpp"
model_path = "~/.models/ggml-base.en.bin"
threads = 4
```

## Editor Integrations

- **Neovim** — plugin at `editors/nvim/` (see `docs/integrations.md`)

## Project Structure

```
stt-daemon/src/
  main.rs              # axum server entrypoint
  provider/            # STT backend implementations
    whisper_cpp.rs     # whisper.cpp via whisper-rs

stt-cli/src/
  main.rs              # clap CLI entrypoint
  capture.rs           # cpal mic capture with lock-free ring buffer
  encode.rs            # WAV encoding (hound)
  transport.rs         # HTTP client (reqwest)
  smoke.rs             # mic → WAV smoke test

config/                # example config
editors/               # editor plugins
specs/                 # architecture specs and behavioral contracts
docs/                  # integration guides
bin/                   # convenience shell scripts
```

## License

All rights reserved.
