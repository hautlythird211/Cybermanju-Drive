<template>
  <OsModal
    :visible="visible"
    title="DELETE FILES"
    variant="glass"
    size="md"
    closable
    @update:visible="$emit('close')"
  >
    <div class="delete-files">
      <div v-for="f in files" :key="f.id" class="delete-file-row">
        <OsIcon :icon="getFileIcon(f)" :size="14" />
        <div class="delete-file-info">
          <span class="delete-file-name truncate">{{ f.name }}</span>
          <span class="delete-file-meta">{{ formatSize(f.sizeBytes) }} · {{ f.mimeType || f.fileType }}</span>
        </div>
        <OsButton variant="ghost" size="xs" @click="onUnbox(f)">UNBOX</OsButton>
      </div>
    </div>

    <OsDivider spacing="sm" />

    <div class="delete-scope">
      <div class="delete-section-title">DELETE WHERE?</div>
      <div class="delete-backends">
        <label class="delete-option" v-for="be in backends" :key="be.id">
          <input type="checkbox" class="delete-cb" v-model="selectedBackendIds" :value="be.id" />
          <OsIcon :icon="getBackendIcon(be.backendType)" :size="12" />
          <span class="del-be-name">{{ be.name || be.backendType }}</span>
          <OsBadge v-if="!be.enabled" variant="danger" size="xs">DISABLED</OsBadge>
        </label>
      </div>
      <div v-if="backends.length === 0" class="text-muted" style="padding: 4px 0;">
        NO SYNC BACKENDS CONFIGURED
      </div>
    </div>

    <OsDivider spacing="sm" />

    <div class="delete-options">
      <label class="delete-option">
        <input type="checkbox" class="delete-cb" v-model="deleteMetadataOnly" />
        <span>DELETE METADATA ONLY</span>
      </label>
      <label class="delete-option">
        <input type="checkbox" class="delete-cb" v-model="deleteFromBackends" />
        <span>ALSO DELETE FROM SELECTED BACKENDS</span>
      </label>
    </div>

    <div v-if="backendErrors.length > 0" class="delete-backend-errors">
      <div v-for="(err, ei) in backendErrors" :key="ei" class="delete-backend-error">
        {{ err }}
      </div>
    </div>

    <template #footer>
      <OsButton variant="ghost" size="sm" :disabled="isDeleting" @click="onCancel">CANCEL</OsButton>
      <OsButton variant="ghost" size="sm" :disabled="isDeleting" @click="onMove">MOVE...</OsButton>
      <OsButton
        v-if="deleteMetadataOnly"
        variant="danger"
        size="sm"
        :loading="isDeleting"
        @click="onDeleteMetadata"
      >DELETE METADATA</OsButton>
      <OsButton
        v-else
        variant="danger"
        size="sm"
        :loading="isDeleting"
        @click="onDelete"
      >DELETE</OsButton>
    </template>
  </OsModal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useAppStore } from '@/stores/app'
import { useHistoryStore } from '@/stores/history'
import { OsModal, OsButton, OsIcon, OsBadge, OsDivider } from '@/components/ui'
import type { FileNode, SyncConfig, SyncBackendType } from '@/types'

const props = defineProps<{ visible: boolean; fileIds: string[] }>()
const emit = defineEmits<{ close: []; move: [fileIds: string[]]; unbox: [file: FileNode] }>()

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
  if (!v) return
  files.value = props.fileIds.map(id => store.files.find(f => f.id === id)).filter((f): f is FileNode => !!f)
  backends.value = store.syncConfigs
  selectedBackendIds.value = store.syncConfigs.filter(c => c.enabled).map(c => c.id)
  deleteMetadataOnly.value = false
  deleteFromBackends.value = false
  isDeleting.value = false
  backendErrors.value = []
})

