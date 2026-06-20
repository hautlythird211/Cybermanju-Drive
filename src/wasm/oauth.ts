export type OAuthProvider = 'googleDrive' | 'googlePhotos' | 'github' | 'gitlab' | 'telegram'

export interface OAuthToken {
  accessToken: string
  refreshToken: string | null
  expiresAt: number | null
  tokenType: string
  scope: string | null
  provider: OAuthProvider
}

export interface OAuthConfig {
  clientId: string
  clientSecret: string
  redirectUri: string
  scopes: string[]
  authUrl: string
  tokenUrl: string
}

function computeRedirectUri(): string {
  const { origin, pathname } = window.location
  const parts = pathname.split('/')
  if (parts.length > 1 && parts[parts.length - 1].includes('.')) {
    parts.pop()
  }
  let dir = parts.join('/').replace(/\/?$/, '/').replace(/\/+/g, '/')
  return `${origin}${dir}oauth/callback`
}

const PROVIDER_CONFIGS: Record<OAuthProvider, OAuthConfig> = {
  googleDrive: {
    clientId: '',
    clientSecret: '',
    redirectUri: computeRedirectUri(),
    scopes: ['https://www.googleapis.com/auth/drive.file'],
    authUrl: 'https://accounts.google.com/o/oauth2/v2/auth',
    tokenUrl: 'https://oauth2.googleapis.com/token',
  },
  googlePhotos: {
    clientId: '',
    clientSecret: '',
    redirectUri: computeRedirectUri(),
    scopes: ['https://www.googleapis.com/auth/photoslibrary.appendonly'],
    authUrl: 'https://accounts.google.com/o/oauth2/v2/auth',
    tokenUrl: 'https://oauth2.googleapis.com/token',
  },
  github: {
    clientId: '',
    clientSecret: '',
    redirectUri: computeRedirectUri(),
    scopes: ['repo'],
    authUrl: 'https://github.com/login/oauth/authorize',
    tokenUrl: 'https://github.com/login/oauth/access_token',
  },
  gitlab: {
    clientId: '',
    clientSecret: '',
    redirectUri: computeRedirectUri(),
    scopes: ['api'],
    authUrl: 'https://gitlab.com/oauth/authorize',
    tokenUrl: 'https://gitlab.com/oauth/token',
  },
  telegram: {
    clientId: '',
    clientSecret: '',
    redirectUri: computeRedirectUri(),
    scopes: ['bot'],
    authUrl: 'https://oauth.telegram.org/auth',
    tokenUrl: 'https://oauth.telegram.org/token',
  },
}

// Primary env var names (with VITE_ prefix, as Vite requires for client exposure)
// Plus fallbacks: with OAUTH prefix, without VITE_ prefix, bare names
const ENV_MAP: Record<string, OAuthProvider>[] = [
  // 1. Standard VITE_ prefix with OAUTH segment
  {
    VITE_OAUTH_GOOGLE_DRIVE_CLIENT_ID: 'googleDrive',
    VITE_OAUTH_GOOGLE_PHOTOS_CLIENT_ID: 'googlePhotos',
    VITE_OAUTH_GITHUB_CLIENT_ID: 'github',
    VITE_OAUTH_GITLAB_CLIENT_ID: 'gitlab',
    VITE_OAUTH_TELEGRAM_CLIENT_ID: 'telegram',
  },
  // 2. VITE_ prefix without OAUTH segment
  {
    VITE_GOOGLE_DRIVE_CLIENT_ID: 'googleDrive',
    VITE_GOOGLE_PHOTOS_CLIENT_ID: 'googlePhotos',
    VITE_GITHUB_CLIENT_ID: 'github',
    VITE_GITLAB_CLIENT_ID: 'gitlab',
    VITE_TELEGRAM_CLIENT_ID: 'telegram',
  },
  // 3. Bare names (no VITE_ prefix — won't be exposed by Vite but may be set at runtime)
  {
    GOOGLE_DRIVE_CLIENT_ID: 'googleDrive',
    GOOGLE_PHOTOS_CLIENT_ID: 'googlePhotos',
    GITHUB_CLIENT_ID: 'github',
    GITLAB_CLIENT_ID: 'gitlab',
    TELEGRAM_CLIENT_ID: 'telegram',
  },
  // 4. Single Google client ID for both Drive & Photos
  {
    VITE_OAUTH_GOOGLE_CLIENT_ID: 'googleDrive',
    VITE_GOOGLE_CLIENT_ID: 'googleDrive',
    GOOGLE_CLIENT_ID: 'googleDrive',
  },
]

