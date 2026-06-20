<template>
  <div class="transfer-window">
    <div class="panel-header">
      <Icon icon="mdi:transfer" width="16" height="16" class="header-icon" />
      <h2 class="panel-title">TRANSFER MANAGER</h2>
    </div>

    <div class="section">
      <h3 class="section-title">[SRC] SOURCE</h3>
      <select v-model="sourceConfig" class="bw-select transfer-select">
        <option value="" disabled>SELECT SOURCE BACKEND</option>
        <option v-for="cfg in configs" :key="cfg.id" :value="cfg.id">
          {{ cfg.name || cfg.backendType }} ({{ cfg.backendType }})
        </option>
      </select>
    </div>

    <div class="section">
      <h3 class="section-title">[DST] DESTINATION</h3>
      <select v-model="destConfig" class="bw-select transfer-select">
        <option value="" disabled>SELECT DESTINATION BACKEND</option>
        <option v-for="cfg in filteredDestConfigs" :key="cfg.id" :value="cfg.id">
          {{ cfg.name || cfg.backendType }} ({{ cfg.backendType }})
        </option>
      </select>
    </div>

    <div class="section">
      <h3 class="section-title">[FILES] FILES TO TRANSFER</h3>
      <div class="file-list">
        <div v-if="sourceFiles.length === 0" class="empty-files">
          <Icon icon="mdi:file-search-outline" width="16" height="16" />
          <span>{{ sourceConfig ? 'Click SCAN to list remote files' : 'Select a source backend first' }}</span>
        </div>
        <div
          v-for="f in sourceFiles"
          :key="f.path"
          class="file-row"
          :class="{ selected: selectedPaths.has(f.path) }"
          @click="toggleFile(f.path)"
        >
          <input type="checkbox" :checked="selectedPaths.has(f.path)" @click.stop="toggleFile(f.path)" />
          <Icon :icon="fileIcon(f)" width="14" height="14" class="file-row-icon" />
          <div class="file-row-info">
            <span class="file-row-name">{{ f.name }}</span>
            <span class="file-row-size">{{ formatSize(f.sizeBytes) }}</span>
          </div>
        </div>
      </div>
      <div class="file-actions" v-if="sourceFiles.length > 0">
        <button class="bw-btn-sm" @click="selectAll">{{ selectedAll ? 'DESELECT ALL' : 'SELECT ALL' }}</button>
        <span class="selected-count">{{ selectedPaths.size }} / {{ sourceFiles.length }} SELECTED</span>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[OPTS] OPTIONS</h3>
      <div class="opts-grid">
        <label class="option-row">
          <input type="checkbox" v-model="deleteSource" />
          <Icon icon="mdi:delete-outline" width="14" height="14" />
          <span>DELETE FROM SOURCE AFTER TRANSFER</span>
        </label>
        <label class="option-row">
          <input type="checkbox" v-model="saveToLocal" />
          <Icon icon="mdi:content-save-outline" width="14" height="14" />
          <span>SAVE TO LOCAL (keep a copy on this device)</span>
        </label>
        <div class="option-hint">
          Files are processed one at a time: resolve → upload → clean temp.
          Local source files are used directly (no download needed).
        </div>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[PROG] PROGRESS</h3>
      <div class="progress-card" v-if="progress">
        <div class="p-row">
          <span class="p-key">STATUS</span>
          <span class="p-val" :class="progress.status">{{ progress.status.toUpperCase() }}</span>
        </div>
        <div class="progress-bar-track">
          <div class="progress-bar-fill" :style="{ width: progressPercent + '%' }"></div>
        </div>
        <div class="p-row">
          <span class="p-key">FILES</span>
          <span class="p-val">{{ progress.processedFiles }} / {{ progress.totalFiles }}</span>
        </div>
        <div class="p-row">
          <span class="p-key">BYTES</span>
          <span class="p-val">{{ formatSize(progress.bytesTransferred) }}</span>
        </div>
        <div class="p-row" v-if="progress.currentFile">
          <span class="p-key">FILE</span>
          <span class="p-val p-val-file">{{ progress.currentFile }}</span>
        </div>
        <div class="p-row p-row-errors" v-if="progress.errors.length > 0">
          <span class="p-key">ERRORS</span>
          <span class="p-val p-val-error">{{ progress.errors.length }}</span>
        </div>
      </div>
      <div v-else class="progress-idle">
        <Icon icon="mdi:progress-check" width="14" height="14" />
        <span>Select source, destination, files, then click TRANSFER</span>
      </div>
    </div>

    <div class="transfer-actions">
      <button v-if="sourceConfig && sourceFiles.length === 0" class="bw-btn" @click="scanSource">
        <Icon icon="mdi:magnify" width="12" height="12" /> [ SCAN ]
      </button>
      <button class="bw-btn bw-btn-danger" @click="cancelTransfer(); isActive = false" :disabled="!isActive">
        [ CANCEL ]
      </button>
      <button
        class="bw-btn bw-btn-inverse"
        :disabled="!canTransfer || isActive"
        @click="doTransfer"
      >
        <Icon v-if="isActive" icon="svg-spinners:blocks-wave" width="12" height="12" />
        {{ isActive ? `TRANSFERRING (${progress?.processedFiles || 0}/${progress?.totalFiles || 0})...` : `[ TRANSFER ${selectedPaths.size} FILES ]` }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import type { RemoteFileInfo } from '@/wasm/sync'
import { transferFiles, getTransferProgress, cancelTransfer as cancelApi, type TransferProgress, type TransferRequest } from '@/wasm/transfer'
import type { SyncConfig } from '@/types'

const props = defineProps<{
  sourceId?: string
  destId?: string
}>()

// rename for template use
const cancelTransfer = cancelApi
const store = useAppStore()

const configs = ref<SyncConfig[]>([])
const sourceConfig = ref<string>('')
const destConfig = ref<string>('')
const sourceFiles = ref<RemoteFileInfo[]>([])
const selectedPaths = ref<Set<string>>(new Set())
const deleteSource = ref(false)
const saveToLocal = ref(false)
const progress = ref<TransferProgress | null>(null)
const isActive = ref(false)

const filteredDestConfigs = computed(() =>
  configs.value.filter(c => c.id !== sourceConfig.value)
)

const canTransfer = computed(() =>
  sourceConfig.value && destConfig.value && selectedPaths.value.size > 0
)

const selectedAll = computed(() =>
  sourceFiles.value.length > 0 && selectedPaths.value.size === sourceFiles.value.length
)

const progressPercent = computed(() => {
  if (!progress.value || progress.value.totalFiles === 0) return 0
  return Math.round((progress.value.processedFiles / progress.value.totalFiles) * 100)
})

function fileIcon(f: RemoteFileInfo): string {
  const ext = f.name.split('.').pop()?.toLowerCase() || ''
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'avif', 'heic', 'heif'].includes(ext)) return 'mdi:file-image-outline'
  if (['mp4', 'mov', 'avi', 'mkv', 'webm'].includes(ext)) return 'mdi:file-video-outline'
  if (['pdf', 'doc', 'docx', 'txt', 'md'].includes(ext)) return 'mdi:file-document-outline'
  if (['zip', 'gz', 'tar', 'bz2', '7z', 'rar'].includes(ext)) return 'mdi:file-archive-outline'
  return 'mdi:file-outline'
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function toggleFile(path: string) {
  const next = new Set(selectedPaths.value)
  if (next.has(path)) next.delete(path)
  else next.add(path)
  selectedPaths.value = next
}

