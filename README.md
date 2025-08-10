# aw-watcher-win-vdesktop

ActivityWatch watcher for Windows virtual desktops. Tracks the current virtual desktop name (or index) and sends periodic heartbeats to ActivityWatch.

Super early version: APIs and flags may change, and features are minimal.

## What it does

- Polls the current Windows virtual desktop every ~8 seconds
- Sends ActivityWatch heartbeats (10s pulsing) with the desktop name
- Creates/uses a bucket named: `aw-watcher-win-vdesktop_<hostname>`

Built on top of the winvd crate:
- winvd: https://docs.rs/winvd/latest/winvd/
- ActivityWatch: https://activitywatch.net/

## Requirements

- Windows 10/11
- Rust toolchain (stable)
- ActivityWatch server running locally (default: `localhost:5600`)

## Build

```powershell
cargo build --release
```

## Run

```powershell
# Default (connects to aw-server on localhost:5600)
cargo run --release

# Specify a custom port
cargo run --release -- --port 5601

# "Testing" mode uses port 5666 unless --port is provided
# Note: current flag name has a typo and is spelled --tesitng
cargo run --release -- --testing
```

## Notes and limitations

- Poll-based (no event hooks yet); may miss rapid changes between polls
- No installer/service; run as a foreground process
- Data schema and bucket metadata are likely to change