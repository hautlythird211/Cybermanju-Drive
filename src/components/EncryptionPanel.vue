<template>
  <div class="encryption-panel">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-shield">[##]</span>
        <h2 class="panel-title">QUANTUM SHIELD</h2>
      </div>
    </div>

    <div class="status-card" :class="{ protected: encryptionStatus?.isEncrypted }">
      <div class="status-top">
        <span class="status-badge" :class="encryptionStatus?.isEncrypted ? 'badge-protected' : 'badge-unprotected'">
          {{ encryptionStatus?.isEncrypted ? 'PROTECTED' : 'UNPROTECTED' }}
        </span>
      </div>

      <div class="status-details" v-if="encryptionStatus?.isEncrypted">
        <div class="algo-name">
          <span>{{ encryptionStatus.algorithm || 'UNKNOWN' }}</span>
          <span class="nist-stars">
            <span v-for="n in (encryptionStatus.nistLevel || 0)" :key="n" class="star filled">*</span>
            <span v-for="n in 5 - (encryptionStatus.nistLevel || 0)" :key="'e' + n" class="star empty">o</span>
          </span>
        </div>
        <div class="status-meta">
          <span class="meta-label">KEY ID:</span>
          <span class="mono">{{ encryptionStatus.keyId || '--' }}</span>
        </div>
        <div class="status-meta" v-if="encryptionStatus.encryptedAt">
          <span class="meta-label">ENCRYPTED:</span>
          <span class="mono">{{ formatDate(encryptionStatus.encryptedAt) }}</span>
        </div>
      </div>
      <div class="status-details" v-else>
        <p class="unprotected-msg">NO QUANTUM-RESISTANT ENCRYPTION ACTIVE. GENERATE A KEYPAIR BELOW.</p>
      </div>

      <div class="nist-viz">
        <span class="nist-label">NIST LEVEL</span>
        <div class="nist-circles">
          <div v-for="n in 5" :key="n" class="nist-circle" :class="{ filled: n <= (encryptionStatus?.nistLevel || 0) }">
            <span class="circle-num">{{ n }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title"><Icon icon="svg-spinners:90-ring" width="12" height="12" class="section-spinner" /> [KEY] GENERATE KEYPAIR</h3>
      <div class="algo-buttons">
        <button v-for="(info, algo) in ENCRYPTION_INFO" :key="algo" class="algo-btn" @click="handleGenerate(algo as EncryptionAlgo)">
          <div class="algo-top">
            <span class="nist-badge">L{{ info.nistLevel }}</span>
          </div>
          <span class="algo-name">{{ info.name }}</span>
          <span class="algo-desc text-muted">{{ info.description }}</span>
        </button>
      </div>
    </div>

    <div class="section" v-if="encryptionKeys.length > 0">
      <h3 class="section-title">[KEY] ACTIVE KEYS ({{ encryptionKeys.length }})</h3>
      <div class="keys-list">
        <div v-for="key in encryptionKeys" :key="key.id" class="key-card">
          <div class="key-header">
            <span class="key-algo">{{ key.algorithmDisplay }}</span>
            <span class="nist-badge small">L{{ key.nistLevel }}</span>
          </div>
          <div class="key-pub-preview mono">{{ key.publicKeyPreview.slice(0, 16) }}..</div>
          <div class="key-date text-muted">{{ formatDate(key.createdAt) }}</div>
        </div>
      </div>
    </div>

    <div class="section" v-if="selectedFile">
      <h3 class="section-title"><Icon icon="svg-spinners:90-ring-with-bg" width="12" height="12" class="section-spinner" /> [LOCK] ENCRYPT SELECTED FILE</h3>
      <p class="selected-file-name">{{ selectedFile.name }}</p>
      <div class="encrypt-actions">
        <select v-model="selectedAlgo" class="encrypt-select">
          <option v-for="(info, algo) in ENCRYPTION_INFO" :key="algo" :value="algo">{{ info.name }} (L{{ info.nistLevel }})</option>
        </select>
        <button class="encrypt-btn" @click="handleEncrypt">[ENC]</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import type { EncryptionAlgo } from '@/types'
import { ENCRYPTION_INFO } from '@/types'

const store = useAppStore()
const emit = defineEmits<{ close: [] }>()

const encryptionStatus = computed(() => store.encryptionStatus)
const encryptionKeys = computed(() => store.encryptionKeys)
const selectedFile = computed(() => store.selectedFile)

const selectedAlgo = ref<EncryptionAlgo>('kyber1024')

function formatDate(iso: string): string {
  if (!iso) return '--'
  const d = new Date(iso)
  return d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

async function handleGenerate(algo: EncryptionAlgo) {
  await store.generateKeypair(algo)
}

async function handleEncrypt() {
  if (!store.selectedFileId) return
  await store.encryptFile(store.selectedFileId, selectedAlgo.value)
}
</script>

<style scoped>
.encryption-panel {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}
.encryption-panel::-webkit-scrollbar { width: 4px; }
.encryption-panel::-webkit-scrollbar-track { background: transparent; }
.encryption-panel::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 2px; }

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-shield {
  font-size: 14px;
  color: var(--text-accent);
}

.panel-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--text-primary);
  margin: 0;
  font-family: var(--font-mono);
}

