# orb task runner. Install: `cargo install just`.

set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Build and run the headless CLI
cli *ARGS:
    cargo run -p orb-cli -- {{ARGS}}

# Run the desktop app in dev mode (vite + tauri)
dev:
    cargo tauri dev

# Build release bundles for the current OS
bundle:
    cargo tauri build

# Format, lint, test everything
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    pnpm --filter orb-desktop exec tsc --noEmit

fmt:
    cargo fmt --all

# Download LGPL ffmpeg/ffprobe static builds as Tauri sidecars for the host triple
fetch-sidecars:
    scripts/fetch-sidecars.sh

# Regenerate app icons from the master PNG
icons:
    cargo tauri icon apps/desktop/src-tauri/app-icon.png -o apps/desktop/src-tauri/icons
