#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: scripts/ios_e2e_project.sh --project <project_path> [options]

Required:
  --project <path>             Project root (must contain main.py)

Optional:
  --bundle-id <id>             App bundle id for simulator install/launch
                               default: com.example.pycro
  --device-name <name>         iOS Simulator device name
                               default: iPhone 16
  --sim-udid <udid>            Explicit simulator UDID (overrides --device-name)
  --configuration <name>       Xcode configuration
                               default: release
  --derived-data <path>        xcodebuild derived data path
                               default: /tmp/pycro-ios-dd
  --pycro-bin <path>           pycro executable path
                               default: <repo>/target/release/pycro
  --cargo <path>               Cargo executable (or command name)
                               default: cargo
  --skip-install               Build only; skip simulator install/launch
  --skip-screenshot            Do not capture screenshot after launch
  --help                       Show this help

Example:
  scripts/ios_e2e_project.sh --project /path/to/example
EOF
}

PROJECT_PATH=""
BUNDLE_ID="com.example.pycro"
DEVICE_NAME="iPhone 16"
SIM_UDID=""
CONFIGURATION="release"
DERIVED_DATA="/tmp/pycro-ios-dd"
SKIP_INSTALL=0
SKIP_SCREENSHOT=0

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_CMD="${CARGO:-cargo}"
PYCRO_BIN="$REPO_ROOT/target/release/pycro"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --project)
      PROJECT_PATH="${2:-}"
      shift 2
      ;;
    --bundle-id)
      BUNDLE_ID="${2:-}"
      shift 2
      ;;
    --device-name)
      DEVICE_NAME="${2:-}"
      shift 2
      ;;
    --sim-udid)
      SIM_UDID="${2:-}"
      shift 2
      ;;
    --configuration)
      CONFIGURATION="${2:-}"
      shift 2
      ;;
    --derived-data)
      DERIVED_DATA="${2:-}"
      shift 2
      ;;
    --pycro-bin)
      PYCRO_BIN="${2:-}"
      shift 2
      ;;
    --cargo)
      CARGO_CMD="${2:-}"
      shift 2
      ;;
    --skip-install)
      SKIP_INSTALL=1
      shift
      ;;
    --skip-screenshot)
      SKIP_SCREENSHOT=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1"
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$PROJECT_PATH" ]]; then
  echo "error: --project is required"
  usage
  exit 2
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: iOS E2E requires macOS/Xcode"
  exit 1
fi

if [[ ! -d "$PROJECT_PATH" ]]; then
  echo "error: project path is not a directory: $PROJECT_PATH"
  exit 1
fi

if [[ ! -f "$PROJECT_PATH/main.py" ]]; then
  echo "error: project root must contain main.py: $PROJECT_PATH"
  exit 1
fi

for cmd in xcodebuild xcrun open; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd"
    exit 1
  fi
done

if ! command -v "$CARGO_CMD" >/dev/null 2>&1; then
  echo "error: cargo command not found: $CARGO_CMD"
  exit 1
fi

if [[ ! -x "$PYCRO_BIN" ]]; then
  echo "building pycro CLI binary: $PYCRO_BIN"
  (cd "$REPO_ROOT" && "$CARGO_CMD" build --release --bin pycro)
fi

echo "== iOS dist build =="
echo "project: $PROJECT_PATH"
echo "pycro:   $PYCRO_BIN"

"$PYCRO_BIN" project build --project "$PROJECT_PATH" --target ios

XCODE_WORKSPACE="$PROJECT_PATH/dist/ios/xcode/pycro.xcodeproj/project.xcworkspace"
if [[ ! -d "$XCODE_WORKSPACE" ]]; then
  echo "error: generated workspace not found: $XCODE_WORKSPACE"
  exit 1
fi

echo "== xcodebuild (clean build) =="
PYCRO_EMBED_PROJECT_ROOT="$PROJECT_PATH" \
  xcodebuild \
    -workspace "$XCODE_WORKSPACE" \
    -scheme pycro_iOS \
    -sdk iphonesimulator \
    -configuration "$CONFIGURATION" \
    -derivedDataPath "$DERIVED_DATA" \
    clean build

APP_PATH="$DERIVED_DATA/Build/Products/${CONFIGURATION}-iphonesimulator/Pycro.app"
if [[ ! -d "$APP_PATH" ]]; then
  echo "error: built app not found: $APP_PATH"
  exit 1
fi

if [[ "$SKIP_INSTALL" -eq 1 ]]; then
  echo "build completed (install/launch skipped)"
  echo "workspace: $XCODE_WORKSPACE"
  echo "app:       $APP_PATH"
  exit 0
fi

if [[ -z "$SIM_UDID" ]]; then
  SIM_UDID="$(xcrun simctl list devices available | sed -nE "s/^[[:space:]]*${DEVICE_NAME//\//\\/} \(([0-9A-F-]+)\) .*/\1/p" | head -n 1)"
fi

if [[ -z "$SIM_UDID" ]]; then
  echo "error: could not resolve simulator UDID for device: $DEVICE_NAME"
  exit 1
fi

echo "== simulator install/launch =="
echo "device: $DEVICE_NAME ($SIM_UDID)"

open -a Simulator || true
xcrun simctl boot "$SIM_UDID" >/dev/null 2>&1 || true
xcrun simctl uninstall "$SIM_UDID" "$BUNDLE_ID" >/dev/null 2>&1 || true
xcrun simctl install "$SIM_UDID" "$APP_PATH"
xcrun simctl launch "$SIM_UDID" "$BUNDLE_ID"

if [[ "$SKIP_SCREENSHOT" -eq 0 ]]; then
  SCREENSHOT_PATH="$PROJECT_PATH/dist/ios/e2e-screenshot.png"
  mkdir -p "$(dirname "$SCREENSHOT_PATH")"
  sleep 4
  xcrun simctl io "$SIM_UDID" screenshot "$SCREENSHOT_PATH"
  echo "screenshot: $SCREENSHOT_PATH"
fi

echo "done"
echo "workspace:  $XCODE_WORKSPACE"
echo "app:        $APP_PATH"
echo "simulator:  $SIM_UDID"
