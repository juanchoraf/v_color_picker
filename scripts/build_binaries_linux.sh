#!/usr/bin/env bash
set -euo pipefail
umask 022

APP="v_color_picker"
PACKAGE="v-color-picker"
PUBLISHER="TheVelasquez.com"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPDATE_DEPS=1
CARGO_LOCKED=""
TMP_ROOT=""

cleanup() {
  if [ -n "$TMP_ROOT" ]; then
    rm -rf "$TMP_ROOT"
  fi
}
trap cleanup EXIT

find_runtime_library() {
  local name="$1"
  local candidate

  if [ -n "${MULTIARCH:-}" ]; then
    for candidate in "/lib/$MULTIARCH/$name" "/usr/lib/$MULTIARCH/$name"; do
      if [ -f "$candidate" ]; then
        printf '%s\n' "$candidate"
        return 0
      fi
    done
  fi

  for candidate in "/lib64/$name" "/usr/lib64/$name" "/lib/$name" "/usr/lib/$name"; do
    if [ -f "$candidate" ]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done

  if command -v ldconfig >/dev/null 2>&1; then
    ldconfig -p 2>/dev/null | awk -v lib="$name" '$1 == lib { print $NF; exit }'
  fi
}

bundle_runtime_library() {
  local name="$1"
  local source

  source="$(find_runtime_library "$name" || true)"
  if [ -n "$source" ] && [ -f "$source" ]; then
    install -Dm644 "$source" "$APP_LIB_DIR/$name"
    echo "Bundled runtime library: $name"
  fi
}

filter_deb_depends() {
  local depends="$1"
  local filtered=""
  local dep
  local dep_name
  local -a dep_array

  IFS=',' read -ra dep_array <<< "$depends"
  for dep in "${dep_array[@]}"; do
    dep="${dep#"${dep%%[![:space:]]*}"}"
    dep="${dep%"${dep##*[![:space:]]}"}"
    dep_name="${dep%% *}"
    case "$dep_name" in
      libbsd0|libgcc-s1|libmd0|libxcb1|libxau6|libxdmcp6)
        continue
        ;;
    esac
    if [ -n "$dep" ]; then
      if [ -n "$filtered" ]; then
        filtered="$filtered, $dep"
      else
        filtered="$dep"
      fi
    fi
  done

  printf '%s\n' "$filtered"
}

write_linux_launcher() {
  local launcher="$1"

  install -Dm755 /dev/stdin "$launcher" <<'LAUNCHER'
#!/usr/bin/env bash
set -e

APP_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
export LD_LIBRARY_PATH="$APP_DIR/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$APP_DIR/bin/v_color_picker" "$@"
LAUNCHER
}

usage() {
  cat <<'USAGE'
Usage: scripts/build_binaries_linux.sh [--locked] [--no-update]

Builds Linux 64-bit v_color_picker artifacts for the current machine.

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
case "$DEB_ARCH" in
  amd64|arm64) ;;
  *)
    echo "Invalid Debian architecture mapping: $DEB_ARCH" >&2
    exit 1
    ;;
esac

OUT_DIR="$ROOT_DIR/versions/${APP}_v${VERSION}"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/${APP}_linux_package.XXXXXX")"
ARTIFACT_ARCH="linux_${ARCH}"
DEB_ROOT="$TMP_ROOT/deb_root"
APP_DIR="$DEB_ROOT/opt/$APP"
APP_BIN_DIR="$APP_DIR/bin"
APP_LIB_DIR="$APP_DIR/lib"
APP_BIN="$APP_BIN_DIR/$APP"
APP_LAUNCHER="$APP_DIR/$APP"
TAR_STAGE="$TMP_ROOT/linux_stage"
BIN_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.bin"
DEB_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.deb"
TAR_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.tar.gz"
MULTIARCH="$(dpkg-architecture -qDEB_HOST_MULTIARCH 2>/dev/null || true)"

mkdir -p "$OUT_DIR"
if [ "$UPDATE_DEPS" -eq 1 ]; then
  cargo update
fi
cargo build --release $CARGO_LOCKED --manifest-path "$ROOT_DIR/Cargo.toml"

cp "$ROOT_DIR/target/release/$APP" "$BIN_OUT"
chmod 755 "$BIN_OUT"
cp "$ROOT_DIR/assets/logo.svg" "$OUT_DIR/logo.svg"

rm -rf "$DEB_ROOT"
install -Dm755 "$ROOT_DIR/target/release/$APP" "$APP_BIN"
write_linux_launcher "$APP_LAUNCHER"
bundle_runtime_library "libgcc_s.so.1"
bundle_runtime_library "libxcb.so.1"
bundle_runtime_library "libXau.so.6"
bundle_runtime_library "libXdmcp.so.6"
bundle_runtime_library "libbsd.so.0"
bundle_runtime_library "libmd.so.0"
install -Dm644 "$ROOT_DIR/assets/logo.svg" "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/$APP.svg"

