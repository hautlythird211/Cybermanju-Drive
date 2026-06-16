<template>
  <Teleport to="body">
    <div v-if="visible" class="move-overlay" @click.self="onCancel">
      <div ref="dialogRef" class="move-modal">
        <div class="move-header">
          <div class="move-title">TRANSFER FILES</div>
          <button class="move-close" @click="onCancel">X</button>
        </div>

        <!-- Source context -->
        <div class="move-source">
          <div class="move-section-title">SOURCE</div>
          <div v-if="currentParentName" class="move-source-info">
            <Icon icon="mdi:folder-open-outline" width="10" height="10" />
            <span>{{ currentParentName }}</span>
          </div>
          <div class="move-file-list">
            <div v-for="f in files" :key="f.id" class="move-file-row" :class="getFileRowClass(f.id)">
              <Icon :icon="getFileIcon(f)" width="14" height="14" class="row-icon" />
              <span class="move-file-name truncate">{{ f.name }}</span>
              <span class="row-size">{{ formatSize(f.sizeBytes) }}</span>
              <div v-if="getSyncedBackends(f).length > 0" class="row-sync-badges" title="ALREADY ON:">
                <span v-for="be in getSyncedBackends(f)" :key="be" class="sync-badge">
                  <Icon :icon="getBackendIcon(be)" width="8" height="8" />
                </span>
              </div>
              <span v-if="fileStatuses[f.id]" class="row-status">{{ fileStatuses[f.id] }}</span>
            </div>
          </div>
        </div>

        <!-- Operation type -->
        <div class="move-op-type">
          <div class="move-section-title">OPERATION</div>
          <div class="op-type-options">
            <button class="op-btn" :class="{ active: opType === 'copy' }" @click="opType = 'copy'">
              <Icon icon="mdi:content-copy" width="12" height="12" />
              <span>COPY</span>
            </button>
            <button class="op-btn" :class="{ active: opType === 'move' }" @click="opType = 'move'">
              <Icon icon="mdi:file-send-outline" width="12" height="12" />
              <span>MOVE</span>
            </button>
            <button class="op-btn op-btn-transfer" :class="{ active: opType === 'transfer' }" @click="opType = 'transfer'">
              <Icon icon="mdi:transfer" width="12" height="12" />
              <span>TRANSFER</span>
            </button>
            <button class="op-btn" :class="{ active: opType === 'metadata' }" @click="opType = 'metadata'">
              <Icon icon="mdi:database-outline" width="12" height="12" />
              <span>INDEX ONLY</span>
            </button>
          </div>
          <div class="op-desc">
            <span v-if="opType === 'copy'">Copy files to destination, keep originals</span>
            <span v-if="opType === 'move'">Transfer files to destination, delete originals after success</span>
            <span v-if="opType === 'transfer'">Relay files between any source &amp; destination backends (no local copy)</span>
            <span v-if="opType === 'metadata'">Register files at destination in database only, no data transfer</span>
          </div>
        </div>

        <!-- Destination -->
        <div class="move-dest-section">
          <div class="move-section-title">DESTINATION</div>

          <div class="move-dest-group">
            <div class="move-group-label">LOCAL FOLDERS</div>
            <div class="move-folder-list">
              <label
                v-for="folder in localFolders"
                :key="folder.id"
                class="move-option"
                :class="{ active: selectedDest === folder.id }"
              >
                <input type="radio" class="move-radio" v-model="selectedDest" :value="folder.id" />
                <Icon :icon="getFolderIcon(folder)" width="12" height="12" class="dest-icon" />
                <span class="move-folder-name truncate">{{ folder.name }}</span>
                <span class="row-size">{{ folder.children?.length || 0 }} ITEMS</span>
              </label>
              <div v-if="localFolders.length === 0" class="no-items-hint">
                (NO LOCAL FOLDERS)
              </div>
            </div>
          </div>

          <div v-if="backends.length > 0" class="move-dest-group">
            <div class="move-group-label">CLOUD BACKENDS</div>
            <div class="move-folder-list">
              <label
                v-for="be in backends"
                :key="be.id"
                class="move-option"
                :class="{ active: selectedDest === be.id }"
              >
                <input type="radio" class="move-radio" v-model="selectedDest" :value="be.id" />
                <Icon :icon="getBackendIcon(be.backendType)" width="12" height="12" class="dest-icon" />
                <span class="move-folder-name truncate">{{ be.name || be.backendType }}</span>
                <span v-if="!be.enabled" class="dest-disabled">DISABLED</span>
                <span v-if="be.enabled" class="dest-extra">{{ be.compressBeforeUpload ? 'COMPRESS' : '' }}{{ be.createPreviews ? ' + PREVIEW' : '' }}</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Summary -->
        <div class="move-summary">
          <div class="move-summary-line" v-if="selectedDest">
            <Icon :icon="getDestIcon()" width="10" height="10" class="summary-icon" />
            <span>{{ summaryText }}</span>
          </div>
          <div v-if="conflicts.length > 0" class="move-conflicts">
            <div v-for="c in conflicts" :key="c.id" class="conflict-row">
              ⚠ {{ c.name }} already exists at destination
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="move-actions">
          <button class="mov-btn mov-cancel" @click="onCancel" :disabled="isMoving">[CANCEL]</button>
          <button
            v-if="!selectedDest"
            class="mov-btn mov-disabled"
            disabled
          >[SELECT A DESTINATION]</button>
          <button
            v-else
            class="mov-btn mov-execute"
            :class="{ 'mov-transfer-exec': opType === 'transfer' }"
            :disabled="isMoving"
            @click="onTransfer"
          >{{ getExecuteLabel() }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, computed, reactive } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useHistoryStore } from '@/stores/history'
