; ── Cybermanju Drive — Custom NSIS Installer ──────────────────────
; Psychodelic high-performance vintage aesthetic
; Black background, terminal-style branding, auto-launch on finish
; ───────────────────────────────────────────────────────────────────

; Black background for MUI2 wizard pages
!define MUI_BGCOLOR "000000"

; Custom welcome page — terminal-style branding
!define MUI_WELCOMEPAGE_TITLE "Cybermanju Drive"
!define MUI_WELCOMEPAGE_TEXT "Post-Quantum Encrypted File System$\r$\n$\r$\n\
  > Quantum-resistant encryption (ML-KEM)$\r$\n\
  > Real-time file synchronization$\r$\n\
  > AI face recognition & smart search$\r$\n\
  > Triple-layer compression (gzip + zstd + bzip)$\r$\n\
  > Web Dashboard with JWT auth$\r$\n$\r$\n\
  Click Next to begin installation."

; Custom finish page with troubleshooting hints
!define MUI_FINISHPAGE_TITLE "Installation Complete"
!define MUI_FINISHPAGE_TEXT "Cybermanju Drive has been installed successfully.$\r$\n$\r$\n\
  If the application does not launch, check:$\r$\n\
  * Windows: ensure WebView2 is installed (go.microsoft.com/fwlink/p/?LinkId=2124703)$\r$\n\
  * Linux: ensure webkit2gtk-4.1 is installed (apt install libwebkit2gtk-4.1-0)$\r$\n$\r$\n\
  Crash logs are saved to: %APPDATA%\cybermanju-drive\crash.log"

; Enable "Launch" checkbox on the finish page (unchecked by default)
!define MUI_FINISHPAGE_RUN "$INSTDIR\Cybermanju Drive.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Cybermanju Drive"
!define MUI_FINISHPAGE_RUN_NOTCHECKED

; Custom branding in the installer footer
BrandingText "Cybermanju Drive — Post-Quantum Encrypted OS"

; No space requirements check
!define MUI_COMPONENTSPAGE_SMALLDESC
