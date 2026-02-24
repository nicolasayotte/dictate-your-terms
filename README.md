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

**Downloading a model:**

All models are hosted at `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/`. Replace `base.en` in the commands below with your chosen model name.

| Model | File | Size | Notes |
|-------|------|------|-------|
| `tiny.en` | `ggml-tiny.en.bin` | ~75 MB | Fastest, lowest accuracy |
| `base.en` | `ggml-base.en.bin` | ~142 MB | Good balance (recommended) |
| `small.en` | `ggml-small.en.bin` | ~466 MB | Better accuracy |
| `medium.en` | `ggml-medium.en.bin` | ~1.5 GB | High accuracy, slower |

**Linux/macOS:**

```bash
mkdir -p ~/.models
curl -L -o ~/.models/ggml-base.en.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"
```

**Windows (PowerShell):**

```powershell
mkdir "$env:USERPROFILE\.models"
curl -L -o "$env:USERPROFILE\.models\ggml-base.en.bin" `
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"
```

After downloading, set `model_path` in your config to the file's absolute path (see [Configuration](#configuration)).

### System dependencies

**Windows** — install build tools:

1. **LLVM/Clang** — required by `bindgen` to generate FFI bindings:
   ```
   winget install LLVM.LLVM
   ```
   Then set the `LIBCLANG_PATH` environment variable:
   ```
   setx LIBCLANG_PATH "C:\Program Files\LLVM\bin"
   ```

2. **CMake** — required to compile `whisper.cpp`:
   ```
   winget install Kitware.CMake
   ```

3. **Visual Studio Build Tools** — C/C++ compiler (`cl.exe`). Install the "Desktop development with C++" workload via the [Visual Studio Build Tools installer](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

Open a **new terminal** after installing so PATH changes take effect. WASAPI (audio) and the Windows clipboard are available natively — no extra packages needed.

**Ubuntu/Debian** — install system dependencies:

```bash
sudo apt install build-essential cmake pkg-config libclang-dev \
  libasound2-dev libpipewire-0.3-dev \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

## Automated Setup

Setup scripts handle model download and config creation interactively:

**Linux/macOS:**
```bash
./scripts/setup.sh
```

**Windows (PowerShell):**
```powershell
.\scripts\setup.ps1
```

The script will:
1. Optionally install system dependencies (apt packages / winget)
2. Let you choose a whisper model size
3. Download the model to `~/.models/` (Linux) or `%USERPROFILE%\.models\` (Windows)
4. Create and configure `config.toml` with the correct model path
5. Optionally build and install `dyt-daemon` and `dyt` to your PATH

All optional steps default to **no** — just press Enter to skip.

## Quickstart

```bash
# 1. Run the setup script (handles deps, model, config, and binary install)
./scripts/setup.sh          # Linux/macOS
# .\scripts\setup.ps1       # Windows (PowerShell)

# 2. Start the daemon
dyt-daemon

# 3. In another terminal — record and transcribe
dyt --record
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

Daemon config lives at:
- **Linux**: `~/.config/dyt/config.toml` (XDG)
- **Windows**: `%APPDATA%\dyt\config.toml`

```toml
[server]
port = 3030
host = "127.0.0.1"

[engine]
provider = "whisper_cpp"
model_path = "~/.models/ggml-base.en.bin"       # Linux/macOS
# model_path = "C:\\Users\\<USERNAME>\\.models\\ggml-base.en.bin"  # Windows
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
scripts/               # setup scripts (model download + config)
```

## License

All rights reserved.