import { useWindowManager } from '@/composables/useWindowManager'
import { invoke } from '@/composables/useTauri'
import type { FileNode, SyncConfig, SyncBackendType } from '@/types'

const props = defineProps<{
  visible: boolean
  fileIds: string[]
}>()

const emit = defineEmits<{
  close: []
}>()

const store = useAppStore()
const history = useHistoryStore()
const wm = useWindowManager()

const files = ref<FileNode[]>([])
const localFolders = ref<FileNode[]>([])
const backends = ref<SyncConfig[]>([])
const selectedDest = ref<string | null>(null)
const opType = ref<'copy' | 'move' | 'transfer' | 'metadata'>('copy')
const isMoving = ref(false)
const fileStatuses = reactive<Record<string, string>>({})

const currentParentName = computed(() => {
  if (files.value.length === 1) {
    const f = files.value[0]
    if (f.parentId) {
      const parent = store.files.find(p => p.id === f.parentId)
      return parent?.name || f.parentId
    }
    return 'ROOT'
  }
  return `${files.value.length} files from multiple folders`
})

const conflicts = computed(() => {
  if (!selectedDest.value) return []
  const destFolder = localFolders.value.find(f => f.id === selectedDest.value)
  if (!destFolder || !destFolder.children) return []
  const destNames = new Set(destFolder.children.map(c => c.name.toLowerCase()))
  return files.value.filter(f => destNames.has(f.name.toLowerCase()) && f.parentId !== selectedDest.value)
})

function getSyncedBackends(f: FileNode): SyncBackendType[] {
  const cd = f.contextData as Record<string, unknown> | undefined
  if (cd?.syncBackend) return [cd.syncBackend as SyncBackendType]
  return []
}

function getFileRowClass(fileId: string): string {
  const s = fileStatuses[fileId]
  if (s === 'error') return 'file-error'
  if (s === 'done') return 'file-done'
  return ''
}

function getFileIcon(f: FileNode): string {
  if (f.fileType === 'folder') return 'mdi:folder-outline'
  if (f.encrypted) return 'mdi:lock-outline'
  if (f.compressionLayers?.length && f.compressionLayers[0] !== 'none') return 'mdi:package-variant-closed'
  return 'mdi:file-outline'
}

function getFolderIcon(f: FileNode): string {
  return 'mdi:folder-outline'
}

