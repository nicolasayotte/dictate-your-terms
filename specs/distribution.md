# Distribution

## Strategy

Release binaries are built on vanilla Linux and Windows CI runners using standard system toolchains.

## Linux (GitHub Actions)

1. Runner: `ubuntu-22.04` (older glibc for broad compatibility).
2. Install system dependencies via `apt`:
   ```bash
   sudo apt install build-essential cmake pkg-config libclang-dev \
     libasound2-dev libpipewire-0.3-dev \
     libx11-dev libxcursor-dev libxrandr-dev libxi-dev
   ```
3. Build: `cargo build --release`
4. The binary dynamically links to standard paths (e.g., `/usr/lib/x86_64-linux-gnu/libasound.so.2`), which exist on any standard Ubuntu/Debian/Fedora installation with ALSA installed.
5. Attach `dyt-daemon` and `dyt` binaries to a GitHub Release.

## Windows (GitHub Actions)

1. Runner: `windows-latest`
2. Install the Rust toolchain via `rustup` (MSVC target).
3. Build: `cargo build --release`
4. WASAPI (audio) and the Windows clipboard are available natively — no extra dependencies.
5. Attach `.exe` binaries to the same GitHub Release.

## Local Development

Developers install system dependencies directly on their machine (see CLAUDE.md Commands section) and use `cargo` commands without any wrapper. The Rust toolchain is installed via [rustup](https://rustup.rs/).

## Design Decisions

- **Why not static linking?** `whisper.cpp` (via `whisper-rs`) builds from source at compile time and statically links. Audio (`libasound`) and X11 clipboard libraries are better left dynamically linked — they must match the user's running audio server and display server.
- **Why Ubuntu 22.04 for CI?** Older glibc ensures the binary runs on 22.04+ without `GLIBC_2.xx not found` errors. Users on newer distros get the same binary.
