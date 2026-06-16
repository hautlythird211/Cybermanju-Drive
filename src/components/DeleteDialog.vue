<template>
  <Teleport to="body">
    <div v-if="visible" class="delete-overlay" @click.self="onCancel">
      <div ref="dialogRef" class="delete-modal">
        <!-- Header -->
        <div class="delete-header">
          <div class="delete-title">DELETE FILES</div>
          <button class="delete-close" @click="onCancel">X</button>
        </div>

        <!-- File list / preview -->
        <div class="delete-files">
          <div v-for="f in files" :key="f.id" class="delete-file-row">
            <div class="delete-file-icon">
              <Icon :icon="getFileIcon(f)" width="16" height="16" class="df-icon" />
            </div>
            <div class="delete-file-info">
              <span class="delete-file-name truncate">{{ f.name }}</span>
              <span class="delete-file-meta text-muted">{{ formatSize(f.sizeBytes) }} · {{ f.mimeType || f.fileType }}</span>
            </div>
            <button class="delete-preview-btn" @click="onUnbox(f)" title="VIEW COMPRESSED DATA / PREVIEW">[UNBOX]</button>
          </div>
        </div>

        <!-- Delete scope -->
        <div class="delete-scope">
          <div class="delete-section-title">DELETE WHERE?</div>
          <div class="delete-backends">
            <label class="delete-option" v-for="be in backends" :key="be.id">
              <input type="checkbox" class="delete-cb" v-model="selectedBackendIds" :value="be.id" />
              <Icon :icon="getBackendIcon(be.backendType)" width="12" height="12" class="del-be-icon" />
              <span class="del-be-name">{{ be.name || be.backendType }}</span>
              <span v-if="!be.enabled" class="text-muted">[DISABLED]</span>
            </label>
          </div>
          <div v-if="backends.length === 0" class="text-muted" style="padding: 4px 0; font-size: 9px;">
            NO SYNC BACKENDS CONFIGURED — DELETE FROM LOCAL STORAGE ONLY
          </div>
        </div>

        <!-- Options -->
        <div class="delete-options">
          <div class="delete-section-title">OPTIONS</div>
          <label class="delete-option">
            <input type="checkbox" class="delete-cb" v-model="deleteMetadataOnly" />
            <span>DELETE METADATA ONLY</span>
            <span class="text-muted" style="font-size: 8px; margin-left: 6px;">(KEEP RAW FILES, REMOVE FROM DATABASE)</span>
          </label>
          <label class="delete-option">
            <input type="checkbox" class="delete-cb" v-model="deleteFromBackends" />
            <span>ALSO DELETE FROM SELECTED BACKENDS</span>
          </label>
        </div>

      <!-- Backend errors -->
      <div v-if="backendErrors.length > 0" class="delete-backend-errors">
        <div v-for="(err, ei) in backendErrors" :key="ei" class="delete-backend-error">
          ⚠ {{ err }}
        </div>
      </div>

        <!-- Actions -->
        <div class="delete-actions">
          <button class="del-btn del-cancel" @click="onCancel" :disabled="isDeleting">[CANCEL]</button>
          <button class="del-btn del-move" @click="onMove" :disabled="isDeleting">[MOVE...]</button>
          <button
            v-if="deleteMetadataOnly"
            class="del-btn del-meta"
            :disabled="isDeleting"
            @click="onDeleteMetadata"
          >{{ isDeleting ? '...' : '[DELETE METADATA ONLY]' }}</button>
          <button
            v-else
            class="del-btn del-delete"
            :disabled="isDeleting"
            @click="onDelete"
          >{{ isDeleting ? '[DELETING...]' : '[DELETE]' }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useHistoryStore } from '@/stores/history'
import type { FileNode, SyncConfig, SyncBackendType } from '@/types'

const props = defineProps<{
  visible: boolean
  fileIds: string[]
}>()

const emit = defineEmits<{
  close: []
  move: [fileIds: string[]]
  unbox: [file: FileNode]
}>()

