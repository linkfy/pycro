#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_interactive_pair() {
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
  local pycro_profile="${PYCRO_CARGO_PROFILE:---release}"

  echo "[compare] starting pycro benchmark..."
  cargo run ${pycro_profile} -- examples/compare_pycro_balls_benchmark.py &
  PYCRO_PID=$!

  echo "[compare] starting pygame benchmark..."
  python3 examples/compare_pygame_balls_benchmark.py &
  PYGAME_PID=$!

  echo "[compare] pycro pid=${PYCRO_PID}, pygame pid=${PYGAME_PID}"
  echo "[compare] close each window or press Ctrl+C to stop both"
  echo "[compare] non-interactive mode: BENCHMARK_MATRIX=1 ./examples/run_compare_benchmarks.sh"

  wait "${PYCRO_PID}" "${PYGAME_PID}"
}

run_auto_matrix() {
  cd "${ROOT_DIR}"

  local runs="${BENCHMARK_RUNS:-3}"
  local targets_csv="${BENCHMARK_TARGET_MATRIX:-3000}"
  local session_seconds="${BENCHMARK_AUTO_SESSION_SECONDS:-3}"
  local step_interval="${BENCHMARK_AUTO_STEP_INTERVAL:-0.08}"
  local hold_seconds="${BENCHMARK_AUTO_HOLD_SECONDS:-2.0}"
  local auto_initial_balls="${BENCHMARK_AUTO_INITIAL_BALLS:-3000}"
  local out_dir="${BENCHMARK_OUT_DIR:-examples/compare/results}"
  local pycro_profile="${PYCRO_CARGO_PROFILE:---release}"

  mkdir -p "${out_dir}"

  local session_seconds_int="${session_seconds%.*}"
  local pycro_frames_default=$((session_seconds_int * 180 + 180))
  local pycro_frames="${PYCRO_FRAMES:-${pycro_frames_default}}"

  IFS=',' read -r -a targets <<< "${targets_csv}"

  echo "[compare] auto matrix runs=${runs} targets=${targets_csv} session=${session_seconds}s initial_balls=${auto_initial_balls}"
  echo "[compare] output_dir=${out_dir}"

  for runtime in pycro pygame; do
    for target in "${targets[@]}"; do
      for run in $(seq 1 "${runs}"); do
        local log_file="${out_dir}/${runtime}_target${target}_run${run}.log"
        echo "[compare] runtime=${runtime} target=${target} run=${run}"

        if [[ "${runtime}" == "pycro" ]]; then
          BENCHMARK_AUTO=1 \
          BENCHMARK_AUTO_TARGETS="${target}" \
          BENCHMARK_AUTO_INITIAL_BALLS="${auto_initial_balls}" \
          BENCHMARK_AUTO_STEP_INTERVAL="${step_interval}" \
          BENCHMARK_AUTO_HOLD_SECONDS="${hold_seconds}" \
          BENCHMARK_AUTO_SESSION_SECONDS="${session_seconds}" \
          PYCRO_FRAMES="${pycro_frames}" \
          cargo run ${pycro_profile} -- examples/compare_pycro_balls_benchmark.py >"${log_file}" 2>&1
        else
          SDL_VIDEODRIVER=dummy \
          BENCHMARK_AUTO=1 \
          BENCHMARK_AUTO_TARGETS="${target}" \
          BENCHMARK_AUTO_INITIAL_BALLS="${auto_initial_balls}" \
          BENCHMARK_AUTO_STEP_INTERVAL="${step_interval}" \
          BENCHMARK_AUTO_HOLD_SECONDS="${hold_seconds}" \
          BENCHMARK_AUTO_SESSION_SECONDS="${session_seconds}" \
          python3 examples/compare_pygame_balls_benchmark.py >"${log_file}" 2>&1
        fi

        local summary_line
        summary_line="$(rg -N "event=summary" "${log_file}" | tail -n 1 || true)"
        if [[ -n "${summary_line}" ]]; then
          echo "[compare] ${summary_line}"
        else
          echo "[compare] warning: no summary line in ${log_file}"
        fi
      done
    done
  done

  echo "[compare] completed auto matrix"
}

if [[ "${BENCHMARK_MATRIX:-0}" == "1" ]]; then
  run_auto_matrix
else
  run_interactive_pair
fi
