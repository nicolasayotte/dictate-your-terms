# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-02-24

First release with Windows support. The daemon now builds and runs on Windows end-to-end, including model inference via OpenBLAS.

### Added

- **Windows support**: `dyt-daemon` builds and runs on Windows. whisper-rs is wired with `features = ["openblas"]` via `[target.'cfg(windows)'.dependencies]`, and `dyt-daemon/build.rs` emits the vcpkg linker search path so `libopenblas.lib` is found automatically.
- **`scripts/setup.ps1`**: Interactive first-time setup script for Windows — installs vcpkg, OpenBLAS, sets `BLAS_INCLUDE_DIRS` / `VCPKG_ROOT`, and creates the `libopenblas.lib` alias required by `whisper-rs-sys`.
- **`scripts/setup.sh`**: Companion interactive setup script for Linux/macOS.
- **CPU thread auto-detection**: Daemon calls `std::thread::available_parallelism()` at startup and passes the result to whisper-rs instead of a hardcoded value.
- **Non-blocking inference**: Transcription is dispatched via `tokio::task::spawn_blocking`, keeping the async executor free during model inference. Inference wall-clock time is logged on each request.
- **`docs/` directory**: Reference documentation covering architecture, API, setup, testing, tech stack, deployment, and workflow.

### Fixed

- **`scripts/setup.ps1`**: Corrected `BLAS_INCLUDE_DIRS` to point to the `openblas` subdirectory (where vcpkg places `cblas.h`), and added the `openblas.lib` → `libopenblas.lib` copy step required by `whisper-rs-sys`.

### Documentation

- Documented all three Windows OpenBLAS build requirements in `CLAUDE.md`, `docs/setup.md`, and `README.md`: the `BLAS_INCLUDE_DIRS` subdirectory, the `libopenblas.lib` alias, and `VCPKG_ROOT`.
- Expanded `README.md` with Windows build toolchain prerequisites, cross-platform config paths, and integration links.
- Updated architecture and tech-stack docs to reflect `spawn_blocking` dispatch and OpenBLAS backend.

## [1.0.0] - 2026-02-24

Initial stable release.

- `dyt-daemon`: Persistent axum inference server exposing `POST /transcribe` on `127.0.0.1:3030`. Keeps whisper-rs model hot in RAM. Accepts raw WAV bytes, returns plain-text transcription.
- `dyt`: Transient CLI capture client. Records mic via cpal into a lock-free ring buffer, WAV-encodes with hound, POSTs to daemon, and injects the transcript into the clipboard.
- `ModelProvider` trait for pluggable STT backends.
- HTTP integration tests and WAV encode/decode roundtrip tests.
- Neovim plugin at `editors/nvim/`.
- CI/CD workflows for Ubuntu and Windows.
