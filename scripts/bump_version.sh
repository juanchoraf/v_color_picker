#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/bump_version.sh NEW_VERSION [--no-build] [build_binaries options]

Updates Cargo.toml to NEW_VERSION, refreshes Cargo.lock, then runs
scripts/build_binaries.sh unless --no-build is passed.

Examples:
  scripts/bump_version.sh 0.1.1
  scripts/bump_version.sh 0.1.1 --no-build
  scripts/bump_version.sh 0.1.1 --locked --no-update
USAGE
}

if [ "$#" -lt 1 ]; then
  usage >&2
  exit 1
fi

if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
  usage
  exit 0
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NEW_VERSION="$1"
shift
RUN_BUILD=1
BUILD_ARGS=()

while [ "$#" -gt 0 ]; do
  case "$1" in
    --no-build)
      RUN_BUILD=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      BUILD_ARGS+=("$1")
      shift
      ;;
  esac
done

if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$ ]]; then
  echo "Version must be semantic, for example 1.2.3" >&2
  exit 1
fi

OLD_VERSION="$(awk -F'"' '/^version = / { print $2; exit }' "$ROOT_DIR/Cargo.toml")"
perl -0pi -e "s/^version = \"[^\"]+\"/version = \"$NEW_VERSION\"/m" "$ROOT_DIR/Cargo.toml"

if command -v cargo >/dev/null 2>&1; then
  cargo generate-lockfile --manifest-path "$ROOT_DIR/Cargo.toml"
else
  echo "cargo not found; Cargo.lock was not refreshed" >&2
fi

echo "Version bumped: $OLD_VERSION -> $NEW_VERSION"
echo "Push tag v$NEW_VERSION to start the GitHub release workflow."

if [ "$RUN_BUILD" -eq 1 ]; then
  "$ROOT_DIR/scripts/build_binaries.sh" "${BUILD_ARGS[@]}"
fi
