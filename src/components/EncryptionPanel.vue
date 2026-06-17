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
  width: 400px;
  height: 100%;
  background: #0a0a0a;
  border-left: 2px solid #00ff41;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  font-family: 'Courier New', monospace;
  color: #00ff41;
}

.encryption-panel::-webkit-scrollbar { width: 4px; }
.encryption-panel::-webkit-scrollbar-track { background: #0a0a0a; }
.encryption-panel::-webkit-scrollbar-thumb { background: #00ff41; }

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 10px;
  border-bottom: 2px solid #00ff41;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-shield {
  font-family: 'Courier New', monospace;
  font-size: 16px;
  color: #00ff41;
}

.panel-title {
  font-size: 14px;
  font-weight: 800;
  letter-spacing: 1px;
  color: #00ff41;
  margin: 0;
}

.status-card {
  border: 2px solid #00ff41;
  padding: 12px;
  background: #0a0a0a;
}

.status-card.protected {
  border-width: 3px;
}

.status-top {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.status-badge {
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 1px;
  padding: 3px 8px;
  border: 2px solid #00ff41;
  color: #00ff41;
}

.badge-protected {
  background: #00ff41;
  color: #0a0a0a;
}

.badge-unprotected {
  background: #0a0a0a;
  color: #00ff41;
}

.status-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.algo-name {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 700;
  font-size: 12px;
}

.nist-stars {
  font-size: 11px;
}

.star.filled { color: #00ff41; }
.star.empty { color: rgba(0, 255, 65, 0.3); }

.status-meta {
  font-size: 10px;
  display: flex;
  gap: 4px;
  color: rgba(0, 255, 65, 0.7);
}

.meta-label {
  color: rgba(0, 255, 65, 0.5);
  min-width: 50px;
}

.unprotected-msg {
  font-size: 11px;
  color: rgba(0, 255, 65, 0.7);
  margin: 0;
}

.nist-viz {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 2px solid rgba(0, 255, 65, 0.3);
  display: flex;
  align-items: center;
  gap: 10px;
}

.nist-label {
  font-size: 9px;
  letter-spacing: 1px;
  color: rgba(0, 255, 65, 0.5);
  white-space: nowrap;
}

.nist-circles {
  display: flex;
  gap: 4px;
}

.nist-circle {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(0, 255, 65, 0.3);
  background: #0a0a0a;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nist-circle.filled {
  border-color: #00ff41;
  background: #00ff41;
}

.nist-circle.filled .circle-num {
  color: #0a0a0a;
}

.circle-num {
  font-size: 10px;
  font-weight: 700;
  color: rgba(0, 255, 65, 0.5);
}

.section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgba(0, 255, 65, 0.6);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  padding-bottom: 4px;
  border-bottom: 2px solid rgba(0, 255, 65, 0.2);
}

.algo-buttons {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.algo-btn {
  background: #0a0a0a;
  border: 2px solid #00ff41;
  padding: 8px 10px;
  cursor: pointer;
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 3px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
}

.algo-btn:hover {
  background: #00ff41;
  color: #0a0a0a;
}

.algo-btn:hover .algo-desc { color: #0a0a0a !important; }

.algo-top {
  display: flex;
  align-items: center;
  gap: 6px;
}

.nist-badge {
  font-size: 9px;
  font-weight: 800;
  padding: 1px 4px;
  border: 1px solid #00ff41;
  color: #00ff41;
}

.nist-badge.small { font-size: 8px; }

.algo-name {
  font-size: 11px;
  font-weight: 700;
}

.algo-desc {
  font-size: 10px;
  line-height: 1.3;
}

.keys-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.key-card {
  border: 2px solid #00ff41;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  background: #0a0a0a;
}

.key-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.key-algo {
  font-size: 11px;
  font-weight: 700;
}

.key-pub-preview {
  font-size: 10px;
  color: rgba(0, 255, 65, 0.5);
  background: rgba(0, 255, 65, 0.05);
  padding: 3px 6px;
  word-break: break-all;
}

.key-date {
  font-size: 10px;
}

.selected-file-name {
  font-size: 11px;
  color: #00ff41;
  background: rgba(0, 255, 65, 0.05);
  padding: 4px 8px;
  border: 2px solid rgba(0, 255, 65, 0.3);
  word-break: break-all;
  margin: 0;
}

.encrypt-actions {
  display: flex;
  gap: 6px;
}

.encrypt-select {
  flex: 1;
  background: #0a0a0a;
  color: #00ff41;
  border: 2px solid #00ff41;
  padding: 6px 8px;
  font-size: 10px;
  font-family: 'Courier New', monospace;
  cursor: pointer;
}

.encrypt-btn {
  background: #00ff41;
  color: #0a0a0a;
  border: 2px solid #00ff41;
  padding: 6px 12px;
  font-size: 10px;
  font-weight: 800;
  cursor: pointer;
  font-family: 'Courier New', monospace;
}

.encrypt-btn:hover {
  background: #0a0a0a;
  color: #00ff41;
}

.mono { font-family: 'Courier New', monospace; }
.text-muted { color: rgba(0, 255, 65, 0.5) !important; }
</style>
