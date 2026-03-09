#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cleanup() {
  if [[ -n "${PYCRO_PID:-}" ]] && kill -0 "${PYCRO_PID}" 2>/dev/null; then
    kill "${PYCRO_PID}" 2>/dev/null || true
  fi
  if [[ -n "${PYGAME_PID:-}" ]] && kill -0 "${PYGAME_PID}" 2>/dev/null; then
    kill "${PYGAME_PID}" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

cd "${ROOT_DIR}"

echo "[compare] starting pycro benchmark..."
cargo run -- examples/compare_pycro_balls_benchmark.py &
PYCRO_PID=$!

echo "[compare] starting pygame benchmark..."
python3 examples/compare_pygame_balls_benchmark.py &
PYGAME_PID=$!

echo "[compare] pycro pid=${PYCRO_PID}, pygame pid=${PYGAME_PID}"
echo "[compare] close each window or press Ctrl+C to stop both"

wait "${PYCRO_PID}" "${PYGAME_PID}"
