<template>
  <main class="explorer">
    <!-- Breadcrumb Navigation -->
    <div class="breadcrumb-bar">
      <div class="breadcrumb-path">
        <button class="breadcrumb-item root" @click="navigateToRoot" title="ROOT">
          <Icon icon="mdi:home-outline" width="14" height="14" />
        </button>
        <template v-for="(segment, i) in pathSegments" :key="i">
          <Icon icon="mdi:chevron-right" width="12" height="12" class="breadcrumb-sep" />
          <button class="breadcrumb-item" @click="navigateToSegment(i)" :title="segment.name">
            <Icon v-if="segment.isFolder" icon="mdi:folder-outline" width="14" height="14" />
            <span class="breadcrumb-name">{{ segment.name }}</span>
          </button>
        </template>
      </div>
      <div class="breadcrumb-actions">
        <span class="item-count">{{ sortedFiles.length }} items</span>
      </div>
    </div>

    <!-- Main Toolbar -->
    <div class="explorer-toolbar">
      <div class="toolbar-left">
        <!-- View Mode Toggle -->
        <div class="view-modes">
          <button
            v-for="mode in viewModes"
            :key="mode.id"
            class="view-btn"
            :class="{ active: store.viewMode === mode.id }"
            @click="store.viewMode = mode.id"
            :title="mode.label"
          >
            <Icon :icon="mode.icon" width="16" height="16" />
          </button>
        </div>

        <div class="toolbar-divider" />

        <!-- Sort Controls -->
        <div class="sort-controls">
          <select v-model="sortField" class="sort-select">
            <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
          <button class="sort-dir-btn" @click="toggleSortDir" :title="sortDir === 'asc' ? 'Ascending' : 'Descending'">
            <Icon :icon="sortDir === 'asc' ? 'mdi:sort-ascending' : 'mdi:sort-descending'" width="14" height="14" />
          </button>
        </div>
      </div>

      <div class="toolbar-center">
        <!-- Tag Filter Pills -->
        <div class="tag-filters" v-if="allTags.length > 0">
          <button
            v-for="tag in allTags.slice(0, 5)"
            :key="tag.name"
            class="tag-pill"
            :class="{ active: activeTagFilters.includes(tag.name) }"
            :style="{ '--tag-color': tag.color }"
            @click="toggleTagFilter(tag.name)"
          >
            <span class="tag-dot" :style="{ background: tag.color }" />
            <span class="tag-label">{{ tag.name }}</span>
            <span class="tag-count">{{ tag.count }}</span>
          </button>
          <button v-if="allTags.length > 5" class="tag-pill more" @click="showAllTags = !showAllTags">
            +{{ allTags.length - 5 }}
          </button>
        </div>
      </div>

      <div class="toolbar-right">
        <!-- Search/Filter -->
        <div class="search-box">
          <Icon icon="mdi:magnify" width="14" height="14" class="search-icon" />
          <input
            v-model="filterQuery"
            class="search-input"
            placeholder="Filter..."
            aria-label="Filter files"
          />
          <button v-if="filterQuery" class="search-clear" @click="filterQuery = ''">
            <Icon icon="mdi:close" width="12" height="12" />
          </button>
        </div>

        <div class="toolbar-divider" />

        <!-- Selection Info -->
        <div v-if="selectedCount > 0" class="selection-info">
          <span class="selection-count">{{ selectedCount }} selected</span>
          <button class="action-btn" @click="clearSelection" title="Clear selection">
            <Icon icon="mdi:close" width="14" height="14" />
          </button>
        </div>
      </div>
    </div>

    <!-- Quick Actions Bar (shown when files selected) -->
    <Transition name="slide-down">
      <div v-if="selectedCount > 0" class="quick-actions">
        <div class="quick-actions-inner">
          <button class="quick-btn" @click="execBulkAction('star')" title="Star">
            <Icon icon="mdi:star-outline" width="16" height="16" />
            <span>Star</span>
          </button>
          <button class="quick-btn" @click="execBulkAction('encrypt')" title="Encrypt">
            <Icon icon="mdi:lock-outline" width="16" height="16" />
            <span>Encrypt</span>
          </button>
          <button class="quick-btn" @click="execBulkAction('compress')" title="Compress">
            <Icon icon="mdi:package-variant" width="16" height="16" />
            <span>Compress</span>
          </button>
          <button class="quick-btn" @click="execBulkAction('tag')" title="Add Tag">
            <Icon icon="mdi:tag-plus-outline" width="16" height="16" />
            <span>Tag</span>
          </button>
          <button class="quick-btn" @click="execBulkAction('move')" title="Move">
            <Icon icon="mdi:folder-move-outline" width="16" height="16" />
            <span>Move</span>
          </button>
          <button class="quick-btn danger" @click="execBulkAction('delete')" title="Delete">
            <Icon icon="mdi:delete-outline" width="16" height="16" />
            <span>Delete</span>
          </button>
        </div>
      </div>
    </Transition>

    <!-- Tag Management Panel (shown when tag button clicked) -->
    <Transition name="slide-down">
      <div v-if="showTagPanel" class="tag-panel">
        <div class="tag-panel-header">
          <span class="tag-panel-title">Manage Tags</span>
          <button class="close-btn" @click="showTagPanel = false">
            <Icon icon="mdi:close" width="16" height="16" />
          </button>
        </div>
        <div class="tag-panel-content">
          <div class="tag-input-row">
            <input
              v-model="newTagName"
              class="tag-input"
              placeholder="New tag name..."
              @keyup.enter="createTag"
            />
            <div class="color-picker">
              <button
                v-for="color in tagColors"
                :key="color"
                class="color-dot"
                :class="{ active: newTagColor === color }"
                :style="{ background: color }"
                @click="newTagColor = color"
              />
            </div>
            <button class="add-tag-btn" @click="createTag" :disabled="!newTagName.trim()">
              <Icon icon="mdi:plus" width="14" height="14" />
            </button>
          </div>
          <div class="existing-tags">
            <div v-for="tag in allTags" :key="tag.name" class="tag-item">
              <span class="tag-dot" :style="{ background: tag.color }" />
              <span class="tag-name">{{ tag.name }}</span>
              <span class="tag-file-count">{{ tag.count }} files</span>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- File Grid View -->
    <div v-if="store.viewMode === 'grid'" class="grid-view" @contextmenu.prevent="ctx.open($event, 'file_grid_bg')">
      <TransitionGroup name="file-grid" tag="div" class="grid-container">
        <div
          v-for="file in sortedFiles"
          :key="file.id"
          class="file-card"
          :class="{
            selected: store.selectedFileId === file.id,
            'bulk-selected': store.selectedFileIds.includes(file.id),
            folder: file.fileType === 'folder'
          }"
          @click="handleClick(file)"
          @dblclick="handleDoubleClick(file)"
          @contextmenu.prevent="showContextMenu($event, file)"
          @mouseenter="showTooltip($event, file)"
          @mousemove="moveTooltip($event)"
          @mouseleave="hideTooltip"
          :draggable="true"
          @dragstart="onDragStart($event, file)"
          @dragover.prevent
        >
          <!-- Selection Checkbox -->
          <div class="file-select" @click.stop="toggleSelect(file.id)">
            <div class="checkbox" :class="{ checked: store.selectedFileIds.includes(file.id) }">
              <Icon v-if="store.selectedFileIds.includes(file.id)" icon="mdi:check" width="10" height="10" />
            </div>
          </div>

          <!-- Thumbnail / Icon -->
          <div class="file-preview">
            <img
              v-if="file.thumbnailPath"
              :src="file.thumbnailPath"
              class="file-thumbnail"
              :alt="file.name"
              @error="(e) => { (e.target as HTMLImageElement).style.display = 'none' }"
            />
            <div v-else class="file-icon-wrapper" :class="getFileTypeClass(file)">
              <Icon :icon="getFileIcon(file)" width="32" height="32" />
            </div>

            <!-- File Type Badge -->
            <div class="type-badge" :class="getFileTypeClass(file)">
              {{ getFileExtension(file) }}
            </div>

            <!-- Status Indicators -->
            <div class="status-badges">
              <span v-if="file.encrypted" class="status-badge encrypted" title="Encrypted">
                <Icon icon="mdi:lock" width="10" height="10" />
              </span>
              <span v-if="file.isStarred" class="status-badge starred" title="Starred">
                <Icon icon="mdi:star" width="10" height="10" />
              </span>
            </div>
          </div>

          <!-- File Info -->
          <div class="file-info">
            <div class="file-name" :title="file.name">{{ file.name }}</div>
            <div class="file-meta">
              <span class="file-size">{{ formatSize(file.sizeBytes) }}</span>
              <span class="file-date">{{ formatDate(file.modifiedAt) }}</span>
            </div>
          </div>

          <!-- Tags (color circles) -->
          <div v-if="file.tags && file.tags.length > 0" class="file-tags">
            <span
              v-for="tag in file.tags.slice(0, 3)"
              :key="tag"
              class="tag-circle"
              :style="{ background: getTagColor(tag) }"
              :title="tag"
            />
            <span v-if="file.tags.length > 3" class="tag-more">+{{ file.tags.length - 3 }}</span>
          </div>

          <!-- Collection Badge -->
          <div v-if="file.collectionIds && file.collectionIds.length > 0" class="collection-badge">
            <Icon icon="mdi:folder-star-outline" width="10" height="10" />
          </div>
        </div>
      </TransitionGroup>

      <!-- Empty State -->
      <div v-if="sortedFiles.length === 0 && !store.isLoading" class="empty-state">
        <Icon icon="mdi:folder-open-outline" width="48" height="48" class="empty-icon" />
        <div class="empty-title">No files here</div>
        <div class="empty-subtitle">Drop files or create a new folder</div>
      </div>
    </div>

    <!-- File List View -->
    <div v-if="store.viewMode === 'list'" class="list-view" @contextmenu.prevent="ctx.open($event, 'file_grid_bg')">
      <div class="list-header">
        <span class="lc lc-check" @click="toggleSelectAll">
          <div class="checkbox" :class="{ checked: allSelected }">
            <Icon v-if="allSelected" icon="mdi:check" width="10" height="10" />
          </div>
        </span>
        <span class="lc lc-name" @click="setSort('name')">
          Name
          <Icon v-if="sortField === 'name'" :icon="sortDir === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'" width="12" height="12" />
        </span>
        <span class="lc lc-tags">Tags</span>
        <span class="lc lc-size" @click="setSort('size')">
          Size
          <Icon v-if="sortField === 'size'" :icon="sortDir === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'" width="12" height="12" />
        </span>
        <span class="lc lc-type" @click="setSort('type')">
          Type
          <Icon v-if="sortField === 'type'" :icon="sortDir === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'" width="12" height="12" />
        </span>
        <span class="lc lc-date" @click="setSort('date')">
          Modified
          <Icon v-if="sortField === 'date'" :icon="sortDir === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'" width="12" height="12" />
        </span>
        <span class="lc lc-status">Status</span>
      </div>

      <div class="list-body">
        <div
          v-for="file in sortedFiles"
          :key="file.id"
          class="list-row"
          :class="{
            selected: store.selectedFileId === file.id,
            'bulk-selected': store.selectedFileIds.includes(file.id),
            folder: file.fileType === 'folder'
          }"
          @click="handleClick(file)"
          @dblclick="handleDoubleClick(file)"
          @contextmenu.prevent="showContextMenu($event, file)"
          @mouseenter="showTooltip($event, file)"
          @mousemove="moveTooltip($event)"
          @mouseleave="hideTooltip"
          :draggable="true"
          @dragstart="onDragStart($event, file)"
        >
          <span class="lc lc-check" @click.stop="toggleSelect(file.id)">
            <div class="checkbox" :class="{ checked: store.selectedFileIds.includes(file.id) }">
              <Icon v-if="store.selectedFileIds.includes(file.id)" icon="mdi:check" width="10" height="10" />
            </div>
          </span>
          <span class="lc lc-name">
            <div class="list-icon" :class="getFileTypeClass(file)">
              <Icon :icon="getFileIcon(file)" width="18" height="18" />
            </div>
            <span class="list-name truncate">{{ file.name }}</span>
          </span>
          <span class="lc lc-tags">
            <div v-if="file.tags && file.tags.length > 0" class="list-tags">
              <span
                v-for="tag in file.tags.slice(0, 2)"
                :key="tag"
                class="tag-circle small"
                :style="{ background: getTagColor(tag) }"
                :title="tag"
              />
              <span v-if="file.tags.length > 2" class="tag-more small">+{{ file.tags.length - 2 }}</span>
            </div>
          </span>
          <span class="lc lc-size">{{ formatSize(file.sizeBytes) }}</span>
          <span class="lc lc-type">
            <span class="type-label" :class="getFileTypeClass(file)">{{ getFileTypeLabel(file) }}</span>
          </span>
          <span class="lc lc-date">{{ formatDate(file.modifiedAt) }}</span>
          <span class="lc lc-status">
            <Icon v-if="file.encrypted" icon="mdi:lock" width="12" height="12" class="status-icon encrypted" />
            <Icon v-if="file.isStarred" icon="mdi:star" width="12" height="12" class="status-icon starred" />
          </span>
        </div>
      </div>

      <div v-if="sortedFiles.length === 0 && !store.isLoading" class="empty-state">
        <Icon icon="mdi:folder-open-outline" width="48" height="48" class="empty-icon" />
        <div class="empty-title">No files here</div>
        <div class="empty-subtitle">Drop files or create a new folder</div>
      </div>
    </div>

    <!-- File Preview Tooltip -->
    <FileTooltip :file="tooltipFile" :visible="tooltipVisible" :x="tooltipX" :y="tooltipY" />

    <!-- Rename Dialog -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showRenameDialog" class="modal-overlay" @click.self="showRenameDialog = false">
          <div class="modal-card">
            <div class="modal-header">
              <Icon icon="mdi:rename-box-outline" width="20" height="20" />
              <span>Rename</span>
            </div>
            <input
              ref="renameInputRef"
              v-model="renameValue"
              class="modal-input"
              @keyup.enter="handleRenameConfirm"
              @keyup.escape="showRenameDialog = false"
            />
            <div class="modal-actions">
              <button class="modal-btn secondary" @click="showRenameDialog = false">Cancel</button>
              <button class="modal-btn primary" @click="handleRenameConfirm">Rename</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </main>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useContextMenu } from '@/composables/useContextMenu'
