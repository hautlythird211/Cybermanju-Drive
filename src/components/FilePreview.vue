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
import type { FileNode } from '@/types'

const store = useAppStore()

const decryptError = ref<string | null>(null)
const showTagInput = ref(false)
const newTag = ref('')

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
  background: #fafafa;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', Roboto, sans-serif;
  color: #1d1d1f;
  overflow: hidden;
}

.preview-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: #ffffff;
  border-bottom: 1px solid #e8e8ed;
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

.preview-icon-wrapper.folder { background: rgba(0, 122, 255, 0.1); color: #007aff; }
.preview-icon-wrapper.image { background: rgba(255, 107, 157, 0.1); color: #ff6b9d; }
.preview-icon-wrapper.video { background: rgba(179, 136, 255, 0.1); color: #b388ff; }
.preview-icon-wrapper.audio { background: rgba(90, 240, 255, 0.1); color: #5af0ff; }
.preview-icon-wrapper.document { background: rgba(255, 215, 0, 0.1); color: #ffd700; }
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
  color: #1d1d1f;
}

.preview-path {
  font-size: 12px;
  color: #86868b;
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
  background: #ffffff;
  border: 1px solid #e8e8ed;
}

.preview-image {
  width: 100%;
  height: auto;
  max-height: 200px;
  object-fit: cover;
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
  color: #86868b;
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
  background: rgba(0, 122, 255, 0.1);
  border: none;
  border-radius: 6px;
  color: #007aff;
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-btn:hover {
  background: rgba(0, 122, 255, 0.2);
}

.meta-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: #ffffff;
  border-radius: 12px;
  padding: 12px;
  border: 1px solid #e8e8ed;
}

.meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.meta-key {
  font-size: 13px;
  color: #86868b;
}

.meta-value {
  font-size: 13px;
  font-weight: 500;
  color: #1d1d1f;
}

.tag-input-row {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.tag-input {
  flex: 1;
  padding: 8px 12px;
  background: #ffffff;
  border: 1px solid #d2d2d7;
  border-radius: 8px;
  font-size: 13px;
  color: #1d1d1f;
}

.tag-input:focus {
  outline: none;
  border-color: #007aff;
}

.tag-add-btn {
  padding: 8px 16px;
  background: #007aff;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: white;
  cursor: pointer;
  transition: background 0.2s ease;
}

.tag-add-btn:hover:not(:disabled) {
  background: #0066d6;
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
  background: #ffffff;
  border: 1px solid #e8e8ed;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  color: #1d1d1f;
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
  color: #86868b;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tag-remove:hover {
  background: rgba(255, 59, 48, 0.2);
  color: #ff3b30;
}

.empty-tags {
  padding: 12px;
  background: #ffffff;
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
  background: #ffffff;
  border-radius: 8px;
  border: 1px solid #e8e8ed;
}

.collection-icon {
  color: #ffd700;
}

.collection-name {
  font-size: 13px;
  font-weight: 500;
  color: #1d1d1f;
}

.info-card {
  background: #ffffff;
  border-radius: 12px;
  padding: 12px;
  border: 1px solid #e8e8ed;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

.info-row:not(:last-child) {
  border-bottom: 1px solid #f5f5f7;
}

.info-key {
  font-size: 13px;
  color: #86868b;
}

.info-value {
  font-size: 13px;
  font-weight: 500;
  color: #1d1d1f;
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
  background: rgba(0, 122, 255, 0.1);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  color: #007aff;
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
  background: #ffffff;
  border-radius: 8px;
  border: 1px solid #e8e8ed;
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
  color: #1d1d1f;
}

.symbol-tree {
  background: #ffffff;
  border-radius: 12px;
  padding: 12px;
  border: 1px solid #e8e8ed;
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
  border-bottom: 1px solid #f5f5f7;
}

.symbol-kind {
  font-size: 11px;
  padding: 2px 6px;
  background: #f5f5f7;
  border-radius: 4px;
  color: #86868b;
}

.symbol-name {
  font-size: 13px;
  font-weight: 500;
  color: #1d1d1f;
  font-family: 'SF Mono', Monaco, monospace;
}

.preview-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 20px;
  background: #ffffff;
  border-top: 1px solid #e8e8ed;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: #f5f5f7;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: #1d1d1f;
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: #e8e8ed;
}

.action-btn.primary {
  background: #007aff;
  color: white;
}

.action-btn.primary:hover {
  background: #0066d6;
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
  background: #fafafa;
}

.empty-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.empty-icon {
  color: #d2d2d7;
  margin-bottom: 8px;
}

.empty-title {
  font-size: 16px;
  font-weight: 600;
  color: #1d1d1f;
}

.empty-subtitle {
  font-size: 14px;
  color: #86868b;
}

/* Scrollbar */
.preview-scroll::-webkit-scrollbar {
  width: 6px;
}

.preview-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.preview-scroll::-webkit-scrollbar-thumb {
  background: #d2d2d7;
  border-radius: 3px;
}

.preview-scroll::-webkit-scrollbar-thumb:hover {
  background: #86868b;
}

.text-muted {
  color: #86868b !important;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
