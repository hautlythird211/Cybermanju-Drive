<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { kvGet, kvSet } from '@/wasm/storage'
import { useAppStore } from '@/stores/app'
import ImportDialog from '@/components/ImportDialog.vue'
import type { OAuthProvider, OAuthToken } from '@/wasm'
import type { SyncConfig } from '@/types'

const store = useAppStore()

const emit = defineEmits<{ (e: 'complete'): void }>()

const step = ref(0)
const steps = ['WELCOME', 'ACCOUNTS', 'COLLECTIONS', 'GROUPS', 'COMPLETE']
const totalSteps = steps.length

// ── Step 1: Welcome ──

// ── Step 2: OAuth Accounts ──
const PROVIDER_ICONS: Record<string, string> = {
  google: 'logos:google-icon',
  github: 'logos:github-icon',
  gitlab: 'logos:gitlab',
  telegram: 'logos:telegram',
  mega: 'logos:mega',
}

const AVAILABLE_PROVIDERS = [
  { id: 'google', label: 'Google (Drive + Photos)', color: '#4285F4', oauth: true },
  { id: 'github', label: 'GitHub', color: '#333', oauth: true },
  { id: 'gitlab', label: 'GitLab', color: '#FC6D26', oauth: true },
  { id: 'telegram', label: 'Telegram', color: '#0088CC', oauth: true },
  { id: 'mega', label: 'Mega.nz', color: '#D9272E', oauth: true },
]

interface ProviderAccount {
  providerId: string
  label: string
  token?: string
  email?: string
  password?: string
}

const accounts = ref<ProviderAccount[]>([])
const isConnecting = ref<string | null>(null)
const accountError = ref('')

// ── Mega login modal ──
const showMegaModal = ref(false)
const megaEmail = ref('')
const megaPassword = ref('')
const megaLabel = ref('')
const megaVerifying = ref(false)
const megaVerifyError = ref('')
const mega2FACode = ref('')

// ── Client ID fallback form ──
const showClientIdForm = ref<string | null>(null)
const clientIdInput = ref('')

// ── Import dialog state ──
const importVisible = ref(false)
const importBackend = ref('')
const importToken = ref('')
const importLabel = ref('')

function showImportDialog(backend: string, token: OAuthToken, label: string) {
  importBackend.value = backend
  importToken.value = token.accessToken
  importLabel.value = label
  importVisible.value = true
}

function onImportComplete() {
  importVisible.value = false
}

function providerIcon(id: string): string {
  return PROVIDER_ICONS[id] || 'mdi:cloud-outline'
}

function isConnected(id: string): boolean {
  return accounts.value.some(a => a.providerId === id)
}

function getProviderLabel(id: string): string {
  return AVAILABLE_PROVIDERS.find(p => p.id === id)?.label || id
}

async function connectOAuth(pid: string) {
  if (isConnecting.value) return

  if (pid === 'mega') {
    openMegaModal()
    return
  }

  if (pid === 'google') {
    await connectGoogle()
    return
  }

  const provider = pid as OAuthProvider
  if (isConnected(provider)) return

  const { oauth } = await import('@/wasm')
  oauth.loadClientIdsFromEnv()
  const clientId = oauth.getProviderClientId(provider)

  if (!clientId) {
    showClientIdForm.value = provider
    return
  }

  isConnecting.value = provider
  accountError.value = ''
  try {
    const { initWasm } = await import('@/wasm')
    await initWasm()

    const existingToken = await oauth.loadTokenFromStorage(provider)
    let token
    if (existingToken) {
      token = await oauth.getValidToken(existingToken)
      if (token) oauth.saveTokenToStorage(token)
    }
    if (!token) {
      token = await oauth.authenticateWithPopup(provider)
      oauth.saveTokenToStorage(token)
    }

    const data = await import('@/wasm/data')
    const account = await data.upsertOAuthAccount(provider, token)

    const label = account.name || getProviderLabel(provider)
    accounts.value.push({ providerId: provider, label, token: token.accessToken })
    showImportDialog(provider, token, label)
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    if (msg.includes('Popup blocked')) {
      accountError.value = `${getProviderLabel(provider)}: POPUP BLOCKED — allow popups for this site and try again`
    } else if (msg.includes('timed out') || msg.includes('closed by the user')) {
      accountError.value = `${getProviderLabel(provider)}: AUTHORIZATION ${msg.includes('timed out') ? 'TIMED OUT' : 'CANCELLED'}`
    } else if (msg.includes('Client ID not configured')) {
      showClientIdForm.value = provider
    } else {
      accountError.value = `${getProviderLabel(provider)}: ${msg}`
    }
  } finally {
    isConnecting.value = null
  }
}

