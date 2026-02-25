# Setup

> **AI Context Summary**: Setting up DictateYourTerms requires Rust (via rustup), a GGML-format whisper model file, and system libraries for audio and clipboard. Ubuntu needs several apt packages; Windows needs LLVM, CMake, and VS Build Tools. The daemon must be running before the CLI can transcribe. Config lives at `~/.config/dyt/config.toml` (Linux) or `%APPDATA%\dyt\config.toml` (Windows). Interactive setup scripts (`scripts/setup.sh` and `scripts/setup.ps1`) automate all of this.

## Automated Setup (Recommended)

Interactive scripts handle everything in one shot:

```bash
./scripts/setup.sh      # Linux/macOS
.\scripts\setup.ps1     # Windows (PowerShell)
```

Each script optionally installs system dependencies, lets you choose and download a whisper model, writes the config, and optionally runs `cargo build --release`. All optional steps default to **no**.

---

## Manual Setup

Follow the steps below if you prefer to set things up by hand.

## Prerequisites

### Rust Toolchain

Install via [rustup](https://rustup.rs/) on all platforms:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### System Dependencies (Ubuntu/Debian)

```bash
sudo apt install build-essential cmake pkg-config libclang-dev \
  libasound2-dev libpipewire-0.3-dev \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

| Category | Packages |
|----------|----------|
| whisper.cpp build | `build-essential`, `cmake`, `pkg-config`, `libclang-dev` |
| Audio (cpal/ALSA) | `libasound2-dev`, `libpipewire-0.3-dev` |
| Clipboard (arboard/X11) | `libx11-dev`, `libxcursor-dev`, `libxrandr-dev`, `libxi-dev` |

### Windows

Install Rust via rustup. WASAPI (audio) and the Windows clipboard API are available natively. Use the MSVC toolchain (default on Windows).

#### OpenBLAS (Required for CPU acceleration)

The Windows build of `dyt-daemon` links against OpenBLAS via the `whisper-rs` `openblas` feature. Install it using [vcpkg](https://github.com/microsoft/vcpkg):

```powershell
# Clone and bootstrap vcpkg (skip if already installed)
git clone https://github.com/microsoft/vcpkg.git $env:USERPROFILE\vcpkg
& "$env:USERPROFILE\vcpkg\bootstrap-vcpkg.bat" -disableMetrics

# Install OpenBLAS for 64-bit Windows
vcpkg install openblas:x64-windows

# Integrate with MSBuild/CMake so the whisper.cpp build finds it automatically
vcpkg integrate install
```

Set `VCPKG_ROOT` to your vcpkg directory if it is not already set:

```powershell
[System.Environment]::SetEnvironmentVariable("VCPKG_ROOT", "$env:USERPROFILE\vcpkg", "User")
```

The `whisper-rs-sys` build script also requires `BLAS_INCLUDE_DIRS` to point to the OpenBLAS headers:

```powershell
[System.Environment]::SetEnvironmentVariable("BLAS_INCLUDE_DIRS", "$env:VCPKG_ROOT\installed\x64-windows\include", "User")
```

After installation, `cargo build --release` will pick up OpenBLAS automatically. If the build fails with `BLAS_INCLUDE_DIRS environment variable must be set`, verify `BLAS_INCLUDE_DIRS` is set and points to the vcpkg OpenBLAS include directory.

> **Automated setup:** `.\scripts\setup.ps1` handles all of the above interactively.

## Build

Build tasks are managed with [cargo-make](https://github.com/sagiegurari/cargo-make). Install it once:

```bash
cargo install cargo-make
```

| Command | When to use |
|---|---|
| `makers build-greedy` | Local dev — enables AVX2/FMA/F16C for your CPU (10–20× faster whisper inference) |
| `makers build-generic` | Portable binary with no CPU-specific flags |

```bash
makers build-greedy
# Outputs:
#   target/release/dyt-daemon    (or .exe on Windows)
#   target/release/dyt           (or .exe on Windows)
```

If you don't have `cargo-make`, you can build directly:

```bash
# Native (local dev)
RUSTFLAGS="-C target-cpu=native" cargo build --release
# Generic
cargo build --release
```

## Whisper Model

Download a GGML-format whisper model. The `base.en` model is a good starting point for English dictation:

```bash
# Check https://huggingface.co/ggerganov/whisper.cpp for current model files
# Example (verify the URL is current before using):
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin \
  -O ~/.local/share/dyt/ggml-base.en.bin
```

See the README for the recommended download command for your platform.

## Config File

Create the config directory and file:

```bash
# Linux
mkdir -p ~/.config/dyt
cat > ~/.config/dyt/config.toml << 'EOF'
[server]
host = "127.0.0.1"
port = 3030

[engine]
provider = "whisper_cpp"
model_path = "/home/you/.local/share/dyt/ggml-base.en.bin"
threads = 4
EOF
```

On Windows, create `%APPDATA%\dyt\config.toml` with the same content, using a Windows-style `model_path` (e.g., `C:\Users\you\dyt\ggml-base.en.bin`).

See the `config/` directory for example configs.

## Running

### Start the Daemon

```bash
./target/release/dyt-daemon
```

The daemon logs to stdout and stays running. It binds `127.0.0.1:3030` by default. Restart it to reload the model or config.

### Record and Transcribe

```bash
# Start recording; speak, then press Enter to stop and transcribe
./target/release/dyt --record
```

The transcript is injected into the clipboard and printed to stdout.

### Smoke Test (no daemon required)

```bash
cargo run --bin dyt-smoke
```

Tests mic capture → WAV encoding pipeline without the daemon. Use this to verify audio hardware works before troubleshooting transcription issues.

## Verify the Setup

```bash
# Terminal 1
./target/release/dyt-daemon

# Terminal 2
./target/release/dyt --record
# Speak, press Enter → transcript appears in clipboard
```

## Cross-References

- System architecture: `docs/architecture.md`
- API reference: `docs/api.md`
- Release builds: `docs/deployment.md`
