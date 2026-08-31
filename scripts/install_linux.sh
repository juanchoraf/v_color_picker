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
INSTALL_BIN="$INSTALL_DIR/bin/$APP"
INSTALL_LAUNCHER="$INSTALL_DIR/$APP"
INSTALL_LIB_DIR="$INSTALL_DIR/lib"
ICON_SRC="$SCRIPT_DIR/logo.svg"

write_installed_launcher() {
  install -Dm755 /dev/stdin "$INSTALL_LAUNCHER" <<'LAUNCHER'
#!/usr/bin/env bash
set -e

APP_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
export LD_LIBRARY_PATH="$APP_DIR/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$APP_DIR/bin/v_color_picker" "$@"
LAUNCHER
}

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
if [ ! -f "$ICON_SRC" ]; then
  ICON_SRC="$ROOT_DIR/assets/logo.svg"
fi

if [ -x "$SCRIPT_DIR/$APP" ] && [ -x "$SCRIPT_DIR/bin/$APP" ]; then
  install -Dm755 "$SCRIPT_DIR/$APP" "$INSTALL_LAUNCHER"
  install -Dm755 "$SCRIPT_DIR/bin/$APP" "$INSTALL_BIN"
  if [ -d "$SCRIPT_DIR/lib" ]; then
    rm -rf "$INSTALL_LIB_DIR"
    mkdir -p "$INSTALL_LIB_DIR"
    cp -a "$SCRIPT_DIR/lib/." "$INSTALL_LIB_DIR/"
    find "$INSTALL_LIB_DIR" -type f -exec chmod 644 {} +
  fi
else
  install -Dm755 "$BINARY" "$INSTALL_BIN"
  write_installed_launcher
fi
if [ -f "$ICON_SRC" ]; then
  install -Dm644 "$ICON_SRC" "/usr/share/icons/hicolor/scalable/apps/$APP.svg"
fi
install -Dm644 /dev/stdin "/usr/share/applications/com.thevelasquez.${APP}.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=v_color_picker
Comment=Pick and inspect exact screen colors
Exec=$INSTALL_LAUNCHER
Icon=$APP
Terminal=false
Categories=Graphics;Utility;
StartupNotify=true
DESKTOP

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi

echo "Installed $APP for all users."