import type { FileNode } from '@/types'
import FileTooltip from './FileTooltip.vue'

const store = useAppStore()
const ctx = useContextMenu()

// State
const sortField = ref<'name' | 'size' | 'date' | 'type'>('name')
const sortDir = ref<'asc' | 'desc'>('asc')
const filterQuery = ref('')
const activeTagFilters = ref<string[]>([])
const showAllTags = ref(false)
const showTagPanel = ref(false)
const newTagName = ref('')
const newTagColor = ref('#00ff41')

// Tooltip state
const tooltipVisible = ref(false)
const tooltipFile = ref<FileNode | null>(null)
const tooltipX = ref(0)
const tooltipY = ref(0)
let tooltipTimer: ReturnType<typeof setTimeout> | null = null

// Rename dialog state
const showRenameDialog = ref(false)
const renameValue = ref('')
const renamingFileId = ref<string | null>(null)
const renameInputRef = ref<HTMLInputElement | null>(null)

// Constants
const viewModes = [
  { id: 'grid' as const, label: 'Grid View', icon: 'mdi:view-grid-outline' },
  { id: 'list' as const, label: 'List View', icon: 'mdi:view-list-outline' },
  { id: 'masonry' as const, label: 'Gallery View', icon: 'mdi:view-dashboard-outline' },
]

