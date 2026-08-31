#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root: sudo scripts/install_linux.sh" >&2
  exit 1
fi

APP="v_color_picker"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$ROOT_DIR/target/release/$APP"

if [ ! -x "$BINARY" ]; then
  echo "Missing release binary: $BINARY" >&2
  echo "Run scripts/build_binaries.sh first." >&2
  exit 1
fi

install -Dm755 "$BINARY" "/usr/local/bin/$APP"
install -Dm644 "$ROOT_DIR/assets/logo.svg" "/usr/local/share/icons/hicolor/scalable/apps/$APP.svg"
install -Dm644 /dev/stdin "/usr/local/share/applications/com.thevelasquez.${APP}.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=V_Color_Picker
Comment=Pick and inspect exact screen colors
Exec=/usr/local/bin/$APP
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
