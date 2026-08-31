#!/usr/bin/env bash
set -euo pipefail

APP="v_color_picker"
PACKAGE="v-color-picker"
PUBLISHER="TheVelasquez.com"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPDATE_DEPS=1
CARGO_LOCKED=""

usage() {
  cat <<'USAGE'
Usage: scripts/build_binaries_linux.sh [--locked] [--no-update]

Builds Linux 64-bit V_Color_Picker artifacts for the current machine.

Options:
  --locked     Use Cargo.lock exactly as-is
  --no-update  Do not run cargo update before building
  -h, --help   Show this help
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --locked)
      UPDATE_DEPS=0
      CARGO_LOCKED="--locked"
      shift
      ;;
    --no-update)
      UPDATE_DEPS=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

cd "$ROOT_DIR"
VERSION="$(awk -F'"' '/^version = / { print $2; exit }' "$ROOT_DIR/Cargo.toml")"
RAW_ARCH="$(uname -m)"

case "$RAW_ARCH" in
  x86_64|amd64)
    ARCH="x86_64"
    DEB_ARCH="amd64"
    ;;
  aarch64|arm64)
    ARCH="arm64"
    DEB_ARCH="arm64"
    ;;
  *)
    echo "Only 64-bit builds are supported. Found: $RAW_ARCH" >&2
    exit 1
    ;;
esac

OUT_DIR="$ROOT_DIR/versions/${APP}_v${VERSION}"
ARTIFACT_ARCH="linux_${ARCH}"
DEB_ROOT="$OUT_DIR/deb_root"
BIN_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.bin"
DEB_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.deb"
TAR_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.tar.gz"

mkdir -p "$OUT_DIR"
if [ "$UPDATE_DEPS" -eq 1 ]; then
  cargo update
fi
cargo build --release $CARGO_LOCKED --manifest-path "$ROOT_DIR/Cargo.toml"

cp "$ROOT_DIR/target/release/$APP" "$BIN_OUT"
chmod 755 "$BIN_OUT"

rm -rf "$DEB_ROOT"
install -Dm755 "$ROOT_DIR/target/release/$APP" "$DEB_ROOT/usr/bin/$APP"
install -Dm644 "$ROOT_DIR/assets/logo.svg" "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/$APP.svg"

install -Dm644 /dev/stdin "$DEB_ROOT/usr/share/applications/com.thevelasquez.${APP}.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=V_Color_Picker
Comment=Pick and inspect exact screen colors
Exec=/usr/bin/$APP
Icon=$APP
Terminal=false
Categories=Graphics;Utility;
StartupNotify=true
DESKTOP

install -Dm644 /dev/stdin "$DEB_ROOT/usr/share/metainfo/com.thevelasquez.${APP}.metainfo.xml" <<METAINFO
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.thevelasquez.$APP</id>
  <name>V_Color_Picker</name>
  <summary>Pick exact screen colors and copy them in many formats</summary>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <developer_name>$PUBLISHER</developer_name>
  <launchable type="desktop-id">com.thevelasquez.$APP.desktop</launchable>
  <provides>
    <binary>$APP</binary>
  </provides>
  <releases>
    <release version="$VERSION"/>
  </releases>
</component>
METAINFO

install -Dm755 /dev/stdin "$DEB_ROOT/DEBIAN/postinst" <<'POSTINST'
#!/usr/bin/env bash
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
POSTINST

install -Dm755 /dev/stdin "$DEB_ROOT/DEBIAN/postrm" <<'POSTRM'
#!/usr/bin/env bash
set -e
if [ "$1" = "purge" ] || [ "$1" = "remove" ]; then
  rm -rf /var/cache/v_color_picker /var/lib/v_color_picker
  rm -rf /root/.config/v_color_picker /root/.local/share/v_color_picker /root/.cache/v_color_picker
  if [ -d /home ]; then
    find /home -maxdepth 3 \( -path '*/.config/v_color_picker' -o -path '*/.local/share/v_color_picker' -o -path '*/.cache/v_color_picker' \) -exec rm -rf {} +
  fi
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
POSTRM

INSTALLED_SIZE="$(du -sk "$DEB_ROOT/usr" | awk '{print $1}')"
install -Dm644 /dev/stdin "$DEB_ROOT/DEBIAN/control" <<CONTROL
Package: $PACKAGE
Version: $VERSION
Section: graphics
Priority: optional
Architecture: $DEB_ARCH
Maintainer: $PUBLISHER <support@thevelasquez.com>
Installed-Size: $INSTALLED_SIZE
Depends: libc6, libxcb1
Homepage: https://thevelasquez.com
Description: Fast desktop color picker
 Pick exact colors from every attached monitor and copy HEX, RGB, RGBA,
 HSL, HWB, LCH, OKLCH, LAB, OKLAB, SRGB, and related formats.
CONTROL

if command -v dpkg-deb >/dev/null 2>&1; then
  dpkg-deb --build --root-owner-group "$DEB_ROOT" "$DEB_OUT"
else
  echo "dpkg-deb not found; skipped .deb packaging" >&2
fi

tar -czf "$TAR_OUT" -C "$OUT_DIR" "$(basename "$BIN_OUT")"
cp "$ROOT_DIR/scripts/install_linux.sh" "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_install.sh"
cp "$ROOT_DIR/scripts/uninstall_linux.sh" "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_uninstall.sh"
chmod 755 "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_install.sh"
chmod 755 "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_uninstall.sh"

echo "Release files written to $OUT_DIR"
