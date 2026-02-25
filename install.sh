#!/usr/bin/env bash
# install.sh – Install demodatagen from GitHub releases.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/user/demodatagen/main/install.sh | bash
#   ./install.sh [--version v0.1.0] [--prefix /usr/local]

set -euo pipefail

REPO="user/demodatagen"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
VERSION=""

# ── Parse arguments ───────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) VERSION="$2"; shift 2 ;;
        --prefix)  INSTALL_PREFIX="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: install.sh [--version VERSION] [--prefix PREFIX]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# ── Detect platform ──────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux*)   PLATFORM="unknown-linux-gnu" ;;
    Darwin*)  PLATFORM="apple-darwin" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="pc-windows-msvc" ;;
    *)        echo "Unsupported OS: ${OS}"; exit 1 ;;
esac

case "${ARCH}" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)            echo "Unsupported architecture: ${ARCH}"; exit 1 ;;
esac

TARGET="${ARCH}-${PLATFORM}"

# ── Resolve version ──────────────────────────────────────────────────
if [[ -z "${VERSION}" ]]; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')
    if [[ -z "${VERSION}" ]]; then
        echo "Error: Could not determine latest version."
        exit 1
    fi
fi

echo "Installing demodatagen ${VERSION} for ${TARGET}..."

# ── Download & install ───────────────────────────────────────────────
TMPDIR="$(mktemp -d)"
trap 'rm -rf "${TMPDIR}"' EXIT

if [[ "${PLATFORM}" == "pc-windows-msvc" ]]; then
    URL="https://github.com/${REPO}/releases/download/${VERSION}/demodatagen-${VERSION}-${TARGET}.zip"
    curl -fsSL "${URL}" -o "${TMPDIR}/demodatagen.zip"
    unzip -q "${TMPDIR}/demodatagen.zip" -d "${TMPDIR}"
else
    URL="https://github.com/${REPO}/releases/download/${VERSION}/demodatagen-${VERSION}-${TARGET}.tar.gz"
    curl -fsSL "${URL}" -o "${TMPDIR}/demodatagen.tar.gz"
    tar xzf "${TMPDIR}/demodatagen.tar.gz" -C "${TMPDIR}"
fi

install -d "${INSTALL_PREFIX}/bin"
install -m 755 "${TMPDIR}/demodatagen" "${INSTALL_PREFIX}/bin/demodatagen"

echo "demodatagen ${VERSION} installed to ${INSTALL_PREFIX}/bin/demodatagen"
echo "Run 'demodatagen --help' to get started."