const sortOptions = [
  { value: 'name', label: 'Name' },
  { value: 'size', label: 'Size' },
  { value: 'date', label: 'Date' },
  { value: 'type', label: 'Type' },
]

const tagColors = [
  '#00ff41', '#5af0ff', '#ff5f57', '#ffd700', '#ff6b9d',
  '#b388ff', '#28c840', '#ff9933', '#3a86ff', '#ff006e'
]

// File type colors for badges
const fileTypeColors: Record<string, string> = {
  image: '#ff6b9d',
  video: '#b388ff',
  audio: '#5af0ff',
  document: '#ffd700',
  archive: '#ff9933',
  code: '#28c840',
  folder: '#00ff41',
  other: '#888',
}

// Computed
const selectedCount = computed(() => store.selectedFileIds.length)

const allSelected = computed(() =>
  sortedFiles.value.length > 0 && sortedFiles.value.every(f => store.selectedFileIds.includes(f.id))
)

const sortedFiles = computed(() => {
  let files = store.currentFolderFiles

  // Apply text filter
  if (filterQuery.value.trim()) {
    const q = filterQuery.value.toLowerCase()
    files = files.filter(f =>
      f.name.toLowerCase().includes(q) ||
      f.tags?.some(t => t.toLowerCase().includes(q)) ||
      f.mimeType?.toLowerCase().includes(q)
    )
  }

  // Apply tag filters
  if (activeTagFilters.value.length > 0) {
    files = files.filter(f =>
      activeTagFilters.value.every(tag => f.tags?.includes(tag))
    )
  }

  // Sort
  return [...files].sort((a, b) => {
    // Folders always first
    if (a.fileType === 'folder' && b.fileType !== 'folder') return -1
    if (a.fileType !== 'folder' && b.fileType === 'folder') return 1

    let cmp = 0
    if (sortField.value === 'name') cmp = a.name.localeCompare(b.name)
    else if (sortField.value === 'size') cmp = a.sizeBytes - b.sizeBytes
    else if (sortField.value === 'date') cmp = new Date(a.modifiedAt).getTime() - new Date(b.modifiedAt).getTime()
    else if (sortField.value === 'type') cmp = (a.mimeType || a.fileType || '').localeCompare(b.mimeType || b.fileType || '')

    return sortDir.value === 'asc' ? cmp : -cmp
  })
})