// Client secret env vars — Google requires client_secret for token exchange
const SECRET_ENV_MAP: Record<string, OAuthProvider>[] = [
  {
    VITE_OAUTH_GOOGLE_DRIVE_CLIENT_SECRET: 'googleDrive',
    VITE_OAUTH_GOOGLE_PHOTOS_CLIENT_SECRET: 'googlePhotos',
    VITE_OAUTH_GITHUB_CLIENT_SECRET: 'github',
    VITE_OAUTH_GITLAB_CLIENT_SECRET: 'gitlab',
    VITE_OAUTH_TELEGRAM_CLIENT_SECRET: 'telegram',
  },
  {
    VITE_GOOGLE_DRIVE_CLIENT_SECRET: 'googleDrive',
    VITE_GOOGLE_PHOTOS_CLIENT_SECRET: 'googlePhotos',
    VITE_GITHUB_CLIENT_SECRET: 'github',
    VITE_GITLAB_CLIENT_SECRET: 'gitlab',
    VITE_TELEGRAM_CLIENT_SECRET: 'telegram',
  },
  {
    GOOGLE_DRIVE_CLIENT_SECRET: 'googleDrive',
    GOOGLE_PHOTOS_CLIENT_SECRET: 'googlePhotos',
    GITHUB_CLIENT_SECRET: 'github',
    GITLAB_CLIENT_SECRET: 'gitlab',
    TELEGRAM_CLIENT_SECRET: 'telegram',
  },
  {
    VITE_OAUTH_GOOGLE_CLIENT_SECRET: 'googleDrive',
    VITE_GOOGLE_CLIENT_SECRET: 'googleDrive',
    GOOGLE_CLIENT_SECRET: 'googleDrive',
  },
]

