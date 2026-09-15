#!/usr/bin/env bash
set -euo pipefail
umask 022

APP="v_color_picker"
PUBLISHER="TheVelasquez.com"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPDATE_DEPS=1
CARGO_LOCKED=""

usage() {
  cat <<'USAGE'
Usage: scripts/build_binaries_macos.sh [--locked] [--no-update]

Builds macOS 64-bit v_color_picker artifacts for the current Mac.

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

command -v pkgbuild >/dev/null 2>&1 || {
  echo "pkgbuild is required to build the macOS installer." >&2
  exit 1
}

cd "$ROOT_DIR"
VERSION="$(awk -F'"' '/^version = / { print $2; exit }' "$ROOT_DIR/Cargo.toml")"
RAW_ARCH="$(uname -m)"

case "$RAW_ARCH" in
  x86_64)
    ARCH="x86_64"
    ;;
  arm64|aarch64)
    ARCH="arm64"
    ;;
  *)
    echo "Only 64-bit builds are supported. Found: $RAW_ARCH" >&2
    exit 1
    ;;
esac

OUT_DIR="$ROOT_DIR/versions/${APP}_v${VERSION}"
ARTIFACT_ARCH="macos_${ARCH}"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/${APP}_macos_package.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT
APP_BUNDLE="$TMP_ROOT/${APP}.app"
PKG_ROOT="$TMP_ROOT/pkg_root"
PKG_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.pkg"

mkdir -p "$OUT_DIR"
if [ "$UPDATE_DEPS" -eq 1 ]; then
  cargo update
fi
cargo build --release $CARGO_LOCKED --manifest-path "$ROOT_DIR/Cargo.toml"

rm -rf "$APP_BUNDLE" "$PKG_ROOT"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"
install -m 755 "$ROOT_DIR/target/release/$APP" "$APP_BUNDLE/Contents/MacOS/$APP"
install -m 644 "$ROOT_DIR/assets/logo.svg" "$APP_BUNDLE/Contents/Resources/logo.svg"

cat > "$APP_BUNDLE/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDisplayName</key><string>v_color_picker</string>
  <key>CFBundleExecutable</key><string>$APP</string>
  <key>CFBundleIdentifier</key><string>com.thevelasquez.$APP</string>
  <key>CFBundleName</key><string>v_color_picker</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>10.15</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSScreenCaptureDescription</key><string>v_color_picker captures screens to let you pick exact colors.</string>
</dict>
</plist>
PLIST

mkdir -p "$PKG_ROOT/Applications"
cp -R "$APP_BUNDLE" "$PKG_ROOT/Applications/"

pkgbuild \
  --root "$PKG_ROOT" \
  --identifier "com.thevelasquez.$APP" \
  --version "$VERSION" \
  --install-location "/" \
  --ownership recommended \
  "$PKG_OUT"

echo "Release files written to $OUT_DIR"
