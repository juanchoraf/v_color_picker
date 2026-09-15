#!/usr/bin/env bash
set -euo pipefail
umask 022

APP="v_color_picker"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPDATE_DEPS=1
CARGO_LOCKED=""

usage() {
  cat <<'USAGE'
Usage: scripts/build_binaries_unix.sh [--locked] [--no-update]

Builds Unix-like 64-bit v_color_picker artifacts for the current machine.

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
    ;;
  aarch64|arm64)
    ARCH="arm64"
    ;;
  *)
    echo "Only 64-bit builds are supported. Found: $RAW_ARCH" >&2
    exit 1
    ;;
esac

OS_NAME="$(uname -s)"
case "$OS_NAME" in
  FreeBSD|DragonFly)
    PACKAGE_TOOL="pkg"
    PREFIX="/usr/local"
    EXT="pkg"
    ;;
  OpenBSD)
    PACKAGE_TOOL="pkg_create"
    PREFIX="/usr/local"
    EXT="tgz"
    ;;
  NetBSD)
    PACKAGE_TOOL="pkg_create"
    PREFIX="/usr/pkg"
    EXT="tgz"
    ;;
  *)
    echo "No native package builder is configured for $OS_NAME." >&2
    exit 1
    ;;
esac
command -v "$PACKAGE_TOOL" >/dev/null 2>&1 || {
  echo "$PACKAGE_TOOL is required to build the $OS_NAME installer." >&2
  exit 1
}

OUT_DIR="$ROOT_DIR/versions/${APP}_v${VERSION}"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/${APP}_unix_package.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT
STAGE="$TMP_ROOT/root"
META="$TMP_ROOT/meta"
ARTIFACT_OS="$(printf '%s' "$OS_NAME" | tr '[:upper:]' '[:lower:]')"
PKG_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_OS}_${ARCH}.$EXT"

mkdir -p "$OUT_DIR" "$META" "$TMP_ROOT/packages"
if [ "$UPDATE_DEPS" -eq 1 ]; then
  cargo update
fi
cargo build --release $CARGO_LOCKED --manifest-path "$ROOT_DIR/Cargo.toml"

mkdir -p "$STAGE$PREFIX/bin" "$STAGE$PREFIX/share/applications" \
  "$STAGE$PREFIX/share/icons/hicolor/scalable/apps"
install -m 755 "$ROOT_DIR/target/release/$APP" "$STAGE$PREFIX/bin/$APP"
install -m 644 "$ROOT_DIR/assets/logo.svg" \
  "$STAGE$PREFIX/share/icons/hicolor/scalable/apps/$APP.svg"
cat > "$STAGE$PREFIX/share/applications/com.thevelasquez.$APP.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=v_color_picker
Comment=Pick and inspect exact screen colors
Exec=$PREFIX/bin/$APP
Icon=$APP
Terminal=false
Categories=Graphics;Utility;
StartupNotify=true
DESKTOP

cat > "$META/plist" <<PLIST
@owner root
@group wheel
bin/$APP
share/applications/com.thevelasquez.$APP.desktop
share/icons/hicolor/scalable/apps/$APP.svg
PLIST
printf '%s\n' 'Fast desktop color picker by TheVelasquez.com.' > "$META/description"

case "$OS_NAME" in
  FreeBSD|DragonFly)
    cat > "$META/+MANIFEST" <<MANIFEST
name: "$APP"
version: "$VERSION"
origin: "graphics/$APP"
comment: "Fast desktop color picker"
desc: "Pick exact screen colors. Published by TheVelasquez.com."
maintainer: "support@thevelasquez.com"
www: "https://thevelasquez.com"
prefix: "$PREFIX"
MANIFEST
    pkg create -m "$META" -p "$META/plist" -r "$STAGE" -o "$TMP_ROOT/packages"
    mv "$TMP_ROOT/packages/$APP-$VERSION.pkg" "$PKG_OUT"
    ;;
  OpenBSD)
    pkg_create -B "$STAGE" -p "$PREFIX" -f "$META/plist" \
      -d "$META/description" -D COMMENT="Fast desktop color picker" \
      -D FULLPKGPATH="graphics/$APP" -D PORTSDIR="/usr/ports" \
      "$TMP_ROOT/packages/$APP-$VERSION.tgz"
    mv "$TMP_ROOT/packages/$APP-$VERSION.tgz" "$PKG_OUT"
    ;;
  NetBSD)
    printf '@name %s-%s\n' "$APP" "$VERSION" >> "$META/plist"
    printf 'OPSYS=NetBSD\nOS_VERSION=%s\nMACHINE_ARCH=%s\n' \
      "$(uname -r)" "$(uname -p)" > "$META/build-info"
    pkg_create -p "$STAGE$PREFIX" -I "$PREFIX" -f "$META/plist" \
      -B "$META/build-info" -u root -g wheel \
      -c '-Fast desktop color picker' -d "$META/description" \
      "$TMP_ROOT/packages/$APP-$VERSION.tgz"
    mv "$TMP_ROOT/packages/$APP-$VERSION.tgz" "$PKG_OUT"
    ;;
esac

echo "Release files written to $OUT_DIR"
