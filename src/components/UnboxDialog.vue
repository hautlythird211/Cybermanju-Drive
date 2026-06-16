<template>
  <Teleport to="body">
    <div v-if="visible" class="unbox-overlay" @click.self="onClose">
      <div ref="dialogRef" class="unbox-modal">
        <div class="unbox-header">
          <div class="unbox-title">UNBOX — {{ file?.name || '' }}</div>
          <button class="unbox-close" @click="onClose">X</button>
        </div>

        <div v-if="file" class="unbox-body">
          <!-- Preview thumbnail -->
          <div v-if="file.thumbnailPath" class="unbox-preview">
            <img :src="file.thumbnailPath" class="unbox-thumb" alt="preview" />
          </div>
          <div v-else class="unbox-preview unbox-no-thumb">
            <Icon :icon="getFileIcon(file)" width="48" height="48" class="no-thumb-icon" />
            <span class="text-muted">NO PREVIEW AVAILABLE</span>
          </div>

          <!-- Metadata -->
          <div class="unbox-meta-grid">
            <div class="unbox-meta-item">
              <span class="unbox-label">SIZE</span>
              <span class="unbox-value">{{ formatSize(file.sizeBytes) }}</span>
            </div>
            <div class="unbox-meta-item">
              <span class="unbox-label">TYPE</span>
              <span class="unbox-value">{{ file.mimeType || file.fileType }}</span>
            </div>
            <div class="unbox-meta-item">
              <span class="unbox-label">CREATED</span>
              <span class="unbox-value">{{ formatDate(file.createdAt) }}</span>
            </div>
            <div class="unbox-meta-item">
              <span class="unbox-label">MODIFIED</span>
              <span class="unbox-value">{{ formatDate(file.modifiedAt) }}</span>
            </div>
            <div class="unbox-meta-item" v-if="file.hashBlake3">
              <span class="unbox-label">BLAKE3</span>
              <span class="unbox-value mono">{{ file.hashBlake3.slice(0, 16) }}...</span>
            </div>
          </div>

          <!-- Compression layers -->
          <div v-if="file.compressionLayers?.length && file.compressionLayers[0] !== 'none'" class="unbox-layers">
            <div class="unbox-section-title">COMPRESSION LAYERS</div>
            <div class="unbox-layer-row" v-for="(layer, li) in file.compressionLayers" :key="li">
              <span class="unbox-layer-num">#{{ li + 1 }}</span>
              <span class="unbox-layer-algo">{{ layer.toUpperCase() }}</span>
              <span v-if="li === 0" class="unbox-layer-status">RAW INPUT</span>
              <span v-else class="unbox-layer-status">STAGE {{ li }}</span>
            </div>
          </div>

          <!-- Encryption -->
          <div v-if="file.encrypted" class="unbox-enc">
            <div class="unbox-section-title">ENCRYPTION</div>
            <span class="unbox-enc-badge">{{ file.encryptionAlgorithm?.toUpperCase() || 'ENCRYPTED' }}</span>
          </div>

          <!-- Tags -->
          <div v-if="file.tags?.length" class="unbox-tags">
            <div class="unbox-section-title">TAGS</div>
            <div class="unbox-tag-list">
              <span v-for="tag in file.tags" :key="tag" class="unbox-tag">{{ tag }}</span>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="unbox-actions">
          <button class="ub-btn" @click="onClose" :disabled="isLoading">[CLOSE]</button>
          <button
            v-if="file && file.compressionLayers?.length && file.compressionLayers[0] !== 'none'"
            class="ub-btn ub-extract"
            :disabled="isLoading"
            @click="onDecompress"
          >{{ isLoading ? '[EXTRACTING...]' : '[DECOMPRESS & EXTRACT]' }}</button>
          <button
            v-if="file && file.encrypted"
            class="ub-btn ub-decrypt"
            :disabled="isLoading"
            @click="onDecrypt"
          >{{ isLoading ? '[DECRYPTING...]' : '[DECRYPT]' }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { ref } from 'vue'
import { useAppStore } from '@/stores/app'
import type { FileNode } from '@/types'

const props = defineProps<{
  visible: boolean
  file: FileNode | null
}>()

const emit = defineEmits<{
  close: []
}>()

const store = useAppStore()
const isLoading = ref(false)

function getFileIcon(f: FileNode): string {
  if (f.fileType === 'folder') return 'mdi:folder-outline'
  if (f.mimeType?.startsWith('image/')) return 'mdi:file-image-outline'
  if (f.mimeType?.startsWith('text/') || f.mimeType?.includes('json')) return 'mdi:file-document-outline'
  if (f.mimeType?.startsWith('audio/')) return 'mdi:file-music-outline'
  if (f.mimeType?.startsWith('video/')) return 'mdi:file-video-outline'
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
    if (updated) {
      Object.assign(props.file, updated)
    }
  } catch (e) {
    store.notifyError('Decompression failed', e)
  } finally {
    isLoading.value = false
  }
}

