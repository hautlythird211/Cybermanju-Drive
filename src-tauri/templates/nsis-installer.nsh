; ── Cybermanju Drive — Post-Install Hook ──────────────────────────
; Runs after all files are extracted, before the finish page.
; Launches the app, creates debug shortcuts, and saves install marker.
; ───────────────────────────────────────────────────────────────────

; Launch the app (user can opt out via finish page checkbox)
ExecShell "" "$INSTDIR\Cybermanju Drive.exe"

; Create desktop shortcut for debug mode (shows console on Windows)
CreateShortCut "$DESKTOP\Cybermanju Drive (Debug).lnk" \
  "$INSTDIR\Cybermanju Drive.exe" "--debug" \
  "$INSTDIR\Cybermanju Drive.exe" 0

; Write install markers (read by the app on first launch)
WriteRegStr HKCU "Software\Cybermanju Drive" "InstallVersion" "0.1.0"
WriteRegStr HKCU "Software\Cybermanju Drive" "InstallPath" "$INSTDIR"
WriteRegStr HKCU "Software\Cybermanju Drive" "InstallDate" "$%DATE%"

; Launch log — saves installer context for debugging
!system 'echo "[Cybermanju Drive Installer] Version: 0.1.0, Path: $INSTDIR" >> "$TEMP\cybermanju-install.log"'
