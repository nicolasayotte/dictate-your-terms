# dyt-cli (Capture Client)

## Responsibility

A transient, highly optimized binary triggered by a hotkey. It handles OS-level hardware bindings and network transmission, then immediately terminates.

## Tech Stack

| Dependency  | Purpose |
|-------------|---------|
| `cpal`      | Cross-Platform Audio Library. Natively hooks into WASAPI on Windows and ALSA/PulseAudio/PipeWire on Ubuntu using the same Rust source. |
| `reqwest`   | HTTP client to fire the captured audio payload to the daemon. |
| `arboard`   | Cross-platform clipboard library. Handles Windows, X11, and Wayland clipboard architectures. |
| `ringbuf`   | Lock-free SPSC ring buffer for real-time safe audio capture (see [audio-capture-design](audio-capture-design.md)). |
| `crossterm` | Terminal event handling for the termination signal. |

## Architecture

When `dyt-cli --record` is executed:

1. Queries `cpal` for the default system microphone.
2. Pre-allocates a lock-free SPSC ring buffer (`ringbuf::HeapRb<f32>`) sized for ~60 seconds of audio (~3MB).
3. Splits the ring buffer into a `Producer` (moves into cpal callback) and `Consumer` (drain thread).
4. The `cpal` callback pushes raw `f32` samples via zero-allocation `push_slice()`.
5. A background drain thread periodically pulls from the consumer, downsamples to 16kHz, and stores in the final payload buffer.
6. The main thread blocks on a termination signal (Enter key via `crossterm` or stdin).
7. On termination: halts the cpal stream, drains remaining ring buffer, encodes payload as WAV in-memory.
8. Fires the WAV as an HTTP POST to `127.0.0.1:3030/transcribe`.
9. On response: overwrites the system clipboard via `arboard`.
10. Exits with code 0.

## Cross-Platform Audio Notes

Windows and Linux handle default sample rates differently. The microphone may natively capture at 48kHz on Windows but 44.1kHz on Ubuntu. Whisper strictly requires 16kHz mono audio. Downsampling is performed by the consumer/drain thread (not in the cpal callback) to keep the audio path real-time safe.

## Binary Name

```
dyt --record
```
