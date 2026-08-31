#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root: sudo scripts/uninstall_macos.sh" >&2
  exit 1
fi

APP="v_color_picker"
rm -rf "/Applications/${APP}.app"
rm -f "/usr/local/bin/$APP"
rm -rf /Library/Application\ Support/v_color_picker /Library/Caches/v_color_picker
rm -rf /root/Library/Application\ Support/v_color_picker /root/Library/Caches/v_color_picker

for home in /Users/*; do
  [ -d "$home" ] || continue
  rm -rf "$home/Library/Application Support/v_color_picker"
  rm -rf "$home/Library/Caches/v_color_picker"
  rm -f "$home/Library/Preferences/com.thevelasquez.v_color_picker.plist"
done

echo "Uninstalled $APP and removed app traces."
