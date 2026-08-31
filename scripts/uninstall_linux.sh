#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root: sudo scripts/uninstall_linux.sh" >&2
  exit 1
fi

APP="v_color_picker"
rm -f "/usr/local/bin/$APP" "/usr/bin/$APP"
rm -f "/usr/local/share/applications/com.thevelasquez.${APP}.desktop"
rm -f "/usr/share/applications/com.thevelasquez.${APP}.desktop"
rm -f "/usr/local/share/icons/hicolor/scalable/apps/$APP.svg"
rm -f "/usr/share/icons/hicolor/scalable/apps/$APP.svg"
rm -f "/usr/share/metainfo/com.thevelasquez.${APP}.metainfo.xml"
rm -rf /var/cache/v_color_picker /var/lib/v_color_picker
rm -rf /root/.config/v_color_picker /root/.local/share/v_color_picker /root/.cache/v_color_picker

if [ -d /home ]; then
  find /home -maxdepth 3 \( -path '*/.config/v_color_picker' -o -path '*/.local/share/v_color_picker' -o -path '*/.cache/v_color_picker' \) -exec rm -rf {} +
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/local/share/applications /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/local/share/icons/hicolor 2>/dev/null || true
  gtk-update-icon-cache -q /usr/share/icons/hicolor 2>/dev/null || true
fi

echo "Uninstalled $APP and removed app traces."