function getFileIcon(f: FileNode): string {
  if (f.fileType === 'folder') return 'mdi:folder-outline'
  if (f.encrypted) return 'mdi:lock-outline'
  if (f.compressionLayers?.length && f.compressionLayers[0] !== 'none') return 'mdi:package-variant-closed'
  if (f.mimeType?.startsWith('image/')) return 'mdi:file-image-outline'
  if (f.mimeType?.startsWith('text/') || f.mimeType?.includes('json')) return 'mdi:file-document-outline'
  return 'mdi:file-outline'
}

function getBackendIcon(t: SyncBackendType): string {
  const map: Record<string, string> = { local: 'mdi:harddisk', github: 'mdi:github', gitlab: 'mdi:gitlab', googleDrive: 'mdi:google-drive', googlePhotos: 'mdi:google-photos', telegram: 'mdi:telegram', mega: 'mdi:cloud-upload-outline' }
  return map[t] || 'mdi:cloud-outline'
}

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0; let s = bytes
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
      history.push('file:delete', `DELETED "${f.name}"`, [f.id],
        { source: 'store', cmd: 'restoreTrashItem', args: { fileId: f.id } },
        { source: 'store', cmd: 'deleteFile', args: { fileId: f.id } })
    }
    if (targets.length > 0) {
      try {
        const { invoke } = await import('@/composables/useTauri')
        await invoke('delete_files_from_backends', { fileIds: props.fileIds, backendConfigIds: targets })
      } catch (e) { backendErrors.value.push(e instanceof Error ? e.message : String(e)) }
    }
    store.notifySuccess(`Deleted ${files.value.length} file(s)`)
    await store.fetchTrashItems()
    emit('close')
  } catch (e) { store.notifyError('Delete failed', e) }
  finally { isDeleting.value = false }
}

async function onDeleteMetadata() {
  try {
    const { invoke } = await import('@/composables/useTauri')
    for (const f of files.value) await invoke('delete_file_metadata_only', { fileId: f.id })
    await store.fetchFiles()
    store.notifySuccess(`Deleted metadata for ${files.value.length} file(s)`)
    emit('close')
  } catch (e) { store.notifyError('Failed to delete metadata', e) }
}

function onMove() { emit('move', props.fileIds) }
function onUnbox(f: FileNode) { emit('unbox', f) }
function onCancel() { emit('close') }
</script>

<style scoped>
.delete-files { display: flex; flex-direction: column; gap: 4px; max-height: 160px; overflow-y: auto; }
.delete-file-row { display: flex; align-items: center; gap: 8px; padding: 6px 8px; background: var(--bg-surface); border-radius: var(--radius-sm); }
.delete-file-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
.delete-file-name { font-size: var(--font-size-sm); font-weight: 600; color: var(--text-primary); }
.delete-file-meta { font-size: var(--font-size-xs); color: var(--text-muted); }
.delete-scope, .delete-options { display: flex; flex-direction: column; gap: 4px; }
.delete-section-title { font-size: var(--font-size-xs); font-weight: 700; color: var(--text-muted); letter-spacing: 0.5px; margin-bottom: 4px; }
.delete-backends { display: flex; flex-direction: column; gap: 2px; }
.delete-option { display: flex; align-items: center; gap: 6px; padding: 4px 6px; cursor: pointer; border-radius: var(--radius-sm); font-size: var(--font-size-sm); color: var(--text-secondary); transition: background var(--transition-fast); }
.delete-option:hover { background: var(--bg-overlay); }
.delete-cb { appearance: none; width: 12px; height: 12px; border: 1px solid var(--text-muted); background: var(--bg-surface); cursor: pointer; border-radius: 2px; flex-shrink: 0; }
.delete-cb:checked { background: var(--accent); border-color: var(--accent); }
.del-be-name { flex: 1; }
.delete-backend-errors { display: flex; flex-direction: column; gap: 2px; }
.delete-backend-error { font-size: var(--font-size-xs); color: var(--text-danger); padding: 2px 6px; background: var(--danger-dim); border-radius: var(--radius-sm); }
.text-muted { color: var(--text-muted) !important; }
.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
