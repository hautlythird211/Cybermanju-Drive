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
            <span v-if="cfg.useGitLfs" class="lfs-badge">LFS</span>
            <span v-if="cfg.repoLayout" class="layout-badge">{{ cfg.repoLayout }}</span>
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
          <option value="codeberg">CODEBERG</option>
          <option value="gitea">GITEA/FORGEJO</option>
          <option value="googleDrive">GOOGLE DRIVE</option>
          <option value="googlePhotos">GOOGLE PHOTOS</option>
          <option value="mega">MEGA</option>
        </select>
        <div v-if="isGitType" class="lfs-options">
          <label class="lfs-toggle">
            <input type="checkbox" v-model="newUseLfs" />
            <span>LFS</span>
          </label>
          <select v-model="newRepoLayout" class="bw-select-sm">
            <option value="flat">FLAT</option>
            <option value="sharded">SHARDED</option>
            <option value="split">SPLIT</option>
          </select>
        </div>
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
const newUseLfs = ref(false)
const newRepoLayout = ref<'flat' | 'sharded' | 'split'>('flat')
const isGitType = computed(() => ['github', 'gitlab', 'codeberg', 'gitea'].includes(newConfigType.value))
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
    const config: Record<string, any> = {
      name: newConfigName.value,
      backendType: newConfigType.value,
      enabled: true,
      basePath: '/',
      autoSync: false,
      compressBeforeUpload: false,
      maxConcurrentUploads: 1,
    }
    if (isGitType.value) {
      config.useGitLfs = newUseLfs.value
      config.repoLayout = newRepoLayout.value
    }
    await store.createSyncConfig(config as any)
    newConfigName.value = ''
    newUseLfs.value = false
    newRepoLayout.value = 'flat'
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
  overflow-y: auto;
  padding: 20px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
  margin-bottom: 16px;
}

.header-left { display: flex; align-items: center; gap: 8px; }
.icon-sync { font-size: 14px; color: var(--text-accent); }
.panel-title { font-size: 13px; font-weight: 700; letter-spacing: 1px; margin: 0; color: var(--text-primary); }

.section { margin-bottom: 16px; }

.section-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0 0 8px;
  font-family: var(--font-mono);
}

.config-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.config-card {
  border: 1px solid var(--border-glass);
  padding: 10px 12px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.config-card:hover {
  border-color: var(--border-accent);
  background: var(--accent-dim);
}

.cfg-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.cfg-name { font-size: 12px; font-weight: 600; flex: 1; color: var(--text-primary); }
.cfg-type { font-size: 9px; color: var(--text-muted); }
.cfg-status { font-size: 9px; font-weight: 600; border: 1px solid var(--border-medium); padding: 1px 6px; color: var(--text-muted); border-radius: var(--radius-sm); }
.cfg-status.on { background: var(--accent-dim); color: var(--text-accent); border-color: var(--border-accent); }
.cfg-meta { font-size: 9px; color: var(--text-muted); display: flex; gap: 4px; flex-wrap: wrap; }
.lfs-badge, .layout-badge {
  font-size: 8px;
  font-weight: 600;
  padding: 1px 4px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-accent);
  color: var(--text-accent);
  background: var(--accent-dim);
}

.progress-card {
  border: 1px solid var(--border-glass);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-md);
}

.p-row { display: flex; justify-content: space-between; }
.p-row-status { display: flex; align-items: center; gap: 6px; }
.sync-spinner { display: inline-flex; flex-shrink: 0; }
.p-key { font-size: 10px; color: var(--text-muted); }
.p-value { font-size: 10px; font-weight: 600; color: var(--text-primary); }

.text-muted { color: var(--text-muted) !important; }

.cfg-actions { display: flex; gap: 4px; margin-top: 6px; }
.bw-btn-sm {
  background: transparent;
  border: 1px solid var(--border-medium);
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 9px;
  padding: 3px 8px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}
.bw-btn-sm:hover { background: var(--accent-dim); border-color: var(--border-accent); color: var(--text-accent); }
.bw-btn-sm:disabled { opacity: 0.4; cursor: default; }

.add-cfg { width: 100%; margin-top: 6px; font-size: 10px; }
.add-config-form {
  display: flex;
  gap: 4px;
  margin-top: 6px;
  align-items: center;
}
.bw-input-sm {
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 5px 8px;
  flex: 1;
  border-radius: var(--radius-sm);
}
.bw-input-sm:focus {
  border-color: var(--border-accent);
  outline: none;
}
.bw-select-sm {
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 9px;
  padding: 5px;
  border-radius: var(--radius-sm);
}
.lfs-options {
  display: flex;
  gap: 4px;
  align-items: center;
}
.lfs-toggle {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 9px;
  color: var(--text-muted);
  cursor: pointer;
}
.lfs-toggle input {
  margin: 0;
}
.p-actions { margin-top: 6px; display: flex; gap: 4px; }
.local-sync-actions { display: flex; gap: 6px; }
.bw-btn {
  padding: 7px 14px;
  background: var(--accent);
  color: var(--text-inverse);
  border: 1px solid var(--accent);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}
.bw-btn:hover { background: #00cc35; border-color: #00cc35; }
</style>
