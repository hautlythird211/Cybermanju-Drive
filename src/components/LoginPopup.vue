<template>
  <Teleport to="body">
    <div
      v-if="store.showLoginPopup"
      class="login-overlay"
      @click.self="store.showLoginPopup = false"
    >
      <div ref="loginRef" class="login-modal" role="dialog" aria-label="Login">
        <div class="login-header">
          <h2>{{ isRegister ? 'REGISTER' : 'LOGIN' }}</h2>
          <button class="close-btn" @click="store.showLoginPopup = false" aria-label="Close">X</button>
        </div>

        <div class="login-body">
          <div class="field">
            <label class="field-label text-muted" for="login-username">USERNAME</label>
            <input
              id="login-username"
              ref="usernameRef"
              v-model="username"
              class="bw-input"
              placeholder="ENTER USERNAME"
              @keyup.enter="isRegister ? handleRegister() : handleLogin()"
            />
          </div>
          <div class="field">
            <label class="field-label text-muted" for="login-password">PASSWORD</label>
            <input
              id="login-password"
              v-model="password"
              type="password"
              class="bw-input"
              placeholder="ENTER PASSWORD"
              @keyup.enter="isRegister ? handleRegister() : handleLogin()"
            />
          </div>
          <div class="field" v-if="isRegister">
            <label class="field-label text-muted" for="login-display">DISPLAY NAME</label>
            <input id="login-display" v-model="displayName" class="bw-input" placeholder="OPTIONAL" />
          </div>

          <div class="login-error text-muted" v-if="errorMsg" role="alert" aria-live="assertive">{{ errorMsg }}</div>

          <div class="login-actions">
            <button class="bw-btn" @click="isRegister = !isRegister; errorMsg = ''">
              {{ isRegister ? '[BACK TO LOGIN]' : '[REGISTER]' }}
            </button>
            <button class="bw-btn bw-btn-inverse" @click="isRegister ? handleRegister() : handleLogin()">
              {{ isRegister ? '[REGISTER]' : '[LOGIN]' }}
            </button>
          </div>

          <div class="oauth-divider">
            <span class="oauth-divider-text">OR</span>
          </div>

          <div class="oauth-buttons">
            <button class="bw-btn oauth-btn" @click="handleOAuth('googleDrive')">
              [GOOGLE DRIVE]
            </button>
            <button class="bw-btn oauth-btn" @click="handleOAuth('googlePhotos')">
              [GOOGLE PHOTOS]
            </button>
            <button class="bw-btn oauth-btn" @click="handleOAuth('github')">
              [GITHUB]
            </button>
            <button class="bw-btn oauth-btn" @click="handleOAuth('gitlab')">
              [GITLAB]
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, toRef, onMounted, nextTick } from 'vue'
import { useAppStore } from '@/stores/app'
import { invoke } from '@/composables/useTauri'
import { useNotifications } from '@/composables/useNotifications'
import { useFocusTrap } from '@/composables/useFocusTrap'
import type { OAuthProvider } from '@/wasm'

const store = useAppStore()
const { notify } = useNotifications()
const usernameRef = ref<HTMLInputElement | null>(null)
const loginRef = ref<HTMLElement | null>(null)

useFocusTrap(loginRef, toRef(store, 'showLoginPopup'))

onMounted(() => {
  nextTick(() => usernameRef.value?.focus())
})

const username = ref('')
const password = ref('')
const displayName = ref('')
const isRegister = ref(false)
const errorMsg = ref('')

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

async function handleOAuth(provider: OAuthProvider) {
  errorMsg.value = ''
  try {
    const { oauth, initWasm } = await import('@/wasm')
    await initWasm()

    // Try to load existing token first
    const existingToken = await oauth.loadTokenFromStorage(provider)
    if (existingToken) {
      const validToken = await oauth.getValidToken(existingToken)
      if (validToken) oauth.saveTokenToStorage(validToken)

      // Create/update account in IndexedDB
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
      store.showLoginPopup = false
      notify('success', `AUTHENTICATED WITH ${provider.toUpperCase()}`)
      return
    }

    const token = await oauth.authenticateWithPopup(provider)
    oauth.saveTokenToStorage(token)

    // Create/update account in IndexedDB from OAuth token
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
    store.showLoginPopup = false
    await store.fetchAccounts()
    notify('success', `AUTHENTICATED WITH ${provider.toUpperCase()}`)
  } catch (e) {
    errorMsg.value = `OAUTH FAILED: ${e instanceof Error ? e.message : String(e)}`
  }
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
}

.login-modal {
  background: #FFFFFF;
  border: 2px solid #000000;
  box-shadow: 4px 4px 0 #000000;
  padding: 20px;
  max-width: 360px;
  width: 90%;
  font-family: 'Courier New', monospace;
}

.login-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 2px solid #000000;
}

.login-header h2 {
  font-size: 13px;
  font-weight: 800;
  letter-spacing: 1px;
  color: #000000;
  margin: 0;
}

.close-btn {
  background: none;
  border: 2px solid #000000;
  color: #000000;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
}

.close-btn:hover {
  background: #000000;
  color: #FFFFFF;
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
  font-size: 10px;
  letter-spacing: 0.5px;
}

.bw-input {
  background: #FFFFFF;
  border: 2px solid #000000;
  padding: 6px 8px;
  color: #000000;
  font-family: 'Courier New', monospace;
  font-size: 11px;
}

.bw-input::placeholder {
  color: rgba(0, 0, 0, 0.3);
}

.bw-input:focus {
  outline: 2px solid #000000;
  outline-offset: -4px;
}

.login-error {
  font-size: 10px;
  color: #000000;
  padding: 4px 0;
}

.login-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 8px;
}

.bw-btn {
  padding: 6px 14px;
  background: #000000;
  border: 2px solid #000000;
  color: #FFFFFF;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
}

.bw-btn:hover {
  background: #FFFFFF;
  color: #000000;
}

.bw-btn-inverse {
  background: #FFFFFF;
  color: #000000;
}

.bw-btn-inverse:hover {
  background: #000000;
  color: #FFFFFF;
}

.text-muted { color: rgba(0, 0, 0, 0.5) !important; }

.oauth-divider {
  display: flex;
  align-items: center;
  margin: 12px 0 8px;
  gap: 8px;
}

.oauth-divider::before,
.oauth-divider::after {
  content: '';
  flex: 1;
  border-top: 1px solid #000000;
}

.oauth-divider-text {
  font-size: 9px;
  letter-spacing: 1px;
  color: rgba(0, 0, 0, 0.4);
}

.oauth-buttons {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.oauth-btn {
  font-size: 9px;
  padding: 4px 6px;
  text-align: center;
  background: #000000;
  color: #FFFFFF;
  border: 1px solid #000000;
}

.oauth-btn:hover {
  background: #FFFFFF;
  color: #000000;
}
</style>
