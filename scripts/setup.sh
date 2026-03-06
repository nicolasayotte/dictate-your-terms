#!/usr/bin/env bash
set -euo pipefail

# ── Step 1: Resolve paths ───────────────────────────────────────────

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEFAULT_CONFIG="$REPO_ROOT/config/default.toml"

if [[ ! -f "$DEFAULT_CONFIG" ]]; then
  echo "ERROR: Default config not found at $DEFAULT_CONFIG" >&2
  exit 1
fi

MODEL_DIR="$HOME/.models"
CONFIG_DIR="$HOME/.config/dyt"
CONFIG_FILE="$CONFIG_DIR/config.toml"
HF_BASE="https://huggingface.co/ggerganov/whisper.cpp/resolve/main"

BINS_INSTALLED=false

# ── Step 2: Install system dependencies (optional) ──────────────────

echo ""
read -rp "Install system dependencies? [y/N]: " install_deps
install_deps="${install_deps:-N}"

if [[ "$install_deps" =~ ^[Yy]$ ]]; then
  platform="$(uname -s)"
  case "$platform" in
    Linux)
      echo "==> Installing Linux dependencies via apt..."
      sudo apt install -y build-essential cmake pkg-config libclang-dev libasound2-dev libpipewire-0.3-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev
      ;;
    Darwin)
      echo "==> Installing macOS dependencies via brew..."
      brew install cmake
      ;;
    *)
      echo "==> Unsupported platform ($platform), skipping dependency install."
      ;;
  esac
else
  echo "==> Skipping dependency install. See docs/setup.md for manual instructions."
fi

# ── Step 3: Interactive model selection ──────────────────────────────

echo ""
echo "Select a whisper model to download:"
echo ""
echo "  1) tiny.en   (~75 MB)  — Fastest, lowest accuracy"
echo "  2) base.en   (~142 MB) — Good balance (recommended)"
echo "  3) small.en  (~466 MB) — Better accuracy"
echo "  4) medium.en (~1.5 GB) — High accuracy, slower"
echo ""

while true; do
  read -rp "Choice [2]: " choice
  choice="${choice:-2}"  # default to 2

  case "$choice" in
    1) MODEL_NAME="ggml-tiny.en.bin";   break ;;
    2) MODEL_NAME="ggml-base.en.bin";   break ;;
    3) MODEL_NAME="ggml-small.en.bin";  break ;;
    4) MODEL_NAME="ggml-medium.en.bin"; break ;;
    *) echo "Invalid choice. Please enter 1-4." ;;
  esac
done

# ── Step 4: Download model ───────────────────────────────────────────

MODEL_PATH="$MODEL_DIR/$MODEL_NAME"
mkdir -p "$MODEL_DIR"

do_download=true
if [[ -f "$MODEL_PATH" ]]; then
  read -rp "Model file $MODEL_PATH already exists. Re-download? [y/N]: " redownload
  redownload="${redownload:-N}"
  if [[ ! "$redownload" =~ ^[Yy]$ ]]; then
    do_download=false
  fi
fi

if $do_download; then
  echo "==> Downloading $MODEL_NAME ..."
  curl -L --progress-bar -o "$MODEL_PATH" "$HF_BASE/$MODEL_NAME"
  echo "==> Model downloaded to $MODEL_PATH"
fi

# ── Step 5: Install config ───────────────────────────────────────────

mkdir -p "$CONFIG_DIR"

if [[ -f "$CONFIG_FILE" ]]; then
  read -rp "Config file $CONFIG_FILE already exists. Overwrite? [y/N]: " overwrite
  overwrite="${overwrite:-N}"
  if [[ ! "$overwrite" =~ ^[Yy]$ ]]; then
    echo "==> Skipping config install. Remember to update model_path manually if needed."
  fi
fi

if [[ ! -f "$CONFIG_FILE" ]] || [[ "${overwrite:-Y}" =~ ^[Yy]$ ]]; then
  echo "==> Installing config to $CONFIG_FILE"
  cp "$DEFAULT_CONFIG" "$CONFIG_FILE"

  # Update model_path — handle macOS vs Linux sed
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s|^model_path = .*|model_path = \"$MODEL_PATH\"|" "$CONFIG_FILE"
  else
    sed -i "s|^model_path = .*|model_path = \"$MODEL_PATH\"|" "$CONFIG_FILE"
  fi

  echo "==> Config written to $CONFIG_FILE"
fi

# ── Step 6: Build and install binaries (optional) ───────────────────

echo ""
read -rp "Build and install dyt binaries to ~/.cargo/bin? [y/N]: " install_bins
install_bins="${install_bins:-N}"

if [[ "$install_bins" =~ ^[Yy]$ ]]; then
  echo "==> Building and installing dyt-daemon..."
  cargo install --path "$REPO_ROOT/dyt-daemon"
  echo "==> Building and installing dyt (CLI)..."
  cargo install --path "$REPO_ROOT/dyt-cli"
  BINS_INSTALLED=true
  echo "==> Binaries installed to ~/.cargo/bin"
else
  echo "==> Skipping binary install."
fi

# ── Step 7: Summary ─────────────────────────────────────────────────

echo ""
echo "Setup complete!"
echo ""
echo "  Model:  $MODEL_PATH"
echo "  Config: $CONFIG_FILE"
echo ""

if $BINS_INSTALLED; then
  echo "Next steps:"
  echo "  1. Start the daemon:        dyt-daemon"
  echo "  2. Record and transcribe:   dyt --record"
else
  echo "Next steps:"
  echo "  1. Build the project:       cargo build --release"
  echo "  2. Start the daemon:        cargo run -p dyt-daemon"
  echo "  3. Record and transcribe:   cargo run -p dyt-cli -- --record"
fi
