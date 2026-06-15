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
  redirectUri: string
  scopes: string[]
  authUrl: string
  tokenUrl: string
}

const PROVIDER_CONFIGS: Record<OAuthProvider, OAuthConfig> = {
  googleDrive: {
    clientId: '',
    redirectUri: `${window.location.origin}/oauth/callback`,
    scopes: ['https://www.googleapis.com/auth/drive.file'],
    authUrl: 'https://accounts.google.com/o/oauth2/v2/auth',
    tokenUrl: 'https://oauth2.googleapis.com/token',
  },
  googlePhotos: {
    clientId: '',
    redirectUri: `${window.location.origin}/oauth/callback`,
    scopes: ['https://www.googleapis.com/auth/photoslibrary.appendonly'],
    authUrl: 'https://accounts.google.com/o/oauth2/v2/auth',
    tokenUrl: 'https://oauth2.googleapis.com/token',
  },
  github: {
    clientId: '',
    redirectUri: `${window.location.origin}/oauth/callback`,
    scopes: ['repo'],
    authUrl: 'https://github.com/login/oauth/authorize',
    tokenUrl: 'https://github.com/login/oauth/access_token',
  },
  gitlab: {
    clientId: '',
    redirectUri: `${window.location.origin}/oauth/callback`,
    scopes: ['api'],
    authUrl: 'https://gitlab.com/oauth/authorize',
    tokenUrl: 'https://gitlab.com/oauth/token',
  },
  telegram: {
    clientId: '',
    redirectUri: `${window.location.origin}/oauth/callback`,
    scopes: ['bot'],
    authUrl: 'https://oauth.telegram.org/auth',
    tokenUrl: 'https://oauth.telegram.org/token',
  },
}

export function setProviderClientId(provider: OAuthProvider, clientId: string): void {
  PROVIDER_CONFIGS[provider].clientId = clientId
}

export function getProviderConfig(provider: OAuthProvider): OAuthConfig {
  return PROVIDER_CONFIGS[provider]
}

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

  if (!clientId) {
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

  return {
    url: `${providerConfig.authUrl}?${params.toString()}`,
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

  const params = new URLSearchParams({
    client_id: config.clientId,
    code,
    redirect_uri: redirectUri,
    grant_type: 'authorization_code',
    code_verifier: verifier,
  })

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
    throw new Error(`Token exchange failed (${response.status}): ${errorText}`)
  }

  const data = await response.json()
  const expiresAt = data.expires_in
    ? Math.floor(Date.now() / 1000) + data.expires_in
    : null

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
  const { url, verifier, state: originalState } = await buildAuthorizationUrlAsync(provider, config)

  return new Promise((resolve, reject) => {
    const popup = openAuthPopup(url)
    if (!popup) {
      reject(new Error('Popup blocked. Please allow popups for this site.'))
      return
    }

    const interval = setInterval(() => {
      try {
        if (popup.closed) {
          clearInterval(interval)
          reject(new Error('Authorization popup was closed by the user'))
          return
        }

        const popupUrl = popup.location.href
        if (!popupUrl || popupUrl === 'about:blank') return

        const urlObj = new URL(popupUrl)
        const code = urlObj.searchParams.get('code')
        const state = urlObj.searchParams.get('state')
        const error = urlObj.searchParams.get('error')

        if (error) {
          clearInterval(interval)
          popup.close()
          reject(new Error(`OAuth error: ${error}`))
          return
        }

        if (code && state === originalState) {
          clearInterval(interval)
          popup.close()

          const redirectUri = config.redirectUri || PROVIDER_CONFIGS[provider].redirectUri
          exchangeCodeForToken(provider, code, verifier, redirectUri)
            .then(resolve)
            .catch(reject)
        }
      } catch {
        // Cross-origin errors until redirected to our origin
      }
    }, 500)

    setTimeout(() => {
      clearInterval(interval)
      popup.close()
      reject(new Error('OAuth authorization timed out after 5 minutes'))
    }, 300000)
  })
}

const TOKEN_STORE_KEY = 'cybermanju:oauth:tokens'

export function saveTokenToStorage(token: OAuthToken): void {
  try {
    const stored = JSON.parse(localStorage.getItem(TOKEN_STORE_KEY) || '{}')
    stored[token.provider] = token
    localStorage.setItem(TOKEN_STORE_KEY, JSON.stringify(stored))
  } catch {
    console.warn('Failed to save OAuth token to localStorage')
  }
}

export function loadTokenFromStorage(provider: OAuthProvider): OAuthToken | null {
  try {
    const stored = JSON.parse(localStorage.getItem(TOKEN_STORE_KEY) || '{}')
    return stored[provider] || null
  } catch {
    return null
  }
}

export function removeTokenFromStorage(provider: OAuthProvider): void {
  try {
    const stored = JSON.parse(localStorage.getItem(TOKEN_STORE_KEY) || '{}')
    delete stored[provider]
    localStorage.setItem(TOKEN_STORE_KEY, JSON.stringify(stored))
  } catch {
    console.warn('Failed to remove OAuth token from localStorage')
  }
}

export function clearAllTokens(): void {
  localStorage.removeItem(TOKEN_STORE_KEY)
}