export function loadClientIdsFromEnv(): void {
  if (typeof import.meta === 'undefined' || !import.meta.env) {
    console.warn('[OAuth][DEBUG] import.meta.env is undefined — env vars unavailable')
    return
  }
  const env = import.meta.env as Record<string, string | undefined>
  const foundIds: string[] = []
  const foundSecrets: string[] = []
  const allKeys = Object.keys(env).filter(k => k.startsWith('VITE_') || k.includes('CLIENT_') || k.includes('OAUTH'))

  console.log('[OAuth][DEBUG] ── ENV VAR SCAN ──')
  console.log('[OAuth][DEBUG] Total env keys:', Object.keys(env).length)
  console.log('[OAuth][DEBUG] Relevant keys:', allKeys.length ? allKeys.join(', ') : '(none found)')

  // Load client IDs
  console.log('[OAuth][DEBUG] Checking provider client IDs:')
  for (const map of ENV_MAP) {
    for (const [key, provider] of Object.entries(map)) {
      const val = env[key]
      const status = val ? `SET (${val.slice(0, 6)}...${val.slice(-3)})` : 'NOT SET'
      console.log(`[OAuth][DEBUG]   ${key} → ${provider}: ${status}`)
      if (val && !PROVIDER_CONFIGS[provider].clientId) {
        PROVIDER_CONFIGS[provider].clientId = val
        foundIds.push(`${key}=${val.slice(0, 8)}...`)
      }
    }
  }

  // Load client secrets
  console.log('[OAuth][DEBUG] Checking provider client secrets:')
  for (const map of SECRET_ENV_MAP) {
    for (const [key, provider] of Object.entries(map)) {
      const val = env[key]
      const status = val ? `SET (${val.slice(0, 4)}...${val.slice(-3)})` : 'NOT SET'
      console.log(`[OAuth][DEBUG]   ${key} → ${provider}: ${status}`)
      if (val && !PROVIDER_CONFIGS[provider].clientSecret) {
        PROVIDER_CONFIGS[provider].clientSecret = val
        foundSecrets.push(`${key}=${val.slice(0, 6)}...`)
      }
    }
  }

  // Also populate googlePhotos from googleDrive if only one Google ID was set
  if (PROVIDER_CONFIGS.googleDrive.clientId && !PROVIDER_CONFIGS.googlePhotos.clientId) {
    PROVIDER_CONFIGS.googlePhotos.clientId = PROVIDER_CONFIGS.googleDrive.clientId
    console.log('[OAuth][DEBUG] googlePhotos inheriting clientId from googleDrive')
  }
  if (PROVIDER_CONFIGS.googleDrive.clientSecret && !PROVIDER_CONFIGS.googlePhotos.clientSecret) {
    PROVIDER_CONFIGS.googlePhotos.clientSecret = PROVIDER_CONFIGS.googleDrive.clientSecret
    console.log('[OAuth][DEBUG] googlePhotos inheriting clientSecret from googleDrive')
  }

  console.log('[OAuth][DEBUG] ── FINAL STATUS ──')
  for (const [provider, config] of Object.entries(PROVIDER_CONFIGS)) {
    const id = config.clientId ? `ID:${config.clientId.slice(0, 6)}...` : 'ID:—'
    const secret = config.clientSecret ? `SECRET:${config.clientSecret.slice(0, 4)}...` : 'SECRET:—'
    console.log(`[OAuth][DEBUG]   ${provider}: ${id} ${secret}`)
  }

  if (foundIds.length > 0) {
    console.log('[OAuth] Client IDs loaded:', foundIds.join(', '))
  } else {
    console.warn('[OAuth] ⚠ NO CLIENT IDs FOUND IN ENVIRONMENT')
    console.warn('[OAuth] Tried env keys:', Object.values(ENV_MAP).flatMap(m => Object.keys(m)).join(', '))
    console.warn('[OAuth] Solution: Create .env file from .env.example and fill in your OAuth client IDs')
    console.warn('[OAuth] Or set VITE_OAUTH_* env vars before running `npm run dev`')
  }
  if (foundSecrets.length > 0) {
    console.log('[OAuth] Client secrets loaded:', foundSecrets.join(', '))
  } else {
    console.warn('[OAuth] ⚠ NO CLIENT SECRETS FOUND — Google token exchange will fail')
    console.warn('[OAuth] Tried env keys:', Object.values(SECRET_ENV_MAP).flatMap(m => Object.keys(m)).join(', '))
    console.warn('[OAuth] Solution: Set VITE_OAUTH_GOOGLE_DRIVE_CLIENT_SECRET in .env')
  }
}

export function getProviderClientId(provider: OAuthProvider): string {
  return PROVIDER_CONFIGS[provider].clientId
}

export function setProviderClientId(provider: OAuthProvider, clientId: string): void {
  PROVIDER_CONFIGS[provider].clientId = clientId
}

export function getProviderClientSecret(provider: OAuthProvider): string {
  return PROVIDER_CONFIGS[provider].clientSecret
}

export function setProviderClientSecret(provider: OAuthProvider, clientSecret: string): void {
  PROVIDER_CONFIGS[provider].clientSecret = clientSecret
}

export function getProviderConfig(provider: OAuthProvider): OAuthConfig {
  return PROVIDER_CONFIGS[provider]
}

// Auto-load client IDs from environment variables
loadClientIdsFromEnv()

function base64UrlEncode(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer)
  let binary = ''
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '')
}

async function generateCodeVerifier(): Promise<string> {
  const array = new Uint8Array(32)
  crypto.getRandomValues(array)
  return base64UrlEncode(array.buffer)
}

async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(verifier)
  const digest = await crypto.subtle.digest('SHA-256', data)
  return base64UrlEncode(digest)
}

function generateState(): string {
  const array = new Uint8Array(16)
  crypto.getRandomValues(array)
  return base64UrlEncode(array.buffer)
}

export function buildAuthorizationUrl(
  provider: OAuthProvider,
  config: { clientId?: string; redirectUri?: string; scopes?: string[] } = {}
): { url: string; verifier: string; state: string } {
  const providerConfig = PROVIDER_CONFIGS[provider]
  const clientId = config.clientId || providerConfig.clientId
  const redirectUri = config.redirectUri || providerConfig.redirectUri
  const scopes = config.scopes || providerConfig.scopes

  if (!clientId) {
    throw new Error(`Client ID not configured for ${provider}. Call setProviderClientId() first.`)
  }

  const verifierPromise = generateCodeVerifier()
  const challengePromise = verifierPromise.then(generateCodeChallenge)
  const state = generateState()

  const params = new URLSearchParams({
    client_id: clientId,
    redirect_uri: redirectUri,
    response_type: 'code',
    scope: scopes.join(' '),
    state,
    access_type: 'offline',
    prompt: 'consent',
  })

  return {
    url: `${providerConfig.authUrl}?${params.toString()}&code_challenge_method=S256&code_challenge=`,
    verifier: '',
    state,
  }
}

