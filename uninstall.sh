#!/usr/bin/env bash
#
# uninstall.sh — remove the demodatagen binary and installer-added PATH entries.
#
# Usage:
#   ./uninstall.sh [--bin-dir DIR] [--prefix DIR] [--purge] [--yes] [--quiet]
set -euo pipefail

BIN_NAME="demodatagen"
BIN_DIR="${BIN_DIR:-}"
PREFIX="${INSTALL_PREFIX:-}"
PURGE=0
ASSUME_YES=0
QUIET=0

if [[ -t 2 && -z "${NO_COLOR:-}" ]]; then
    C_BOLD=$'\033[1m'; C_RED=$'\033[31m'; C_YELLOW=$'\033[33m'; C_RESET=$'\033[0m'
else
    C_BOLD=""; C_RED=""; C_YELLOW=""; C_RESET=""
fi
say()  { [[ "${QUIET}" -eq 1 ]] || printf '%s\n' "$*"; }
info() { [[ "${QUIET}" -eq 1 ]] || printf '%s==>%s %s\n' "${C_BOLD}" "${C_RESET}" "$*"; }
warn() { printf '%swarning:%s %s\n' "${C_YELLOW}" "${C_RESET}" "$*" >&2; }
err()  { printf '%serror:%s %s\n' "${C_RED}" "${C_RESET}" "$*" >&2; exit 1; }

usage() {
    cat <<EOF
${C_BOLD}uninstall.sh${C_RESET} — remove ${BIN_NAME}

Options:
  --bin-dir <DIR>   Directory the binary was installed into
  --prefix <DIR>    Look in <DIR>/bin (ignored if --bin-dir is set)
  --purge           Also remove config/cache/data directories
  --yes             Do not prompt for confirmation
  --quiet           Only print errors
  -h, --help        Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --bin-dir) BIN_DIR="${2:-}"; shift 2 ;;
        --prefix)  PREFIX="${2:-}"; shift 2 ;;
        --purge)   PURGE=1; shift ;;
        --yes|-y)  ASSUME_YES=1; shift ;;
        --quiet)   QUIET=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) err "unknown option: $1 (try --help)" ;;
    esac
done

have_cmd() { command -v "$1" >/dev/null 2>&1; }

# ── Locate installed binaries ────────────────────────────────────────
declare -a CANDIDATES=()
if [[ -n "${BIN_DIR}" ]]; then
    CANDIDATES+=("${BIN_DIR}/${BIN_NAME}")
elif [[ -n "${PREFIX}" ]]; then
    CANDIDATES+=("${PREFIX}/bin/${BIN_NAME}")
else
    CANDIDATES+=(
        "/usr/local/bin/${BIN_NAME}"
        "${HOME}/.local/bin/${BIN_NAME}"
        "${HOME}/.demodatagen/bin/${BIN_NAME}"
    )
    # Also catch whatever is first on PATH.
    if have_cmd "${BIN_NAME}"; then
        CANDIDATES+=("$(command -v "${BIN_NAME}")")
    fi
fi

# Deduplicate while preserving order.
declare -a FOUND=()
for path in "${CANDIDATES[@]}"; do
    [[ -e "${path}" ]] || continue
    skip=0
    for seen in "${FOUND[@]:-}"; do [[ "${seen}" == "${path}" ]] && skip=1; done
    [[ "${skip}" -eq 0 ]] && FOUND+=("${path}")
done

if [[ "${#FOUND[@]}" -eq 0 ]]; then
    warn "no ${BIN_NAME} installation found"
else
    info "Found ${#FOUND[@]} installation(s):"
    for path in "${FOUND[@]}"; do say "  ${path}"; done
    if [[ "${ASSUME_YES}" -ne 1 && -t 0 ]]; then
        printf 'Remove these? [y/N] ' >&2
        read -r reply
        [[ "${reply}" =~ ^[Yy]$ ]] || err "aborted"
    fi
    for path in "${FOUND[@]}"; do
        if rm -f "${path}" 2>/dev/null; then
            say "removed ${path}"
        elif have_cmd sudo; then
            sudo rm -f "${path}" && say "removed ${path} (sudo)"
        else
            warn "could not remove ${path} (insufficient permissions)"
        fi
    done
fi

# ── Remove installer-added PATH entries ──────────────────────────────
marker="# added by demodatagen install.sh"
for profile in "${HOME}/.bashrc" "${HOME}/.zshrc" "${HOME}/.profile"; do
    [[ -e "${profile}" ]] || continue
    if grep -qF "${marker}" "${profile}" 2>/dev/null; then
        # Delete the marker line and the export line that follows it.
        tmp="$(mktemp)"
        awk -v m="${marker}" '
            $0==m { skip=2; next }
            skip>0 { skip--; next }
            { print }
        ' "${profile}" > "${tmp}"
        cat "${tmp}" > "${profile}"
        rm -f "${tmp}"
        say "cleaned PATH entry from ${profile}"
    fi
done

# ── Purge config / cache / data ──────────────────────────────────────
if [[ "${PURGE}" -eq 1 ]]; then
    for dir in \
        "${XDG_CONFIG_HOME:-${HOME}/.config}/demodatagen" \
        "${XDG_CACHE_HOME:-${HOME}/.cache}/demodatagen" \
        "${XDG_DATA_HOME:-${HOME}/.local/share}/demodatagen" \
        "${HOME}/.demodatagen"; do
        if [[ -d "${dir}" ]]; then
            rm -rf "${dir}" && say "purged ${dir}"
        fi
    done
fi

info "${BIN_NAME} has been uninstalled."
