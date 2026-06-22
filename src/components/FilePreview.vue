<template>
  <aside v-if="store.selectedFile" class="file-preview">
    <div class="preview-header">
      <div class="preview-icon-wrapper" :class="getFileTypeClass(store.selectedFile)">
        <Icon :icon="getFileIcon(store.selectedFile)" width="24" height="24" />
      </div>
      <div class="preview-file-info">
        <span class="preview-filename truncate">{{ store.selectedFile.name }}</span>
        <span class="preview-path truncate text-muted">{{ store.selectedFile.path }}</span>
      </div>
    </div>

    <div class="preview-scroll">
      <!-- Preview Image -->
      <div v-if="store.selectedFile.thumbnailPath" class="preview-image-section">
        <img :src="store.selectedFile.thumbnailPath" class="preview-image" :alt="store.selectedFile.name" />
      </div>

      <!-- Resolution Picker (for media files) -->
      <div v-if="isMediaFile" class="preview-section">
        <div class="section-label">RESOLUTION</div>
        <div class="resolution-grid">
          <button
            v-for="res in resolutions"
            :key="res.level"
            class="res-btn"
            :class="{ active: selectedResolution === res.level }"
            @click="selectedResolution = res.level"
          >
            <div class="res-level">{{ res.label }}</div>
            <div class="res-desc">{{ res.desc }}</div>
            <div v-if="res.level === 'r3'" class="res-badge full">ORIGINAL</div>
            <div v-else-if="res.level === 'r2'" class="res-badge high">HD</div>
            <div v-else-if="res.level === 'r1'" class="res-badge mid">SD</div>
            <div v-else class="res-badge low">THUMB</div>
          </button>
        </div>
        <div class="resolution-info">
          <div class="res-info-row">
            <span class="res-info-key">Key Tier</span>
            <span class="res-info-value" :class="currentResKeyTier">{{ currentResKeyTier }}</span>
          </div>
          <div class="res-info-row">
            <span class="res-info-key">Encrypted</span>
            <span class="res-info-value">Yes (ChaCha20-Poly1305)</span>
          </div>
          <div class="res-info-row">
            <span class="res-info-key">Estimated Size</span>
            <span class="res-info-value">{{ currentResEstSize }}</span>
          </div>
        </div>
        <button class="open-viewer-btn" @click="openInViewer">
          <Icon icon="mdi:image-outline" width="14" height="14" />
          OPEN IN VIEWER
        </button>
      </div>

      <!-- Metadata -->
      <div class="preview-section">
        <div class="section-label">Info</div>
        <div class="meta-grid">
          <div class="meta-row">
            <span class="meta-key">Size</span>
            <span class="meta-value">{{ formatSize(store.selectedFile.sizeBytes) }}</span>
          </div>
          <div class="meta-row">
            <span class="meta-key">Type</span>
            <span class="meta-value">{{ store.selectedFile.mimeType || store.selectedFile.fileType }}</span>
          </div>
          <div class="meta-row">
            <span class="meta-key">Created</span>
            <span class="meta-value">{{ formatDate(store.selectedFile.createdAt) }}</span>
          </div>
          <div class="meta-row">
            <span class="meta-key">Modified</span>
            <span class="meta-value">{{ formatDate(store.selectedFile.modifiedAt) }}</span>
          </div>
        </div>
      </div>

      <!-- Tags Section -->
      <div class="preview-section">
        <div class="section-header">
          <span class="section-label">Tags</span>
          <button class="add-btn" @click="showTagInput = !showTagInput" title="Add tag">
            <Icon icon="mdi:plus" width="14" height="14" />
          </button>
        </div>
        <div v-if="showTagInput" class="tag-input-row">
          <input
            v-model="newTag"
            class="tag-input"
            placeholder="New tag..."
            @keyup.enter="addTag"
          />
          <button class="tag-add-btn" @click="addTag" :disabled="!newTag.trim()">Add</button>
        </div>
        <div v-if="store.selectedFile.tags && store.selectedFile.tags.length > 0" class="tag-list">
          <span
            v-for="tag in store.selectedFile.tags"
            :key="tag"
            class="tag-chip"
            :style="{ '--tag-color': getTagColor(tag) }"
          >
            <span class="tag-dot" :style="{ background: getTagColor(tag) }" />
            <span class="tag-name">{{ tag }}</span>
            <button class="tag-remove" @click="removeTag(tag)">
              <Icon icon="mdi:close" width="10" height="10" />
            </button>
          </span>
        </div>
        <div v-else class="empty-tags">
          <span class="text-muted">No tags</span>
        </div>
      </div>

      <!-- Collections Section -->
      <div class="preview-section" v-if="store.selectedFile.collectionIds && store.selectedFile.collectionIds.length > 0">
        <div class="section-label">Collections</div>
        <div class="collection-list">
          <div v-for="colId in store.selectedFile.collectionIds" :key="colId" class="collection-item">
            <Icon icon="mdi:folder-star-outline" width="14" height="14" class="collection-icon" />
            <span class="collection-name">{{ getCollectionName(colId) }}</span>
          </div>
        </div>
      </div>

      <!-- Encryption Info -->
      <div v-if="store.selectedFile.encrypted" class="preview-section">
        <div class="section-label">Encryption</div>
        <div class="info-card">
          <div class="info-row">
            <span class="info-key">Algorithm</span>
            <span class="info-badge">{{ store.selectedFile.encryptionAlgorithm?.toUpperCase() }}</span>
          </div>
          <div v-if="store.selectedFile.contextData?.keyId" class="info-row">
            <span class="info-key">Key ID</span>
            <span class="info-value mono">{{ String(store.selectedFile.contextData.keyId).substring(0, 16) }}..</span>
          </div>
        </div>
      </div>

      <!-- Compression Info -->
      <div v-if="store.selectedFile.compressionLayers && store.selectedFile.compressionLayers[0] && store.selectedFile.compressionLayers[0] !== 'none'" class="preview-section">
        <div class="section-label">Compression</div>
        <div class="info-card">
          <div class="info-row">
            <span class="info-key">Layer</span>
            <span class="info-badge">{{ store.selectedFile.compressionLayers[0]?.toUpperCase() }}</span>
          </div>
          <div v-if="store.selectedFile.hashBlake3" class="info-row">
            <span class="info-key">BLAKE3</span>
            <span class="info-value mono hash">{{ store.selectedFile.hashBlake3.substring(0, 20) }}..</span>
          </div>
        </div>
      </div>

      <!-- Face Groups -->
      <div v-if="store.selectedFile.faceGroupIds && store.selectedFile.faceGroupIds.length" class="preview-section">
        <div class="section-label">People</div>
        <div class="face-list">
          <div v-for="groupId in store.selectedFile.faceGroupIds" :key="groupId" class="face-item">
            <div class="face-avatar">
              <Icon icon="mdi:face-man-outline" width="14" height="14" />
            </div>
            <span class="face-name">{{ getFaceGroupName(groupId) }}</span>
          </div>
        </div>
      </div>

      <!-- GPS Location -->
      <div v-if="store.selectedFile.gpsLat" class="preview-section">
        <div class="section-label">Location</div>
        <div class="info-card">
          <div class="info-row">
            <span class="info-key">Coordinates</span>
            <span class="info-value mono">{{ store.selectedFile.gpsLat.toFixed(4) }}, {{ store.selectedFile.gpsLon?.toFixed(4) }}</span>
          </div>
        </div>
      </div>

      <!-- Code Symbols -->
      <div v-if="store.parseResult && store.parseResult.symbols.length" class="preview-section">
        <div class="section-label">Symbols ({{ store.parseResult.language }})</div>
        <div class="symbol-tree">
          <div v-for="sym in store.parseResult.symbols" :key="sym.name + sym.startLine" class="symbol-row">
            <span class="symbol-kind">{{ sym.kind }}</span>
            <span class="symbol-name">{{ sym.name }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="preview-actions">
      <button class="action-btn primary" @click="handleOpen" title="Open file">
        <Icon icon="mdi:folder-open-outline" width="16" height="16" />
        <span>Open</span>
      </button>
      <button class="action-btn" @click="handleStar" :class="{ starred: store.selectedFile.isStarred }" title="Star file">
        <Icon :icon="store.selectedFile.isStarred ? 'mdi:star' : 'mdi:star-outline'" width="16" height="16" />
      </button>
      <button class="action-btn" @click="handleCopyPath" title="Copy path">
        <Icon icon="mdi:content-copy" width="16" height="16" />
      </button>
      <button class="action-btn" @click="handleEncrypt" :title="store.selectedFile.encrypted ? 'Decrypt' : 'Encrypt'">
        <Icon :icon="store.selectedFile.encrypted ? 'mdi:lock-open-outline' : 'mdi:lock-outline'" width="16" height="16" />
      </button>
      <button class="action-btn" @click="handleCompress" title="Compress">
        <Icon icon="mdi:package-variant" width="16" height="16" />
      </button>
      <button class="action-btn danger" @click="handleDelete" title="Delete">
        <Icon icon="mdi:delete-outline" width="16" height="16" />
      </button>
    </div>
  </aside>

  <aside v-else class="file-preview empty-preview">
    <div class="empty-content">
      <Icon icon="mdi:file-outline" width="48" height="48" class="empty-icon" />
      <span class="empty-title">No file selected</span>
      <span class="empty-subtitle">Select a file to view details</span>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useMedia } from '@/composables/useMedia'
