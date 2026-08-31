#!/usr/bin/env bash
set -euo pipefail

APP="v_color_picker"

if [ "${EUID:-$(id -u)}" -ne 0 ]; then
  echo "Run as root: sudo $0" >&2
  exit 1
fi

rm -rf "/opt/$APP"
rm -f "/usr/local/bin/$APP"
rm -f "/usr/local/share/applications/com.thevelasquez.${APP}.desktop"
rm -f "/usr/local/share/icons/hicolor/scalable/apps/$APP.svg"
rm -rf "/var/cache/$APP" "/var/lib/$APP"
rm -rf "/root/.config/$APP" "/root/.local/share/$APP" "/root/.cache/$APP"

if [ -d /home ]; then
  find /home -maxdepth 3 \( -path "*/.config/$APP" -o -path "*/.local/share/$APP" -o -path "*/.cache/$APP" \) -exec rm -rf {} +
fi

echo "Uninstalled $APP and removed app traces."