export async function buildAuthorizationUrlAsync(
  provider: OAuthProvider,
  config: { clientId?: string; redirectUri?: string; scopes?: string[] } = {}
): Promise<{ url: string; verifier: string; state: string }> {
  const providerConfig = PROVIDER_CONFIGS[provider]
  const clientId = config.clientId || providerConfig.clientId
  const redirectUri = config.redirectUri || providerConfig.redirectUri
  const scopes = config.scopes || providerConfig.scopes

  console.log(`[OAuth][DEBUG] buildAuthorizationUrlAsync(${provider})`)
  console.log(`[OAuth][DEBUG]   clientId: ${clientId ? `${clientId.slice(0, 6)}...${clientId.slice(-3)}` : '(EMPTY)'}`)
  console.log(`[OAuth][DEBUG]   redirectUri: ${redirectUri}`)
  console.log(`[OAuth][DEBUG]   scopes: ${scopes.join(' ')}`)
  console.log(`[OAuth][DEBUG]   authUrl: ${providerConfig.authUrl}`)

  if (!clientId) {
    console.error(`[OAuth][ERROR] Client ID not configured for ${provider}. Cannot build auth URL.`)
    throw new Error(`Client ID not configured for ${provider}. Call setProviderClientId() first.`)
  }

  const verifier = await generateCodeVerifier()
  const challenge = await generateCodeChallenge(verifier)
  const state = generateState()

  const params = new URLSearchParams({
    client_id: clientId,
    redirect_uri: redirectUri,
    response_type: 'code',
    scope: scopes.join(' '),
    state,
    access_type: 'offline',
    prompt: 'consent',
    code_challenge_method: 'S256',
    code_challenge: challenge,
  })

  const fullUrl = `${providerConfig.authUrl}?${params.toString()}`
  console.log(`[OAuth][DEBUG]   Full auth URL: ${fullUrl}`)
  console.log(`[OAuth][DEBUG]   PKCE state: ${state.slice(0, 8)}...`)
  console.log(`[OAuth][DEBUG]   PKCE challenge: ${challenge.slice(0, 12)}...`)

  return {
    url: fullUrl,
    verifier,
    state,
  }
}

export async function exchangeCodeForToken(
  provider: OAuthProvider,
  code: string,
  verifier: string,
  redirectUri: string
): Promise<OAuthToken> {
  const config = PROVIDER_CONFIGS[provider]

  console.log(`[OAuth][DEBUG] exchangeCodeForToken(${provider})`)
  console.log(`[OAuth][DEBUG]   tokenUrl: ${config.tokenUrl}`)
  console.log(`[OAuth][DEBUG]   clientId: ${config.clientId ? `${config.clientId.slice(0, 6)}...${config.clientId.slice(-3)}` : '(EMPTY)'}`)
  console.log(`[OAuth][DEBUG]   clientSecret: ${config.clientSecret ? 'SET' : '(EMPTY)'}`)
  console.log(`[OAuth][DEBUG]   code: ${code.slice(0, 8)}...`)
  console.log(`[OAuth][DEBUG]   redirectUri: ${redirectUri}`)
  console.log(`[OAuth][DEBUG]   verifier: ${verifier.slice(0, 12)}...`)

  const params = new URLSearchParams({
    client_id: config.clientId,
    code,
    redirect_uri: redirectUri,
    grant_type: 'authorization_code',
    code_verifier: verifier,
  })

  if (config.clientSecret) {
    params.set('client_secret', config.clientSecret)
  }

  console.log(`[OAuth][DEBUG]   POST ${config.tokenUrl}`)
  console.log(`[OAuth][DEBUG]   Body params: ${[...params.keys()].join(', ')}`)

  const response = await fetch(config.tokenUrl, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      Accept: 'application/json',
    },
    body: params.toString(),
  })

  console.log(`[OAuth][DEBUG]   Response status: ${response.status} ${response.statusText}`)

  if (!response.ok) {
    const errorText = await response.text()
    console.error(`[OAuth][ERROR] Token exchange FAILED (${response.status}):`, errorText)
    throw new Error(`Token exchange failed (${response.status}): ${errorText}`)
  }

  const data = await response.json()
  const expiresAt = data.expires_in
    ? Math.floor(Date.now() / 1000) + data.expires_in
    : null

  console.log(`[OAuth][DEBUG]   Token exchange SUCCESS`)
  console.log(`[OAuth][DEBUG]   token_type: ${data.token_type || 'Bearer'}`)
  console.log(`[OAuth][DEBUG]   expires_in: ${data.expires_in || 'N/A'}s`)
  console.log(`[OAuth][DEBUG]   scope: ${data.scope || 'N/A'}`)
  console.log(`[OAuth][DEBUG]   has_refresh_token: ${!!data.refresh_token}`)
  console.log(`[OAuth][DEBUG]   access_token: ${data.access_token ? `${data.access_token.slice(0, 8)}...` : '(MISSING)'}`)

  return {
    accessToken: data.access_token,
    refreshToken: data.refresh_token || null,
    expiresAt,
    tokenType: data.token_type || 'Bearer',
    scope: data.scope || null,
    provider,
  }
}

