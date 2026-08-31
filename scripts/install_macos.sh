#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root: sudo scripts/install_macos.sh" >&2
  exit 1
fi

APP="v_color_picker"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_BUNDLE="$ROOT_DIR/versions/${APP}.app"

if [ ! -d "$APP_BUNDLE" ]; then
  APP_BUNDLE="$ROOT_DIR/versions/${APP}_v$(awk -F'"' '/^version = / { print $2; exit }' "$ROOT_DIR/Cargo.toml")/${APP}.app"
fi
if [ ! -d "$APP_BUNDLE" ]; then
  echo "Missing app bundle. Run scripts/build_binaries.sh on macOS first." >&2
  exit 1
fi

rm -rf "/Applications/${APP}.app"
cp -R "$APP_BUNDLE" "/Applications/${APP}.app"
ln -sf "/Applications/${APP}.app/Contents/MacOS/$APP" "/usr/local/bin/$APP"

echo "Installed $APP for all users."
