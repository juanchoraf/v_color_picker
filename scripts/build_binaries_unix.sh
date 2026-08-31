#!/usr/bin/env bash
set -euo pipefail

APP="v_color_picker"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPDATE_DEPS=1
CARGO_LOCKED=""

usage() {
  cat <<'USAGE'
Usage: scripts/build_binaries_unix.sh [--locked] [--no-update]

Builds Unix-like 64-bit V_Color_Picker artifacts for the current machine.

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

OUT_DIR="$ROOT_DIR/versions/${APP}_v${VERSION}"
STAGE="$OUT_DIR/unix_stage"
ARTIFACT_ARCH="unix_${ARCH}"
BIN_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.bin"
TAR_OUT="$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}.tar.gz"

mkdir -p "$OUT_DIR" "$STAGE"
if [ "$UPDATE_DEPS" -eq 1 ]; then
  cargo update
fi
cargo build --release $CARGO_LOCKED --manifest-path "$ROOT_DIR/Cargo.toml"

cp "$ROOT_DIR/target/release/$APP" "$BIN_OUT"
chmod 755 "$BIN_OUT"

rm -rf "$STAGE"
mkdir -p "$STAGE"
cp "$BIN_OUT" "$STAGE/$APP"
cp "$ROOT_DIR/assets/logo.svg" "$STAGE/logo.svg"
tar -czf "$TAR_OUT" -C "$STAGE" "$APP" "logo.svg"

cp "$ROOT_DIR/scripts/install_unix.sh" "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_install.sh"
cp "$ROOT_DIR/scripts/uninstall_unix.sh" "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_uninstall.sh"
chmod 755 "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_install.sh"
chmod 755 "$OUT_DIR/${APP}_v${VERSION}_${ARTIFACT_ARCH}_uninstall.sh"

echo "Release files written to $OUT_DIR"
