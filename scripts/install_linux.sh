#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root: sudo scripts/install_linux.sh" >&2
  exit 1
fi

APP="v_color_picker"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY="$ROOT_DIR/target/release/$APP"
INSTALL_DIR="/opt/$APP"
INSTALL_BIN="$INSTALL_DIR/$APP"

if [ ! -x "$BINARY" ]; then
  BINARY="$SCRIPT_DIR/$APP"
fi
if [ ! -x "$BINARY" ]; then
  BINARY="$(ls "$SCRIPT_DIR"/${APP}_v*_linux_*.bin 2>/dev/null | head -n 1 || true)"
fi
if [ -z "$BINARY" ] || [ ! -f "$BINARY" ]; then
  echo "Missing release binary next to this installer script." >&2
  exit 1
fi

install -Dm755 "$BINARY" "$INSTALL_BIN"
install -Dm644 "$ROOT_DIR/assets/logo.svg" "/usr/local/share/icons/hicolor/scalable/apps/$APP.svg"
install -Dm644 /dev/stdin "/usr/local/share/applications/com.thevelasquez.${APP}.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=v_color_picker
Comment=Pick and inspect exact screen colors
Exec=$INSTALL_BIN
Icon=$APP
Terminal=false
Categories=Graphics;Utility;
StartupNotify=true
DESKTOP

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/local/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/local/share/icons/hicolor || true
fi

echo "Installed $APP for all users."