function getBackendIcon(t: SyncBackendType): string {
  const map: Record<string, string> = {
    local: 'mdi:harddisk', github: 'mdi:github', gitlab: 'mdi:gitlab',
    googleDrive: 'mdi:google-drive', googlePhotos: 'mdi:google-photos', telegram: 'mdi:telegram',
    mega: 'mdi:cloud-upload-outline',
  }
  return map[t] || 'mdi:cloud-outline'
}

function getDestIcon(): string {
  if (!selectedDest.value) return 'mdi:help-circle-outline'
  if (backends.value.some(b => b.id === selectedDest.value)) {
    const be = backends.value.find(b => b.id === selectedDest.value)
    return getBackendIcon(be!.backendType)
  }
  return 'mdi:folder-outline'
}

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0; let s = bytes
  while (s >= 1024 && i < units.length - 1) { s /= 1024; i++ }
  return `${s.toFixed(1)} ${units[i]}`
}

const summaryText = computed(() => {
  if (!selectedDest.value) return ''
  const count = files.value.length
  const size = files.value.reduce((s, f) => s + (f.sizeBytes || 0), 0)
  const opLabel = { copy: 'COPY', move: 'MOVE', transfer: 'RELAY', metadata: 'INDEX ONLY' }[opType.value]
  if (backends.value.some(b => b.id === selectedDest.value)) {
    const be = backends.value.find(b => b.id === selectedDest.value)
    return `${opLabel} ${count} file(s) (${formatSize(size)}) → ${be?.name || be?.backendType}`
  }
  const folder = localFolders.value.find(f => f.id === selectedDest.value)
  return `${opLabel} ${count} file(s) (${formatSize(size)}) → ${folder?.name || 'folder'}`
})

function getExecuteLabel(): string {
  if (isMoving.value) return '[TRANSFERRING...]'
  const count = files.value.length
  if (opType.value === 'metadata') return `[INDEX ${count} FILE(S)]`
  if (opType.value === 'transfer') return `[RELAY ${count} FILE(S)]`
  if (opType.value === 'move' && backends.value.some(b => b.id === selectedDest.value)) return `[SYNC & DELETE ${count} FILE(S)]`
  return `[${opType.value.toUpperCase()} ${count} FILE(S)]`
}

watch(() => props.visible, (v) => {
  if (v) {
    files.value = props.fileIds
      .map(id => store.files.find(f => f.id === id))
      .filter((f): f is FileNode => !!f)
    localFolders.value = store.files.filter(f => f.fileType === 'folder')
    backends.value = store.syncConfigs
    selectedDest.value = null
    opType.value = 'copy'
    isMoving.value = false
    Object.keys(fileStatuses).forEach(k => delete fileStatuses[k])
  }
})