async function connectGoogle() {
  if (isConnected('google')) return
  isConnecting.value = 'google'
  accountError.value = ''
  try {
    const { oauth, initWasm } = await import('@/wasm')
    await initWasm()
    oauth.loadClientIdsFromEnv()

    const clientId = oauth.getProviderClientId('googleDrive')
    if (!clientId) {
      showClientIdForm.value = 'google'
      return
    }

    const GOOGLE_SCOPES = [
      'https://www.googleapis.com/auth/drive.file',
      'https://www.googleapis.com/auth/photoslibrary.appendonly',
    ]

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

    const label = driveAccount.name || 'Google'
    accounts.value.push({ providerId: 'google', label, token: token.accessToken })
    showImportDialog('google', token, 'Google (Drive + Photos)')
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    if (msg.includes('Popup blocked')) {
      accountError.value = 'GOOGLE: POPUP BLOCKED — allow popups for this site and try again'
    } else if (msg.includes('timed out') || msg.includes('closed by the user')) {
      accountError.value = `GOOGLE: AUTHORIZATION ${msg.includes('timed out') ? 'TIMED OUT' : 'CANCELLED'}`
    } else if (msg.includes('Client ID not configured')) {
      showClientIdForm.value = 'google'
    } else {
      accountError.value = `GOOGLE: ${msg}`
    }
  } finally {
    isConnecting.value = null
  }
}

async function saveClientId() {
  if (!showClientIdForm.value || !clientIdInput.value.trim()) return
  const provider = showClientIdForm.value as OAuthProvider
  const { oauth } = await import('@/wasm')
  oauth.setProviderClientId(provider, clientIdInput.value.trim())
  showClientIdForm.value = null
  clientIdInput.value = ''
  await connectOAuth(provider)
}

function cancelClientIdForm() {
  showClientIdForm.value = null
  clientIdInput.value = ''
}

function openMegaModal() {
  megaEmail.value = ''
  megaPassword.value = ''
  megaLabel.value = 'Mega'
  megaVerifyError.value = ''
  mega2FACode.value = ''
  showMegaModal.value = true
}

function closeMegaModal() {
  showMegaModal.value = false
  megaVerifying.value = false
  megaVerifyError.value = ''
  mega2FACode.value = ''
}

async function verifyAndConnectMega() {
  megaVerifyError.value = ''
  if (!megaEmail.value.trim() || !megaPassword.value.trim()) {
    megaVerifyError.value = 'EMAIL AND PASSWORD ARE REQUIRED'
    return
  }

  megaVerifying.value = true
  isConnecting.value = 'mega'
  try {
    const label = megaLabel.value.trim() || 'Mega'
    const token = `${megaEmail.value.trim()}|${megaPassword.value}`

    const testConfig: SyncConfig & { secondFactorCode?: string } = {
      id: '',
      backendType: 'mega',
      enabled: true,
      name: label,
      basePath: '/',
      token,
      secondFactorCode: mega2FACode.value.trim() || undefined,
      autoSync: false,
      compressBeforeUpload: false,
      createPreviews: false,
      deleteRawAfterSync: false,
      maxConcurrentUploads: 1,
    }

    await store.testSyncConnection(testConfig)

    accounts.value.push({
      providerId: 'mega',
      label,
      email: megaEmail.value.trim(),
      password: megaPassword.value,
      token,
    })
    closeMegaModal()
    showImportDialog('mega', { accessToken: token } as OAuthToken, label)
  } catch (e) {
    megaVerifyError.value = e instanceof Error ? e.message : 'CONNECTION FAILED'
  } finally {
    megaVerifying.value = false
    isConnecting.value = null
  }
}

function removeAccount(idx: number) {
  accounts.value.splice(idx, 1)
}