function selectAll() {
  if (selectedAll.value) {
    selectedPaths.value = new Set()
  } else {
    selectedPaths.value = new Set(sourceFiles.value.map(f => f.path))
  }
}

async function scanSource() {
  const cfg = configs.value.find(c => c.id === sourceConfig.value)
  if (!cfg) return
  try {
    const { listRemoteFiles } = await import('@/wasm/sync')
    const files = await listRemoteFiles(cfg as any, '')
    sourceFiles.value = files.filter(f => !f.name.startsWith('.'))
  } catch (e) {
    sourceFiles.value = []
  }
}

async function doTransfer() {
  const src = configs.value.find(c => c.id === sourceConfig.value)
  const dst = configs.value.find(c => c.id === destConfig.value)
  if (!src || !dst) return

  isActive.value = true
  const request: TransferRequest = {
    sourceConfig: src as any,
    destConfig: dst as any,
    filePaths: Array.from(selectedPaths.value),
    deleteSourceAfter: deleteSource.value,
    saveToLocal: saveToLocal.value,
  }

  try {
    const result = await transferFiles(request)
    if (result.errors.length > 0) {
      console.error('Transfer errors:', result.errors)
    }
    selectedPaths.value = new Set()
  } catch (e) {
    console.error('Transfer failed:', e)
  } finally {
    isActive.value = false
  }
}

