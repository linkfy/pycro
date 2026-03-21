#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "" ]]; then
  echo "usage: scripts/android_build_project.sh <project_path>"
  echo "example: scripts/android_build_project.sh /path/to/project"
  exit 2
fi

PROJECT_PATH="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$ANDROID_HOME}"
export NDK_HOME="${NDK_HOME:-$ANDROID_HOME/ndk/21.4.7075529}"
export CARGO="${CARGO:-$(rustup which cargo 2>/dev/null || command -v cargo)}"

if ! command -v "${CARGO}" >/dev/null 2>&1; then
  echo "error: cargo not found. install Rust stable with rustup."
  exit 1
fi

if ! command -v cargo-quad-apk >/dev/null 2>&1; then
  echo "error: cargo-quad-apk not found. run: cargo install cargo-quad-apk --force"
  exit 1
fi

if [[ ! -d "$NDK_HOME" ]]; then
  echo "error: NDK_HOME does not exist: $NDK_HOME"
  exit 1
fi

# Apple Silicon + NDK 21 compatibility shims required by cargo-quad-apk.
if [[ "$(uname -s)" == "Darwin" && "$(uname -m)" == "arm64" ]]; then
  CLANG_BASE="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/lib64/clang/9.0.9/lib/linux"
  BIN_BASE="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
  if [[ -d "$CLANG_BASE/aarch64" && -f "$CLANG_BASE/libclang_rt.builtins-aarch64-android.a" ]]; then
    ln -sfn "$CLANG_BASE/libclang_rt.builtins-aarch64-android.a" "$CLANG_BASE/aarch64/libunwind.a"
  fi
  if [[ -f "$BIN_BASE/aarch64-linux-android-ld" ]]; then
    ln -sfn "$BIN_BASE/aarch64-linux-android-ld" "$BIN_BASE/ld"
  fi
fi

cd "$REPO_ROOT"
if [[ ! -x "./target/release/pycro" ]]; then
  "$CARGO" build --release --bin pycro
fi

echo "building Android package for project: $PROJECT_PATH"
echo "ANDROID_HOME=$ANDROID_HOME"
echo "NDK_HOME=$NDK_HOME"

CARGO="$CARGO" ./target/release/pycro \
  project build \
  --project "$PROJECT_PATH" \
  --target android

echo "done"
echo "apk: $PROJECT_PATH/dist/android/apk/pycro.apk"