// ── Step 3: Quick Collections ──
const COLLECTION_PRESETS = [
  { name: 'PHOTOS', type: 'media', color: '#FF6B6B', selected: true },
  { name: 'DOCUMENTS', type: 'document', color: '#4ECDC4', selected: true },
  { name: 'MUSIC', type: 'media', color: '#45B7D1', selected: false },
  { name: 'VIDEOS', type: 'media', color: '#96CEB4', selected: false },
  { name: 'DOWNLOADS', type: 'download', color: '#FFEAA7', selected: false },
  { name: 'PROJECTS', type: 'custom', color: '#DDA0DD', selected: false },
]

const collections = ref(COLLECTION_PRESETS.map(c => ({ ...c })))
const customCollectionName = ref('')
const customCollectionColor = ref('#FFFFFF')

function addCustomCollection() {
  if (!customCollectionName.value.trim()) return
  collections.value.push({
    name: customCollectionName.value.trim().toUpperCase(),
    type: 'custom',
    color: customCollectionColor.value,
    selected: true,
  })
  customCollectionName.value = ''
}

function toggleCollection(idx: number) {
  collections.value[idx].selected = !collections.value[idx].selected
}

// ── Step 4: Groups ──
const GROUPS_PRESETS = [
  { name: 'FAMILY', selected: true },
  { name: 'FRIENDS', selected: true },
  { name: 'WORK', selected: false },
  { name: 'TRAVEL', selected: false },
]
const groups = ref(GROUPS_PRESETS.map(g => ({ ...g })))
const customGroupName = ref('')

function addCustomGroup() {
  if (!customGroupName.value.trim()) return
  groups.value.push({ name: customGroupName.value.trim().toUpperCase(), selected: true })
  customGroupName.value = ''
}

function toggleGroup(idx: number) {
  groups.value[idx].selected = !groups.value[idx].selected
}

// ── Navigation ──
const isCompleting = ref(false)
const completeError = ref('')

function canProceed(): boolean {
  switch (step.value) {
    case 0: return true // welcome — always can proceed
    case 1: return true // accounts are optional — can skip
    case 2: return collections.value.some(c => c.selected)
    case 3: return groups.value.some(g => g.selected)
    case 4: return true
    default: return false
  }
}

async function next() {
  if (step.value < totalSteps - 1) {
    step.value++
  } else {
    await finish()
  }
}

function prev() {
  if (step.value > 0) step.value--
}

async function finish() {
  isCompleting.value = true
  completeError.value = ''
  try {
    const config = {
      accounts: accounts.value.map(a => ({ ...a })),
      collections: collections.value.filter(c => c.selected).map(c => ({
        name: c.name,
        type: c.type,
        color: c.color,
      })),
      groups: groups.value.filter(g => g.selected).map(g => ({
        name: g.name,
      })),
      completedAt: new Date().toISOString(),
    }
    await kvSet('setup_config', config)
    await kvSet('setup_complete', true)

    // Create sync configs for each account
    const errors: string[] = []
    for (const acct of accounts.value) {
      try {
        const backends = acct.providerId === 'google' ? ['googleDrive', 'googlePhotos'] : [acct.providerId]
        for (const backend of backends) {
          const base: Record<string, unknown> = {
            name: `${acct.label} (${backend.replace(/([A-Z])/g, ' $1').trim()})`,
            backendType: backend,
            enabled: true,
            basePath: '/',
            autoSync: false,
            compressBeforeUpload: false,
            maxConcurrentUploads: 1,
          }
          if (backend === 'mega') {
            base.token = `${acct.email}|${acct.password}`
          } else if (acct.token) {
            base.token = acct.token
          }
          await store.createSyncConfig(base as any)
        }
      } catch (e) {
        errors.push(`${acct.label}: ${e}`)
      }
    }
    if (errors.length) {
      completeError.value = `SOME SYNC CONFIGS FAILED:\n${errors.join('\n')}`
    }
    emit('complete')
  } catch (err) {
    completeError.value = `FAILED TO SAVE: ${err}`
    isCompleting.value = false
  }
}

onMounted(async () => {
  const existing = await kvGet<any>('setup_config')
  if (existing?.accounts?.length) {
    accounts.value = existing.accounts
  }
  if (existing?.collections?.length) {
    for (const c of existing.collections) {
      const preset = collections.value.find(p => p.name === c.name)
      if (preset) preset.selected = true
    }
  }
  if (existing?.groups?.length) {
    for (const g of existing.groups) {
      const preset = groups.value.find(p => p.name === g.name)
      if (preset) preset.selected = true
    }
  }
})
</script>