const store = useAppStore()
const history = useHistoryStore()

const files = ref<FileNode[]>([])
const backends = ref<SyncConfig[]>([])
const selectedBackendIds = ref<string[]>([])
const deleteMetadataOnly = ref(false)
const deleteFromBackends = ref(false)
const isDeleting = ref(false)
const backendErrors = ref<string[]>([])

watch(() => props.visible, (v) => {
  if (v) {
    files.value = props.fileIds
      .map(id => store.files.find(f => f.id === id))
      .filter((f): f is FileNode => !!f)
    backends.value = store.syncConfigs.filter(c => c.enabled || true)
    selectedBackendIds.value = store.syncConfigs
      .filter(c => c.enabled)
      .map(c => c.id)
    deleteMetadataOnly.value = false
    deleteFromBackends.value = false
  }
})

function getFileIcon(f: FileNode): string {
  if (f.fileType === 'folder') return 'mdi:folder-outline'
  if (f.encrypted) return 'mdi:lock-outline'
  if (f.compressionLayers?.length && f.compressionLayers[0] !== 'none') return 'mdi:package-variant-closed'
  if (f.mimeType?.startsWith('image/')) return 'mdi:file-image-outline'
  if (f.mimeType?.startsWith('text/') || f.mimeType?.includes('json')) return 'mdi:file-document-outline'
  if (f.mimeType?.startsWith('audio/')) return 'mdi:file-music-outline'
  if (f.mimeType?.startsWith('video/')) return 'mdi:file-video-outline'
  return 'mdi:file-outline'
}

function getBackendIcon(t: SyncBackendType): string {
  const map: Record<string, string> = {
    local: 'mdi:harddisk',
    github: 'mdi:github',
    gitlab: 'mdi:gitlab',
    googleDrive: 'mdi:google-drive',
    googlePhotos: 'mdi:google-photos',
    telegram: 'mdi:telegram',
    mega: 'mdi:cloud-upload-outline',
  }
  return map[t] || 'mdi:cloud-outline'
}

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  let s = bytes
  while (s >= 1024 && i < units.length - 1) { s /= 1024; i++ }
  return `${s.toFixed(1)} ${units[i]}`
}

async function onDelete() {
  isDeleting.value = true
  backendErrors.value = []
  const targets = deleteFromBackends.value ? selectedBackendIds.value : []
  try {
    for (const f of files.value) {
      await store.deleteFile(f.id, true)
      history.push('file:delete', `DELETED "${f.name}" [BACKENDS: ${targets.length}]`, [f.id],
        { source: 'store', cmd: 'restoreTrashItem', args: { fileId: f.id } },
        { source: 'store', cmd: 'deleteFile', args: { fileId: f.id } })
    }
    if (targets.length > 0) {
      try {
        const { invoke } = await import('@/composables/useTauri')
        await invoke('delete_files_from_backends', { fileIds: props.fileIds, backendConfigIds: targets })
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e)
        backendErrors.value.push(msg)
      }
    }
    store.notifySuccess(`Deleted ${files.value.length} file(s)`)
    await store.fetchTrashItems()
    emit('close')
  } catch (e) {
    store.notifyError('Delete failed', e)
  } finally {
    isDeleting.value = false
  }
}

async function onDeleteMetadata() {
  try {
    const { invoke: invokeMeta } = await import('@/composables/useTauri')
    for (const f of files.value) {
      await invokeMeta('delete_file_metadata_only', { fileId: f.id })
    }
    await store.fetchFiles()
    store.notifySuccess(`Deleted metadata for ${files.value.length} file(s)`)
    emit('close')
  } catch (e) {
    store.notifyError('Failed to delete metadata', e)
  }
}

async function onMove() {
  emit('move', props.fileIds)
}

function onUnbox(f: FileNode) {
  emit('unbox', f)
}

function onCancel() {
  emit('close')
}
</script>

