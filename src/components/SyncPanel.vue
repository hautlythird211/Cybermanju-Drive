<template>
  <div class="sync-panel">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-sync">[~]</span>
        <h2 class="panel-title">STORAGE SYNC</h2>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[CFG] SYNC CONFIGS ({{ syncConfigs.length }})</h3>
      <div class="config-list">
        <div v-for="cfg in syncConfigs" :key="cfg.id" class="config-card">
          <div class="cfg-header">
            <span class="cfg-name">{{ cfg.name || cfg.backendType }}</span>
            <span class="cfg-type text-muted">{{ cfg.backendType }}</span>
            <span class="cfg-status" :class="{ on: cfg.enabled }">{{ cfg.enabled ? 'ON' : 'OFF' }}</span>
          </div>
          <div class="cfg-meta text-muted">
            <span v-if="cfg.basePath">PATH: {{ cfg.basePath }}</span>
            <span v-if="cfg.repoName">REPO: {{ cfg.repoName }}</span>
          </div>
          <div class="cfg-actions">
            <button class="bw-btn-sm" @click="startSyncForCfg(cfg.id)" :disabled="syncActive">[SYNC]</button>
            <button class="bw-btn-sm" @click="deleteCfg(cfg.id)">[DEL]</button>
          </div>
        </div>
        <button class="bw-btn add-cfg" @click="showAddConfig = !showAddConfig">[+ ADD CONFIG]</button>
      </div>

      <div v-if="showAddConfig" class="add-config-form">
        <input v-model="newConfigName" class="bw-input-sm" placeholder="NAME" @keyup.enter="addConfig" />
        <select v-model="newConfigType" class="bw-select-sm">
          <option value="local">LOCAL</option>
          <option value="github">GITHUB</option>
          <option value="gitlab">GITLAB</option>
          <option value="googleDrive">GOOGLE DRIVE</option>
          <option value="googlePhotos">GOOGLE PHOTOS</option>
        </select>
        <button class="bw-btn-sm" @click="addConfig">[SAVE]</button>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[PROG] SYNC PROGRESS</h3>
      <div class="progress-card" v-if="syncProgress">
        <div class="p-row p-row-status">
          <Icon v-if="syncActive" icon="svg-spinners:bars-scale" width="14" height="14" class="sync-spinner" />
          <span class="p-key text-muted">STATUS</span>
          <span class="p-value">{{ syncProgress.status }}</span>
        </div>
        <div class="p-row"><span class="p-key text-muted">FILES</span><span class="p-value">{{ syncProgress.processedFiles }}/{{ syncProgress.totalFiles }}</span></div>
        <div class="p-row"><span class="p-key text-muted">BYTES</span><span class="p-value">{{ formatSize((syncProgress as any).bytesUploaded || (syncProgress as any).bytesProcessed || 0) }}</span></div>
        <div class="p-row" v-if="syncProgress.errors?.length"><span class="p-key text-muted">ERRORS</span><span class="p-value">{{ syncProgress.errors.length }}</span></div>
        <div class="p-row" v-if="syncProgress.currentFile"><span class="p-key text-muted">CURRENT</span><span class="p-value" style="font-size:8px;word-break:break-all;">{{ syncProgress.currentFile }}</span></div>
        <div class="p-actions" v-if="syncActive">
          <button class="bw-btn-sm" @click="cancelCurrentSync">[CANCEL]</button>
        </div>
      </div>
      <div class="progress-card" v-else-if="syncSummary">
        <div class="p-row"><span class="p-key text-muted">TOTAL</span><span class="p-value">{{ syncSummary.totalFiles }}</span></div>
        <div class="p-row"><span class="p-key text-muted">SYNCED</span><span class="p-value">{{ syncSummary.syncedFiles }}</span></div>
        <div class="p-row"><span class="p-key text-muted">PENDING</span><span class="p-value">{{ syncSummary.changedFiles }}</span></div>
        <div class="p-row"><span class="p-key text-muted">ERRORS</span><span class="p-value">{{ syncSummary.errorFiles }}</span></div>
        <div class="p-row"><span class="p-key text-muted">SIZE</span><span class="p-value">{{ formatSize(syncSummary.totalBytes) }}</span></div>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[LOCAL] SYNC-WITHOUT-UPLOAD</h3>
      <div class="local-sync-actions">
        <button class="bw-btn" @click="markAllSynced">[MARK ALL SYNCED]</button>
        <button class="bw-btn" @click="refreshSummary">[REFRESH STATUS]</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { invoke } from '@/composables/useTauri'
import { useNotifications } from '@/composables/useNotifications'

const store = useAppStore()
const { notify } = useNotifications()
const syncConfigs = computed(() => store.syncConfigs)
const syncProgress = computed(() => store.syncProgress)
const syncActive = computed(() => {
  const s = syncProgress.value?.status
  if (!s) return false
  return ['uploading', 'scanning', 'compressing', 'linking', 'cleaning'].includes(s)
})