<template>
  <div class="setup-wizard">
    <div class="wizard-container">
      <!-- Header -->
      <div class="wizard-header">
        <div class="wizard-title">CYBERMANJU DRIVE — SETUP</div>
        <div class="wizard-progress">
          <div
            v-for="(s, i) in steps"
            :key="i"
            class="progress-step"
            :class="{ active: i === step, done: i < step }"
            @click="i < step ? step = i : null"
          >
            <div class="step-number">{{ i < step ? '✓' : i + 1 }}</div>
            <div class="step-label">{{ s }}</div>
          </div>
        </div>
      </div>

      <!-- Body -->
      <div class="wizard-body">
        <!-- Step 0: Welcome -->
        <div v-if="step === 0" class="step-content">
          <div class="step-icon">
            <Icon icon="mdi:shield-check-outline" width="48" height="48" />
          </div>
          <div class="step-title">WELCOME TO CYBERMANJU DRIVE</div>
          <div class="step-desc">
            Your post-quantum encrypted file system needs a little configuration before you dive in.
            This wizard will help you set up:
          </div>
          <div class="feature-list">
            <div class="feature-item">
              <Icon icon="mdi:cloud-outline" width="18" height="18" />
              <span>Cloud provider accounts (Google Drive, GitHub, Mega.nz, etc.)</span>
            </div>
            <div class="feature-item">
              <Icon icon="mdi:folder-multiple-outline" width="18" height="18" />
              <span>Quick collections to organise your files</span>
            </div>
            <div class="feature-item">
              <Icon icon="mdi:account-group-outline" width="18" height="18" />
              <span>Groups for sharing and collaboration</span>
            </div>
          </div>
          <div class="step-hint">You can change all settings later.</div>
          <button class="bw-btn bw-btn-inverse welcome-continue" @click="next">
            [ CONTINUE ]
          </button>
        </div>

        <!-- Step 1: Accounts -->
        <div v-if="step === 1" class="step-content">
          <div class="step-icon">
            <Icon icon="mdi:cloud-key-outline" width="48" height="48" />
          </div>
          <div class="step-title">CONFIGURE ACCOUNTS</div>
          <div class="step-desc">
            Add cloud provider accounts for syncing. You can skip this and configure later.
          </div>

          <div class="accounts-list">
            <div v-for="(acct, idx) in accounts" :key="idx" class="account-card">
              <div class="account-provider-icon">
                <Icon :icon="providerIcon(acct.providerId)" width="20" height="20" />
              </div>
              <div class="account-info">
                <div class="account-label">{{ acct.label }}</div>
                <div class="account-provider">{{ acct.providerId }}{{ acct.email ? ' — ' + acct.email : '' }}</div>
              </div>
              <button class="btn-remove" @click="removeAccount(idx)">✕</button>
            </div>
          </div>

          <div class="oauth-connect-grid">
            <div
              v-for="p in AVAILABLE_PROVIDERS"
              :key="p.id"
              class="oauth-connect-card"
              :class="{
                connected: isConnected(p.id),
                connecting: isConnecting === p.id,
                'card-clickable': !isConnected(p.id),
              }"
              :tabindex="!isConnected(p.id) ? 0 : undefined"
              @click="!isConnected(p.id) ? connectOAuth(p.id) : undefined"
              @keydown.enter="!isConnected(p.id) ? connectOAuth(p.id) : undefined"
            >
              <div class="oauth-connect-icon">
                <Icon :icon="providerIcon(p.id)" width="24" height="24" />
              </div>
              <div class="oauth-connect-name">{{ p.label }}</div>
              <div class="oauth-connect-status">
                <span v-if="isConnected(p.id)" class="status-connected">CONNECTED</span>
                <span v-else-if="isConnecting === p.id" class="status-connecting">
                  <Icon icon="svg-spinners:blocks-wave" width="12" height="12" />
                  AUTH...
                </span>
                <span v-else class="status-disconnected">NOT CONNECTED</span>
              </div>
              <button
                v-if="p.oauth && !isConnected(p.id)"
                class="bw-btn bw-btn-inverse connect-btn"
                :disabled="isConnecting !== null"
                @click.stop="connectOAuth(p.id)"
              >
                [ CONNECT ]
              </button>
            </div>
          </div>

          <div v-if="showClientIdForm" class="add-account-form client-id-form">
            <div class="form-row">
              <label>ENTER CLIENT ID FOR {{ getProviderLabel(showClientIdForm) }}</label>
              <input v-model="clientIdInput" class="bw-input" placeholder="OAuth Client ID" />
              <div class="hint-text">Set via <code>.env</code> or enter here (session only)</div>
            </div>
            <div class="form-actions">
              <button class="bw-btn" @click="cancelClientIdForm">CANCEL</button>
              <button class="bw-btn bw-btn-inverse" :disabled="!clientIdInput.trim()" @click="saveClientId">SAVE</button>
            </div>
          </div>

          <div v-if="accountError && !showClientIdForm" class="form-error">{{ accountError }}</div>
        </div>

        <!-- Step 2: Collections -->
        <div v-if="step === 2" class="step-content">
          <div class="step-icon">
            <Icon icon="mdi:folder-multiple-outline" width="48" height="48" />
          </div>
          <div class="step-title">QUICK COLLECTIONS</div>
          <div class="step-desc">
            Select collections to create. You can add more later.
          </div>

          <div class="preset-grid">
            <div
              v-for="(col, idx) in collections"
              :key="idx"
              class="preset-card"
              :class="{ selected: col.selected }"
              @click="toggleCollection(idx)"
            >
              <div class="preset-color" :style="{ background: col.color }" />
              <div class="preset-name">{{ col.name }}</div>
              <div class="preset-type">{{ col.type }}</div>
              <div class="preset-check">{{ col.selected ? '✓' : '' }}</div>
            </div>
          </div>

          <div class="custom-add">
            <input v-model="customCollectionName" class="bw-input" placeholder="CUSTOM COLLECTION NAME" @keyup.enter="addCustomCollection" />
            <input v-model="customCollectionColor" class="bw-input color-input" type="color" />
            <button class="bw-btn" @click="addCustomCollection">ADD</button>
          </div>
        </div>

        <!-- Step 3: Groups -->
        <div v-if="step === 3" class="step-content">
          <div class="step-icon">
            <Icon icon="mdi:account-group-outline" width="48" height="48" />
          </div>
          <div class="step-title">GROUPS</div>
          <div class="step-desc">
            Create groups for organising shared files and collaboration.
          </div>

          <div class="preset-grid">
            <div
              v-for="(g, idx) in groups"
              :key="idx"
              class="preset-card"
              :class="{ selected: g.selected }"
              @click="toggleGroup(idx)"
            >
              <div class="preset-icon">
                <Icon icon="mdi:account-group-outline" width="24" height="24" />
              </div>
              <div class="preset-name">{{ g.name }}</div>
              <div class="preset-check">{{ g.selected ? '✓' : '' }}</div>
            </div>
          </div>

          <div class="custom-add">
            <input v-model="customGroupName" class="bw-input" placeholder="CUSTOM GROUP NAME" @keyup.enter="addCustomGroup" />
            <button class="bw-btn" @click="addCustomGroup">ADD</button>
          </div>
        </div>

        <!-- Step 4: Complete -->
        <div v-if="step === 4" class="step-content">
          <div class="step-icon">
            <Icon icon="mdi:check-circle-outline" width="48" height="48" />
          </div>
          <div class="step-title">SETUP COMPLETE</div>
          <div class="step-desc">
            Your configuration is ready. Here's a summary:
          </div>

          <div class="summary">
            <div class="summary-section">
              <div class="summary-label">ACCOUNTS ({{ accounts.length }})</div>
              <div v-for="a in accounts" :key="a.label" class="summary-item">
                <Icon :icon="providerIcon(a.providerId)" width="14" height="14" />
                {{ a.label }} ({{ a.providerId }})
              </div>
              <div v-if="!accounts.length" class="summary-empty">No accounts configured</div>
            </div>
            <div class="summary-section">
              <div class="summary-label">COLLECTIONS ({{ collections.filter(c => c.selected).length }})</div>
              <div v-for="c in collections.filter(c => c.selected)" :key="c.name" class="summary-item">
                <span class="summary-dot" :style="{ background: c.color }" />
                {{ c.name }}
              </div>
            </div>
            <div class="summary-section">
              <div class="summary-label">GROUPS ({{ groups.filter(g => g.selected).length }})</div>
              <div v-for="g in groups.filter(g => g.selected)" :key="g.name" class="summary-item">
                <Icon icon="mdi:account-group-outline" width="14" height="14" />
                {{ g.name }}
              </div>
            </div>
          </div>

          <div v-if="completeError" class="form-error">{{ completeError }}</div>
          <div v-if="isCompleting" class="completing">
            <Icon icon="svg-spinners:blocks-wave" width="24" height="24" />
            SAVING CONFIGURATION...
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="wizard-footer">
        <div class="footer-left">
          <button v-if="step > 0" class="bw-btn" @click="prev">[ BACK ]</button>
        </div>
        <div class="footer-right">
          <button
            class="bw-btn bw-btn-inverse"
            :disabled="!canProceed() || isCompleting"
            @click="next"
          >
            {{ step < totalSteps - 1 ? '[ NEXT ]' : '[ LAUNCH ]' }}
          </button>
        </div>
      </div>
    </div>
  </div>
  <!-- Mega login modal -->
  <Teleport to="body">
    <div v-if="showMegaModal" class="mega-modal-overlay" @click.self="closeMegaModal">
      <div class="mega-modal">
        <div class="mega-modal-header">
          <Icon icon="logos:mega" width="28" height="28" />
          <span>CONNECT MEGA.NZ</span>
        </div>
        <div class="mega-modal-body">
          <div class="form-row">
            <label>EMAIL</label>
            <input
              v-model="megaEmail"
              class="bw-input"
              placeholder="Mega.nz account email"
              autocomplete="email"
              @keyup.enter="verifyAndConnectMega"
            />
          </div>
          <div class="form-row">
            <label>PASSWORD</label>
            <input
              v-model="megaPassword"
              class="bw-input"
              type="password"
              placeholder="Mega.nz password"
              autocomplete="current-password"
              @keyup.enter="verifyAndConnectMega"
            />
          </div>
          <div class="form-row">
            <label>LABEL (OPTIONAL)</label>
            <input v-model="megaLabel" class="bw-input" placeholder="e.g. My Mega" />
          </div>
          <div class="form-row">
            <label>2FA CODE (OPTIONAL)</label>
            <input
              v-model="mega2FACode"
              class="bw-input"
              placeholder="Six-digit authenticator code"
              autocomplete="one-time-code"
              inputmode="numeric"
              maxlength="6"
              @keyup.enter="verifyAndConnectMega"
            />
          </div>
          <div v-if="megaVerifyError" class="mega-modal-error">{{ megaVerifyError }}</div>
          <div v-if="megaVerifying" class="mega-modal-verifying">
            <Icon icon="svg-spinners:blocks-wave" width="16" height="16" />
            VERIFYING...
          </div>
        </div>
        <div class="mega-modal-footer">
          <button class="bw-btn" :disabled="megaVerifying" @click="closeMegaModal">CANCEL</button>
          <button
            class="bw-btn bw-btn-inverse"
            :disabled="megaVerifying || !megaEmail.trim() || !megaPassword.trim()"
            @click="verifyAndConnectMega"
          >
            {{ megaVerifying ? 'VERIFYING...' : 'VERIFY & CONNECT' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
  <ImportDialog
    :visible="importVisible"
    :backendType="importBackend"
    :token="importToken"
    :label="importLabel"
    @close="onImportComplete"
    @import="onImportComplete"
  />
</template>

<style scoped>
.setup-wizard {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  font-family: 'Courier New', 'Fira Code', monospace;
  z-index: 1000;
  overflow: auto;
}

.wizard-container {
  width: 600px;
  max-width: 96vw;
  max-height: 90vh;
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 60px rgba(0, 255, 65, 0.03);
  overflow: hidden;
}

/* ── Header ── */
.wizard-header {
  padding: 20px 24px 12px;
  border-bottom: 1px solid #1a1a1a;
}

.wizard-title {
  font-size: 14px;
  font-weight: 800;
  color: #00ff41;
  letter-spacing: 2px;
  margin-bottom: 16px;
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.15);
}

.wizard-progress {
  display: flex;
  gap: 4px;
}

.progress-step {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  cursor: default;
  padding: 4px 0;
  opacity: 0.4;
  transition: opacity 0.2s;
}

.progress-step.active { opacity: 1; }
.progress-step.done { opacity: 0.7; cursor: pointer; }
.progress-step.done:hover { opacity: 1; }

.step-number {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 1px solid #333;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  color: #666;
  transition: all 0.2s;
}

.progress-step.active .step-number {
  border-color: #00ff41;
  color: #00ff41;
  box-shadow: 0 0 8px rgba(0, 255, 65, 0.2);
}

.progress-step.done .step-number {
  border-color: #00ff41;
  background: rgba(0, 255, 65, 0.1);
  color: #00ff41;
}

.step-label {
  font-size: 8px;
  color: #666;
  letter-spacing: 1px;
}

.progress-step.active .step-label { color: #00ff41; }
.progress-step.done .step-label { color: #00ff41; }

/* ── Body ── */
.wizard-body {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
  min-height: 300px;
}

.step-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
}

.step-icon {
  color: #00ff41;
  opacity: 0.8;
  margin-bottom: 4px;
}

.step-title {
  font-size: 16px;
  font-weight: 800;
  color: #e0e0e0;
  letter-spacing: 2px;
}

.step-desc {
  font-size: 11px;
  color: #888;
  line-height: 1.6;
  max-width: 440px;
}

.step-hint {
  font-size: 10px;
  color: #555;
  font-style: italic;
  margin-top: 8px;
}

.feature-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
  text-align: left;
  width: 100%;
  max-width: 360px;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 11px;
  color: #aaa;
  padding: 6px 10px;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
}

.feature-item svg { flex-shrink: 0; color: #00ff41; }

.welcome-continue {
  margin-top: 12px;
  padding: 8px 32px;
  font-size: 11px;
}

/* ── Accounts ── */
.accounts-list {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 8px 0;
}

.account-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  background: #0d0d0d;
}

.account-provider-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  background: #111;
}

