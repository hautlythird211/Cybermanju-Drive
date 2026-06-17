<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'

const emit = defineEmits<{ (e: 'login', user: string): void }>()

const users = ref([
  { name: 'admin', avatar: 'AD', lastLogin: 'Just now', session: 'cybermanju-shell' },
  { name: 'operator', avatar: 'OP', lastLogin: 'Yesterday', session: 'cybermanju-shell' },
  { name: 'guest', avatar: 'GT', lastLogin: 'Never', session: 'cybermanju-shell' },
])

const selectedUserIdx = ref(0)
const password = ref('')
const showPassword = ref(false)
const showUserList = ref(true)
const unlocking = ref(false)
const showSessionOptions = ref(false)
const showAccessibility = ref(false)
const loginError = ref('')

const sessions = [
  { id: 'cybermanju-shell', label: 'Cybermanju Shell', desc: 'Post-quantum encrypted desktop' },
  { id: 'kde', label: 'KDE Plasma', desc: 'Full-featured desktop environment' },
  { id: 'gnome', label: 'GNOME', desc: 'Modern user-friendly desktop' },
  { id: 'i3', label: 'i3 WM', desc: 'Minimal tiling window manager' },
]

const accessibilityOptions = [
  { id: 'screen-reader', label: 'Screen Reader', desc: 'Orca screen reader' },
  { id: 'large-text', label: 'Large Text', desc: 'Increased font sizes' },
  { id: 'high-contrast', label: 'High Contrast', desc: 'Enhanced contrast theme' },
  { id: 'sticky-keys', label: 'Sticky Keys', desc: 'One-handed modifier keys' },
]

function selectUser(idx: number) {
  selectedUserIdx.value = idx
  showUserList.value = false
  password.value = ''
  loginError.value = ''
}

function showAllUsers() {
  showUserList.value = true
  password.value = ''
  loginError.value = ''
  unlocking.value = false
}

function doLogin() {
  if (!password.value.trim()) {
    loginError.value = 'Password required'
    return
  }
  unlocking.value = true
  loginError.value = ''
  setTimeout(() => {
    emit('login', users.value[selectedUserIdx.value].name)
  }, 1800 + Math.random() * 600)
}

function handleKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (showSessionOptions.value) { showSessionOptions.value = false; return }
    if (showAccessibility.value) { showAccessibility.value = false; return }
    if (!showUserList.value) { showAllUsers(); return }
  }
  if (e.key === 'Enter' && !showUserList.value) {
    doLogin()
  }
  if (showUserList.value) {
    if (e.key === 'ArrowUp') { e.preventDefault(); selectedUserIdx.value = (selectedUserIdx.value - 1 + users.value.length) % users.value.length }
    if (e.key === 'ArrowDown') { e.preventDefault(); selectedUserIdx.value = (selectedUserIdx.value + 1) % users.value.length }
    if (e.key === 'ArrowRight' || e.key === 'Enter') { selectUser(selectedUserIdx.value) }
  }
}

onMounted(() => {
  const saved = localStorage.getItem('cybermanju_username')
  if (saved) {
    const idx = users.value.findIndex(u => u.name === saved)
    if (idx >= 0) {
      selectedUserIdx.value = idx
      showUserList.value = false
    }
  }
})
</script>

