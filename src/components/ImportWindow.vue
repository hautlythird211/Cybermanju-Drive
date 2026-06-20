<template>
  <div class="import-window">
    <div class="panel-header">
      <Icon icon="mdi:file-import-outline" width="16" height="16" class="header-icon" />
      <h2 class="panel-title">IMPORT MANAGER</h2>
    </div>

    <div class="section">
      <h3 class="section-title">[SRC] IMPORT SOURCES ({{ sources.length }})</h3>
      <div class="sources-list">
        <div v-for="src in sources" :key="src.id" class="source-card" :class="{ active: src.expanded }">
          <div class="sc-header" @click="src.expanded = !src.expanded">
            <Icon :icon="providerIcon(src.backendType)" width="18" height="18" class="sc-icon" />
            <div class="sc-info">
              <span class="sc-name">{{ src.name }}</span>
              <span class="sc-type">{{ src.backendType }}</span>
            </div>
            <span class="sc-status" :class="{ idle: src.status === 'idle', scanning: src.status === 'scanning', error: src.status === 'error', done: src.status === 'done' }">{{ src.status?.toUpperCase() || 'IDLE' }}</span>
            <Icon :icon="src.expanded ? 'mdi:chevron-up' : 'mdi:chevron-down'" width="16" height="16" class="sc-chevron" />
          </div>

          <div v-if="src.expanded" class="sc-body">
            <div class="sc-config">
              <div class="config-row">
                <label class="config-label">BASE PATH</label>
                <input v-model="src.basePath" class="bw-input-sm config-input" placeholder="/" @change="saveSource(src)" />
              </div>
              <div class="config-row">
                <label class="config-label">FILE TYPES</label>
                <div class="filter-tags">
                  <button :class="{ active: src.filter === 'all' }" @click="src.filter = 'all'; saveSource(src)">ALL</button>
                  <button :class="{ active: src.filter === 'files' }" @click="src.filter = 'files'; saveSource(src)">DOCUMENTS</button>
                  <button :class="{ active: src.filter === 'images' }" @click="src.filter = 'images'; saveSource(src)">IMAGES</button>
                  <button :class="{ active: src.filter === 'videos' }" @click="src.filter = 'videos'; saveSource(src)">VIDEOS</button>
                </div>
              </div>
              <div class="config-row toggles">
                <label class="toggle-item">
                  <input type="checkbox" v-model="src.compress" @change="saveSource(src)" />
                  <Icon icon="mdi:package-variant-closed" width="14" height="14" />
                  <span>COMPRESS</span>
                </label>
                <label class="toggle-item">
                  <input type="checkbox" v-model="src.encrypt" @change="saveSource(src)" />
                  <Icon icon="mdi:lock-outline" width="14" height="14" />
                  <span>ENCRYPT</span>
                </label>
                <label class="toggle-item">
                  <input type="checkbox" v-model="src.autoSync" @change="saveSource(src)" />
                  <Icon icon="mdi:sync" width="14" height="14" />
                  <span>AUTO-SYNC</span>
                </label>
              </div>
              <div class="config-row">
                <label class="config-label">OUTPUT DIR</label>
                <input v-model="src.outputDir" class="bw-input-sm config-input" placeholder="/imported" @change="saveSource(src)" />
              </div>
            </div>

            <div class="sc-summary">
              <div class="sum-item">
                <span class="sum-label">RAW</span>
                <span class="sum-val">{{ src.rawCount }}</span>
              </div>
              <div class="sum-item">
                <span class="sum-label">COMPRESSED</span>
                <span class="sum-val">{{ src.compressedCount }}</span>
              </div>
              <div class="sum-item">
                <span class="sum-label">IMPORTED</span>
                <span class="sum-val">{{ src.importedCount }}</span>
              </div>
            </div>

            <div class="sc-actions">
              <button class="bw-btn-sm" :disabled="src.scanning" @click="scanSource(src)">
                <Icon v-if="src.scanning" icon="svg-spinners:blocks-wave" width="10" height="10" />
                {{ src.scanning ? 'SCANNING...' : '[ SCAN ]' }}
              </button>
              <button class="bw-btn-sm" :disabled="src.importing" @click="importSource(src)">
                <Icon v-if="src.importing" icon="svg-spinners:blocks-wave" width="10" height="10" />
                {{ src.importing ? 'IMPORTING...' : '[ IMPORT ]' }}
              </button>
              <button class="bw-btn-sm bw-btn-danger" @click="openMoveDialog(src)">
                <Icon icon="mdi:file-move-outline" width="12" height="12" />
                [ MOVE FILES ]
              </button>
              <button class="bw-btn-sm bw-btn-danger" @click="removeSource(src)">[ REMOVE ]</button>
            </div>
          </div>
        </div>

        <div v-if="sources.length === 0" class="empty-state">
          <Icon icon="mdi:cloud-outline" width="24" height="24" />
          <span>No storage accounts configured. Connect accounts in Settings or Setup Wizard.</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[PROG] IMPORT PROGRESS</h3>
      <div class="progress-card" v-if="syncProgress">
        <div class="p-row p-row-status">
          <Icon v-if="syncActive" icon="svg-spinners:bars-scale" width="14" height="14" class="sync-spinner" />
          <span class="p-key text-muted">STATUS</span>
          <span class="p-val">{{ syncProgress.status.toUpperCase() }}</span>
        </div>
        <div class="progress-bar-track">
          <div class="progress-bar-fill" :style="{ width: progressPercent + '%' }"></div>
        </div>
        <div class="p-row p-row-files">
          <span class="p-key text-muted">FILES</span>
          <span class="p-val">{{ syncProgress.processedFiles }} / {{ syncProgress.totalFiles }}</span>
        </div>
        <div class="p-row p-row-bytes">
          <span class="p-key text-muted">BYTES</span>
          <span class="p-val">{{ formatBytes(syncProgress.bytesProcessed) }}</span>
        </div>
        <div class="p-row p-row-errors" v-if="syncProgress.errors.length > 0">
          <span class="p-key text-muted">ERRORS</span>
          <span class="p-val p-val-error">{{ syncProgress.errors.length }}</span>
        </div>
      </div>
      <div v-else class="progress-idle">
        <Icon icon="mdi:progress-check" width="14" height="14" />
        <span>No active import. Scan or import from a source above.</span>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'
