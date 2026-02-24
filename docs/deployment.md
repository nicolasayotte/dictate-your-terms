# Deployment

> **AI Context Summary**: Release binaries are built via GitHub Actions CI/CD. Linux targets ubuntu-22.04 for broad glibc compatibility with dynamic linking (audio and X11 libs must be installed on the host); Windows targets windows-latest with the MSVC toolchain. Local release build is `cargo build --release`. Static linking is not supported — audio and clipboard libraries require dynamic linking.

## Local Release Build

```bash
cargo build --release
# Outputs:
#   target/release/dyt-daemon    (Linux) / target/release/dyt-daemon.exe (Windows)
#   target/release/dyt           (Linux) / target/release/dyt.exe (Windows)
```

Strip symbols for smaller binaries (Linux):

```bash
strip target/release/dyt-daemon target/release/dyt
```

## Installing Locally

```bash
# Linux — system-wide
sudo cp target/release/dyt-daemon target/release/dyt /usr/local/bin/

# Linux — user-local
mkdir -p ~/.local/bin
cp target/release/dyt-daemon target/release/dyt ~/.local/bin/
# Ensure ~/.local/bin is in $PATH
```

## GitHub Actions CI/CD

Release strategy per `specs/distribution.md`:

### Linux

- **Runner**: `ubuntu-22.04` — older glibc for maximum binary compatibility across distros
- **Linking**: Dynamic — ALSA, PipeWire, and X11 libs link at runtime. Static linking is rejected because these libraries are not designed for static use.
- **Pre-build step** (install system deps):

```yaml
- name: Install system dependencies
  run: |
    sudo apt install -y build-essential cmake pkg-config libclang-dev \
      libasound2-dev libpipewire-0.3-dev \
      libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

### Windows

- **Runner**: `windows-latest`
- **Toolchain**: MSVC (default via rustup on Windows)
- **No extra deps**: WASAPI and Win32 clipboard are part of the Windows SDK

## Running the Daemon as a Service

### Linux (systemd user service)

Create `~/.config/systemd/user/dyt-daemon.service`:

```ini
[Unit]
Description=DictateYourTerms STT Daemon

[Service]
ExecStart=/usr/local/bin/dyt-daemon
Restart=on-failure
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
```

```bash
systemctl --user enable dyt-daemon
systemctl --user start dyt-daemon
systemctl --user status dyt-daemon
journalctl --user -u dyt-daemon -f    # Follow logs
```

### Windows (Task Scheduler)

Create a basic task: trigger = at logon, action = path to `dyt-daemon.exe`, start in `%APPDATA%\dyt\`. Or simply run it in a terminal for development.

## Hotkey Integration

The CLI is designed to be invoked from a hotkey daemon:

```bash
# sxhkd (Linux)
super + v
    dyt --record

# i3/sway config
bindsym $mod+v exec dyt --record

# AutoHotkey (Windows)
^v::Run, dyt.exe --record
```

## Environment Variables

| Variable | Effect |
|----------|--------|
| `RUST_LOG` | Tracing filter (e.g., `info`, `debug`, `dyt_daemon=trace`) |

## Release Checklist

- [ ] `cargo test` passes on both Linux and Windows
- [ ] `cargo build --release` succeeds on both platforms
- [ ] Daemon starts and loads model without error
- [ ] `dyt --record` produces correct transcript end-to-end
- [ ] Linux binaries stripped
- [ ] Release artifacts uploaded to GitHub Releases

## Cross-References

- System dependencies: `docs/setup.md`
- Architecture overview: `docs/architecture.md`
- Distribution spec: `specs/distribution.md`