.account-info {
  flex: 1;
  text-align: left;
}

.account-label {
  font-size: 11px;
  font-weight: 700;
  color: #e0e0e0;
}

.account-provider {
  font-size: 9px;
  color: #666;
}

.btn-remove {
  background: none;
  border: none;
  color: #ff5f57;
  cursor: pointer;
  font-size: 12px;
  padding: 4px;
  opacity: 0.6;
  transition: opacity 0.15s;
}

.btn-remove:hover { opacity: 1; }

/* ── OAuth Connect Grid ── */
.oauth-connect-grid {
  width: 100%;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 8px;
  margin: 8px 0;
}

.oauth-connect-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px 8px;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  background: #0d0d0d;
  transition: all 0.15s;
}

.oauth-connect-card.connected {
  border-color: #00ff41;
  background: rgba(0, 255, 65, 0.03);
}

.oauth-connect-card.connecting {
  border-color: #febc2e;
  background: rgba(254, 188, 46, 0.03);
}

.oauth-connect-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  background: #111;
}

.oauth-connect-card.card-clickable {
  cursor: pointer;
}

.oauth-connect-card.card-clickable:hover {
  border-color: #333;
  background: #111;
}

.oauth-connect-card.card-clickable:focus-visible {
  outline: none;
  border-color: #00ff41;
  box-shadow: 0 0 12px rgba(0, 255, 65, 0.15);
}