import { onProgress, getProgress, type SyncProgress } from '@/wasm/sync'

interface ImportSource {
  id: string
  name: string
  backendType: string
  expanded: boolean
  basePath: string
  outputDir: string
  filter: 'all' | 'files' | 'images' | 'videos'
  compress: boolean
  encrypt: boolean
  autoSync: boolean
  status: string
  rawCount: number
  compressedCount: number
  importedCount: number
  scanning: boolean
  importing: boolean
}

const store = useAppStore()
const wm = useWindowManager()

const sources = ref<ImportSource[]>([])
const syncProgress = ref<SyncProgress | null>(null)
const syncActive = ref(false)

const progressPercent = computed(() => {
  if (!syncProgress.value || syncProgress.value.totalFiles === 0) return 0
  return Math.round((syncProgress.value.processedFiles / syncProgress.value.totalFiles) * 100)
})

const PROVIDER_ICONS: Record<string, string> = {
  googleDrive: 'mdi:google-drive',
  googlePhotos: 'mdi:google-photos',
  google: 'mdi:google',
  github: 'mdi:github',
  gitlab: 'mdi:gitlab',
  telegram: 'mdi:send',
  mega: 'logos:mega',
}

function providerIcon(type: string): string {
  return PROVIDER_ICONS[type] || 'mdi:cloud-outline'
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

async function loadSources() {
  const importAccounts = store.accounts.filter(a => a.oauthProvider || a.backendType === 'mega')
  const existingIds = new Set(sources.value.map(s => s.id))
  for (const acc of importAccounts) {
    if (!existingIds.has(acc.id)) {
      sources.value.push({
        id: acc.id,
        name: acc.name,
        backendType: acc.backendType || acc.oauthProvider || 'unknown',
        expanded: false,
        basePath: acc.path || '/',
        outputDir: '/imported',
        filter: 'all',
        compress: false,
        encrypt: false,
        autoSync: false,
        status: 'idle',
        rawCount: 0,
        compressedCount: 0,
        importedCount: 0,
        scanning: false,
        importing: false,
      })
    }
  }
}

async function saveSource(src: ImportSource) {
  const { updateAccount } = await import('@/wasm/data')
  await updateAccount(src.id, {
    path: src.basePath,
  } as any)
}

async function scanSource(src: ImportSource) {
  src.scanning = true
  src.status = 'scanning'
  try {
    const { listRemoteFiles } = await import('@/wasm/sync')
    const config = {
      id: src.id,
      name: src.name,
      backendType: src.backendType,
      enabled: true,
      basePath: src.basePath,
      autoSync: false,
      compressBeforeSync: false,
      maxConcurrentOps: 1,
      createdAt: '',
      updatedAt: '',
    }
    const files = await listRemoteFiles(config as any, src.basePath)
    const ext = (name: string) => name.split('.').pop()?.toLowerCase() || ''
    const imageExts = new Set(['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg', 'tiff', 'avif', 'heic', 'heif'])
    const videoExts = new Set(['mp4', 'mov', 'avi', 'mkv', 'webm', 'wmv', 'flv', 'm4v', '3gp', 'ogv'])
    const textExts = new Set(['txt', 'md', 'json', 'xml', 'csv', 'pdf', 'doc', 'docx', 'xls', 'xlsx'])

    let rawCount = 0, compressedCount = 0
    for (const f of files) {
      const e = ext(f.name)
      if (imageExts.has(e) || videoExts.has(e) || textExts.has(e)) {
        rawCount++
      } else if (['zip', 'gz', 'tar', 'bz2', 'xz', '7z', 'rar'].includes(e)) {
        compressedCount++
      } else {
        rawCount++
      }
    }

    src.rawCount = rawCount
    src.compressedCount = compressedCount
    src.status = 'done'
  } catch (e) {
    src.status = 'error'
    store.notifyError(`Scan failed: ${e instanceof Error ? e.message : String(e)}`, '')
  } finally {
    src.scanning = false
  }
}

async function importSource(src: ImportSource) {
  src.importing = true
  src.status = 'importing'
  syncActive.value = true
  try {
    const config = {
      id: src.id,
      name: src.name,
      backendType: src.backendType,
      enabled: true,
      basePath: src.basePath,
      autoSync: src.autoSync,
      compressBeforeSync: src.compress,
      maxConcurrentOps: 1,
      createdAt: '',
      updatedAt: '',
    }
    const { listRemoteFiles } = await import('@/wasm/sync')
    const files = await listRemoteFiles(config as any, src.basePath)

    const ext = (name: string) => name.split('.').pop()?.toLowerCase() || ''
    const imageExts = new Set(['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg', 'tiff', 'avif', 'heic', 'heif'])
    const videoExts = new Set(['mp4', 'mov', 'avi', 'mkv', 'webm', 'wmv', 'flv', 'm4v', '3gp', 'ogv'])

    const filtered = files.filter(f => {
      const e = ext(f.name)
      if (src.filter === 'images') return imageExts.has(e)
      if (src.filter === 'videos') return videoExts.has(e)
      if (src.filter === 'files') return !imageExts.has(e) && !videoExts.has(e)
      return true
    })

    const { startSync } = await import('@/wasm/sync')
    await startSync(config as any, filtered.map(f => f.path))

    src.importedCount += filtered.length
    src.status = 'done'
  } catch (e) {
    src.status = 'error'
    store.notifyError(`Import failed: ${e instanceof Error ? e.message : String(e)}`, '')
  } finally {
    src.importing = false
    syncActive.value = false
  }
}

function openMoveDialog(src: ImportSource) {
  // Open the TransferWindow with this source pre-selected
  wm.open('transfer', { sourceId: src.id })
}

async function removeSource(src: ImportSource) {
  const { deleteAccount } = await import('@/wasm/data')
  await deleteAccount(src.id)
  sources.value = sources.value.filter(s => s.id !== src.id)
}

function onSyncProgress(progress: SyncProgress) {
  syncProgress.value = progress
  if (progress.status === 'done' || progress.status === 'error') {
    syncActive.value = false
  }
}

let unsubProgress: (() => void) | null = null

onMounted(async () => {
  await loadSources()
  unsubProgress = onProgress(onSyncProgress)
})

onUnmounted(() => {
  if (unsubProgress) unsubProgress()
})
</script>

<style scoped>
.import-window {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 16px 18px;
  box-sizing: border-box;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 18px;
  padding-bottom: 12px;
  border-bottom: 1px solid #1a1a1a;
}

.header-icon {
  color: var(--text-primary);
  flex-shrink: 0;
}

.panel-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 2px;
  margin: 0;
}