import type { FileNode, ResolutionLevel } from '@/types'

const store = useAppStore()
const { openMediaOverlay, getFileBytesForPreview, getMediaInfo } = useMedia()

const decryptError = ref<string | null>(null)
const showTagInput = ref(false)
const newTag = ref('')
const selectedResolution = ref<ResolutionLevel>('r3')
const openingInViewer = ref(false)

const resolutions = [
  { level: 'r0' as ResolutionLevel, label: 'R0', desc: '200x150 Thumb', estSize: '~3 KB' },
  { level: 'r1' as ResolutionLevel, label: 'R1', desc: '640x480 SD', estSize: '~45 KB' },
  { level: 'r2' as ResolutionLevel, label: 'R2', desc: '1920x1080 HD', estSize: '~450 KB' },
  { level: 'r3' as ResolutionLevel, label: 'R3', desc: 'Original', estSize: 'Full' },
]

const isMediaFile = computed(() => {
  if (!store.selectedFile) return false
  const mime = store.selectedFile.mimeType || ''
  return mime.startsWith('image/') || mime.startsWith('video/') || mime.startsWith('audio/')
})

const currentResKeyTier = computed(() => {
  return selectedResolution.value === 'r0' || selectedResolution.value === 'r1' ? 'preview' : 'content'
})