.oauth-connect-name {
  font-size: 10px;
  font-weight: 700;
  color: #e0e0e0;
  letter-spacing: 1px;
}

.oauth-connect-status {
  font-size: 8px;
  letter-spacing: 1px;
}

.status-connected {
  color: #00ff41;
  font-weight: 700;
}

.status-connecting {
  color: #febc2e;
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-disconnected {
  color: #555;
}

.connect-btn {
  margin-top: 4px;
  padding: 4px 16px;
  font-size: 9px;
}

.client-id-form {
  border-color: #febc2e;
}

.hint-text {
  font-size: 9px;
  color: #555;
  font-style: italic;
  margin-top: 2px;
}

.hint-text code {
  color: #888;
  background: #111;
  padding: 0 3px;
  border-radius: 2px;
}

.add-account-trigger {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  color: #00ff41;
  cursor: pointer;
  padding: 8px;
  margin-top: 4px;
  opacity: 0.7;
  transition: opacity 0.15s;
}

.add-account-trigger:hover { opacity: 1; }

.add-account-form {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  background: #0d0d0d;
  margin-top: 8px;
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  text-align: left;
}

.form-row label {
  font-size: 9px;
  color: #666;
  letter-spacing: 1px;
  font-weight: 700;
}

.bw-input, .bw-select {
  background: #111;
  border: 1px solid #2a2a2a;
  border-radius: 4px;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  padding: 6px 10px;
  outline: none;
  transition: border-color 0.15s;
}

.bw-input:focus, .bw-select:focus {
  border-color: #00ff41;
  box-shadow: 0 0 8px rgba(0, 255, 65, 0.1);
}

.bw-select option {
  background: #111;
  color: #e0e0e0;
}

.form-error {
  font-size: 10px;
  color: #ff5f57;
  padding: 4px 0;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.bw-btn {
  background: transparent;
  border: 1px solid #333;
  border-radius: 4px;
  color: #888;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 6px 14px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.bw-btn:hover {
  border-color: #555;
  color: #e0e0e0;
}

.bw-btn-inverse {
  border-color: #00ff41;
  color: #00ff41;
  text-shadow: 0 0 4px rgba(0, 255, 65, 0.2);
}

.bw-btn-inverse:hover {
  background: rgba(0, 255, 65, 0.1);
  box-shadow: 0 0 12px rgba(0, 255, 65, 0.15);
}

.bw-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

/* ── Collections & Groups ── */
.preset-grid {
  width: 100%;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
  margin: 8px 0;
}

.preset-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px 8px;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
  background: #0d0d0d;
  position: relative;
}

.preset-card:hover {
  border-color: #333;
  background: #111;
}

.preset-card.selected {
  border-color: #00ff41;
  background: rgba(0, 255, 65, 0.03);
}

.preset-color {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: 2px solid #1a1a1a;
}

.preset-icon { opacity: 0.7; }

.preset-name {
  font-size: 10px;
  font-weight: 700;
  color: #e0e0e0;
  letter-spacing: 1px;
}

.preset-type {
  font-size: 8px;
  color: #555;
}

.preset-check {
  position: absolute;
  top: 6px;
  right: 8px;
  font-size: 10px;
  color: #00ff41;
  font-weight: 700;
}

.custom-add {
  display: flex;
  gap: 6px;
  width: 100%;
  margin-top: 8px;
}

.custom-add .bw-input { flex: 1; }

.color-input {
  width: 40px !important;
  padding: 2px !important;
  cursor: pointer;
}

/* ── Summary ── */
.summary {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin: 8px 0;
  text-align: left;
}

.summary-section {
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  padding: 10px 12px;
}

.summary-label {
  font-size: 9px;
  color: #00ff41;
  letter-spacing: 1px;
  font-weight: 700;
  margin-bottom: 6px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  color: #aaa;
  padding: 2px 0;
}

.summary-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.summary-empty {
  font-size: 10px;
  color: #555;
  font-style: italic;
}

.completing {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: #00ff41;
  margin-top: 8px;
}

/* ── Footer ── */
.wizard-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 24px 20px;
  border-top: 1px solid #1a1a1a;
}

