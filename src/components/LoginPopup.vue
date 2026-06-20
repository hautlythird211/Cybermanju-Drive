<template>
  <Teleport to="body">
    <div
      v-if="store.showLoginPopup"
      class="login-overlay"
      @click.self="store.showLoginPopup = false"
    >
      <div ref="loginRef" class="login-modal" role="dialog" aria-label="Login">
        <button class="close-btn" @click="store.showLoginPopup = false">✕</button>

        <div class="login-brand">
          <svg class="login-logo" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="#00ff41" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
          <div class="login-title">CYBERMANJU DRIVE</div>
          <div class="login-subtitle">{{ isRegister ? 'CREATE ACCOUNT' : 'SIGN IN' }}</div>
        </div>

        <div class="login-body">
          <div class="field">
            <label class="field-label" for="login-username">USERNAME</label>
            <div class="input-wrap">
              <svg class="input-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
              </svg>
              <input
                id="login-username"
                ref="usernameRef"
                v-model="username"
                class="login-input"
                placeholder="Enter username"
                @keyup.enter="isRegister ? handleRegister() : handleLogin()"
              />
            </div>
          </div>
          <div class="field">
            <label class="field-label" for="login-password">PASSWORD</label>
            <div class="input-wrap">
              <svg class="input-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
              </svg>
              <input
                id="login-password"
                v-model="password"
                type="password"
                class="login-input"
                placeholder="Enter password"
                @keyup.enter="isRegister ? handleRegister() : handleLogin()"
              />
            </div>
          </div>
          <div class="field" v-if="isRegister">
            <label class="field-label" for="login-display">DISPLAY NAME</label>
            <div class="input-wrap">
              <svg class="input-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
              </svg>
              <input id="login-display" v-model="displayName" class="login-input" placeholder="Optional display name" />
            </div>
          </div>

          <div class="login-error" v-if="errorMsg">{{ errorMsg }}</div>

          <div class="login-actions">
            <button class="btn btn-secondary" @click="isRegister = !isRegister; errorMsg = ''">
              {{ isRegister ? 'SIGN IN' : 'REGISTER' }}
            </button>
            <button class="btn btn-primary" @click="isRegister ? handleRegister() : handleLogin()">
              {{ isRegister ? 'CREATE' : 'LOGIN' }}
            </button>
          </div>

          <div class="oauth-divider">
            <span>OR CONTINUE WITH</span>
          </div>

          <div class="oauth-providers">
            <button
              v-for="p in OAUTH_PROVIDERS"
              :key="p.id"
              class="oauth-btn"
              :class="{ connecting: connectingProvider === p.id }"
              :disabled="!!connectingProvider"
              @click="handleOAuth(p.id)"
              :title="p.label"
            >
              <div class="oauth-icon-circle">
                <Icon v-if="p.icon" :icon="p.icon" width="20" height="20" />
                <span v-else class="oauth-icon-fallback">{{ p.label[0] }}</span>
              </div>
              <span class="oauth-label">{{ p.label }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
    <ImportDialog
      :visible="importVisible"
      :backendType="importBackend"
      :token="importToken"
      :label="importLabel"
      @close="importVisible = false"
      @import="onImportDone"
    />
  </Teleport>
</template>

<script setup lang="ts">
import { ref, toRef, onMounted, nextTick } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { invoke } from '@/composables/useTauri'
import { useNotifications } from '@/composables/useNotifications'
import { useFocusTrap } from '@/composables/useFocusTrap'
import ImportDialog from '@/components/ImportDialog.vue'
import type { OAuthProvider, OAuthToken } from '@/wasm'

const store = useAppStore()
const { notify } = useNotifications()
const usernameRef = ref<HTMLInputElement | null>(null)
const loginRef = ref<HTMLElement | null>(null)

useFocusTrap(loginRef, { active: toRef(store, 'showLoginPopup') })

onMounted(() => {
  nextTick(() => usernameRef.value?.focus())
})

const OAUTH_PROVIDERS = [
  { id: 'google', label: 'Google', icon: 'logos:google-icon', color: '#4285F4' },
  { id: 'github' as OAuthProvider, label: 'GitHub', icon: 'logos:github-icon', color: '#fff' },
  { id: 'gitlab' as OAuthProvider, label: 'GitLab', icon: 'logos:gitlab', color: '#FC6D26' },
]

const username = ref('')
const password = ref('')
const displayName = ref('')
const isRegister = ref(false)
const errorMsg = ref('')
const connectingProvider = ref<string | null>(null)

// ── Import dialog state ──
const importVisible = ref(false)
const importBackend = ref('')
const importToken = ref('')
const importLabel = ref('')
let pendingOAuthToken: OAuthToken | null = null

function showImportDialog(backend: string, token: OAuthToken, label: string) {
  pendingOAuthToken = token
  importBackend.value = backend
  importToken.value = token.accessToken
  importLabel.value = label
  importVisible.value = true
}

function onImportDone() {
  store.showLoginPopup = false
  notify('success', 'IMPORT COMPLETE')
}

async function handleLogin() {
  errorMsg.value = ''
  if (!username.value.trim() || !password.value.trim()) {
    errorMsg.value = 'ENTER USERNAME AND PASSWORD'
    return
  }
  try {
    const result = await invoke<{ userId: string; username: string; role: string; displayName?: string; token: string }>('authenticate_user', {
      username: username.value,
      password: password.value,
    })
    store.currentUser = result
    store.showLoginPopup = false
    notify('success', `LOGGED IN AS ${result.username}`)
    store.authToken = result.token
    username.value = ''
    password.value = ''
  } catch (e) {
    errorMsg.value = `LOGIN FAILED: ${e instanceof Error ? e.message : String(e)}`
  }
}

async function handleRegister() {
  errorMsg.value = ''
  if (!username.value.trim() || !password.value.trim()) {
    errorMsg.value = 'ENTER USERNAME AND PASSWORD'
    return
  }
  try {
    await invoke('register_user', {
      username: username.value,
      password: password.value,
      displayName: displayName.value || undefined,
      role: 'user',
    })
    notify('success', 'USER REGISTERED')
    isRegister.value = false
    displayName.value = ''
  } catch (e) {
    errorMsg.value = `REGISTRATION FAILED: ${e instanceof Error ? e.message : String(e)}`
  }
}

async function handleOAuth(provider: string) {
  errorMsg.value = ''
  connectingProvider.value = provider
  try {
    const { oauth, initWasm } = await import('@/wasm')
    await initWasm()
    oauth.loadClientIdsFromEnv()

    if (provider === 'google') {
      await handleGoogleOAuth(oauth)
      return
    }

    const p = provider as OAuthProvider
    const clientId = oauth.getProviderClientId(p)
    if (!clientId) {
      errorMsg.value = `${provider}: CLIENT ID NOT CONFIGURED — set VITE_OAUTH_${provider.toUpperCase()}_CLIENT_ID or ${provider.toUpperCase()}_CLIENT_ID in GitHub Secrets`
      return
    }

    await doOAuth(p, p, oauth)
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    if (msg.includes('Popup blocked')) {
      errorMsg.value = 'POPUP BLOCKED — allow popups for this site'
    } else if (msg.includes('timed out')) {
      errorMsg.value = 'AUTHORIZATION TIMED OUT'
    } else if (msg.includes('closed by the user')) {
      errorMsg.value = 'AUTHORIZATION CANCELLED'
    } else {
      errorMsg.value = msg
    }
  } finally {
    connectingProvider.value = null
  }
}

async function handleGoogleOAuth(oauth: typeof import('@/wasm/oauth')) {
  const GOOGLE_SCOPES = [
    'https://www.googleapis.com/auth/drive.file',
    'https://www.googleapis.com/auth/photoslibrary.appendonly',
  ]

  const clientId = oauth.getProviderClientId('googleDrive')
  if (!clientId) {
    errorMsg.value = 'GOOGLE: CLIENT ID NOT CONFIGURED — set VITE_OAUTH_GOOGLE_CLIENT_ID or GOOGLE_CLIENT_ID in GitHub Secrets'
    return
  }

  const existingToken = await oauth.loadTokenFromStorage('googleDrive')
  let token
  if (existingToken) {
    token = await oauth.getValidToken(existingToken)
    if (token) oauth.saveTokenToStorage(token)
  }
  if (!token) {
    token = await oauth.authenticateWithPopup('googleDrive', { scopes: GOOGLE_SCOPES })
    oauth.saveTokenToStorage(token)
  }

  const data = await import('@/wasm/data')
  const driveAccount = await data.upsertOAuthAccount('googleDrive', token)
  await data.upsertOAuthAccount('googlePhotos', token)

  store.currentUser = {
    userId: driveAccount.id,
    username: driveAccount.name,
    role: 'user',
    displayName: driveAccount.name,
    token: token.accessToken,
  }
  store.authToken = token.accessToken
  await store.fetchAccounts()
  store.showLoginPopup = false
  showImportDialog('google', token, 'Google (Drive + Photos)')
  notify('success', 'AUTHENTICATED WITH GOOGLE')
}

async function doOAuth(provider: OAuthProvider, storageKey: OAuthProvider, oauth: typeof import('@/wasm/oauth')) {
  const existingToken = await oauth.loadTokenFromStorage(storageKey)
  if (existingToken) {
    const validToken = await oauth.getValidToken(existingToken)
    if (validToken) {
      oauth.saveTokenToStorage(validToken)
      const data = await import('@/wasm/data')
      const account = await data.upsertOAuthAccount(provider, validToken)
      store.currentUser = {
        userId: account.id,
        username: account.name,
        role: 'user',
        displayName: account.name,
        token: validToken.accessToken,
      }
      store.authToken = validToken.accessToken
      await store.fetchAccounts()
      store.showLoginPopup = false
      showImportDialog(provider, validToken, account.name)
      notify('success', `AUTHENTICATED WITH ${provider.toUpperCase()}`)
      return
    }
  }

  const token = await oauth.authenticateWithPopup(provider)
  oauth.saveTokenToStorage(token)
  const data = await import('@/wasm/data')
  const account = await data.upsertOAuthAccount(provider, token)
  store.currentUser = {
    userId: account.id,
    username: account.name,
    role: 'user',
    displayName: account.name,
    token: token.accessToken,
  }
  store.authToken = token.accessToken
  await store.fetchAccounts()
  store.showLoginPopup = false
  showImportDialog(provider, token, account.name)
  notify('success', `AUTHENTICATED WITH ${provider.toUpperCase()}`)
}
</script>

<style scoped>
.login-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10002;
  backdrop-filter: blur(4px);
}

