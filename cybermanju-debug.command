#!/usr/bin/env bash
# ── Cybermanju Drive — Debug Launcher (macOS) ─────────────────────
# Double-click this file in Finder to run the app with a visible
# Terminal window showing logs and crash diagnostics.
# ───────────────────────────────────────────────────────────────────
set -euo pipefail

APP_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_BIN="${APP_DIR}/Cybermanju Drive.app/Contents/MacOS/cybermanju-drive"
DATA_DIR="${HOME}/Library/Application Support/cybermanju-drive"
CRASH_LOG="${DATA_DIR}/crash.log"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Cybermanju Drive — Debug Console                        ║"
echo "║  Post-Quantum Encrypted OS                               ║"
echo "║                                                          ║"
echo "║  This Terminal shows app logs to help diagnose issues.   ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

echo "[DEBUG] App directory: ${APP_DIR}"
echo "[DEBUG] Data directory: ${DATA_DIR}"

# Check binary exists
if [ ! -x "${APP_BIN}" ]; then
    echo "[ERROR] App binary not found at: ${APP_BIN}"
    echo "        Try: ${APP_DIR}/cybermanju-drive (if not in .app bundle)"
    # Fallback: try the unpackaged binary
    if [ -x "${APP_DIR}/cybermanju-drive" ]; then
        APP_BIN="${APP_DIR}/cybermanju-drive"
        echo "[DEBUG] Using unpackaged binary: ${APP_BIN}"
    else
        exit 1
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

# macOS: WebKit is bundled with the OS, no extra deps needed.
# Run with --debug flag for verbose output.
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

echo ""
echo "Press Cmd+Q to close this Terminal."
