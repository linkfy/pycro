#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "" ]]; then
  echo "usage: scripts/ios_build_project.sh <project_path>"
  echo "example: scripts/ios_build_project.sh /path/to/project"
  exit 2
fi

PROJECT_PATH="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO="${CARGO:-cargo}"

if [[ ! -d "$PROJECT_PATH" ]]; then
  echo "error: project path is not a directory: $PROJECT_PATH"
  exit 1
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: iOS builds require macOS/Xcode"
  exit 1
fi

if ! command -v "$CARGO" >/dev/null 2>&1; then
  echo "error: cargo not found. install Rust stable with rustup."
  exit 1
fi

if ! "$CARGO" apple --help >/dev/null 2>&1; then
  echo "error: \`cargo apple\` not available. install cargo-mobile2:"
  echo "  cargo install --git https://github.com/tauri-apps/cargo-mobile2"
  exit 1
fi

cd "$REPO_ROOT"
if [[ ! -x "./target/release/pycro" ]]; then
  "$CARGO" build --release --bin pycro
fi

echo "building iOS package for project: $PROJECT_PATH"

./target/release/pycro \
  project build \
  --project "$PROJECT_PATH" \
  --target ios

echo "done"
echo "xcode: $PROJECT_PATH/dist/ios/xcode"
