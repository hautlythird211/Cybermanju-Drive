@echo off
chcp 65001 >nul
title Cybermanju Drive Debug
setlocal enabledelayedexpansion

set "APP_DIR=C:\Program Files\Cybermanju Drive"
set "EXE=%APP_DIR%\cybermanju-drive.exe"
set "DATA_DIR=%LOCALAPPDATA%\cybermanju-drive"
set "LOG=%TEMP%\cybermanju-debug-%RANDOM%.log"
set "CRASH_LOG=%DATA_DIR%\crash.log"
set "APP_LOG=%DATA_DIR%\cybermanju.log"

echo =============================================
echo Cybermanju Drive — Full Debug
echo =============================================
echo.
echo App:       %EXE%
echo Data:      %DATA_DIR%
echo Log:       %LOG%
echo.
echo Setting RUST_LOG=trace for maximum detail...
set RUST_LOG=trace

echo.
echo Starting app with --debug flag (console + verbose logs)...
echo.

:: Run the app and capture all output
"%EXE%" --debug > "%LOG%" 2>&1

echo.
echo =============================================
echo App exited with code %ERRORLEVEL%
echo =============================================
echo.
echo Checking crash.log...
if exist "%CRASH_LOG%" (
    echo -- crash.log found --
    type "%CRASH_LOG%"
    echo.
) else (
    echo [none]
)
echo.
echo Checking cybermanju.log...
if exist "%APP_LOG%" (
    echo -- cybermanju.log found --
    type "%APP_LOG%"
    echo.
) else (
    echo [none]
)
echo.
echo Full debug log saved to %LOG%
echo.
echo Opening log in Notepad...
start notepad "%LOG%"
if exist "%CRASH_LOG%" start notepad "%CRASH_LOG%"
if exist "%APP_LOG%" start notepad "%APP_LOG%"
echo.
echo Done. Press any key to close...
pause >nul