const allTags = computed(() => {
  const tagMap = new Map<string, { name: string; color: string; count: number }>()
  store.files.forEach(f => {
    f.tags?.forEach(tag => {
      const existing = tagMap.get(tag)
      if (existing) {
        existing.count++
      } else {
        tagMap.set(tag, { name: tag, color: getTagColor(tag), count: 1 })
      }
    })
  })
  return Array.from(tagMap.values()).sort((a, b) => b.count - a.count)
})

const pathSegments = computed(() => {
  if (!store.selectedFileId) return []
  const segments: { id: string; name: string; isFolder: boolean }[] = []
  let currentId = store.selectedFileId

  while (currentId) {
    const file = store.files.find(f => f.id === currentId)
    if (!file) break
    segments.unshift({ id: file.id, name: file.name, isFolder: file.fileType === 'folder' })
    currentId = file.parentId || ''
  }

  return segments
})

// Tag color cache
const tagColorCache = new Map<string, string>()
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
  if (mime.includes('zip') || mime.includes('tar') || mime.includes('gz') || mime.includes('rar')) return 'archive'
  if (mime.includes('json') || mime.includes('javascript') || mime.includes('typescript') || mime.includes('python') || mime.includes('rust')) return 'code'
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
  if (mime.includes('javascript')) return 'mdi/language-javascript'
  if (mime.includes('typescript')) return 'mdi:language-typescript'
  if (mime.includes('python')) return 'mdi:language-python'
  if (mime.includes('rust')) return 'mdi:language-rust'
  if (mime.includes('text')) return 'mdi:file-document-outline'
  return 'mdi:file-outline'
}

function getFileExtension(file: FileNode): string {
  const parts = file.name.split('.')
  return parts.length > 1 ? parts.pop()!.toUpperCase() : ''
}

function getFileTypeLabel(file: FileNode): string {
  if (file.fileType === 'folder') return 'Folder'
  const mime = file.mimeType || ''
  if (mime.startsWith('image/')) return 'Image'
  if (mime.startsWith('video/')) return 'Video'
  if (mime.startsWith('audio/')) return 'Audio'
  if (mime.includes('pdf')) return 'PDF'
  if (mime.includes('zip') || mime.includes('tar') || mime.includes('gz')) return 'Archive'
  if (mime.includes('json') || mime.includes('javascript') || mime.includes('typescript')) return 'Code'
  if (mime.includes('text')) return 'Document'
  return 'File'
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
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

  if (diffDays === 0) return 'Today'
  if (diffDays === 1) return 'Yesterday'
  if (diffDays < 7) return `${diffDays} days ago`
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
}

