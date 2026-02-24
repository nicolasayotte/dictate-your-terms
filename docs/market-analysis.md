# Market Analysis

Why building DictateYourTerms is the right move: the exact tool we need does not exist as a polished, out-of-the-box solution.

## Existing Tools

### 1. The Heavyweights (Talon Voice)

Talon is the undisputed king of voice coding. It uses local, fast STT engines.

**Why it doesn't fit:** Designed for 100% hands-free accessibility. Requires learning a custom phonetic alphabet and syntax commands. It's an entire paradigm shift, not a lightweight push-to-talk tool for dumping context into an agent while keeping hands on the keyboard.

### 2. The IDE-Locked Tools (Cursor / Copilot Voice)

Massive surge in voice-to-text for agentic coding, but locked inside proprietary GUI editors.

**Why it doesn't fit:** Completely useless if your environment is Neovim, WezTerm, and Tmux. Relies on cloud endpoints, introducing flow-breaking latency.

### 3. The GitHub Hack-Jobs (Python Scripts)

Dozens of Python scripts wrapping local Whisper on GitHub.

**Why they don't fit:**
- **Cold starts**: Load the Whisper model from scratch every invocation (2-4 second delay).
- **OS dependency**: Rely on Linux-specific tools like `xdotool` for keystroke simulation, breaking Windows compatibility.
- **Resource hogs**: Python wrapping raw audio streams and ML models is bloated and fragile.

## The Gap

No existing tool:
- Drops text into a Neovim buffer instantly
- Works identically on Ubuntu and Windows
- Runs entirely locally without eating all RAM
- Keeps the model hot to eliminate cold starts
- Is terminal-native (no GUI requirement)

DictateYourTerms fills this gap with a Rust client/daemon architecture that decouples the heavy ML inference from lightweight audio capture.