.section {
  margin-bottom: 20px;
}

.section-title {
  font-size: 9px;
  font-weight: 700;
  color: #555;
  letter-spacing: 1.5px;
  margin: 0 0 10px 0;
}

.sources-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.source-card {
  background: var(--bg-glass-light);
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: border-color 0.15s;
  backdrop-filter: blur(var(--glass-blur-light));
}

.source-card.active {
  border-color: #2a2a2a;
}

.sc-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  cursor: pointer;
  user-select: none;
  transition: background 0.1s;
}

.sc-header:hover {
  background: var(--bg-surface);
}

.sc-icon {
  color: #aaa;
  flex-shrink: 0;
}

.sc-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.sc-name {
  font-size: 11px;
  font-weight: 700;
  color: #ccc;
}

.sc-type {
  font-size: 8px;
  color: #555;
  letter-spacing: 0.5px;
}

.sc-status {
  font-size: 8px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid transparent;
}

.sc-status.idle { color: #555; border-color: #2a2a2a; }
.sc-status.scanning { color: #45B7D1; border-color: rgba(69, 183, 209, 0.2); }
.sc-status.error { color: #ff5f57; border-color: rgba(255, 95, 87, 0.2); }
.sc-status.done { color: var(--text-accent); border-color: var(--border-glass); }

.sc-chevron {
  color: #444;
  flex-shrink: 0;
}

.sc-body {
  padding: 0 14px 14px;
  border-top: 1px solid #121212;
}

.sc-config {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 0;
}

.config-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.config-label {
  font-size: 8px;
  font-weight: 700;
  color: #555;
  letter-spacing: 1px;
  width: 64px;
  flex-shrink: 0;
}

.config-input {
  flex: 1;
}

.filter-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.filter-tags button {
  padding: 2px 10px;
  background: transparent;
  border: 1px solid var(--border-medium);
  border-radius: 4px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 8px;
  font-weight: 700;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.12s;
}

.filter-tags button.active {
  border-color: var(--border-accent);
  color: var(--text-accent);
  background: var(--bg-surface);
}

.filter-tags button:hover {
  border-color: #333;
  color: #888;
}

.toggles {
  display: flex;
  gap: 14px;
}

.toggle-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 9px;
  color: #666;
  cursor: pointer;
  user-select: none;
}

.toggle-item input {
  accent-color: var(--accent);
}

.toggle-item:hover {
  color: #888;
}

.sc-summary {
  display: flex;
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid #121212;
  margin-bottom: 8px;
}

.sum-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 6px 14px;
  background: var(--bg-surface);
  border-radius: 6px;
  border: 1px solid var(--border-glass);
}

.sum-label {
  font-size: 7px;
  font-weight: 700;
  color: #555;
  letter-spacing: 1px;
}

.sum-val {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
}

.sc-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  padding-top: 6px;
  border-top: 1px solid #121212;
}