export async function refreshAccessToken(token: OAuthToken): Promise<OAuthToken> {
  if (!token.refreshToken) {
    throw new Error('No refresh token available')
  }

  const config = PROVIDER_CONFIGS[token.provider]

  const params = new URLSearchParams({
    client_id: config.clientId,
    refresh_token: token.refreshToken,
    grant_type: 'refresh_token',
  })

  if (config.clientSecret) {
    params.set('client_secret', config.clientSecret)
  }

  const response = await fetch(config.tokenUrl, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      Accept: 'application/json',
    },
    body: params.toString(),
  })

  if (!response.ok) {
    const errorText = await response.text()
    throw new Error(`Token refresh failed (${response.status}): ${errorText}`)
  }

  const data = await response.json()
  const expiresAt = data.expires_in
    ? Math.floor(Date.now() / 1000) + data.expires_in
    : null

  return {
    ...token,
    accessToken: data.access_token,
    refreshToken: data.refresh_token || token.refreshToken,
    expiresAt: expiresAt || token.expiresAt,
  }
}

export function isTokenExpired(token: OAuthToken, bufferSeconds = 300): boolean {
  if (!token.expiresAt) return false
  return Math.floor(Date.now() / 1000) + bufferSeconds >= token.expiresAt
}

export async function getValidToken(token: OAuthToken): Promise<OAuthToken> {
  if (isTokenExpired(token)) {
    return refreshAccessToken(token)
  }
  return token
}

export function openAuthPopup(
  url: string,
  title = 'OAuth Authorization',
  width = 600,
  height = 700
): Window | null {
  const left = Math.round(window.screenX + (window.innerWidth - width) / 2)
  const top = Math.round(window.screenY + (window.innerHeight - height) / 2)
  return window.open(
    url,
    title,
    `width=${width},height=${height},left=${left},top=${top},popup=1`
  )
}