async function onTransfer() {
  if (!selectedDest.value || isMoving.value) return

  const destIsBackend = backends.value.some(b => b.id === selectedDest.value)
  const destFolder = localFolders.value.find(f => f.id === selectedDest.value)
  isMoving.value = true
  const succeeded: string[] = []
  const failed: Array<{ id: string; name: string; error: string }> = []

  // ── TRANSFER operation — opens TransferWindow for cross-backend relay ──
  if (opType.value === 'transfer') {
    emit('close')
    wm.open('transfer')
    return
  }

  for (const f of files.value) {
    fileStatuses[f.id] = destIsBackend ? 'SYNCING' : 'MOVING'

    try {
      if (destIsBackend) {
        if (opType.value === 'metadata') {
          fileStatuses[f.id] = 'INDEXED'
          succeeded.push(f.id)
          continue
        }
        const be = backends.value.find(b => b.id === selectedDest.value)
        if (be) {
          const result = await invoke<{ filesSynced: number }>('start_sync', { configId: be.id, fileIds: [f.id] })
          if (result.filesSynced > 0) {
            history.push('file:move', `SYNCED "${f.name}" → ${be.name || be.backendType}`, [f.id],
              { source: 'invoke', cmd: 'delete_file', args: { fileId: f.id } },
              { source: 'invoke', cmd: 'start_sync', args: { configId: be.id, fileIds: [f.id] } })
            succeeded.push(f.id)
            fileStatuses[f.id] = 'DONE'
          } else {
            throw new Error('Sync returned 0 files synced')
          }
        }
      } else if (destFolder) {
        if (opType.value === 'metadata') {
          fileStatuses[f.id] = 'INDEXED'
          succeeded.push(f.id)
          continue
        }
        await invoke('move_file', { fileId: f.id, newParentId: destFolder.id })
        history.push('file:move', `MOVED "${f.name}" → ${destFolder.name}`, [f.id],
          { source: 'invoke', cmd: 'move_file', args: { fileId: f.id, newParentId: f.parentId || '' } },
          { source: 'invoke', cmd: 'move_file', args: { fileId: f.id, newParentId: destFolder.id } })
        succeeded.push(f.id)
        fileStatuses[f.id] = 'DONE'
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      failed.push({ id: f.id, name: f.name, error: msg })
      fileStatuses[f.id] = 'ERROR'
    }
  }

  // Source cleanup (only for MOVE operations)
  if (opType.value === 'move' && succeeded.length > 0) {
    for (const id of succeeded) {
      fileStatuses[id] = 'CLEANING'
      try {
        await store.deleteFile(id, true)
        fileStatuses[id] = 'CLEANED'
      } catch {
        fileStatuses[id] = 'CLEANUP FAILED'
      }
    }
    await store.fetchTrashItems()
  }

  await store.fetchFiles()

  if (failed.length === 0) {
    store.notifySuccess(`Transferred ${succeeded.length} file(s)`)
    emit('close')
  } else {
    const errNames = failed.map(f => `"${f.name}"`).join(', ')
    store.notifyError(`Transfer incomplete — ${failed.length} failed: ${errNames}`, new Error(failed[0].error))
    isMoving.value = false
  }
}

function onCancel() {
  if (isMoving.value) return
  emit('close')
}
</script>

<style scoped>
.move-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.85);
  display: flex; align-items: center; justify-content: center; z-index: 10001;
}

.move-modal {
  background: #1a1a1a; border: 1px solid #333; border-radius: 10px;
  width: 520px; max-width: 94%; max-height: 85vh;
  display: flex; flex-direction: column;
  font-family: 'Courier New', monospace;
  box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}

.move-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 16px; border-bottom: 1px solid #2a2a2a;
}