.bw-btn-sm {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 12px;
  background: transparent;
  border: 1px solid var(--border-medium);
  border-radius: 4px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 700;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.12s;
}

.bw-btn-sm:hover {
  border-color: var(--border-accent);
  color: var(--text-accent);
  background: var(--accent-dim);
}

.bw-btn-sm:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.bw-btn-danger {
  border-color: #3a1a1a;
  color: #aa5555;
}

.bw-btn-danger:hover {
  border-color: #ff5f57;
  color: #ff5f57;
}

.bw-input-sm {
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 5px 8px;
  outline: none;
  transition: border-color 0.12s;
}

.bw-input-sm:focus {
  border-color: var(--border-accent);
  outline: none;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 32px 18px;
  color: #444;
  font-size: 10px;
  text-align: center;
}

.empty-state svg {
  color: #333;
}

.progress-card {
  background: var(--bg-glass-light);
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-md);
  padding: 14px;
  backdrop-filter: blur(var(--glass-blur-light));
}

.p-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.p-row:last-child {
  margin-bottom: 0;
}

.p-key {
  font-size: 8px;
  font-weight: 700;
  letter-spacing: 1px;
}

.p-val {
  font-size: 11px;
  font-weight: 700;
  color: #ccc;
}

.p-val-error {
  color: #ff5f57;
}

.progress-bar-track {
  height: 4px;
  background: #141414;
  border-radius: 2px;
  margin: 8px 0;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.sync-spinner {
  color: #45B7D1;
}

.progress-idle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  font-size: 9px;
  color: #555;
}
</style>
