/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_OAUTH_GOOGLE_DRIVE_CLIENT_ID?: string
  readonly VITE_OAUTH_GOOGLE_PHOTOS_CLIENT_ID?: string
  readonly VITE_OAUTH_GITHUB_CLIENT_ID?: string
  readonly VITE_OAUTH_GITLAB_CLIENT_ID?: string
  readonly VITE_OAUTH_TELEGRAM_CLIENT_ID?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