.footer-left, .footer-right {
  display: flex;
  gap: 8px;
}

@media (max-width: 640px) {
  .wizard-container { border-radius: 0; max-height: 100vh; }
  .wizard-body { padding: 16px; }
  .preset-grid { grid-template-columns: repeat(auto-fill, minmax(100px, 1fr)); }
  .step-title { font-size: 13px; }
}

/* ── Mega modal (teleported to body) ── */
.mega-modal-overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.8);
  z-index: 9999;
}

.mega-modal {
  width: 380px;
  max-width: 92vw;
  background: #0a0a0a;
  border: 1px solid #D9272E;
  border-radius: 12px;
  box-shadow: 0 0 60px rgba(217, 39, 46, 0.08);
  overflow: hidden;
}

.mega-modal-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 20px 12px;
  font-size: 13px;
  font-weight: 800;
  color: #e0e0e0;
  letter-spacing: 1px;
  border-bottom: 1px solid #1a1a1a;
}

.mega-modal-body {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mega-modal-error {
  font-size: 10px;
  color: #ff5f57;
  padding: 6px 8px;
  border: 1px solid rgba(255, 95, 87, 0.2);
  border-radius: 4px;
  background: rgba(255, 95, 87, 0.05);
  text-align: center;
}

.mega-modal-verifying {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 10px;
  color: #febc2e;
  letter-spacing: 1px;
}

.mega-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px 16px;
  border-top: 1px solid #1a1a1a;
}
</style>
