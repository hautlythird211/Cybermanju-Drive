<template>
  <Teleport to="body">
    <div v-if="visible" class="import-overlay" @click.self="close">
      <div class="import-modal">
        <div class="import-header">
          <div class="import-title">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#00ff41" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            IMPORT FROM {{ providerLabel }}
          </div>
          <button class="import-close" @click="close">✕</button>
        </div>

        <div v-if="loading" class="import-loading">
          <Icon icon="svg-spinners:blocks-wave" width="20" height="20" />
          <span>LISTING FILES...</span>
        </div>

        <div v-else-if="error" class="import-error">{{ error }}</div>

        <template v-else>
          <div class="import-toolbar">
            <label class="import-select-all">
              <input type="checkbox" :checked="allSelected" :indeterminate="someSelected && !allSelected" @change="toggleAll" />
              <span>{{ selectedCount }} / {{ files.length }} SELECTED</span>
            </label>
            <div class="import-type-filter" v-if="hasPhotos">
              <button :class="{ active: filter === 'all' }" @click="filter = 'all'">ALL</button>
              <button :class="{ active: filter === 'drive' }" @click="filter = 'drive'">FILES</button>
              <button :class="{ active: filter === 'photos' }" @click="filter = 'photos'">PHOTOS</button>
            </div>
          </div>

          <div class="import-list">
            <div
              v-for="(f, i) in filteredFiles"
              :key="f.path"
              class="import-file"
              :class="{ checked: f.selected }"
              @click="f.selected = !f.selected"
            >
              <input type="checkbox" v-model="f.selected" @click.stop />
              <div class="import-file-icon">
                <svg v-if="isImage(f)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#45B7D1" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
                  <circle cx="8.5" cy="8.5" r="1.5"/>
                  <polyline points="21 15 16 10 5 21"/>
                </svg>
                <svg v-else-if="isVideo(f)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#96CEB4" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="23 7 16 12 23 17 23 7"/>
                  <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
                </svg>
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#888" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                </svg>
              </div>
              <div class="import-file-info">
                <div class="import-file-name">{{ f.name }}</div>
                <div class="import-file-meta">
                  <span v-if="f.sizeBytes > 0">{{ formatSize(f.sizeBytes) }}</span>
                  <span v-if="f.modifiedAt"> · {{ formatDate(f.modifiedAt) }}</span>
                </div>
              </div>
            </div>
            <div v-if="filteredFiles.length === 0" class="import-empty">NO FILES FOUND</div>
          </div>
        </template>

        <div class="import-footer">
          <button class="bw-btn" @click="close">SKIP</button>
          <button
            class="bw-btn bw-btn-inverse"
            :disabled="selectedCount === 0 || importing"
            @click="importSelected"
          >
            <Icon v-if="importing" icon="svg-spinners:blocks-wave" width="12" height="12" />
            {{ importing ? 'IMPORTING...' : `[ IMPORT ${selectedCount} ]` }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import type { RemoteFileInfo } from '@/wasm/sync'

const store = useAppStore()

interface FileEntry extends RemoteFileInfo {
  selected: boolean
  type: 'file' | 'image' | 'video'
}

const props = defineProps<{
  visible: boolean
  backendType: string
  token: string
  label: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'import', files: RemoteFileInfo[]): void
}>()

const loading = ref(false)
const error = ref('')
const files = ref<FileEntry[]>([])
const importing = ref(false)
const filter = ref<'all' | 'drive' | 'photos'>('all')

const providerLabel = computed(() => props.label || props.backendType)
const hasPhotos = computed(() => props.backendType === 'google')
const filteredFiles = computed(() => {
  if (filter.value === 'all') return files.value
  if (props.backendType === 'google') {
    if (filter.value === 'drive') return files.value.filter(f => f.type === 'file')
    if (filter.value === 'photos') return files.value.filter(f => f.type !== 'file')
  }
  return files.value
})
const allSelected = computed(() => files.value.length > 0 && files.value.every(f => f.selected))
const someSelected = computed(() => files.value.some(f => f.selected))
const selectedCount = computed(() => files.value.filter(f => f.selected).length)

function isImage(f: FileEntry) { return f.type === 'image' }
function isVideo(f: FileEntry) { return f.type === 'video' }

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  } catch { return iso }
}

function toggleAll() {
  const newVal = !allSelected.value
  files.value.forEach(f => f.selected = newVal)
}

function classifyFile(name: string): 'image' | 'video' | 'file' {
  const ext = name.split('.').pop()?.toLowerCase() || ''
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg', 'tiff', 'avif', 'heic', 'heif'].includes(ext)) return 'image'
  if (['mp4', 'mov', 'avi', 'mkv', 'webm', 'wmv', 'flv', 'm4v', '3gp', 'ogv'].includes(ext)) return 'video'
  return 'file'
}