.login-modal {
  position: relative;
  background: #0d0d0d;
  border: 1px solid #1a1a1a;
  border-radius: 12px;
  padding: 28px 24px 20px;
  max-width: 380px;
  width: 92%;
  font-family: 'Courier New', 'Fira Code', monospace;
  box-shadow: 0 0 80px rgba(0, 255, 65, 0.04), 0 8px 32px rgba(0, 0, 0, 0.6);
}

.close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid #2a2a2a;
  border-radius: 6px;
  color: #555;
  font-size: 10px;
  cursor: pointer;
  transition: all 0.15s;
}

.close-btn:hover {
  border-color: #ff5f57;
  color: #ff5f57;
  background: rgba(255, 95, 87, 0.06);
}

.login-brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  margin-bottom: 20px;
}

.login-logo {
  opacity: 0.7;
}

.login-title {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-accent);
  letter-spacing: 2px;
  text-shadow: 0 0 8px rgba(var(--accent-rgb), 0.12);
}

.login-subtitle {
  font-size: 9px;
  color: #555;
  letter-spacing: 2px;
}

.login-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field-label {
  font-size: 9px;
  color: #666;
  letter-spacing: 1px;
  font-weight: 700;
  padding-left: 2px;
}

.input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.input-icon {
  position: absolute;
  left: 10px;
  color: #444;
  pointer-events: none;
  flex-shrink: 0;
}

