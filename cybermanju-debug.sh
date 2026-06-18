#!/usr/bin/env bash
# ── Cybermanju Drive — Debug Launcher (Linux) ────────────────────
# Runs the app with verbose stderr, checks for crash logs,
# and diagnoses common startup issues.
# ───────────────────────────────────────────────────────────────────
set -euo pipefail

APP_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_BIN="${APP_DIR}/cybermanju-drive"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/cybermanju-drive"
CRASH_LOG="${DATA_DIR}/crash.log"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Cybermanju Drive — Debug Console                        ║"
echo "║  Post-Quantum Encrypted OS                               ║"
echo "║                                                          ║"
echo "║  This terminal shows app logs to help diagnose issues.   ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

echo "[DEBUG] App directory: ${APP_DIR}"
echo "[DEBUG] Data directory: ${DATA_DIR}"
echo "[DEBUG] Binary: ${APP_BIN}"

# Check if binary exists
if [ ! -x "${APP_BIN}" ]; then
    echo "[ERROR] App binary not found at: ${APP_BIN}"
    echo "        Did you extract all files?"
    exit 1
fi

# Check for WebKit2GTK
if command -v dpkg &>/dev/null; then
    if ! dpkg -l libwebkit2gtk-4.1-0 &>/dev/null 2>&1; then
        echo "[WARN] libwebkit2gtk-4.1-0 not found. Install it:"
        echo "       sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0"
    fi
elif command -v rpm &>/dev/null; then
    if ! rpm -q webkit2gtk4.1 &>/dev/null 2>&1; then
        echo "[WARN] webkit2gtk4.1 not found. Install it:"
        echo "       sudo dnf install webkit2gtk4.1 gtk3"
    fi
fi

# Check for previous crash log
if [ -f "${CRASH_LOG}" ]; then
    echo ""
    echo "[DEBUG] Previous crash log found:"
    echo "----------------------------------------"
    cat "${CRASH_LOG}"
    echo "----------------------------------------"
    echo ""
    rm -f "${CRASH_LOG}"
fi

echo ""
echo "[DEBUG] Launching app..."
echo "════════════════════════════════════════════════════════════"
echo ""

# Run with debug flag — stderr is displayed in this terminal
"${APP_BIN}" --debug 2>&1

EXIT_CODE=$?

echo ""
echo "════════════════════════════════════════════════════════════"
echo "[DEBUG] App exited with code: ${EXIT_CODE}"

if [ -f "${CRASH_LOG}" ]; then
    echo ""
    echo "[DEBUG] Crash log was generated:"
    echo "----------------------------------------"
    cat "${CRASH_LOG}"
    echo "----------------------------------------"
    echo ""
    echo "Please report the crash at:"
    echo "  https://github.com/cybermanju/cybermanju-drive/issues"
else
    echo "[DEBUG] No crash log found."
    if [ "${EXIT_CODE}" -ne 0 ]; then
        echo "        Check the terminal output above for errors."
    fi
fi