.move-title { font-size: 11px; font-weight: 700; color: #e0e0e0; letter-spacing: 1px; }

.move-close {
  background: none; border: 1px solid #444; color: #888;
  width: 22px; height: 22px; cursor: pointer; border-radius: 4px;
  font-size: 10px; font-family: inherit;
}
.move-close:hover { background: #ff5f57; color: #fff; border-color: #ff5f57; }

.move-section-title {
  font-size: 9px; font-weight: 700; color: #999; letter-spacing: 0.5px; margin-bottom: 6px;
}

/* Source */
.move-source { padding: 8px 16px; border-bottom: 1px solid #2a2a2a; }

.move-source-info {
  display: flex; align-items: center; gap: 4px;
  padding: 3px 8px; font-size: 9px; color: #888; margin-bottom: 4px;
}

.move-file-list {
  display: flex; flex-direction: column; gap: 2px;
  max-height: 110px; overflow-y: auto;
}

.move-file-row {
  display: flex; align-items: center; gap: 6px;
  padding: 4px 8px; background: #151515; border-radius: 4px;
  font-size: 10px; color: #ccc; transition: background 0.15s;
}
.move-file-row.file-error { background: rgba(255,95,87,0.1); }
.move-file-row.file-done { background: rgba(0,255,65,0.05); }

.row-icon { flex-shrink: 0; color: #888; }
.move-file-name { flex: 1; min-width: 0; }
.row-size { font-size: 8px; color: #555; flex-shrink: 0; }
.row-sync-badges { display: flex; gap: 2px; margin-left: 4px; }
.sync-badge {
  display: flex; align-items: center; padding: 1px 4px;
  background: rgba(0,255,65,0.06); border: 1px solid rgba(0,255,65,0.1);
  border-radius: 3px; color: #00ff41; font-size: 7px;
}
.row-status { font-size: 7px; color: #888; margin-left: 4px; font-weight: 700; }

/* Operation type */
.move-op-type { padding: 8px 16px; border-bottom: 1px solid #2a2a2a; }

.op-type-options {
  display: flex; gap: 4px; margin-bottom: 6px;
}

.op-btn {
  flex: 1; display: flex; align-items: center; justify-content: center; gap: 4px;
  padding: 6px 8px; background: #151515; border: 1px solid #333;
  border-radius: 5px; color: #888; font-family: inherit; font-size: 9px; font-weight: 700;
  cursor: pointer; transition: all 0.1s;
}
.op-btn:hover { border-color: #555; color: #ccc; }
.op-btn.active { border-color: #00ff41; color: #00ff41; background: rgba(0,255,65,0.04); }
.op-btn-transfer.active { border-color: #45B7D1; color: #45B7D1; background: rgba(69,183,209,0.04); }

.op-desc { font-size: 8px; color: #555; padding: 0 2px; }

/* Destination */
.move-dest-section { padding: 8px 16px; border-bottom: 1px solid #2a2a2a; }

.move-dest-group { margin-bottom: 6px; }
.move-group-label { font-size: 8px; color: #666; margin-bottom: 3px; letter-spacing: 0.3px; }

.move-folder-list { display: flex; flex-direction: column; gap: 2px; max-height: 130px; overflow-y: auto; }

.move-option {
  display: flex; align-items: center; gap: 6px;
  padding: 4px 6px; cursor: pointer; border-radius: 4px; font-size: 10px; color: #ccc;
  border: 1px solid transparent; transition: all 0.08s;
}
.move-option:hover { background: #222; border-color: #333; }
.move-option.active { border-color: rgba(0,255,65,0.3); background: rgba(0,255,65,0.04); }

.move-radio {
  appearance: none; width: 12px; height: 12px;
  border: 1px solid #555; background: #111; cursor: pointer;
  border-radius: 50%; flex-shrink: 0;
}
.move-radio:checked { background: #00ff41; border-color: #00ff41; }

.dest-icon { flex-shrink: 0; color: #888; }
.move-folder-name { flex: 1; min-width: 0; }
.dest-disabled { font-size: 8px; color: #ff5f57; }
.dest-extra { font-size: 7px; color: #555; }
.no-items-hint { font-size: 9px; color: #555; padding: 4px 6px; }

/* Summary */
.move-summary {
  padding: 8px 16px; border-bottom: 1px solid #2a2a2a;
  min-height: 28px; display: flex; flex-direction: column; gap: 4px;
}
.move-summary-line {
  display: flex; align-items: center; gap: 6px;
  font-size: 9px; color: #888;
}
.summary-icon { flex-shrink: 0; }
.move-conflicts { display: flex; flex-direction: column; gap: 2px; }
.conflict-row {
  font-size: 8px; color: #f0b27a; padding: 2px 6px;
  background: rgba(240,178,122,0.06); border-radius: 3px;
}

/* Actions */
.move-actions {
  display: flex; gap: 6px; justify-content: flex-end;
  padding: 12px 16px;
}

.mov-btn {
  padding: 5px 14px; font-family: inherit; font-size: 10px; font-weight: 700;
  cursor: pointer; border: 1px solid #444; border-radius: 5px;
  background: transparent; color: #ccc; transition: all 0.1s;
}
.mov-cancel:hover { border-color: #666; background: #222; }
.mov-execute:hover:not(:disabled) { border-color: #5dade2; color: #5dade2; }
.mov-execute.mov-transfer-exec:hover:not(:disabled) { border-color: #45B7D1; color: #45B7D1; }
.mov-btn:disabled, .mov-disabled { opacity: 0.3; cursor: not-allowed; }

.text-muted { color: #555 !important; }
.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
