#!/usr/bin/env bash
# uninstall.sh – Remove demodatagen from the system.
#
# Usage:
#   ./uninstall.sh [--prefix /usr/local]

set -euo pipefail

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix) INSTALL_PREFIX="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: uninstall.sh [--prefix PREFIX]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

BINARY="${INSTALL_PREFIX}/bin/demodatagen"

if [[ -f "${BINARY}" ]]; then
    rm -f "${BINARY}"
    echo "Removed ${BINARY}"
else
    echo "demodatagen not found at ${BINARY}"
    exit 1
fi

echo "demodatagen has been uninstalled."