install -Dm644 /dev/stdin "$DEB_ROOT/usr/share/applications/com.thevelasquez.${APP}.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=v_color_picker
Comment=Pick and inspect exact screen colors
Exec=/opt/$APP/$APP
Icon=$APP
Terminal=false
Categories=Graphics;Utility;
StartupNotify=true
DESKTOP

install -Dm644 /dev/stdin "$DEB_ROOT/usr/share/metainfo/com.thevelasquez.${APP}.metainfo.xml" <<METAINFO
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.thevelasquez.$APP</id>
  <name>v_color_picker</name>
  <summary>Pick exact screen colors and copy them in many formats</summary>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <developer_name>$PUBLISHER</developer_name>
  <icon type="stock">$APP</icon>
  <launchable type="desktop-id">com.thevelasquez.$APP.desktop</launchable>
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
  rm -rf /opt/v_color_picker
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

INSTALLED_SIZE="$(du -sk "$DEB_ROOT/opt" "$DEB_ROOT/usr" | awk '{total += $1} END {print total}')"
DEB_DEPENDS="libc6"
if command -v dpkg-shlibdeps >/dev/null 2>&1; then
  SHLIBS_WORK="$TMP_ROOT/shlibdeps_work"
  rm -rf "$SHLIBS_WORK"
  mkdir -p "$SHLIBS_WORK/debian"
  install -Dm644 /dev/stdin "$SHLIBS_WORK/debian/control" <<SHLIBS_CONTROL
Source: $PACKAGE
Section: graphics
Priority: optional
Maintainer: $PUBLISHER <support@thevelasquez.com>
Standards-Version: 4.7.0

Package: $PACKAGE
Architecture: any
Depends: \${shlibs:Depends}
Description: Fast desktop color picker
 Fast desktop color picker.
SHLIBS_CONTROL
  SHLIBS_LINE="$(cd "$SHLIBS_WORK" && dpkg-shlibdeps --ignore-missing-info -O "$APP_BIN" 2>/dev/null || true)"
  rm -rf "$SHLIBS_WORK"
  if [ -n "$SHLIBS_LINE" ] && [ "${SHLIBS_LINE#shlibs:Depends=}" != "$SHLIBS_LINE" ]; then
    DEB_DEPENDS="${SHLIBS_LINE#shlibs:Depends=}"
  fi
fi
DEB_DEPENDS="$(filter_deb_depends "$DEB_DEPENDS")"
if [ -z "$DEB_DEPENDS" ]; then
  DEB_DEPENDS="libc6"
fi
install -Dm644 /dev/stdin "$DEB_ROOT/DEBIAN/control" <<CONTROL
Package: $PACKAGE
Version: $VERSION
Section: graphics
Priority: optional
Architecture: $DEB_ARCH
Maintainer: $PUBLISHER <support@thevelasquez.com>
Installed-Size: $INSTALLED_SIZE
Depends: $DEB_DEPENDS
Homepage: https://thevelasquez.com
Description: Fast desktop color picker
 Pick exact colors from every attached monitor and copy HEX, RGB, RGBA,
 HSL, HWB, LCH, OKLCH, LAB, OKLAB, SRGB, and related formats.
CONTROL

chmod 755 "$DEB_ROOT" "$DEB_ROOT/DEBIAN"
chmod 755 "$APP_DIR" "$APP_BIN_DIR"
if [ -d "$APP_LIB_DIR" ]; then
  chmod 755 "$APP_LIB_DIR"
  find "$APP_LIB_DIR" -type f -exec chmod 644 {} +
fi
chmod 644 "$DEB_ROOT/DEBIAN/control"
chmod 755 "$DEB_ROOT/DEBIAN/postinst" "$DEB_ROOT/DEBIAN/postrm"

if command -v dpkg-deb >/dev/null 2>&1; then
  dpkg-deb --build --root-owner-group "$DEB_ROOT" "$DEB_OUT"
else
  echo "dpkg-deb not found; skipped .deb packaging" >&2
fi

rm -rf "$TAR_STAGE"
mkdir -p "$TAR_STAGE/bin" "$TAR_STAGE/lib"
cp "$APP_LAUNCHER" "$TAR_STAGE/$APP"
cp "$APP_BIN" "$TAR_STAGE/bin/$APP"
if [ -d "$APP_LIB_DIR" ]; then
  cp -a "$APP_LIB_DIR/." "$TAR_STAGE/lib/"
fi
cp "$ROOT_DIR/assets/logo.svg" "$TAR_STAGE/logo.svg"
tar -czf "$TAR_OUT" -C "$TAR_STAGE" "$APP" "bin" "lib" "logo.svg"
cp "$ROOT_DIR/scripts/install_linux.sh" "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_install.sh"
cp "$ROOT_DIR/scripts/uninstall_linux.sh" "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_uninstall.sh"
chmod 755 "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_install.sh"
chmod 755 "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_uninstall.sh"

echo "Release files written to $OUT_DIR"