const currentResEstSize = computed(() => {
  const res = resolutions.find(r => r.level === selectedResolution.value)
  return res?.estSize || 'Unknown'
})

async function openInViewer() {
  if (!store.selectedFile || openingInViewer.value) return
  openingInViewer.value = true
  try {
    const file = store.selectedFile
    const mime = file.mimeType || ''
    const type = mime.startsWith('image/') ? 'image' : mime.startsWith('video/') ? 'video' : 'audio'

    // Uncompress-on-demand: fetch real bytes from disk
    const fileBytes = await getFileBytesForPreview(file.id)
    const mediaData = await getMediaInfo(file.id, file.name, fileBytes)

    openMediaOverlay(type, mediaData, fileBytes)
  } catch (e) {
    console.error('Failed to open in viewer:', e)
    store.notifyError?.(`Failed to open: ${e}`)
  } finally {
    openingInViewer.value = false
  }
}

// Tag color cache
const tagColorCache = new Map<string, string>()
const tagColors = [
  '#00ff41', '#5af0ff', '#ff5f57', '#ffd700', '#ff6b9d',
  '#b388ff', '#28c840', '#ff9933', '#3a86ff', '#ff006e'
]
let colorIndex = 0

function getTagColor(tagName: string): string {
  if (tagColorCache.has(tagName)) {
    return tagColorCache.get(tagName)!
  }
  const color = tagColors[colorIndex % tagColors.length]
  tagColorCache.set(tagName, color)
  colorIndex++
  return color
}

