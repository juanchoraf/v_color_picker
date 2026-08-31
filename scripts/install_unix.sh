#!/usr/bin/env bash
set -euo pipefail

APP="v_color_picker"
INSTALL_BIN="/usr/local/bin/$APP"
ICON_DIR="/usr/local/share/icons/hicolor/scalable/apps"
DESKTOP_DIR="/usr/local/share/applications"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "${EUID:-$(id -u)}" -ne 0 ]; then
  echo "Run as root: sudo $0" >&2
  exit 1
fi

SOURCE_BIN="$SCRIPT_DIR/$APP"
if [ ! -f "$SOURCE_BIN" ]; then
  SOURCE_BIN="$(ls "$SCRIPT_DIR"/${APP}_v*_*.bin 2>/dev/null | head -n 1 || true)"
fi

if [ -z "$SOURCE_BIN" ] || [ ! -f "$SOURCE_BIN" ]; then
  echo "Missing $APP binary next to this installer script." >&2
  exit 1
fi

install -Dm755 "$SOURCE_BIN" "$INSTALL_BIN"
if [ -f "$SCRIPT_DIR/logo.svg" ]; then
  install -Dm644 "$SCRIPT_DIR/logo.svg" "$ICON_DIR/$APP.svg"
fi

install -Dm644 /dev/stdin "$DESKTOP_DIR/com.thevelasquez.${APP}.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=V_Color_Picker
Comment=Pick and inspect exact screen colors
Exec=$INSTALL_BIN
Icon=$APP
Terminal=false
Categories=Graphics;Utility;
StartupNotify=true
DESKTOP

echo "Installed $APP for all users."