async function fetchFiles() {
  loading.value = true
  error.value = ''
  try {
    const { sync } = await import('@/wasm')
    const config = {
      id: '',
      name: props.label,
      backendType: props.backendType === 'google' ? 'googleDrive' : props.backendType,
      enabled: true,
      basePath: '/',
      autoSync: false,
      compressBeforeSync: false,
      maxConcurrentOps: 1,
      createdAt: '',
      updatedAt: '',
    }
    const remoteFiles = await sync.listRemoteFiles(config as any, '')
    files.value = (remoteFiles || [])
      .filter(f => f && f.name && !f.name.startsWith('.'))
      .map(f => ({ ...f, selected: false, type: classifyFile(f.name || 'unknown') }))
  } catch (e) {
    error.value = `FAILED TO LIST FILES: ${e instanceof Error ? e.message : String(e)}`
  } finally {
    loading.value = false
  }
}

async function importSelected() {
  if (selectedCount.value === 0 || importing.value) return
  importing.value = true
  try {
    const selected = files.value.filter(f => f.selected)
    const { sync } = await import('@/wasm')
    for (const file of selected) {
      if (props.backendType !== 'google' && file.url) {
        const drive = await import('@/wasm/drive')
        await drive.importFromUrl(file.url, null)
      }
    }
    emit('import', selected)
    emit('close')
  } catch (e) {
    error.value = `IMPORT FAILED: ${e instanceof Error ? e.message : String(e)}`
  } finally {
    importing.value = false
  }
}

function close() {
  if (importing.value) return
  emit('close')
}

watch(() => props.visible, (v) => {
  if (v) {
    files.value = []
    filter.value = 'all'
    loading.value = true
    error.value = ''
    fetchFiles()
  }
})
</script>

<style scoped>
.import-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10003;
  backdrop-filter: blur(4px);
}

.import-modal {
  width: 560px;
  max-width: 94vw;
  max-height: 80vh;
  background: #0d0d0d;
  border: 1px solid #1a1a1a;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 80px rgba(0, 255, 65, 0.04), 0 8px 32px rgba(0, 0, 0, 0.6);
  overflow: hidden;
}

.import-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 18px;
  border-bottom: 1px solid #1a1a1a;
}

.import-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 800;
  color: #e0e0e0;
  letter-spacing: 1px;
}

.import-title svg {
  flex-shrink: 0;
}

.import-close {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid #2a2a2a;
  border-radius: 6px;
  color: #555;
  font-size: 10px;
  cursor: pointer;
  transition: all 0.15s;
}

.import-close:hover {
  border-color: #ff5f57;
  color: #ff5f57;
}

.import-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 48px 20px;
  font-size: 11px;
  color: #888;
  letter-spacing: 1px;
}

.import-error {
  padding: 20px;
  font-size: 10px;
  color: #ff5f57;
  text-align: center;
}

.import-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 18px;
  border-bottom: 1px solid #0a0a0a;
  background: #080808;
}

.import-select-all {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 10px;
  color: #888;
  cursor: pointer;
  user-select: none;
}

.import-select-all input {
  accent-color: #00ff41;
}

.import-type-filter {
  display: flex;
  gap: 4px;
}

.import-type-filter button {
  padding: 3px 10px;
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 4px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 8px;
  font-weight: 700;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.12s;
}

.import-type-filter button.active {
  border-color: #00ff41;
  color: #00ff41;
  background: rgba(0, 255, 65, 0.04);
}

.import-type-filter button:hover {
  border-color: #333;
  color: #888;
}

.import-list {
  flex: 1;
  overflow-y: auto;
  max-height: 360px;
  padding: 4px 0;
}

.import-list::-webkit-scrollbar {
  width: 4px;
}
.import-list::-webkit-scrollbar-track {
  background: transparent;
}
.import-list::-webkit-scrollbar-thumb {
  background: #1a1a1a;
  border-radius: 2px;
}

.import-file {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 18px;
  cursor: pointer;
  transition: background 0.1s;
}

.import-file:hover {
  background: rgba(255, 255, 255, 0.03);
}

.import-file.checked {
  background: rgba(0, 255, 65, 0.03);
}

.import-file input {
  accent-color: #00ff41;
  flex-shrink: 0;
}

.import-file-icon {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: 4px;
  background: #0a0a0a;
}

.import-file-info {
  flex: 1;
  min-width: 0;
}

.import-file-name {
  font-size: 11px;
  font-weight: 600;
  color: #ccc;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.import-file-meta {
  font-size: 9px;
  color: #555;
  margin-top: 1px;
}

.import-empty {
  padding: 32px 18px;
  text-align: center;
  font-size: 10px;
  color: #444;
  letter-spacing: 1px;
}

.import-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 18px 16px;
  border-top: 1px solid #1a1a1a;
}

.bw-btn {
  background: transparent;
  border: 1px solid #2a2a2a;
  border-radius: 6px;
  color: #888;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 7px 18px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  gap: 6px;
}

.bw-btn:hover {
  border-color: #444;
  color: #ccc;
}

.bw-btn-inverse {
  border-color: #00ff41;
  color: #00ff41;
  text-shadow: 0 0 4px rgba(0, 255, 65, 0.15);
}

.bw-btn-inverse:hover {
  background: rgba(0, 255, 65, 0.08);
  box-shadow: 0 0 12px rgba(0, 255, 65, 0.1);
}

.bw-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
</style>