async function onDecrypt() {
  if (!props.file) return
  isLoading.value = true
  try {
    await store.decryptFile(props.file.id)
    const updated = await store.getFile(props.file.id)
    if (updated) {
      Object.assign(props.file, updated)
    }
  } catch (e) {
    store.notifyError('Decryption failed', e)
  } finally {
    isLoading.value = false
  }
}

function onClose() { emit('close') }
</script>

<style scoped>
.unbox-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.85);
  display: flex; align-items: center; justify-content: center; z-index: 10001;
}

.unbox-modal {
  background: #1a1a1a; border: 1px solid #333; border-radius: 10px;
  width: 420px; max-width: 94%; max-height: 80vh;
  display: flex; flex-direction: column;
  font-family: 'Courier New', monospace;
  box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}

.unbox-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 16px; border-bottom: 1px solid #2a2a2a;
}
.unbox-title { font-size: 11px; font-weight: 700; color: #e0e0e0; letter-spacing: 1px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
.unbox-close {
  background: none; border: 1px solid #444; color: #888;
  width: 22px; height: 22px; cursor: pointer; border-radius: 4px;
  font-size: 10px; font-family: inherit; flex-shrink: 0; margin-left: 8px;
}
.unbox-close:hover { background: #ff5f57; color: #fff; border-color: #ff5f57; }

.unbox-body {
  padding: 12px 16px; overflow-y: auto; flex: 1;
}

.unbox-preview {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: 16px; background: #111; border-radius: 8px; margin-bottom: 12px;
  min-height: 100px;
}
.unbox-thumb { max-width: 100%; max-height: 200px; border-radius: 4px; object-fit: contain; }
.unbox-no-thumb { gap: 8px; }
.no-thumb-icon { color: #444; }

.unbox-meta-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin-bottom: 12px;
}
.unbox-meta-item {
  display: flex; flex-direction: column; gap: 1px;
  padding: 6px 8px; background: #151515; border-radius: 4px;
}
.unbox-label { font-size: 8px; color: #666; letter-spacing: 0.3px; }
.unbox-value { font-size: 10px; color: #ccc; }
.unbox-value.mono { font-family: 'Courier New', monospace; font-size: 9px; }

.unbox-section-title { font-size: 9px; font-weight: 700; color: #999; margin-bottom: 4px; letter-spacing: 0.5px; }

.unbox-layers { margin-bottom: 12px; }
.unbox-layer-row {
  display: flex; align-items: center; gap: 8px;
  padding: 5px 8px; background: #151515; border-radius: 4px; margin-bottom: 2px;
  font-size: 9px;
}
.unbox-layer-num { color: #666; font-weight: 700; width: 20px; }
.unbox-layer-algo { flex: 1; color: #ccc; font-weight: 600; }
.unbox-layer-status { color: #555; font-size: 8px; }

.unbox-enc { margin-bottom: 12px; }
.unbox-enc-badge {
  display: inline-block; padding: 3px 10px;
  background: rgba(255, 179, 71, 0.1); border: 1px solid rgba(255, 179, 71, 0.2);
  color: #ffb347; font-size: 9px; font-weight: 700; border-radius: 4px;
}

.unbox-tags { margin-bottom: 12px; }
.unbox-tag-list { display: flex; flex-wrap: wrap; gap: 3px; }
.unbox-tag {
  padding: 2px 8px; background: #151515; border: 1px solid #2a2a2a;
  border-radius: 3px; font-size: 8px; color: #888;
}

.unbox-actions {
  display: flex; gap: 6px; justify-content: flex-end;
  padding: 12px 16px; border-top: 1px solid #2a2a2a;
}

.ub-btn {
  padding: 5px 14px; font-family: inherit; font-size: 10px; font-weight: 700;
  cursor: pointer; border: 1px solid #444; border-radius: 5px;
  background: transparent; color: #ccc; transition: all 0.1s;
}
.ub-btn:hover { border-color: #666; background: #222; }
.ub-btn:disabled { opacity: 0.3; cursor: not-allowed; }
.ub-extract { color: #58d68d; border-color: #58d68d; }
.ub-extract:hover { background: rgba(88,214,141,0.1); }
.ub-decrypt { color: #ffb347; border-color: #ffb347; }
.ub-decrypt:hover { background: rgba(255,179,71,0.1); }

.text-muted { color: #555 !important; }
</style>