<template>
  <div class="login-screen" @keydown="handleKey" tabindex="0" autofocus>
    <div class="login-backdrop"></div>
    <div class="login-container">
      <!-- User List -->
      <div v-if="showUserList" class="user-list-panel">
        <div class="login-brand">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="#00ff41" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" class="login-logo">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            <circle cx="12" cy="16" r="1" fill="#00ff41"/>
          </svg>
          <div class="login-brand-text">
            <div class="login-brand-title">CyberManju Drive</div>
            <div class="login-brand-sub">Post-Quantum Encrypted OS</div>
          </div>
        </div>
        <div class="user-list-title">Select a user</div>
        <div
          v-for="(user, i) in users"
          :key="user.name"
          class="user-card"
          :class="{ 'user-selected': selectedUserIdx === i }"
          @click="selectUser(i)"
        >
          <div class="user-avatar">{{ user.avatar }}</div>
          <div class="user-info">
            <div class="user-name">{{ user.name }}</div>
            <div class="user-meta">{{ user.lastLogin }}</div>
          </div>
          <div class="user-session">{{ user.session }}</div>
        </div>
        <div class="login-actions">
          <button class="login-action-btn" @click="showSessionOptions = true">Session Type</button>
          <button class="login-action-btn" @click="showAccessibility = true">Accessibility</button>
          <button class="login-action-btn power-btn">Power Off</button>
        </div>
      </div>

      <!-- Password Prompt -->
      <div v-else class="password-panel">
        <div class="pwd-avatar-large" @click="showAllUsers">
          <div class="avatar-circle">{{ users[selectedUserIdx].avatar }}</div>
          <div class="avatar-name">{{ users[selectedUserIdx].name }}</div>
          <div class="avatar-hint">Click to switch user</div>
        </div>
        <div class="pwd-field">
          <div class="pwd-label">Password</div>
          <div class="pwd-input-wrap">
            <input
              v-model="password"
              :type="showPassword ? 'text' : 'password'"
              class="pwd-input"
              placeholder="Enter password"
              autocomplete="current-password"
              spellcheck="false"
              @keyup.enter="doLogin"
            />
            <button class="pwd-toggle" @click="showPassword = !showPassword">
              {{ showPassword ? 'HIDE' : 'SHOW' }}
            </button>
          </div>
          <div v-if="loginError" class="pwd-error">{{ loginError }}</div>
        </div>
        <div v-if="unlocking" class="unlocking-bar">
          <div class="unlocking-text">Unlocking session...</div>
          <div class="unlocking-track">
            <div class="unlocking-fill"></div>
          </div>
        </div>
        <div v-else class="pwd-actions">
          <button class="pwd-submit" @click="doLogin">[ UNLOCK ]</button>
          <button class="pwd-cancel" @click="showAllUsers">[ CANCEL ]</button>
        </div>
      </div>
    </div>

    <!-- Session Options Modal -->
    <div v-if="showSessionOptions" class="modal-overlay" @click.self="showSessionOptions = false">
      <div class="modal-panel">
        <div class="modal-title">Desktop Session</div>
        <div v-for="s in sessions" :key="s.id" class="modal-option" :class="{ 'modal-option-active': s.id === users[selectedUserIdx].session }" @click="users[selectedUserIdx].session = s.id">
          <div class="modal-option-label">{{ s.label }}</div>
          <div class="modal-option-desc">{{ s.desc }}</div>
        </div>
        <button class="modal-close" @click="showSessionOptions = false">[ CLOSE ]</button>
      </div>
    </div>

    <!-- Accessibility Modal -->
    <div v-if="showAccessibility" class="modal-overlay" @click.self="showAccessibility = false">
      <div class="modal-panel">
        <div class="modal-title">Accessibility</div>
        <div v-for="opt in accessibilityOptions" :key="opt.id" class="modal-option">
          <div class="modal-option-label">{{ opt.label }}</div>
          <div class="modal-option-desc">{{ opt.desc }}</div>
        </div>
        <button class="modal-close" @click="showAccessibility = false">[ CLOSE ]</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-screen {
  position: fixed;
  inset: 0;
  z-index: 99999;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Courier New', 'Fira Code', monospace;
  background: #050505;
  overflow: hidden;
}

.login-backdrop {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse at 20% 50%, rgba(0, 255, 65, 0.03) 0%, transparent 60%),
    radial-gradient(ellipse at 80% 50%, rgba(0, 255, 65, 0.02) 0%, transparent 60%);
  pointer-events: none;
}

.login-container {
  position: relative;
  z-index: 1;
  width: 480px;
  max-width: 92vw;
}

.user-list-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.login-brand {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 8px;
  padding: 0 4px;
}

.login-logo {
  filter: drop-shadow(0 0 8px rgba(0, 255, 65, 0.3));
}

.login-brand-title {
  font-size: 16px;
  font-weight: 800;
  color: #00ff41;
  letter-spacing: 3px;
  text-shadow: 0 0 10px rgba(0, 255, 65, 0.2);
}

.login-brand-sub {
  font-size: 9px;
  color: #555;
  letter-spacing: 1px;
}

.user-list-title {
  font-size: 11px;
  color: #666;
  letter-spacing: 2px;
  margin-bottom: 4px;
  padding: 0 4px;
}

.user-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 16px;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.user-card:hover {
  border-color: #2a2a2a;
  background: rgba(255, 255, 255, 0.02);
}

.user-selected {
  border-color: rgba(0, 255, 65, 0.3);
  background: rgba(0, 255, 65, 0.04);
}

.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: linear-gradient(135deg, #0a0a0a, #1a1a1a);
  border: 1px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 800;
  color: #00ff41;
  text-shadow: 0 0 6px rgba(0, 255, 65, 0.3);
}

.user-info {
  flex: 1;
}

.user-name {
  font-size: 13px;
  font-weight: 700;
  color: #ddd;
}

.user-meta {
  font-size: 9px;
  color: #555;
  margin-top: 2px;
}

.user-session {
  font-size: 8px;
  color: #444;
  letter-spacing: 1px;
}

