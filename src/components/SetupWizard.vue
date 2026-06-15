<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { kvGet, kvSet } from '@/wasm/storage'

const emit = defineEmits<{ (e: 'complete'): void }>()

const step = ref(0)
const steps = ['WELCOME', 'ACCOUNTS', 'COLLECTIONS', 'GROUPS', 'COMPLETE']
const totalSteps = steps.length

// ── Step 1: Welcome ──
const welcomeDone = ref(false)

// ── Step 2: OAuth Accounts ──
const PROVIDER_ICONS: Record<string, string> = {
  googleDrive: 'logos:google-drive',
  googlePhotos: 'logos:google-photos',
  github: 'logos:github-icon',
  gitlab: 'logos:gitlab',
  telegram: 'logos:telegram',
}

const AVAILABLE_PROVIDERS = [
  { id: 'googleDrive', label: 'Google Drive', color: '#4285F4' },
  { id: 'googlePhotos', label: 'Google Photos', color: '#FBBC04' },
  { id: 'github', label: 'GitHub', color: '#333' },
  { id: 'gitlab', label: 'GitLab', color: '#FC6D26' },
  { id: 'telegram', label: 'Telegram', color: '#0088CC' },
]

interface ProviderAccount {
  providerId: string
  label: string
  clientId: string
  clientSecret: string
  email: string
}

const accounts = ref<ProviderAccount[]>([])
const showAddAccount = ref(false)
const newAccount = ref<ProviderAccount>({
  providerId: 'googleDrive',
  label: '',
  clientId: '',
  clientSecret: '',
  email: '',
})
const accountError = ref('')

function providerIcon(id: string): string {
  return PROVIDER_ICONS[id] || 'mdi:cloud-outline'
}

function addAccount() {
  accountError.value = ''
  if (!newAccount.value.label.trim()) {
    accountError.value = 'LABEL IS REQUIRED'
    return
  }
  if (!newAccount.value.clientId.trim()) {
    accountError.value = 'CLIENT ID IS REQUIRED'
    return
  }
  accounts.value.push({ ...newAccount.value })
  newAccount.value = { providerId: 'googleDrive', label: '', clientId: '', clientSecret: '', email: '' }
  showAddAccount.value = false
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
    case 0: return welcomeDone.value
    case 1: return true // accounts are optional — can skip
    case 2: return collections.value.some(c => c.selected)
    case 3: return groups.value.some(g => g.selected)
    case 4: return true
    default: return false
  }
}

function markWelcomeDone() {
  welcomeDone.value = true
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
      accounts: accounts.value,
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
              <span>Cloud provider accounts (Google Drive, GitHub, etc.)</span>
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
          <button class="bw-btn bw-btn-inverse welcome-continue" @click="markWelcomeDone">
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
                <div class="account-provider">{{ acct.providerId }} — {{ acct.email || 'no email' }}</div>
              </div>
              <button class="btn-remove" @click="removeAccount(idx)">✕</button>
            </div>
          </div>

          <div v-if="!showAddAccount" class="add-account-trigger" @click="showAddAccount = true">
            <Icon icon="mdi:plus-circle-outline" width="16" height="16" />
            ADD ACCOUNT
          </div>

          <div v-if="showAddAccount" class="add-account-form">
            <div class="form-row">
              <label>PROVIDER</label>
              <select v-model="newAccount.providerId" class="bw-select">
                <option v-for="p in AVAILABLE_PROVIDERS" :key="p.id" :value="p.id">
                  {{ p.label }}
                </option>
                <option value="custom">CUSTOM</option>
              </select>
            </div>
            <div class="form-row">
              <label>LABEL</label>
              <input v-model="newAccount.label" class="bw-input" placeholder="e.g. Work Google" />
            </div>
            <div class="form-row">
              <label>CLIENT ID</label>
              <input v-model="newAccount.clientId" class="bw-input" placeholder="OAuth Client ID" />
            </div>
            <div class="form-row">
              <label>CLIENT SECRET</label>
              <input v-model="newAccount.clientSecret" class="bw-input" type="password" placeholder="OAuth Client Secret" />
            </div>
            <div class="form-row">
              <label>EMAIL</label>
              <input v-model="newAccount.email" class="bw-input" placeholder="account email (optional)" />
            </div>
            <div v-if="accountError" class="form-error">{{ accountError }}</div>
            <div class="form-actions">
              <button class="bw-btn" @click="showAddAccount = false">CANCEL</button>
              <button class="bw-btn bw-btn-inverse" @click="addAccount">ADD</button>
            </div>
          </div>
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
</style>