export async function authenticateWithPopup(
  provider: OAuthProvider,
  config: { clientId?: string; redirectUri?: string; scopes?: string[] } = {}
): Promise<OAuthToken> {
  console.log(`[OAuth][DEBUG] authenticateWithPopup(${provider}) — starting popup flow`)

  const { url, verifier, state: originalState } = await buildAuthorizationUrlAsync(provider, config)

  console.log(`[OAuth][DEBUG]   Opening popup for: ${provider}`)
  console.log(`[OAuth][DEBUG]   Auth URL (first 120 chars): ${url.slice(0, 120)}...`)

  return new Promise((resolve, reject) => {
    const popup = openAuthPopup(url)
    if (!popup) {
      console.error(`[OAuth][ERROR] Popup was blocked by browser for ${provider}`)
      reject(new Error('Popup blocked. Please allow popups for this site.'))
      return
    }

    console.log(`[OAuth][DEBUG]   Popup opened, polling for redirect...`)

    const interval = setInterval(() => {
      try {
        if (popup.closed) {
          clearInterval(interval)
          console.warn(`[OAuth][WARN] Popup closed by user for ${provider}`)
          reject(new Error('Authorization popup was closed by the user'))
          return
        }

        const popupUrl = popup.location.href
        if (!popupUrl || popupUrl === 'about:blank') return

        const urlObj = new URL(popupUrl)
        const code = urlObj.searchParams.get('code')
        const state = urlObj.searchParams.get('state')
        const error = urlObj.searchParams.get('error')

        console.log(`[OAuth][DEBUG]   Popup URL: ${popupUrl.slice(0, 100)}...`)

        if (error) {
          clearInterval(interval)
          const errorDesc = urlObj.searchParams.get('error_description')
          console.error(`[OAuth][ERROR] OAuth error from ${provider}: ${error}`)
          if (errorDesc) console.error(`[OAuth][ERROR]   description: ${errorDesc}`)
          popup.close()
          reject(new Error(`OAuth error: ${error}${errorDesc ? ` — ${errorDesc}` : ''}`))
          return
        }

        if (code && state === originalState) {
          clearInterval(interval)
          console.log(`[OAuth][DEBUG]   Received auth code: ${code.slice(0, 8)}...`)
          console.log(`[OAuth][DEBUG]   State verified ✓`)
          popup.close()

          const redirectUri = config.redirectUri || PROVIDER_CONFIGS[provider].redirectUri
          console.log(`[OAuth][DEBUG]   Exchanging code for token...`)
          exchangeCodeForToken(provider, code, verifier, redirectUri)
            .then((token) => {
              console.log(`[OAuth][DEBUG]   Token obtained for ${provider} ✓`)
              resolve(token)
            })
            .catch((err) => {
              console.error(`[OAuth][ERROR] Token exchange failed for ${provider}:`, err)
              reject(err)
            })
        }
      } catch {
        // Cross-origin errors until redirected to our origin
      }
    }, 500)

    setTimeout(() => {
      clearInterval(interval)
      console.error(`[OAuth][ERROR] Authorization timed out after 5 minutes for ${provider}`)
      popup.close()
      reject(new Error('OAuth authorization timed out after 5 minutes'))
    }, 300000)
  })
}

const DB_NAME = 'CybermanjuDrive'
const DB_VERSION = 1
const STORE_NAME = 'oauth_tokens'

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onupgradeneeded = (event) => {
      const database = (event.target as IDBOpenDBRequest).result
      if (!database.objectStoreNames.contains(STORE_NAME)) {
        database.createObjectStore(STORE_NAME, { keyPath: 'provider' })
      }
    }

    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}

async function saveTokenToIndexedDB(provider: string, token: OAuthToken): Promise<void> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite')
    const store = transaction.objectStore(STORE_NAME)
    const request = store.put(token)
    request.onsuccess = () => resolve()
    request.onerror = () => reject(request.error)
  })
}

async function loadTokenFromIndexedDB(provider: string): Promise<OAuthToken | null> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readonly')
    const store = transaction.objectStore(STORE_NAME)
    const request = store.get(provider)
    request.onsuccess = () => resolve(request.result || null)
    request.onerror = () => reject(request.error)
  })
}

async function removeTokenFromIndexedDB(provider: string): Promise<void> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite')
    const store = transaction.objectStore(STORE_NAME)
    const request = store.delete(provider)
    request.onsuccess = () => resolve()
    request.onerror = () => reject(request.error)
  })
}

async function clearAllTokensFromIndexedDB(): Promise<void> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite')
    const store = transaction.objectStore(STORE_NAME)
    const request = store.clear()
    request.onsuccess = () => resolve()
    request.onerror = () => reject(request.error)
  })
}

export async function saveTokenToStorage(token: OAuthToken): Promise<void> {
  await saveTokenToIndexedDB(token.provider, token)
}

export async function loadTokenFromStorage(provider: OAuthProvider): Promise<OAuthToken | null> {
  return loadTokenFromIndexedDB(provider)
}

export async function removeTokenFromStorage(provider: OAuthProvider): Promise<void> {
  await removeTokenFromIndexedDB(provider)
}

export async function clearAllTokens(): Promise<void> {
  await clearAllTokensFromIndexedDB()
}
