#!/usr/bin/env bash
#
# install.sh — install the demodatagen binary from GitHub releases.
#
# Quick install (latest):
#   curl -fsSL https://raw.githubusercontent.com/josunlp/demodatagen/main/install.sh | bash
#
# With options (run the downloaded script directly):
#   ./install.sh [--version vX.Y.Z] [--bin-dir DIR] [--prefix DIR]
#                [--repo owner/name] [--no-modify-path] [--force] [--quiet]
#
# Honors env vars: DEMODATAGEN_REPO, DEMODATAGEN_VERSION, INSTALL_PREFIX, BIN_DIR.
set -euo pipefail

# ── Configuration & defaults ─────────────────────────────────────────
REPO="${DEMODATAGEN_REPO:-josunlp/demodatagen}"
VERSION="${DEMODATAGEN_VERSION:-}"
BIN_DIR="${BIN_DIR:-}"
PREFIX="${INSTALL_PREFIX:-}"
BIN_NAME="demodatagen"
MODIFY_PATH=1
FORCE=0
QUIET=0

# ── Pretty output ────────────────────────────────────────────────────
if [[ -t 2 && -z "${NO_COLOR:-}" ]]; then
    C_BOLD=$'\033[1m'; C_RED=$'\033[31m'; C_GREEN=$'\033[32m'
    C_YELLOW=$'\033[33m'; C_DIM=$'\033[2m'; C_RESET=$'\033[0m'
else
    C_BOLD=""; C_RED=""; C_GREEN=""; C_YELLOW=""; C_DIM=""; C_RESET=""
fi
say()  { [[ "${QUIET}" -eq 1 ]] || printf '%s\n' "$*"; }
info() { [[ "${QUIET}" -eq 1 ]] || printf '%s==>%s %s\n' "${C_BOLD}" "${C_RESET}" "$*"; }
warn() { printf '%swarning:%s %s\n' "${C_YELLOW}" "${C_RESET}" "$*" >&2; }
err()  { printf '%serror:%s %s\n' "${C_RED}" "${C_RESET}" "$*" >&2; exit 1; }

usage() {
    cat <<EOF
${C_BOLD}install.sh${C_RESET} — install ${BIN_NAME} from GitHub releases

Options:
  --version <vX.Y.Z>   Install a specific version (default: latest release)
  --bin-dir <DIR>      Directory to install the binary into
  --prefix <DIR>       Install to <DIR>/bin (ignored if --bin-dir is set)
  --repo <owner/name>  Source repository (default: ${REPO})
  --no-modify-path     Do not touch shell profiles for PATH
  --force              Overwrite an existing installation without prompting
  --quiet              Only print errors
  -h, --help           Show this help

Default bin dir: /usr/local/bin when writable/root, otherwise \$HOME/.local/bin
EOF
}

# ── Parse arguments ──────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) VERSION="${2:-}"; shift 2 ;;
        --bin-dir) BIN_DIR="${2:-}"; shift 2 ;;
        --prefix)  PREFIX="${2:-}"; shift 2 ;;
        --repo)    REPO="${2:-}"; shift 2 ;;
        --no-modify-path) MODIFY_PATH=0; shift ;;
        --force)   FORCE=1; shift ;;
        --quiet)   QUIET=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) err "unknown option: $1 (try --help)" ;;
    esac
done

# ── Helpers ──────────────────────────────────────────────────────────
have_cmd() { command -v "$1" >/dev/null 2>&1; }
need_cmd() { have_cmd "$1" || err "required command not found: $1"; }

# Pick an HTTP client.
if have_cmd curl; then
    DL() { curl -fsSL --retry 3 "$1" -o "$2"; }
    DL_STDOUT() { curl -fsSL --retry 3 "$1"; }
elif have_cmd wget; then
    DL() { wget -qO "$2" "$1"; }
    DL_STDOUT() { wget -qO- "$1"; }