// Navigation
function navigateToRoot() {
  store.selectedFileId = null
  store.currentPanel = 'files'
}

function navigateToSegment(index: number) {
  const segment = pathSegments.value[index]
  if (segment) {
    store.selectedFileId = segment.id
  }
}

// Selection
function toggleSelect(fileId: string) {
  const idx = store.selectedFileIds.indexOf(fileId)
  if (idx === -1) {
    store.selectedFileIds.push(fileId)
  } else {
    store.selectedFileIds.splice(idx, 1)
  }
  store.isMultiSelect = store.selectedFileIds.length > 0
}

function toggleSelectAll() {
  if (allSelected.value) {
    store.selectedFileIds = []
  } else {
    store.selectedFileIds = sortedFiles.value.map(f => f.id)
  }
  store.isMultiSelect = store.selectedFileIds.length > 0
}

function clearSelection() {
  store.selectedFileIds = []
  store.isMultiSelect = false
}

function handleClick(file: FileNode) {
  if (store.isMultiSelect) {
    toggleSelect(file.id)
  } else {
    store.selectedFileId = file.id
  }
}

function handleDoubleClick(file: FileNode) {
  if (file.fileType === 'folder') {
    store.selectedFileId = file.id
  } else {
    store.selectedFileId = file.id
    // Could open preview or file
  }
}

// Tag filtering
function toggleTagFilter(tagName: string) {
  const idx = activeTagFilters.value.indexOf(tagName)
  if (idx === -1) {
    activeTagFilters.value.push(tagName)
  } else {
    activeTagFilters.value.splice(idx, 1)
  }
}

// Sorting
function setSort(field: 'name' | 'size' | 'date' | 'type') {
  if (sortField.value === field) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortField.value = field
    sortDir.value = 'asc'
  }
}

function toggleSortDir() {
  sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
}

// Tooltip
function showTooltip(e: MouseEvent, file: FileNode) {
  if (tooltipTimer) clearTimeout(tooltipTimer)
  tooltipTimer = setTimeout(() => {
    tooltipFile.value = file
    tooltipX.value = e.clientX
    tooltipY.value = e.clientY
    tooltipVisible.value = true
  }, 500)
}

function moveTooltip(e: MouseEvent) {
  if (tooltipVisible.value) {
    tooltipX.value = e.clientX
    tooltipY.value = e.clientY
  }
}

function hideTooltip() {
  if (tooltipTimer) clearTimeout(tooltipTimer)
  tooltipVisible.value = false
  tooltipFile.value = null
}

// Drag and drop
function onDragStart(e: DragEvent, file: FileNode) {
  e.dataTransfer?.setData('text/plain', file.id)
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'copy'
  }
}

// Context menu
function showContextMenu(e: MouseEvent, file: FileNode) {
  const entries = [
    { id: 'open', label: 'Open', icon: 'mdi:folder-open', action: () => { store.selectFile(file.id) } },
    { id: 'preview', label: 'Preview', icon: 'mdi:eye-outline', action: () => { store.selectedFileId = file.id } },
    { id: 'div1', label: '', divider: true },
    { id: 'star', label: file.isStarred ? 'Unstar' : 'Star', icon: 'mdi:star-outline', action: () => store.toggleStar(file.id) },
    { id: 'rename', label: 'Rename', icon: 'mdi:rename-box-outline', action: () => {
      renameValue.value = file.name
      renamingFileId.value = file.id
      showRenameDialog.value = true
      setTimeout(() => renameInputRef.value?.focus(), 50)
    }},
    { id: 'duplicate', label: 'Duplicate', icon: 'mdi:content-copy', action: () => store.duplicateFileContext?.(file.id) },
    { id: 'div2', label: '', divider: true },
    { id: 'encrypt', label: file.encrypted ? 'Decrypt' : 'Encrypt', icon: file.encrypted ? 'mdi:lock-open-outline' : 'mdi:lock-outline', action: () => {
      if (file.encrypted) {
        store.notifySuccess('Decrypt: ' + file.name)
      } else {
        store.encryptFile(file.id, 'hybrid')
      }
    }},
    { id: 'compress', label: 'Compress', icon: 'mdi:package-variant', action: () => store.compressFile(file.id, 'zstd') },
    { id: 'div3', label: '', divider: true },
    { id: 'delete', label: 'Delete', icon: 'mdi:delete-outline', danger: true, action: () => window.dispatchEvent(new CustomEvent('cybermanju:show-delete-dialog', { detail: { fileIds: [file.id] } })) },
  ]
  ctx.replaceEntries('file_grid_item', entries)
  ctx.open(e, 'file_grid_item')
}

