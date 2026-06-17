<template>
  <OsModal
    :visible="visible"
    :title="'UNBOX — ' + (file?.name || '')"
    variant="glass"
    size="sm"
    closable
    @update:visible="onClose"
  >
    <div v-if="file" class="unbox-body">
      <div v-if="file.thumbnailPath" class="unbox-preview">
        <img :src="file.thumbnailPath" class="unbox-thumb" alt="preview" />
      </div>
      <div v-else class="unbox-preview unbox-no-thumb">
        <OsIcon :icon="getFileIcon(file)" :size="48" />
        <span class="text-muted">NO PREVIEW AVAILABLE</span>
      </div>

      <div class="unbox-meta-grid">
        <div class="unbox-meta-item"><span class="unbox-label">SIZE</span><span class="unbox-value">{{ formatSize(file.sizeBytes) }}</span></div>
        <div class="unbox-meta-item"><span class="unbox-label">TYPE</span><span class="unbox-value">{{ file.mimeType || file.fileType }}</span></div>
        <div class="unbox-meta-item"><span class="unbox-label">CREATED</span><span class="unbox-value">{{ formatDate(file.createdAt) }}</span></div>
        <div class="unbox-meta-item"><span class="unbox-label">MODIFIED</span><span class="unbox-value">{{ formatDate(file.modifiedAt) }}</span></div>
        <div class="unbox-meta-item" v-if="file.hashBlake3"><span class="unbox-label">BLAKE3</span><span class="unbox-value mono">{{ file.hashBlake3.slice(0, 16) }}...</span></div>
      </div>

      <div v-if="file.compressionLayers?.length && file.compressionLayers[0] !== 'none'" class="unbox-layers">
        <div class="unbox-section-title">COMPRESSION LAYERS</div>
        <div class="unbox-layer-row" v-for="(layer, li) in file.compressionLayers" :key="li">
          <span class="unbox-layer-num">#{{ li + 1 }}</span>
          <span class="unbox-layer-algo">{{ layer.toUpperCase() }}</span>
          <OsBadge variant="accent" size="xs">{{ li === 0 ? 'RAW INPUT' : 'STAGE ' + li }}</OsBadge>
        </div>
      </div>

      <div v-if="file.encrypted" class="unbox-enc">
        <div class="unbox-section-title">ENCRYPTION</div>
        <OsBadge variant="gold">{{ file.encryptionAlgorithm?.toUpperCase() || 'ENCRYPTED' }}</OsBadge>
      </div>

      <div v-if="file.tags?.length" class="unbox-tags">
        <div class="unbox-section-title">TAGS</div>
        <div class="unbox-tag-list">
          <OsBadge v-for="tag in file.tags" :key="tag" variant="default" size="xs">{{ tag }}</OsBadge>
        </div>
      </div>
    </div>

    <template #footer>
      <OsButton variant="ghost" size="sm" :disabled="isLoading" @click="onClose">CLOSE</OsButton>
      <OsButton
        v-if="file && file.compressionLayers?.length && file.compressionLayers[0] !== 'none'"
        variant="neon"
        size="sm"
        :loading="isLoading"
        @click="onDecompress"
      >DECOMPRESS & EXTRACT</OsButton>
      <OsButton
        v-if="file && file.encrypted"
        variant="primary"
        size="sm"
        :loading="isLoading"
        @click="onDecrypt"
      >DECRYPT</OsButton>
    </template>
  </OsModal>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '@/stores/app'
import { OsModal, OsButton, OsIcon, OsBadge } from '@/components/ui'
import type { FileNode } from '@/types'

const props = defineProps<{ visible: boolean; file: FileNode | null }>()
const emit = defineEmits<{ close: [] }>()
const store = useAppStore()
const isLoading = ref(false)

function getFileIcon(f: FileNode): string {
  if (f.fileType === 'folder') return 'mdi:folder-outline'
  if (f.mimeType?.startsWith('image/')) return 'mdi:file-image-outline'
  if (f.mimeType?.startsWith('text/') || f.mimeType?.includes('json')) return 'mdi:file-document-outline'
  return 'mdi:file-outline'
}

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0; let s = bytes
  while (s >= 1024 && i < units.length - 1) { s /= 1024; i++ }
  return `${s.toFixed(1)} ${units[i]}`
}

function formatDate(d: string): string {
  if (!d) return '-'
  try { return new Date(d).toLocaleDateString() } catch { return d }
}

async function onDecompress() {
  if (!props.file) return
  isLoading.value = true
  try {
    await store.decompressFile(props.file.id)
    const updated = await store.getFile(props.file.id)
    if (updated) Object.assign(props.file, updated)
  } catch (e) { store.notifyError('Decompression failed', e) }
  finally { isLoading.value = false }
}

async function onDecrypt() {
  if (!props.file) return
  isLoading.value = true
  try {
    await store.decryptFile(props.file.id)
    const updated = await store.getFile(props.file.id)
    if (updated) Object.assign(props.file, updated)
  } catch (e) { store.notifyError('Decryption failed', e) }
  finally { isLoading.value = false }
}

function onClose() { emit('close') }
</script>

<style scoped>
.unbox-body { display: flex; flex-direction: column; gap: 12px; }
.unbox-preview { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 16px; background: var(--bg-surface); border-radius: var(--radius-lg); min-height: 100px; gap: 8px; }
.unbox-thumb { max-width: 100%; max-height: 200px; border-radius: var(--radius-sm); object-fit: contain; }
.unbox-meta-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
.unbox-meta-item { display: flex; flex-direction: column; gap: 1px; padding: 6px 8px; background: var(--bg-surface); border-radius: var(--radius-sm); }
.unbox-label { font-size: var(--font-size-xs); color: var(--text-muted); letter-spacing: 0.3px; }
.unbox-value { font-size: var(--font-size-sm); color: var(--text-secondary); }
.unbox-value.mono { font-size: var(--font-size-xs); }
.unbox-section-title { font-size: var(--font-size-xs); font-weight: 700; color: var(--text-muted); margin-bottom: 4px; letter-spacing: 0.5px; text-transform: uppercase; }
.unbox-layers { display: flex; flex-direction: column; gap: 2px; }
.unbox-layer-row { display: flex; align-items: center; gap: 8px; padding: 5px 8px; background: var(--bg-surface); border-radius: var(--radius-sm); font-size: var(--font-size-xs); }
.unbox-layer-num { color: var(--text-muted); font-weight: 700; width: 20px; }
.unbox-layer-algo { flex: 1; color: var(--text-secondary); font-weight: 600; }
.unbox-tag-list { display: flex; flex-wrap: wrap; gap: 3px; }
.text-muted { color: var(--text-muted) !important; }
</style>
