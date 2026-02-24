# DictateYourTerms

Zero-latency voice pipeline that translates speech into terminal-native text — keeping developer hands on the keyboard during agentic programming sessions.

Speak into your mic, get text on your clipboard. That's it.

## Architecture

Two Rust binaries, split for latency:

| Crate | Binary | Role |
|-------|--------|------|
| `dyt-daemon` | `dyt-daemon` | Persistent inference server (axum + whisper-rs). Keeps the STT model hot in RAM. |
| `dyt-cli` | `dyt` | Transient capture client (cpal + ringbuf). Records mic, sends WAV to daemon, copies transcript to clipboard. |

```
  mic ──► dyt (capture + WAV encode)
              │
              │ POST /transcribe (raw WAV bytes)
              ▼
          dyt-daemon (whisper.cpp inference)
              │
              ▼
          plain text ──► clipboard + stdout
```

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (rustup)
- A whisper.cpp GGML model file (e.g. `ggml-base.en.bin`)

**Ubuntu/Debian** — install system dependencies:

```bash
sudo apt install build-essential cmake pkg-config libclang-dev \
  libasound2-dev libpipewire-0.3-dev \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

## Quickstart

```bash
# 1. Build both crates
cargo build --release

# 2. Set up daemon config
mkdir -p ~/.config/dyt
cp config/default.toml ~/.config/dyt/config.toml
# Edit model_path to point at your GGML model file

# 3. Start the daemon
./bin/dyt-daemon

# 4. In another terminal — record and transcribe
./bin/dyt-record
# Speak, then press Enter. Transcript lands on your clipboard.
```

## Usage

### Daemon

```bash
# Start (or restart) the daemon
./bin/dyt-daemon

# Or run directly
cargo run -p dyt-daemon
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

Daemon config lives at `~/.config/dyt/config.toml` (XDG).

```toml
[server]
port = 3030
host = "127.0.0.1"

[engine]
provider = "whisper_cpp"
model_path = "~/.models/ggml-base.en.bin"
threads = 4
```

## Integrations

- **Neovim** — [dyt.nvim](https://github.com/nicolasayotte/dyt.nvim)
- **Tmux** — [dyt.tmux](https://github.com/nicolasayotte/dyt.tmux)

## Project Structure

```
dyt-daemon/src/
  main.rs              # axum server entrypoint
  provider/            # STT backend implementations
    whisper_cpp.rs     # whisper.cpp via whisper-rs

dyt-cli/src/
  main.rs              # clap CLI entrypoint
  capture.rs           # cpal mic capture with lock-free ring buffer
  encode.rs            # WAV encoding (hound)
  transport.rs         # HTTP client (reqwest)
  smoke.rs             # mic → WAV smoke test

config/                # example config
specs/                 # architecture specs and behavioral contracts
docs/                  # integration guides
bin/                   # convenience shell scripts
```

## License

All rights reserved.
