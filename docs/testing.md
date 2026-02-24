# Testing

> **AI Context Summary**: Tests are standard Rust `#[test]` unit tests co-located in source files, plus one integration test in `dyt-cli/tests/`. Run with `cargo test`. Coverage spans WAV encoding/decoding, resampling, config loading, and a full encode→decode roundtrip. The `dyt-smoke` binary tests live mic capture without the daemon. No mocking framework — tests use synthetic `Vec<f32>` samples and in-memory hound WAV construction.

## Commands

```bash
cargo test                            # Run all tests (all crates)
cargo test -p dyt-daemon              # Daemon tests only
cargo test -p dyt-cli                 # CLI tests only
cargo test encode                     # Tests matching "encode"
cargo test -- --nocapture             # Show println! output
cargo run --bin dyt-smoke             # Live mic smoke test (no daemon needed)
```

## Test Coverage

### dyt-cli/src/encode.rs

Unit tests for the WAV encoder and linear-interpolation resampler:

| Test | What it verifies |
|------|-----------------|
| `passthrough_16k` | 16 kHz input passes through unchanged, sample count preserved |
| `resample_48k_to_16k` | 48 kHz → 16 kHz produces ~1/3 the samples (±1 tolerance) |
| `empty_input` | Zero samples produces a valid WAV with zero PCM samples |
| `clamps_positive` | Samples > 1.0 clamp to `i16::MAX` |
| `clamps_negative` | Samples < -1.0 clamp to `-i16::MAX` (not `i16::MIN`) |
| `wav_metadata` | Output WAV has channels=1, sample_rate=16000, bits_per_sample=16 |

### dyt-daemon/src/server.rs

Unit tests for WAV decoding and audio normalization in the server handler:

| Test | What it verifies |
|------|-----------------|
| `decode_i16_wav` | I16 PCM WAV bytes decode to correct f32 range |
| `decode_f32_wav` | F32 WAV bytes decode to correct f32 values |
| `stereo_to_mono` | Stereo channel pairs averaged correctly |
| `resample_in_server` | Server-side resampling produces expected output length |

### dyt-daemon/src/config.rs

Unit tests for config loading and defaults:

| Test | What it verifies |
|------|-----------------|
| `defaults` | Omitted fields produce correct default values |
| `full_config` | All fields parse correctly from TOML |
| `config_path` | Platform config path resolves correctly |

### Integration Test

```bash
cargo test --test encode_roundtrip    # dyt-cli/tests/encode_roundtrip.rs
```

Encodes a synthetic sine wave to WAV via `encode::to_wav()`, decodes it back with hound, and asserts round-trip fidelity within tolerance.

## Smoke Test

`dyt-smoke` (`dyt-cli/src/smoke.rs`) is a standalone binary:

1. Opens the default mic via cpal
2. Records for 2 seconds
3. Encodes to WAV in memory
4. Reports WAV byte size and sample count

Does not require the daemon. Use this to verify the mic capture pipeline on new hardware or after OS audio config changes.

```bash
cargo run --bin dyt-smoke
```

## Test Patterns

### WAV Construction in Tests

Use the hound in-memory pattern from CLAUDE.md:

```rust
let spec = hound::WavSpec {
    channels: 1,
    sample_rate: 16000,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};
let mut buf = Vec::new();
let mut writer = hound::WavWriter::new(std::io::Cursor::new(&mut buf), spec)?;
for s in &samples {
    writer.write_sample((*s * i16::MAX as f32) as i16)?;
}
writer.finalize()?;
// buf is now valid WAV bytes
```

### Testing Decode Logic Directly

`server.rs` tests call the WAV decode and resample helpers directly rather than going through the axum handler, avoiding the need for a live ModelProvider or HTTP stack.

### No Live Audio in Unit Tests

Unit tests always use synthetic `Vec<f32>` samples. Only `dyt-smoke` opens the physical microphone. This keeps `cargo test` fast and hermetic.

### Clamp Behavior Note

`clamps_negative` asserts `-i16::MAX` (−32767), not `i16::MIN` (−32768). This is intentional: the encoder computes `(-1.0 * 32767) as i16`, which gives −32767. Tests document this asymmetry explicitly.

## Cross-References

- WAV encode implementation: `dyt-cli/src/encode.rs`
- Server WAV decode: `dyt-daemon/src/server.rs`
- Architecture: `docs/architecture.md`