else
    err "need either 'curl' or 'wget' installed"
fi

# Compute a SHA-256 hex digest of a file.
sha256_of() {
    if have_cmd sha256sum; then sha256sum "$1" | awk '{print $1}'
    elif have_cmd shasum; then shasum -a 256 "$1" | awk '{print $1}'
    else echo ""; fi
}

# ── Detect platform ──────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"
EXT="tar.gz"

case "${ARCH}" in
    x86_64|amd64)   ARCH="x86_64" ;;
    aarch64|arm64)  ARCH="aarch64" ;;
    *)              err "unsupported architecture: ${ARCH}" ;;
esac

case "${OS}" in
    Linux*)
        # Distinguish musl from glibc so static builds install on Alpine etc.
        if have_cmd ldd && ldd --version 2>&1 | grep -qi musl; then
            VENDOR="unknown-linux-musl"
        else
            VENDOR="unknown-linux-gnu"
        fi
        ;;
    Darwin*) VENDOR="apple-darwin" ;;
    MINGW*|MSYS*|CYGWIN*) VENDOR="pc-windows-msvc"; EXT="zip"; BIN_NAME="demodatagen.exe" ;;
    *) err "unsupported OS: ${OS}" ;;
esac
TARGET="${ARCH}-${VENDOR}"

# ── Resolve version ──────────────────────────────────────────────────
if [[ -z "${VERSION}" ]]; then
    info "Querying latest release of ${REPO}…"
    VERSION="$(DL_STDOUT "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
    [[ -n "${VERSION}" ]] || err "could not determine latest version (check ${REPO} has releases)"
fi
# Normalize to a leading 'v' (release tags are vX.Y.Z).
[[ "${VERSION}" == v* ]] || VERSION="v${VERSION}"

# ── Resolve install directory ────────────────────────────────────────
if [[ -z "${BIN_DIR}" ]]; then
    if [[ -n "${PREFIX}" ]]; then
        BIN_DIR="${PREFIX}/bin"
    elif [[ "$(id -u)" -eq 0 ]]; then
        BIN_DIR="/usr/local/bin"
    elif [[ -w /usr/local/bin ]]; then
        BIN_DIR="/usr/local/bin"
    else
        BIN_DIR="${HOME}/.local/bin"
    fi
fi

# Decide whether we need sudo to write into BIN_DIR.
SUDO=""
ensure_bin_dir() {
    if [[ -d "${BIN_DIR}" && -w "${BIN_DIR}" ]]; then
        return
    fi
    if mkdir -p "${BIN_DIR}" 2>/dev/null && [[ -w "${BIN_DIR}" ]]; then
        return
    fi
    if [[ "$(id -u)" -ne 0 ]] && have_cmd sudo; then
        warn "elevated permissions needed to write to ${BIN_DIR}; using sudo"
        SUDO="sudo"
        ${SUDO} mkdir -p "${BIN_DIR}"
    else
        err "cannot write to ${BIN_DIR} (try --bin-dir \$HOME/.local/bin)"
    fi
}

DEST="${BIN_DIR}/${BIN_NAME}"
if [[ -e "${DEST}" && "${FORCE}" -ne 1 ]]; then
    if [[ -t 0 ]]; then
        printf '%s already exists. Overwrite? [y/N] ' "${DEST}" >&2
        read -r reply
        [[ "${reply}" =~ ^[Yy]$ ]] || err "aborted"
    else
        warn "${DEST} exists; overwriting (non-interactive). Use --force to silence."
    fi
fi

# ── Download, verify, install ────────────────────────────────────────
info "Installing ${BIN_NAME} ${VERSION} (${TARGET})"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "${TMPDIR}"' EXIT

ARCHIVE="demodatagen-${VERSION}-${TARGET}.${EXT}"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

say "${C_DIM}  downloading ${ARCHIVE}${C_RESET}"
DL "${BASE_URL}/${ARCHIVE}" "${TMPDIR}/${ARCHIVE}" \
    || err "download failed: ${BASE_URL}/${ARCHIVE}"