let pollInterval: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  configs.value = store.syncConfigs as any

  // Pre-select source/dest if opened from another component (e.g. ImportWindow)
  if (props.sourceId && configs.value.some(c => c.id === props.sourceId)) {
    sourceConfig.value = props.sourceId
  }
  if (props.destId && configs.value.some(c => c.id === props.destId)) {
    destConfig.value = props.destId
  }

  // Auto-scan if source was pre-selected
  if (sourceConfig.value) {
    await scanSource()
  }

  pollInterval = setInterval(async () => {
    try {
      progress.value = await getTransferProgress()
      if (progress.value && (progress.value.status === 'completed' || progress.value.status === 'cancelled' || progress.value.status === 'error')) {
        if (progress.value.status === 'completed' || progress.value.status === 'cancelled') {
          isActive.value = false
        }
      }
      if (progress.value && progress.value.status === 'completed') {
        progress.value = null
      }
    } catch {
      // ignore poll errors
    }
  }, 1000)
})

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval)
})
</script>

<style scoped>
.transfer-window {
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

.header-icon { color: var(--text-primary); flex-shrink: 0; }

.panel-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 2px;
  margin: 0;
}

.section {
  margin-bottom: 16px;
}

.section-title {
  font-size: 9px;
  font-weight: 700;
  color: #555;
  letter-spacing: 1.5px;
  margin: 0 0 8px 0;
}

.transfer-select {
  width: 100%;
  padding: 7px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  border-radius: 6px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 10px;
  outline: none;
  cursor: pointer;
}

.transfer-select:focus {
  border-color: var(--border-accent);
  outline: none;
}

.file-list {
  max-height: 180px;
  overflow-y: auto;
  background: var(--bg-surface);
  border: 1px solid var(--border-glass);
  border-radius: 6px;
}

.file-list::-webkit-scrollbar { width: 4px; }
.file-list::-webkit-scrollbar-track { background: transparent; }
.file-list::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 2px; }

.empty-files {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px;
  font-size: 9px;
  color: #555;
  justify-content: center;
}

.file-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  cursor: pointer;
  transition: background 0.1s;
}

.file-row:hover { background: var(--bg-surface); }
.file-row.selected { background: var(--bg-surface); }

.file-row input { accent-color: var(--accent); flex-shrink: 0; }

.file-row-icon { color: #666; flex-shrink: 0; }

.file-row-info {
  flex: 1;
  min-width: 0;
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.file-row-name {
  font-size: 10px;
  color: #ccc;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-row-size {
  font-size: 9px;
  color: #555;
  flex-shrink: 0;
}

.file-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 6px;
}

.selected-count {
  font-size: 9px;
  color: #666;
}

.option-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 9px;
  color: #666;
  cursor: pointer;
}

.option-row input { accent-color: #ff5f57; }

.opts-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.option-hint {
  font-size: 8px;
  color: #444;
  padding: 4px 0 0 0;
  line-height: 1.4;
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

.p-row:last-child { margin-bottom: 0; }

.p-key {
  font-size: 8px;
  font-weight: 700;
  color: #555;
  letter-spacing: 1px;
  width: 48px;
  flex-shrink: 0;
}

.p-val { font-size: 10px; font-weight: 700; color: #ccc; }
.p-val-file { font-size: 9px; color: #888; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.p-val-error { color: #ff5f57; }

.p-row-errors { border-top: 1px solid #1a1a1a; padding-top: 6px; }

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
  transition: width 0.3s;
}

.progress-idle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  font-size: 9px;
  color: var(--text-muted);
  background: var(--bg-glass-light);
  border-radius: 6px;
  border: 1px solid var(--border-glass);
}

.transfer-actions {
  display: flex;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid #1a1a1a;
}

.bw-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  background: transparent;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.bw-btn:hover { background: var(--accent-dim); border-color: var(--border-accent); color: var(--text-accent); }
.bw-btn:disabled { opacity: 0.3; cursor: not-allowed; }

.bw-btn-inverse {
  border-color: var(--accent);
  color: var(--text-accent);
  text-shadow: 0 0 4px rgba(var(--accent-rgb), 0.15);
}

.bw-btn-inverse:hover {
  background: var(--accent-dim);
  box-shadow: 0 0 12px rgba(var(--accent-rgb), 0.1);
}

.bw-btn-danger {
  border-color: #3a1a1a;
  color: #aa5555;
}

.bw-btn-danger:hover { border-color: #ff5f57; color: #ff5f57; }

.bw-btn-sm {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
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

.bw-btn-sm:hover { background: var(--accent-dim); border-color: var(--border-accent); color: var(--text-accent); }
</style>