.status-card {
  border: 1px solid var(--border-glass);
  padding: 16px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-lg);
}

.status-card.protected {
  border-color: var(--border-accent);
}

.status-top {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}

.status-badge {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--accent);
  font-family: var(--font-mono);
}

.badge-protected {
  background: var(--accent-dim);
  color: var(--text-accent);
}

.badge-unprotected {
  background: transparent;
  color: var(--text-muted);
  border-color: var(--border-medium);
}

.status-details {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.algo-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 12px;
  color: var(--text-primary);
}

.nist-stars {
  font-size: 12px;
}

.star.filled { color: var(--text-accent); }
.star.empty { color: var(--text-muted); opacity: 0.3; }

.status-meta {
  font-size: 10px;
  display: flex;
  gap: 6px;
  color: var(--text-secondary);
}

.meta-label {
  color: var(--text-muted);
  min-width: 50px;
  font-family: var(--font-mono);
}

.unprotected-msg {
  font-size: 11px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
}

.nist-viz {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-glass);
  display: flex;
  align-items: center;
  gap: 12px;
}

.nist-label {
  font-size: 9px;
  letter-spacing: 1px;
  color: var(--text-muted);
  white-space: nowrap;
  font-family: var(--font-mono);
}

.nist-circles {
  display: flex;
  gap: 6px;
}

.nist-circle {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-medium);
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
}

.nist-circle.filled {
  border-color: var(--accent);
  background: var(--accent-dim);
}

.nist-circle.filled .circle-num {
  color: var(--text-accent);
}

.circle-num {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-glass);
  font-family: var(--font-mono);
}

.algo-buttons {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.algo-btn {
  background: var(--bg-glass-light);
  border: 1px solid var(--border-glass);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  padding: 10px 12px;
  cursor: pointer;
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.algo-btn:hover {
  background: var(--accent-dim);
  border-color: var(--border-accent);
}

.algo-btn:hover .algo-desc { color: var(--text-accent) !important; }

.algo-top {
  display: flex;
  align-items: center;
  gap: 8px;
}

.nist-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border: 1px solid var(--accent);
  color: var(--text-accent);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
}

.nist-badge.small { font-size: 8px; }

.algo-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.algo-desc {
  font-size: 10px;
  line-height: 1.4;
  color: var(--text-muted);
}

.keys-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.key-card {
  border: 1px solid var(--border-glass);
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-md);
}

.key-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.key-algo {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.key-pub-preview {
  font-size: 10px;
  color: var(--text-muted);
  background: var(--bg-surface);
  padding: 4px 8px;
  word-break: break-all;
  border-radius: var(--radius-sm);
}

.key-date {
  font-size: 10px;
  color: var(--text-muted);
}

.selected-file-name {
  font-size: 11px;
  color: var(--text-primary);
  background: var(--bg-elevated);
  padding: 6px 10px;
  border: 1px solid var(--border-subtle);
  word-break: break-all;
  margin: 0;
  border-radius: var(--radius-sm);
}

.encrypt-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.encrypt-select {
  flex: 1;
  background: var(--bg-surface);
  color: var(--text-primary);
  border: 1px solid var(--border-medium);
  padding: 7px 10px;
  font-size: 11px;
  font-family: var(--font-mono);
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.encrypt-btn {
  background: var(--accent);
  color: var(--text-inverse);
  border: 1px solid var(--accent);
  padding: 7px 14px;
  font-size: 10px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--font-mono);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.encrypt-btn:hover {
  background: #00cc35;
  border-color: #00cc35;
}

.mono { font-family: var(--font-mono); }
.text-muted { color: var(--text-muted) !important; }
</style>