// Bulk actions
async function execBulkAction(action: string) {
  const ids = [...store.selectedFileIds]
  for (const id of ids) {
    try {
      switch (action) {
        case 'encrypt': await store.encryptFile(id, 'hybrid'); break
        case 'compress': await store.compressFile(id, 'zstd'); break
        case 'star': store.toggleStar(id); break
        case 'delete': window.dispatchEvent(new CustomEvent('cybermanju:show-delete-dialog', { detail: { fileIds: store.selectedFileIds } })); break
        case 'tag': showTagPanel.value = true; break
        case 'move': store.notifySuccess('Move: ' + ids.length + ' files'); break
      }
    } catch {}
  }
  if (action !== 'tag') {
    clearSelection()
  }
}

// Rename
async function handleRenameConfirm() {
  if (renamingFileId.value && renameValue.value.trim()) {
    await store.renameFile(renamingFileId.value, renameValue.value.trim())
  }
  showRenameDialog.value = false
  renamingFileId.value = null
}

// Tag creation
function createTag() {
  if (!newTagName.value.trim()) return
  // This would normally call a store action to add the tag
  store.notifySuccess('Tag created: ' + newTagName.value)
  newTagName.value = ''
}

// Initialize
onMounted(() => {
  if (store.selectedFileId) {
    // Scroll to selected file if exists
  }
})
</script>

<style scoped>
/* CSS Variables for theming */
.explorer {
  --bg-primary: #fafafa;
  --bg-secondary: #ffffff;
  --bg-tertiary: #f5f5f7;
  --bg-hover: rgba(0, 0, 0, 0.04);
  --bg-active: rgba(0, 122, 255, 0.08);
  --text-primary: #1d1d1f;
  --text-secondary: #6e6e73;
  --text-tertiary: #86868b;
  --border-color: #d2d2d7;
  --border-light: #e8e8ed;
  --accent: #007aff;
  --accent-hover: #0066d6;
  --danger: #ff3b30;
  --success: #34c759;
  --warning: #ff9500;
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.08);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.12);
  --transition: 0.2s ease;

  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', Roboto, sans-serif;
  color: var(--text-primary);
  overflow: hidden;
}

/* Breadcrumb Bar */
.breadcrumb-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
}

.breadcrumb-path {
  display: flex;
  align-items: center;
  gap: 4px;
}

.breadcrumb-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  background: none;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--accent);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background var(--transition);
}

.breadcrumb-item:hover {
  background: var(--bg-hover);
}

.breadcrumb-item.root {
  padding: 6px 8px;
}

.breadcrumb-sep {
  color: var(--text-tertiary);
}

.breadcrumb-name {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-count {
  font-size: 13px;
  color: var(--text-secondary);
}

/* Main Toolbar */
.explorer-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 52px;
  padding: 0 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
  gap: 16px;
}

.toolbar-left,
.toolbar-center,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toolbar-center {
  flex: 1;
  justify-content: center;
}

.toolbar-divider {
  width: 1px;
  height: 24px;
  background: var(--border-light);
}

/* View Modes */
.view-modes {
  display: flex;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  padding: 3px;
}

.view-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  background: none;
  border: none;
  border-radius: 6px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition);
}

.view-btn:hover {
  color: var(--text-primary);
}

.view-btn.active {
  background: var(--bg-secondary);
  color: var(--accent);
  box-shadow: var(--shadow-sm);
}

/* Sort Controls */
.sort-controls {
  display: flex;
  align-items: center;
  gap: 6px;
}

.sort-select {
  padding: 6px 28px 6px 12px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: var(--radius-sm);
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24'%3E%3Cpath fill='%236e6e73' d='M7 10l5 5 5-5z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
}

.sort-dir-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition);
}

.sort-dir-btn:hover {
  background: var(--border-light);
  color: var(--text-primary);
}

/* Tag Filters */
.tag-filters {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: nowrap;
  overflow-x: auto;
  max-width: 400px;
  scrollbar-width: none;
}

.tag-filters::-webkit-scrollbar {
  display: none;
}

.tag-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--bg-tertiary);
  border: 1px solid transparent;
  border-radius: 20px;
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  white-space: nowrap;
  transition: all var(--transition);
}

.tag-pill:hover {
  background: var(--border-light);
}

.tag-pill.active {
  background: var(--tag-color, var(--accent));
  color: white;
  border-color: transparent;
}

.tag-pill.active .tag-dot {
  background: white !important;
}

.tag-pill.active .tag-count {
  background: rgba(255, 255, 255, 0.3);
  color: white;
}

.tag-pill.more {
  background: none;
  border: 1px dashed var(--border-color);
  color: var(--text-secondary);
}

.tag-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.tag-label {
  font-weight: 500;
}

.tag-count {
  padding: 2px 6px;
  background: var(--bg-hover);
  border-radius: 10px;
  font-size: 11px;
  color: var(--text-secondary);
}

/* Search Box */
.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 200px;
  padding: 8px 32px 8px 32px;
  background: var(--bg-tertiary);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  font-size: 13px;
  color: var(--text-primary);
  transition: all var(--transition);
}

.search-input:focus {
  outline: none;
  background: var(--bg-secondary);
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
  width: 260px;
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.search-clear {
  position: absolute;
  right: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: var(--border-light);
  border: none;
  border-radius: 50%;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition);
}