# Verify checksum against the release's SHA256SUMS (if present).
if DL "${BASE_URL}/SHA256SUMS" "${TMPDIR}/SHA256SUMS" 2>/dev/null; then
    expected="$(grep -E "  ${ARCHIVE}\$|\\*${ARCHIVE}\$| ${ARCHIVE}\$" "${TMPDIR}/SHA256SUMS" \
        | head -1 | awk '{print $1}')"
    actual="$(sha256_of "${TMPDIR}/${ARCHIVE}")"
    if [[ -z "${actual}" ]]; then
        warn "no sha256 tool available; skipping checksum verification"
    elif [[ -z "${expected}" ]]; then
        warn "checksum for ${ARCHIVE} not found in SHA256SUMS; skipping verification"
    elif [[ "${expected}" != "${actual}" ]]; then
        err "checksum mismatch for ${ARCHIVE}\n  expected ${expected}\n  actual   ${actual}"
    else
        say "${C_DIM}  checksum OK${C_RESET}"
    fi
else
    warn "SHA256SUMS not published for ${VERSION}; skipping checksum verification"
fi

# Extract.
if [[ "${EXT}" == "zip" ]]; then
    need_cmd unzip
    unzip -q "${TMPDIR}/${ARCHIVE}" -d "${TMPDIR}/extract"
else
    need_cmd tar
    mkdir -p "${TMPDIR}/extract"
    tar xzf "${TMPDIR}/${ARCHIVE}" -C "${TMPDIR}/extract"
fi

SRC="$(find "${TMPDIR}/extract" -type f -name "${BIN_NAME}" | head -1)"
[[ -n "${SRC}" ]] || err "binary ${BIN_NAME} not found inside ${ARCHIVE}"
chmod +x "${SRC}"

ensure_bin_dir
${SUDO} install -m 0755 "${SRC}" "${DEST}" 2>/dev/null \
    || { ${SUDO} cp "${SRC}" "${DEST}" && ${SUDO} chmod 0755 "${DEST}"; }

# ── Verify the installed binary runs ─────────────────────────────────
if "${DEST}" --version >/dev/null 2>&1; then
    installed_ver="$("${DEST}" --version 2>/dev/null | head -1)"
    info "${C_GREEN}Installed${C_RESET} ${installed_ver} → ${DEST}"
else
    warn "installed binary at ${DEST} did not run cleanly"
fi

# ── PATH setup ───────────────────────────────────────────────────────
case ":${PATH}:" in
    *":${BIN_DIR}:"*) on_path=1 ;;
    *) on_path=0 ;;
esac

if [[ "${on_path}" -eq 0 ]]; then
    if [[ "${MODIFY_PATH}" -eq 1 ]]; then
        line="export PATH=\"${BIN_DIR}:\$PATH\""
        marker="# added by demodatagen install.sh"
        added=0
        for profile in "${HOME}/.bashrc" "${HOME}/.zshrc" "${HOME}/.profile"; do
            [[ -e "${profile}" ]] || continue
            if ! grep -qF "${marker}" "${profile}" 2>/dev/null; then
                printf '\n%s\n%s\n' "${marker}" "${line}" >> "${profile}"
                say "${C_DIM}  added ${BIN_DIR} to PATH in ${profile}${C_RESET}"
                added=1
            fi
        done
        if [[ "${added}" -eq 1 ]]; then
            info "Restart your shell or run: ${C_BOLD}export PATH=\"${BIN_DIR}:\$PATH\"${C_RESET}"
        fi
    else
        warn "${BIN_DIR} is not on your PATH. Add it with:"
        say  "  export PATH=\"${BIN_DIR}:\$PATH\""
    fi
fi

info "Done. Try: ${C_BOLD}${BIN_NAME} list${C_RESET}"