.login-actions {
  display: flex;
  gap: 8px;
  justify-content: center;
  margin-top: 8px;
}

.login-action-btn {
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #666;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  font-weight: 600;
  padding: 8px 16px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.login-action-btn:hover {
  border-color: #333;
  color: #aaa;
}

.power-btn:hover {
  border-color: rgba(255, 95, 87, 0.3);
  color: #ff5f57;
}

.password-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.pwd-avatar-large {
  text-align: center;
  cursor: pointer;
}

.avatar-circle {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background: linear-gradient(135deg, #0a0a0a, #1a1a1a);
  border: 2px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  font-weight: 800;
  color: #00ff41;
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.3);
  box-shadow: 0 0 20px rgba(0, 255, 65, 0.05);
  margin: 0 auto;
}

.avatar-name {
  font-size: 14px;
  font-weight: 800;
  color: #ddd;
  margin-top: 10px;
  letter-spacing: 2px;
}

.avatar-hint {
  font-size: 9px;
  color: #444;
  margin-top: 4px;
}

.pwd-field {
  width: 100%;
}

.pwd-label {
  font-size: 9px;
  color: #555;
  letter-spacing: 2px;
  margin-bottom: 6px;
  text-transform: uppercase;
}

.pwd-input-wrap {
  display: flex;
  gap: 8px;
}

.pwd-input {
  flex: 1;
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  padding: 10px 12px;
  outline: none;
  transition: border-color 0.15s;
}

.pwd-input:focus {
  border-color: rgba(0, 255, 65, 0.3);
}

.pwd-toggle {
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  font-weight: 600;
  padding: 0 12px;
  cursor: pointer;
  letter-spacing: 1px;
}

.pwd-toggle:hover {
  border-color: #333;
  color: #888;
}

.pwd-error {
  color: #ff5f57;
  font-size: 10px;
  margin-top: 6px;
  font-weight: 600;
}

.unlocking-bar {
  width: 100%;
}

.unlocking-text {
  font-size: 9px;
  color: #00ff41;
  letter-spacing: 2px;
  margin-bottom: 8px;
  text-align: center;
}

.unlocking-track {
  width: 100%;
  height: 3px;
  background: #111;
  border-radius: 2px;
  overflow: hidden;
}

.unlocking-fill {
  height: 100%;
  width: 100%;
  background: linear-gradient(90deg, #00ff41, #00ff88, #00ff41);
  background-size: 200% 100%;
  animation: unlock-progress 1.8s ease-out forwards;
  border-radius: 2px;
  box-shadow: 0 0 10px rgba(0, 255, 65, 0.3);
}

@keyframes unlock-progress {
  0% { width: 0%; }
  30% { width: 30%; }
  60% { width: 65%; }
  85% { width: 88%; }
  100% { width: 100%; }
}

.pwd-actions {
  display: flex;
  gap: 12px;
}

.pwd-submit {
  background: transparent;
  border: 1px solid rgba(0, 255, 65, 0.3);
  border-radius: 6px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  font-weight: 700;
  padding: 10px 28px;
  cursor: pointer;
  letter-spacing: 2px;
  transition: all 0.15s;
}

.pwd-submit:hover {
  background: rgba(0, 255, 65, 0.08);
  border-color: #00ff41;
  box-shadow: 0 0 16px rgba(0, 255, 65, 0.15);
}

.pwd-cancel {
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  font-weight: 600;
  padding: 10px 28px;
  cursor: pointer;
  letter-spacing: 2px;
  transition: all 0.15s;
}

.pwd-cancel:hover {
  border-color: #333;
  color: #888;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100000;
}

.modal-panel {
  width: 400px;
  max-width: 90vw;
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 10px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.modal-title {
  font-size: 12px;
  font-weight: 800;
  color: #ddd;
  letter-spacing: 2px;
  margin-bottom: 12px;
}

.modal-option {
  padding: 10px 14px;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.modal-option:hover {
  border-color: #2a2a2a;
  background: rgba(255, 255, 255, 0.02);
}

.modal-option-active {
  border-color: rgba(0, 255, 65, 0.3);
  background: rgba(0, 255, 65, 0.04);
}

.modal-option-label {
  font-size: 11px;
  font-weight: 700;
  color: #ccc;
}

.modal-option-desc {
  font-size: 9px;
  color: #555;
  margin-top: 2px;
}

.modal-close {
  margin-top: 12px;
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 600;
  padding: 8px 20px;
  cursor: pointer;
  letter-spacing: 1px;
  align-self: flex-end;
}

.modal-close:hover {
  border-color: #333;
  color: #888;
}
</style>