.search-clear:hover {
  background: var(--text-tertiary);
  color: white;
}

/* Selection Info */
.selection-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--bg-active);
  border-radius: var(--radius-sm);
}

.selection-count {
  font-size: 13px;
  font-weight: 500;
  color: var(--accent);
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: none;
  border: none;
  border-radius: 50%;
  color: var(--accent);
  cursor: pointer;
  transition: all var(--transition);
}

.action-btn:hover {
  background: var(--accent);
  color: white;
}

/* Quick Actions Bar */
.quick-actions {
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
  padding: 12px 20px;
}

.quick-actions-inner {
  display: flex;
  align-items: center;
  gap: 8px;
  justify-content: center;
}

.quick-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  cursor: pointer;
  transition: all var(--transition);
}

.quick-btn:hover {
  background: var(--border-light);
  transform: translateY(-1px);
}

.quick-btn.danger {
  color: var(--danger);
}

.quick-btn.danger:hover {
  background: rgba(255, 59, 48, 0.1);
}

/* Tag Panel */
.tag-panel {
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
  padding: 16px 20px;
}

.tag-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.tag-panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: 50%;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition);
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tag-input-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.tag-input {
  flex: 1;
  max-width: 240px;
  padding: 8px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-sm);
  font-size: 13px;
  color: var(--text-primary);
}

.tag-input:focus {
  outline: none;
  border-color: var(--accent);
}

.color-picker {
  display: flex;
  gap: 6px;
}

.color-dot {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all var(--transition);
}

.color-dot:hover {
  transform: scale(1.1);
}

.color-dot.active {
  border-color: var(--text-primary);
  box-shadow: 0 0 0 2px white, 0 0 0 4px var(--text-primary);
}

.add-tag-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--accent);
  border: none;
  border-radius: var(--radius-sm);
  color: white;
  cursor: pointer;
  transition: all var(--transition);
}

.add-tag-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.add-tag-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.existing-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
}

.tag-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.tag-file-count {
  font-size: 12px;
  color: var(--text-secondary);
}

/* Grid View */
.grid-view {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px;
}

.grid-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
}

.file-card {
  position: relative;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-light);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition);
}

.file-card:hover {
  border-color: var(--border-color);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.file-card.selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.15);
}

.file-card.bulk-selected {
  border-color: var(--accent);
  background: var(--bg-active);
}

.file-card.folder {
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%);
}

/* File Select Checkbox */
.file-select {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 2;
  opacity: 0;
  transition: opacity var(--transition);
}

.file-card:hover .file-select,
.file-card.selected .file-select,
.file-card.bulk-selected .file-select {
  opacity: 1;
}

.checkbox {
  width: 20px;
  height: 20px;
  background: var(--bg-secondary);
  border: 2px solid var(--border-color);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition);
}