<style scoped>
.delete-overlay {
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.85);
  display: flex; align-items: center; justify-content: center;
  z-index: 10001;
}

.delete-modal {
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 10px;
  width: 520px;
  max-width: 94%;
  max-height: 80vh;
  display: flex; flex-direction: column;
  font-family: 'Courier New', monospace;
  box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}

.delete-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 16px; border-bottom: 1px solid #2a2a2a;
}

.delete-title {
  font-size: 11px; font-weight: 700; color: #e0e0e0; letter-spacing: 1px;
}

.delete-close {
  background: none; border: 1px solid #444; color: #888;
  width: 22px; height: 22px; cursor: pointer; border-radius: 4px;
  font-size: 10px; font-family: inherit;
}

.delete-close:hover { background: #ff5f57; color: #fff; border-color: #ff5f57; }

/* Backend errors */
.delete-backend-errors { padding: 4px 16px; }
.delete-backend-error {
  font-size: 8px; color: #ff5f57; padding: 2px 6px;
  background: rgba(255,95,87,0.08); border-radius: 3px; margin-bottom: 2px;
}

/* Files */
.delete-files {
  padding: 8px 16px; display: flex; flex-direction: column; gap: 4px;
  max-height: 160px; overflow-y: auto;
}

.delete-file-row {
  display: flex; align-items: center; gap: 8px;
  padding: 6px 8px; background: #151515; border-radius: 4px;
}

.delete-file-icon { width: 20px; flex-shrink: 0; display: flex; align-items: center; justify-content: center; }
.df-icon { color: #888; }
.delete-file-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
.delete-file-name { font-size: 10px; font-weight: 600; color: #ccc; }
.delete-file-meta { font-size: 8px; }

.delete-preview-btn {
  background: transparent; border: 1px solid #333; color: #888;
  padding: 2px 8px; font-family: inherit; font-size: 8px; font-weight: 700;
  cursor: pointer; border-radius: 3px; flex-shrink: 0;
}
.delete-preview-btn:hover { border-color: #00ff41; color: #00ff41; }

/* Scope */
.delete-scope { padding: 8px 16px; border-top: 1px solid #2a2a2a; }
.delete-section-title {
  font-size: 9px; font-weight: 700; color: #999; letter-spacing: 0.5px; margin-bottom: 6px;
}

.delete-backends { display: flex; flex-direction: column; gap: 3px; }

.delete-option {
  display: flex; align-items: center; gap: 6px;
  padding: 4px 6px; cursor: pointer; border-radius: 4px; font-size: 10px; color: #ccc;
  transition: background 0.08s;
}
.delete-option:hover { background: #222; }

.delete-cb {
  appearance: none; width: 12px; height: 12px;
  border: 1px solid #555; background: #111; cursor: pointer; border-radius: 2px; flex-shrink: 0;
}
.delete-cb:checked { background: #00ff41; border-color: #00ff41; }

.del-be-icon { flex-shrink: 0; color: #777; }
.del-be-name { flex: 1; }

/* Options */
.delete-options { padding: 8px 16px; border-top: 1px solid #2a2a2a; }

/* Actions */
.delete-actions {
  display: flex; gap: 6px; justify-content: flex-end;
  padding: 12px 16px; border-top: 1px solid #2a2a2a;
}

.del-btn {
  padding: 5px 14px; font-family: inherit; font-size: 10px; font-weight: 700;
  cursor: pointer; border: 1px solid #444; border-radius: 5px;
  background: transparent; color: #ccc; transition: all 0.1s;
}

.del-cancel:hover { border-color: #666; background: #222; }
.del-move:hover { border-color: #5dade2; color: #5dade2; }
.del-meta:hover { border-color: #f0b27a; color: #f0b27a; }
.del-delete:hover { border-color: #ff5f57; color: #ff5f57; }
.del-btn:disabled { opacity: 0.3; cursor: not-allowed; }

.text-muted { color: #555 !important; }
.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
