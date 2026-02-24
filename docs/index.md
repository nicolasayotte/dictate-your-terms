# Documentation Index

> DictateYourTerms (dyt) — zero-latency voice-to-clipboard pipeline for developers.
> Browse by task below, or start with `docs/architecture.md` for a system overview.

## By Task

### Starting Out

| Document | Read when you want to… |
|----------|------------------------|
| [`docs/setup.md`](./setup.md) | Install dependencies and run for the first time |
| [`docs/architecture.md`](./architecture.md) | Understand the two-process pipeline |
| [`docs/tech-stack.md`](./tech-stack.md) | Know which crates are used and why |

### Building Features

| Document | Read when you want to… |
|----------|------------------------|
| [`docs/api.md`](./api.md) | Understand or test `POST /transcribe` |
| [`docs/testing.md`](./testing.md) | Run tests or add new test cases |
| [`docs/workflow.md`](./workflow.md) | Implement a multi-part feature with planner/builder agents |

### Releasing & Deploying

| Document | Read when you want to… |
|----------|------------------------|
| [`docs/deployment.md`](./deployment.md) | Build release binaries, set up systemd, integrate hotkeys |
| [`docs/setup.md`](./setup.md) | Install system dependencies on a new machine |

## Authoritative Specs

Architecture decisions live in `specs/` — read these before modifying core systems:

| Spec | Covers |
|------|--------|
| `specs/architecture.md` | System overview and design decisions |
| `specs/dyt-daemon.md` | Daemon internals (axum, ModelProvider, config) |
| `specs/dyt-cli.md` | CLI internals (cpal, ringbuf, transport) |
| `specs/audio-capture-design.md` | Why lock-free ringbuf (not mutex) in the audio callback |
| `specs/behavioral-contracts.md` | Gherkin-style behavioral contracts |
| `specs/distribution.md` | CI/CD release strategy (Linux dynamic linking, MSVC Windows) |

## Other Docs

- [`docs/integrations.md`](./integrations.md) — Editor and tool integration guides
- [`docs/idea.md`](./idea.md) — Original design ideation
- [`docs/market-analysis.md`](./market-analysis.md) — Market context

## Quick Reference

```bash
cargo check                           # Type-check without building
cargo test                            # Run all tests
cargo build --release                 # Build release binaries
cargo run -p dyt-daemon               # Start the inference daemon
cargo run -p dyt-cli -- --record      # Record and transcribe
cargo run --bin dyt-smoke             # Smoke-test mic pipeline (no daemon)
```