.checkbox.checked {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

/* File Preview (Thumbnail/Icon) */
.file-preview {
  position: relative;
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
}

.file-thumbnail {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.file-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  border-radius: var(--radius-md);
  transition: all var(--transition);
}

.file-icon-wrapper.folder { background: rgba(0, 122, 255, 0.1); color: #007aff; }
.file-icon-wrapper.image { background: rgba(255, 107, 157, 0.1); color: #ff6b9d; }
.file-icon-wrapper.video { background: rgba(179, 136, 255, 0.1); color: #b388ff; }
.file-icon-wrapper.audio { background: rgba(90, 240, 255, 0.1); color: #5af0ff; }
.file-icon-wrapper.document { background: rgba(255, 215, 0, 0.1); color: #ffd700; }
.file-icon-wrapper.archive { background: rgba(255, 152, 51, 0.1); color: #ff9933; }
.file-icon-wrapper.code { background: rgba(40, 200, 64, 0.1); color: #28c840; }
.file-icon-wrapper.other { background: rgba(136, 136, 136, 0.1); color: #888; }

/* Type Badge */
.type-badge {
  position: absolute;
  bottom: 8px;
  right: 8px;
  padding: 3px 8px;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  border-radius: 6px;
  font-size: 10px;
  font-weight: 600;
  color: white;
  text-transform: uppercase;
}

/* Status Badges */
.status-badges {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
}

.status-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  backdrop-filter: blur(4px);
}

.status-badge.encrypted {
  background: rgba(255, 152, 51, 0.9);
  color: white;
}

.status-badge.starred {
  background: rgba(255, 204, 0, 0.9);
  color: #1d1d1f;
}

/* File Info */
.file-info {
  padding: 12px;
  flex: 1;
}

.file-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}

/* File Tags (Color Circles) */
.file-tags {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 12px 12px;
}

.tag-circle {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid var(--bg-secondary);
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.1);
}

.tag-circle.small {
  width: 10px;
  height: 10px;
}

.tag-more {
  font-size: 11px;
  color: var(--text-secondary);
}

.tag-more.small {
  font-size: 10px;
}

/* Collection Badge */
.collection-badge {
  position: absolute;
  top: 10px;
  left: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: rgba(255, 204, 0, 0.9);
  border-radius: 50%;
  color: #1d1d1f;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  text-align: center;
}

.empty-icon {
  color: var(--text-tertiary);
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.empty-subtitle {
  font-size: 14px;
  color: var(--text-secondary);
}

/* List View */
.list-view {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.list-header {
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 20px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  position: sticky;
  top: 0;
  z-index: 2;
}

.list-header .lc {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  user-select: none;
}

.list-header .lc:hover {
  color: var(--text-primary);
}

.list-body {
  background: var(--bg-secondary);
}

.list-row {
  display: flex;
  align-items: center;
  height: 48px;
  padding: 0 20px;
  border-bottom: 1px solid var(--border-light);
  cursor: pointer;
  transition: background var(--transition);
}

.list-row:hover {
  background: var(--bg-hover);
}

.list-row.selected {
  background: var(--bg-active);
}

.list-row.bulk-selected {
  background: rgba(0, 122, 255, 0.04);
}

.list-row.folder {
  background: linear-gradient(90deg, rgba(0, 122, 255, 0.04) 0%, transparent 100%);
}

.lc {
  display: flex;
  align-items: center;
  overflow: hidden;
  white-space: nowrap;
}

.lc-check { flex: 0 0 32px; justify-content: center; }
.lc-name { flex: 3; min-width: 0; gap: 12px; }
.lc-tags { flex: 1; min-width: 80px; }
.lc-size { flex: 0.8; min-width: 70px; justify-content: flex-end; }
.lc-type { flex: 1; min-width: 80px; }
.lc-date { flex: 1; min-width: 100px; }
.lc-status { flex: 0.5; min-width: 60px; gap: 6px; }

.list-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  flex-shrink: 0;
}

.list-icon.folder { background: rgba(0, 122, 255, 0.1); color: #007aff; }
.list-icon.image { background: rgba(255, 107, 157, 0.1); color: #ff6b9d; }
.list-icon.video { background: rgba(179, 136, 255, 0.1); color: #b388ff; }
.list-icon.audio { background: rgba(90, 240, 255, 0.1); color: #5af0ff; }
.list-icon.document { background: rgba(255, 215, 0, 0.1); color: #ffd700; }
.list-icon.archive { background: rgba(255, 152, 51, 0.1); color: #ff9933; }
.list-icon.code { background: rgba(40, 200, 64, 0.1); color: #28c840; }
.list-icon.other { background: rgba(136, 136, 136, 0.1); color: #888; }

.list-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.list-tags {
  display: flex;
  align-items: center;
  gap: 4px;
}

.type-label {
  font-size: 12px;
  padding: 3px 8px;
  border-radius: 4px;
  text-transform: capitalize;
}

.type-label.folder { background: rgba(0, 122, 255, 0.1); color: #007aff; }
.type-label.image { background: rgba(255, 107, 157, 0.1); color: #ff6b9d; }
.type-label.video { background: rgba(179, 136, 255, 0.1); color: #b388ff; }
.type-label.audio { background: rgba(90, 240, 255, 0.1); color: #5af0ff; }
.type-label.document { background: rgba(255, 215, 0, 0.1); color: #ffd700; }
.type-label.archive { background: rgba(255, 152, 51, 0.1); color: #ff9933; }
.type-label.code { background: rgba(40, 200, 64, 0.1); color: #28c840; }
.type-label.other { background: rgba(136, 136, 136, 0.1); color: #888; }

.status-icon {
  flex-shrink: 0;
}

.status-icon.encrypted { color: var(--warning); }
.status-icon.starred { color: #ffcc00; }

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-card {
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  width: 360px;
  padding: 24px;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-input {
  width: 100%;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-sm);
  font-size: 15px;
  color: var(--text-primary);
  margin-bottom: 20px;
}

.modal-input:focus {
  outline: none;
  border-color: var(--accent);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.modal-btn {
  padding: 10px 20px;
  border-radius: var(--radius-sm);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
}

.modal-btn.secondary {
  background: var(--bg-tertiary);
  border: none;
  color: var(--text-primary);
}

.modal-btn.secondary:hover {
  background: var(--border-light);
}

.modal-btn.primary {
  background: var(--accent);
  border: none;
  color: white;
}

.modal-btn.primary:hover {
  background: var(--accent-hover);
}

/* Transitions */
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.3s ease;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-card,
.modal-leave-to .modal-card {
  transform: scale(0.95);
}

.file-grid-enter-active,
.file-grid-leave-active {
  transition: all 0.3s ease;
}

.file-grid-enter-from,
.file-grid-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

/* Scrollbar */
.grid-view::-webkit-scrollbar,
.list-view::-webkit-scrollbar {
  width: 8px;
}

.grid-view::-webkit-scrollbar-track,
.list-view::-webkit-scrollbar-track {
  background: transparent;
}

.grid-view::-webkit-scrollbar-thumb,
.list-view::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
}

.grid-view::-webkit-scrollbar-thumb:hover,
.list-view::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}

/* Responsive */
@media (max-width: 768px) {
  .toolbar-center {
    display: none;
  }

  .search-input {
    width: 140px;
  }

  .search-input:focus {
    width: 180px;
  }

  .grid-container {
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 12px;
  }

  .file-preview {
    height: 100px;
  }
}
</style>