.login-input {
  width: 100%;
  background: var(--bg-surface);
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #e0e0e0;
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 8px 10px 8px 32px;
  outline: none;
  transition: border-color 0.15s;
}

.login-input:focus {
  border-color: var(--text-accent);
  box-shadow: 0 0 8px rgba(var(--accent-rgb), 0.08);
}

.login-input::placeholder {
  color: #333;
}

.login-error {
  font-size: 10px;
  color: #ff5f57;
  padding: 4px 0;
  text-align: center;
}

.login-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.btn {
  flex: 1;
  padding: 8px 14px;
  border-radius: 6px;
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.15s;
  text-align: center;
}

.btn-primary {
  background: rgba(var(--accent-rgb), 0.08);
  border: 1px solid #00ff41;
  color: var(--text-accent);
  text-shadow: 0 0 4px rgba(var(--accent-rgb), 0.15);
}

.btn-primary:hover {
  background: rgba(var(--accent-rgb), 0.15);
  box-shadow: 0 0 12px rgba(var(--accent-rgb), 0.1);
}

.btn-secondary {
  background: transparent;
  border: 1px solid #2a2a2a;
  color: #888;
}

.btn-secondary:hover {
  border-color: #444;
  color: #ccc;
}

.oauth-divider {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 6px 0 2px;
  font-size: 8px;
  color: #444;
  letter-spacing: 1px;
}

.oauth-divider::before,
.oauth-divider::after {
  content: '';
  flex: 1;
  border-top: 1px solid #1a1a1a;
}

.oauth-providers {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.oauth-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: var(--bg-surface);
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
  font-family: var(--font-mono);
  text-align: left;
}

.oauth-btn:hover {
  border-color: #333;
  background: #0f0f0f;
}

.oauth-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.oauth-btn.connecting {
  border-color: #febc2e;
  background: rgba(254, 188, 46, 0.04);
}

.oauth-icon-circle {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #111;
  flex-shrink: 0;
  border: 1px solid #1a1a1a;
}

.oauth-icon-fallback {
  font-size: 12px;
  font-weight: 700;
  color: #555;
}

.oauth-label {
  font-size: 9px;
  font-weight: 600;
  color: #ccc;
  letter-spacing: 0.5px;
  line-height: 1.2;
}
</style>