const showAddConfig = ref(false)
const newConfigName = ref('')
const newConfigType = ref('local')
const syncSummary = ref<{
  totalFiles: number; syncedFiles: number; changedFiles: number; errorFiles: number; totalBytes: number
} | null>(null)

let progressUnsub: (() => void) | null = null

onMounted(async () => {
  await refreshSummary()
  // Subscribe to WASM sync progress
  try {
    const { sync } = await import('@/wasm')
    progressUnsub = sync.onProgress((p) => {
      store.syncProgress = {
        status: p.status as any,
        processedFiles: p.processedFiles,
        totalFiles: p.totalFiles,
        bytesUploaded: p.bytesProcessed,
        currentFile: p.currentFile ?? undefined,
        errors: p.errors,
        startedAt: p.startedAt ?? undefined,
      }
    })
  } catch { /* WASM not available */ }
})

onUnmounted(() => {
  progressUnsub?.()
})

async function refreshSummary() {
  try {
    const { sync } = await import('@/wasm')
    syncSummary.value = await sync.getSyncSummary()
  } catch {
    syncSummary.value = null
  }
}

async function addConfig() {
  if (!newConfigName.value.trim()) return
  try {
    await store.createSyncConfig({
      name: newConfigName.value,
      backendType: newConfigType.value,
      enabled: true,
      basePath: '/',
      autoSync: false,
      compressBeforeUpload: false,
      maxConcurrentUploads: 1,
    } as any)
    newConfigName.value = ''
    showAddConfig.value = false
    notify('success', 'SYNC CONFIG CREATED')
  } catch (e) {
    notify('error', `FAILED: ${e}`)
  }
}

async function deleteCfg(id: string) {
  try {
    await store.deleteSyncConfig(id)
    notify('success', 'CONFIG DELETED')
  } catch (e) {
    notify('error', `FAILED: ${e}`)
  }
}

async function startSyncForCfg(configId: string) {
  try {
    await store.startSync(configId, [])
  } catch (e) {
    notify('error', `SYNC FAILED: ${e}`)
  }
}

function cancelCurrentSync() {
  store.cancelSync()
}

async function markAllSynced() {
  try {
    const { sync } = await import('@/wasm')
    await sync.markAllFilesSynced()
    await refreshSummary()
    notify('success', 'ALL FILES MARKED AS SYNCED')
  } catch (e) {
    notify('error', `FAILED: ${e}`)
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const k = 1024
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + units[i]
}
</script>

<style scoped>
.sync-panel {
  width: 100%;
  height: 100%;
  background: #000;
  overflow-y: auto;
  padding: 16px;
  font-family: 'Courier New', monospace;
  color: #FFFFFF;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 10px;
  border-bottom: 2px solid #FFFFFF;
  margin-bottom: 16px;
}

.header-left { display: flex; align-items: center; gap: 8px; }
.icon-sync { font-size: 16px; }
.panel-title { font-size: 14px; font-weight: 800; letter-spacing: 1px; margin: 0; }

.section { margin-bottom: 16px; }

.section-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgba(255,255,255,0.6);
  margin: 0 0 8px;
}

.config-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.config-card {
  border: 2px solid #FFFFFF;
  padding: 8px 10px;
}

.cfg-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.cfg-name { font-size: 12px; font-weight: 700; flex: 1; }
.cfg-type { font-size: 9px; }
.cfg-status { font-size: 9px; font-weight: 700; border: 1px solid #FFFFFF; padding: 0 4px; }
.cfg-status.on { background: #FFFFFF; color: #000; }
.cfg-meta { font-size: 9px; }

.progress-card {
  border: 2px solid #FFFFFF;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.p-row { display: flex; justify-content: space-between; }
.p-row-status { display: flex; align-items: center; gap: 6px; }
.sync-spinner { display: inline-flex; flex-shrink: 0; }
.p-key { font-size: 10px; }
.p-value { font-size: 10px; font-weight: 700; }

.text-muted { color: rgba(255,255,255,0.5) !important; }

.cfg-actions { display: flex; gap: 4px; margin-top: 6px; }
.bw-btn-sm {
  background: #000;
  border: 1px solid #FFF;
  color: #FFF;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  padding: 2px 6px;
  cursor: pointer;
}
.bw-btn-sm:hover { background: #FFF; color: #000; }
.bw-btn-sm:disabled { opacity: 0.4; cursor: default; }

.add-cfg { width: 100%; margin-top: 6px; font-size: 10px; }
.add-config-form {
  display: flex;
  gap: 4px;
  margin-top: 6px;
  align-items: center;
}
.bw-input-sm {
  background: #000;
  border: 1px solid #FFF;
  color: #FFF;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  padding: 4px 6px;
  flex: 1;
}
.bw-select-sm {
  background: #000;
  border: 1px solid #FFF;
  color: #FFF;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  padding: 4px;
}
.p-actions { margin-top: 6px; display: flex; gap: 4px; }
.local-sync-actions { display: flex; gap: 6px; }
.bw-btn {
  padding: 6px 12px;
  background: #000;
  border: 2px solid #FFF;
  color: #FFF;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
}
.bw-btn:hover { background: #FFF; color: #000; }
</style>
