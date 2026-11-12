#!/usr/bin/env bash
# Build the Rust cdylib for Android and place it where Gradle expects it.
#
# Prerequisites:
#   rustup target add aarch64-linux-android
#   cargo install cargo-ndk
#   Android NDK installed and ANDROID_NDK_HOME set (or auto-detected by cargo-ndk)
#
# Usage:
#   ./build-rust.sh            # debug build (arm64)
#   ./build-rust.sh --release  # release build (arm64)
set -euo pipefail

PROFILE="debug"
CARGO_FLAGS=()
if [[ "${1:-}" == "--release" ]]; then
    PROFILE="release"
    CARGO_FLAGS+=("--release")
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="$SCRIPT_DIR/app/src/main/jniLibs"

cargo ndk -t arm64-v8a -o "$OUT_DIR" build --lib "${CARGO_FLAGS[@]}"

echo "Built libsudoku.so ($PROFILE) into $OUT_DIR/arm64-v8a"
