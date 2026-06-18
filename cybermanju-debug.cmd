@echo off
title Cybermanju Drive — Debug Console
echo ╔═══════════════════════════════════════════════════════════╗
echo ║  Cybermanju Drive — Debug Console                        ║
║  Post-Quantum Encrypted OS                                 ║
║                                                             ║
║  This window shows app logs to help diagnose issues.        ║
╚═══════════════════════════════════════════════════════════════╝
echo.

set APP_DIR=%~dp0
echo [DEBUG] App directory: %APP_DIR%
echo [DEBUG] Starting Cybermanju Drive...
echo.

if exist "%APPDATA%\cybermanju-drive\crash.log" (
    echo [DEBUG] Previous crash log found:
    echo ----------------------------------------
    type "%APPDATA%\cybermanju-drive\crash.log"
    echo ----------------------------------------
    echo.
    del "%APPDATA%\cybermanju-drive\crash.log" 2>nul
)

echo [DEBUG] Launching app...
echo.

start /wait "" "%APP_DIR%\Cybermanju Drive.exe" --debug

echo.
echo [DEBUG] App exited with code: %ERRORLEVEL%
echo.

if exist "%APPDATA%\cybermanju-drive\crash.log" (
    echo [DEBUG] Crash log was generated:
    echo ----------------------------------------
    type "%APPDATA%\cybermanju-drive\crash.log"
    echo ----------------------------------------
) else (
    echo [DEBUG] No crash log found. If nothing appeared, check:
    echo   - WebView2 is installed (https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
    echo   - Windows is up to date
    echo   - No antivirus is blocking the app
)

echo.
echo Press any key to close this window.
pause >nul