// File type helpers
function getFileTypeClass(file: FileNode): string {
  if (file.fileType === 'folder') return 'folder'
  const mime = file.mimeType || ''
  if (mime.startsWith('image/')) return 'image'
  if (mime.startsWith('video/')) return 'video'
  if (mime.startsWith('audio/')) return 'audio'
  if (mime.includes('zip') || mime.includes('tar') || mime.includes('gz')) return 'archive'
  if (mime.includes('json') || mime.includes('javascript') || mime.includes('typescript')) return 'code'
  if (mime.includes('pdf') || mime.includes('document') || mime.includes('text')) return 'document'
  return 'other'
}

function getFileIcon(file: FileNode): string {
  if (file.fileType === 'folder') return 'mdi:folder'
  const mime = file.mimeType || ''
  if (mime.startsWith('image/')) return 'mdi:image'
  if (mime.startsWith('video/')) return 'mdi:video'
  if (mime.startsWith('audio/')) return 'mdi:music'
  if (mime.includes('pdf')) return 'mdi:file-pdf-box'
  if (mime.includes('zip') || mime.includes('tar') || mime.includes('gz')) return 'mdi:zip-box'
  if (mime.includes('json')) return 'mdi:code-json'
  if (mime.includes('javascript')) return 'mdi:language-javascript'
  if (mime.includes('typescript')) return 'mdi:language-typescript'
  if (mime.includes('python')) return 'mdi:language-python'
  if (mime.includes('rust')) return 'mdi:language-rust'
  if (mime.includes('text')) return 'mdi:file-document-outline'
  return 'mdi:file-outline'
}

// Format helpers
function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const k = 1024
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + units[i]
}

function formatDate(dateStr: string): string {
  if (!dateStr) return '--'
  const d = new Date(dateStr)
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
}

// Collection helpers
function getCollectionName(colId: string): string {
  const col = store.collections.find(c => c.id === colId)
  return col?.name || 'Unknown'
}

// Face group helpers
function getFaceGroupName(groupId: string): string {
  const group = store.faceGroups.find(g => g.id === groupId)
  return group?.name || 'Unknown'
}

// Tag management
function addTag() {
  if (!newTag.value.trim() || !store.selectedFile) return
  // This would normally call a store action
  store.notifySuccess('Tag added: ' + newTag.value)
  newTag.value = ''
  showTagInput.value = false
}

function removeTag(tag: string) {
  // This would normally call a store action
  store.notifySuccess('Tag removed: ' + tag)
}

// Actions
function handleOpen() {
  if (store.selectedFile) {
    store.selectFile(store.selectedFile.id)
  }
}

function handleStar() {
  if (store.selectedFile) {
    store.toggleStar(store.selectedFile.id)
  }
}

function handleCopyPath() {
  if (store.selectedFile?.path) {
    navigator.clipboard.writeText(store.selectedFile.path)
    store.notifySuccess('Path copied to clipboard')
  }
}

function handleEncrypt() {
  if (store.selectedFile) {
    if (store.selectedFile.encrypted) {
      store.notifySuccess('Decrypt: ' + store.selectedFile.name)
    } else {
      store.encryptFile(store.selectedFile.id, 'hybrid')
    }
  }
}

