#!/usr/bin/env bash
# Downloads LGPL (non-GPL) static ffmpeg + ffprobe for the host target triple into
# apps/desktop/src-tauri/binaries/, named the way Tauri's externalBin expects:
#   ffmpeg-<triple>[.exe], ffprobe-<triple>[.exe]
# Source: https://github.com/BtbN/FFmpeg-Builds (gpl-free "lgpl" variants) for Linux/Windows,
#         https://evermeet.cx / https://www.osxexperts.net style builds for macOS are GPL; on macOS
#         we build from source via Homebrew formula flags in CI instead (see release.yml).
set -euo pipefail
cd "$(dirname "$0")/.."
OUT=apps/desktop/src-tauri/binaries
mkdir -p "$OUT"
TRIPLE=$(rustc -vV | sed -n 's/^host: //p')
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

case "$TRIPLE" in
  x86_64-unknown-linux-gnu)
    URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-lgpl.tar.xz"
    curl -fL "$URL" -o "$TMP/ff.tar.xz"
    tar -xJf "$TMP/ff.tar.xz" -C "$TMP"
    cp "$TMP"/ffmpeg-*/bin/ffmpeg  "$OUT/ffmpeg-$TRIPLE"
    cp "$TMP"/ffmpeg-*/bin/ffprobe "$OUT/ffprobe-$TRIPLE"
    ;;
  aarch64-unknown-linux-gnu)
    URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linuxarm64-lgpl.tar.xz"
    curl -fL "$URL" -o "$TMP/ff.tar.xz"
    tar -xJf "$TMP/ff.tar.xz" -C "$TMP"
    cp "$TMP"/ffmpeg-*/bin/ffmpeg  "$OUT/ffmpeg-$TRIPLE"
    cp "$TMP"/ffmpeg-*/bin/ffprobe "$OUT/ffprobe-$TRIPLE"
    ;;
  x86_64-pc-windows-msvc)
    URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-lgpl.zip"
    curl -fL "$URL" -o "$TMP/ff.zip"
    unzip -q "$TMP/ff.zip" -d "$TMP"
    cp "$TMP"/ffmpeg-*/bin/ffmpeg.exe  "$OUT/ffmpeg-$TRIPLE.exe"
    cp "$TMP"/ffmpeg-*/bin/ffprobe.exe "$OUT/ffprobe-$TRIPLE.exe"
    ;;
  *-apple-darwin)
    echo "macOS: no prebuilt LGPL static ffmpeg available; build with 'brew install ffmpeg' and copy" >&2
    echo "  \$(brew --prefix)/bin/ffmpeg  -> $OUT/ffmpeg-$TRIPLE" >&2
    echo "  \$(brew --prefix)/bin/ffprobe -> $OUT/ffprobe-$TRIPLE" >&2
    echo "(Homebrew ffmpeg is GPL; for commercial distribution build with --disable-gpl.)" >&2
    exit 1
    ;;
  *) echo "unsupported triple: $TRIPLE" >&2; exit 1 ;;
esac
chmod +x "$OUT"/ff*
ls -la "$OUT"
