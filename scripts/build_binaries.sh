#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OS_NAME="$(uname -s)"

case "$OS_NAME" in
  Linux*)
    exec "$ROOT_DIR/scripts/build_binaries_linux.sh" "$@"
    ;;
  Darwin*)
    exec "$ROOT_DIR/scripts/build_binaries_macos.sh" "$@"
    ;;
  FreeBSD*|OpenBSD*|NetBSD*|DragonFly*)
    exec "$ROOT_DIR/scripts/build_binaries_unix.sh" "$@"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$ROOT_DIR/scripts/build_binaries_windows.ps1" "$@"
    ;;
  *)
    echo "Unsupported build OS: $OS_NAME" >&2
    exit 1
    ;;
esac