function handleCompress() {
  if (store.selectedFile) {
    store.compressFile(store.selectedFile.id, 'zstd')
  }
}

function handleDelete() {
  if (store.selectedFile) {
    window.dispatchEvent(new CustomEvent('cybermanju:show-delete-dialog', { detail: { fileIds: [store.selectedFile.id] } }))
  }
}
</script>

<style scoped>
.file-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-elevated);
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', Roboto, sans-serif;
  color: var(--text-primary);
  overflow: hidden;
}

.preview-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-bottom: 1px solid var(--border-glass);
}

.preview-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: 12px;
  flex-shrink: 0;
}

.preview-icon-wrapper.folder { background: var(--accent-dim); color: var(--text-accent); }
.preview-icon-wrapper.image { background: rgba(255, 107, 157, 0.1); color: var(--text-pink); }
.preview-icon-wrapper.video { background: rgba(179, 136, 255, 0.1); color: var(--text-purple); }
.preview-icon-wrapper.audio { background: rgba(90, 240, 255, 0.1); color: var(--text-info); }
.preview-icon-wrapper.document { background: rgba(255, 215, 0, 0.1); color: var(--text-gold); }
.preview-icon-wrapper.archive { background: rgba(255, 152, 51, 0.1); color: #ff9933; }
.preview-icon-wrapper.code { background: rgba(40, 200, 64, 0.1); color: #28c840; }
.preview-icon-wrapper.other { background: rgba(136, 136, 136, 0.1); color: #888; }

.preview-file-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.preview-filename {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.preview-path {
  font-size: 12px;
  color: var(--text-muted);
}

.preview-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.preview-image-section {
  margin-bottom: 16px;
  border-radius: 12px;
  overflow: hidden;
  background: var(--bg-surface);
  border: 1px solid var(--border-glass);
}

.preview-image {
  width: 100%;
  height: auto;
  max-height: 200px;
  object-fit: cover;
}

.resolution-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
  margin-bottom: 10px;
}

.res-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 8px 6px;
  background: #16161c;
  border: 1px solid #22222a;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
  font-family: inherit;
}

.res-btn:hover {
  background: #1e1e26;
  border-color: #333;
}

.res-btn.active {
  background: rgba(0, 255, 65, 0.08);
  border-color: #00ff41;
}

.res-level {
  font-size: 11px;
  font-weight: 800;
  color: #ececf0;
  letter-spacing: 1px;
}

.res-btn.active .res-level {
  color: #00ff41;
}

.res-desc {
  font-size: 8px;
  color: #50505e;
  letter-spacing: 0.5px;
}

.res-badge {
  font-size: 7px;
  font-weight: 800;
  padding: 1px 4px;
  border-radius: 3px;
  letter-spacing: 0.5px;
  margin-top: 2px;
}

.res-badge.full { background: rgba(0, 255, 65, 0.15); color: #00ff41; }
.res-badge.high { background: rgba(179, 136, 255, 0.15); color: #b388ff; }
.res-badge.mid { background: rgba(90, 240, 255, 0.15); color: #5af0ff; }
.res-badge.low { background: rgba(80, 80, 94, 0.3); color: #50505e; }

.resolution-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 10px;
}

.res-info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 9px;
}

.res-info-key { color: #50505e; letter-spacing: 0.5px; }
.res-info-value { color: #a0a0b0; font-weight: 600; }
.res-info-value.preview { color: #5af0ff; }
.res-info-value.content { color: #b388ff; }

.open-viewer-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 8px;
  background: rgba(0, 255, 65, 0.1);
  border: 1px solid rgba(0, 255, 65, 0.2);
  border-radius: 8px;
  color: #00ff41;
  font-family: inherit;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.15s;
}

.open-viewer-btn:hover {
  background: rgba(0, 255, 65, 0.15);
  border-color: rgba(0, 255, 65, 0.4);
}

.preview-section {
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 12px;
}

.section-header .section-label {
  margin-bottom: 0;
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: var(--accent-dim);
  border: none;
  border-radius: 6px;
  color: var(--text-accent);
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-btn:hover {
  background: var(--accent-dim);
  opacity: 0.8;
}

.meta-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-radius: 12px;
  padding: 12px;
  border: 1px solid var(--border-glass);
}

.meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.meta-key {
  font-size: 13px;
  color: var(--text-muted);
}

.meta-value {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.tag-input-row {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.tag-input {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-primary);
}

.tag-input:focus {
  outline: none;
  border-color: var(--accent);
}

.tag-add-btn {
  padding: 8px 16px;
  background: var(--accent);
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-inverse);
  cursor: pointer;
  transition: background 0.2s ease;
}

.tag-add-btn:hover:not(:disabled) {
  background: var(--accent);
  opacity: 0.85;
}

.tag-add-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border: 1px solid var(--border-glass);
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.tag-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.tag-name {
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  background: rgba(0, 0, 0, 0.1);
  border: none;
  border-radius: 50%;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s ease;
}

.tag-remove:hover {
  background: rgba(255, 59, 48, 0.2);
  color: #ff3b30;
}

.empty-tags {
  padding: 12px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-radius: 8px;
  text-align: center;
}

.collection-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.collection-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-radius: 8px;
  border: 1px solid var(--border-glass);
}

.collection-icon {
  color: #ffd700;
}

.collection-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.info-card {
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-radius: 12px;
  padding: 12px;
  border: 1px solid var(--border-glass);
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

.info-row:not(:last-child) {
  border-bottom: 1px solid var(--border-glass);
}

.info-key {
  font-size: 13px;
  color: var(--text-muted);
}

.info-value {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.info-value.mono {
  font-family: 'SF Mono', Monaco, monospace;
}

.info-value.hash {
  font-size: 11px;
  word-break: break-all;
}

.info-badge {
  padding: 4px 8px;
  background: var(--accent-dim);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-accent);
}

.face-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.face-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-radius: 8px;
  border: 1px solid var(--border-glass);
}

.face-avatar {
  width: 32px;
  height: 32px;
  background: linear-gradient(135deg, #ff6b9d 0%, #b388ff 100%);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.face-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.symbol-tree {
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-radius: 12px;
  padding: 12px;
  border: 1px solid var(--border-glass);
  max-height: 200px;
  overflow-y: auto;
}

.symbol-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}

.symbol-row:not(:last-child) {
  border-bottom: 1px solid var(--border-glass);
}

.symbol-kind {
  font-size: 11px;
  padding: 2px 6px;
  background: var(--bg-surface);
  border-radius: 4px;
  color: var(--text-muted);
}

.symbol-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  font-family: 'SF Mono', Monaco, monospace;
}

.preview-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 20px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border-top: 1px solid var(--border-glass);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: var(--bg-surface);
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: var(--border-glass);
}

.action-btn.primary {
  background: var(--accent);
  color: var(--text-inverse);
}

.action-btn.primary:hover {
  background: var(--accent);
  opacity: 0.85;
}

.action-btn.starred {
  background: rgba(255, 204, 0, 0.2);
  color: #ff9500;
}

.action-btn.danger {
  color: #ff3b30;
}

.action-btn.danger:hover {
  background: rgba(255, 59, 48, 0.1);
}

/* Empty State */
.empty-preview {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
}

.empty-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.empty-icon {
  color: var(--text-muted);
  margin-bottom: 8px;
}

.empty-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.empty-subtitle {
  font-size: 14px;
  color: var(--text-muted);
}

/* Scrollbar */
.preview-scroll::-webkit-scrollbar {
  width: 6px;
}

.preview-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.preview-scroll::-webkit-scrollbar-thumb {
  background: var(--border-glass);
  border-radius: 3px;
}

.preview-scroll::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}

.text-muted {
  color: var(--text-muted) !important;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
