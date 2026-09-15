#!/usr/bin/env bash
set -euo pipefail

if [ "${EUID:-$(id -u)}" -ne 0 ]; then
  echo "Run as root: sudo $0" >&2
  exit 1
fi

if ! command -v apt-get >/dev/null 2>&1; then
  echo "This helper currently supports Debian/Ubuntu apt systems." >&2
  exit 1
fi

apt-get update
apt-get install -y \
  build-essential \
  ca-certificates \
  curl \
  desktop-file-utils \
  dpkg \
  dpkg-dev \
  hicolor-icon-theme \
  libbsd0 \
  libegl1-mesa-dev \
  libgl1-mesa-dev \
  libmd0 \
  libpipewire-0.3-dev \
  libxau6 \
  libxcb1 \
  libxcb1-dev \
  libxdmcp6 \
  libxkbcommon-dev \
  libwayland-dev \
  pkg-config \
  shared-mime-info
