# Architecture

## Overview

DictateYourTerms uses a two-tier, client-server architecture that strictly decouples the heavyweight machine learning context from the lightweight hardware interaction. This guarantees a modular, type-safe system that executes instantly and seamlessly bridges both Windows and Ubuntu environments.

## Cargo Workspace

The project is a Cargo workspace comprising two specialized crates:

| Crate        | Role              | Tech Stack                        | Description |
|--------------|-------------------|-----------------------------------|-------------|
| `dyt-daemon` | Inference Engine  | axum, tokio, whisper-rs, hound    | Persistent background server. Holds the STT model hot in RAM/VRAM, eliminating cold starts. Exposes a single local API endpoint (`POST /transcribe`). |
| `dyt-cli`    | Capture Client    | cpal, reqwest, arboard            | Transient CLI binary. Captures raw f32 audio from the OS default mic, fires it to the daemon, dumps returned text to the system clipboard, and exits. |

```toml
[workspace]
members = [
    "dyt-daemon",
    "dyt-cli"
]
```

## System Flow

1. You are in Neovim (Ubuntu or Windows). You press `<leader>v`.
2. A terminal split briefly opens, running `dyt-cli --record`.
3. `cpal` captures your voice detailing a complex architectural constraint.
4. You hit Enter to end the capture.
5. `dyt-cli` fires the payload to `dyt-daemon` (running `whisper-rs`).
6. The daemon responds in ~400ms.
7. `dyt-cli` pipes the text to the clipboard and exits.
8. Your Neovim Lua script immediately pastes the clipboard contents into your buffer.

## Design Principles

- **Zero-latency**: Reject cloud APIs and cold-started Python scripts; the persistent daemon architecture keeps the model hot.
- **Cross-platform**: Strict use of `cpal` and `arboard` in Rust ensures identical binary behavior on Windows and Ubuntu.
- **Terminal-native**: Inject text directly into the system clipboard or stdout; reject proprietary GUI integrations.
- **Hands off the keyboard**: Push-to-talk interrupt loop ensures you never reach for a mouse.
- **Modular**: Decoupling the ML daemon from the capture CLI means swapping the inference backend leaves the client untouched.

## Distribution

Release binaries are built on vanilla Linux (GitHub Actions `ubuntu-22.04` for broad glibc compatibility). System dependencies are installed via `apt` so the resulting binary dynamically links to standard system libraries (e.g., `/usr/lib/x86_64-linux-gnu/libasound.so.2`), ensuring compatibility across standard Linux distros. See `specs/distribution.md` for full details.

- **Linux**: CI installs system deps via `apt`, runs `cargo build --release`, attaches binaries to GitHub Releases.
- **Windows**: CI uses the MSVC Rust toolchain; WASAPI and clipboard are available natively with no extra dependencies.
