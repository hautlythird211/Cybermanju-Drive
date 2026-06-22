<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useMedia } from '@/composables/useMedia'
import type { FileNode, FileMediaData } from '@/types'

const store = useAppStore()
const { openMediaOverlay, getFileBytesForPreview, getMediaInfo } = useMedia()

const emit = defineEmits<{
  close: []
}>()

const filterType = ref<'all' | 'image' | 'video' | 'audio'>('all')
const sortBy = ref<'name' | 'date' | 'size'>('date')
const sortDir = ref<'asc' | 'desc'>('desc')
const loading = ref(false)
const openingFileId = ref<string | null>(null)

const mediaFiles = computed(() => {
  let files = store.files.filter(f => {
    const mime = f.mimeType || ''
    return mime.startsWith('image/') || mime.startsWith('video/') || mime.startsWith('audio/')
  })

  if (filterType.value !== 'all') {
    files = files.filter(f => {
      const mime = f.mimeType || ''
      if (filterType.value === 'image') return mime.startsWith('image/')
      if (filterType.value === 'video') return mime.startsWith('video/')
      if (filterType.value === 'audio') return mime.startsWith('audio/')
      return true
    })
  }

  return [...files].sort((a, b) => {
    let cmp = 0
    if (sortBy.value === 'name') cmp = a.name.localeCompare(b.name)
    else if (sortBy.value === 'date') cmp = new Date(a.modifiedAt).getTime() - new Date(b.modifiedAt).getTime()
    else if (sortBy.value === 'size') cmp = a.sizeBytes - b.sizeBytes
    return sortDir.value === 'asc' ? cmp : -cmp
  })
})

const imageCount = computed(() => store.files.filter(f => f.mimeType?.startsWith('image/')).length)
const videoCount = computed(() => store.files.filter(f => f.mimeType?.startsWith('video/')).length)
const audioCount = computed(() => store.files.filter(f => f.mimeType?.startsWith('audio/')).length)
const totalCount = computed(() => imageCount.value + videoCount.value + audioCount.value)

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
}

function formatDate(date: string): string {
  try {
    const d = new Date(date)
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  } catch { return date }
}

function getMediaClass(file: FileNode): string {
  const mime = file.mimeType || ''
  if (mime.startsWith('image/')) return 'image'
  if (mime.startsWith('video/')) return 'video'
  if (mime.startsWith('audio/')) return 'audio'
  return 'other'
}

function getMediaIcon(file: FileNode): string {
  const mime = file.mimeType || ''
  if (mime.startsWith('image/')) return 'mdi:image-outline'
  if (mime.startsWith('video/')) return 'mdi:video-outline'
  if (mime.startsWith('audio/')) return 'mdi:music-note-outline'
  return 'mdi:file-outline'
}

async function openMedia(file: FileNode) {
  if (openingFileId.value) return
  openingFileId.value = file.id
  loading.value = true
  try {
    const mime = file.mimeType || ''
    const type = mime.startsWith('image/') ? 'image' : mime.startsWith('video/') ? 'video' : 'audio'

    // Uncompress-on-demand: fetch file bytes from disk, decompress layers
    const fileBytes = await getFileBytesForPreview(file.id)

    // Get media info from the decompressed bytes
    const mediaData = await getMediaInfo(file.id, file.name, fileBytes)

    openMediaOverlay(type, mediaData, fileBytes)
  } catch (e) {
    console.error('Failed to open media:', e)
    store.notifyError?.(`Failed to open ${file.name}: ${e}`)
  } finally {
    loading.value = false
    openingFileId.value = null
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="media-library">
    <div class="library-header">
      <Icon icon="mdi:image-multiple-outline" width="20" height="20" />
      <span class="header-title">MEDIA LIBRARY</span>
      <span class="header-count">{{ totalCount }} files</span>
      <button class="close-btn" @click="emit('close')">
        <Icon icon="mdi:close" width="18" height="18" />
      </button>
    </div>

    <div class="library-stats">
      <div class="stat" :class="{ active: filterType === 'all' }" @click="filterType = 'all'">
        <Icon icon="mdi:folder-multiple-outline" width="16" height="16" />
        <span class="stat-count">{{ totalCount }}</span>
        <span class="stat-label">All</span>
      </div>
      <div class="stat image" :class="{ active: filterType === 'image' }" @click="filterType = 'image'">
        <Icon icon="mdi:image-outline" width="16" height="16" />
        <span class="stat-count">{{ imageCount }}</span>
        <span class="stat-label">Images</span>
      </div>
      <div class="stat video" :class="{ active: filterType === 'video' }" @click="filterType = 'video'">
        <Icon icon="mdi:video-outline" width="16" height="16" />
        <span class="stat-count">{{ videoCount }}</span>
        <span class="stat-label">Videos</span>
      </div>
      <div class="stat audio" :class="{ active: filterType === 'audio' }" @click="filterType = 'audio'">
        <Icon icon="mdi:music-note-outline" width="16" height="16" />
        <span class="stat-count">{{ audioCount }}</span>
        <span class="stat-label">Audio</span>
      </div>
    </div>

    <div class="library-toolbar">
      <div class="sort-group">
        <button class="sort-btn" :class="{ active: sortBy === 'name' }" @click="sortBy = 'name'">Name</button>
        <button class="sort-btn" :class="{ active: sortBy === 'date' }" @click="sortBy = 'date'">Date</button>
        <button class="sort-btn" :class="{ active: sortBy === 'size' }" @click="sortBy = 'size'">Size</button>
        <button class="sort-dir" @click="sortDir = sortDir === 'asc' ? 'desc' : 'asc'">
          <Icon :icon="sortDir === 'asc' ? 'mdi:sort-ascending' : 'mdi:sort-descending'" width="14" height="14" />
        </button>
      </div>
    </div>

    <div class="library-grid">
      <div
        v-for="file in mediaFiles"
        :key="file.id"
        class="media-card"
        :class="getMediaClass(file)"
        @click="openMedia(file)"
        @dblclick="openMedia(file)"
      >
        <div class="media-thumb">
          <img v-if="file.thumbnailPath" :src="file.thumbnailPath" :alt="file.name" class="thumb-img" />
          <div v-else class="thumb-icon">
            <Icon :icon="getMediaIcon(file)" width="28" height="28" />
          </div>
          <div class="media-overlay">
            <Icon :icon="getMediaIcon(file)" width="32" height="32" />
          </div>
        </div>
        <div class="media-info">
          <div class="media-name truncate">{{ file.name }}</div>
          <div class="media-meta">
            <span class="media-size">{{ formatSize(file.sizeBytes) }}</span>
            <span class="media-date">{{ formatDate(file.modifiedAt) }}</span>
          </div>
        </div>
      </div>

      <div v-if="mediaFiles.length === 0" class="empty-state">
        <Icon icon="mdi:image-off-outline" width="48" height="48" class="empty-icon" />
        <div class="empty-title">No media files</div>
        <div class="empty-subtitle">Import images, videos, or audio files</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.media-library {
  position: fixed;
  inset: 0;
  background: #08080a;
  z-index: 10001;
  display: flex;
  flex-direction: column;
  font-family: var(--font-mono, monospace);
}

.library-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 20px;
  background: #0e0e12;
  border-bottom: 1px solid #22222a;
  color: #ececf0;
}

.header-title {
  font-size: 12px;
  font-weight: 800;
  letter-spacing: 2px;
}

.header-count {
  font-size: 10px;
  color: #50505e;
  margin-left: auto;
}

.close-btn {
  background: none;
  border: none;
  color: #50505e;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.close-btn:hover {
  color: #ff453a;
  background: rgba(255, 69, 58, 0.1);
}

.library-stats {
  display: flex;
  gap: 4px;
  padding: 12px 20px;
  background: #0e0e12;
  border-bottom: 1px solid #1a1a22;
}

.stat {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 8px;
  cursor: pointer;
  color: #50505e;
  transition: all 0.15s;
  font-size: 10px;
}

.stat:hover { background: #1e1e26; color: #a0a0b0; }
.stat.active { background: #1e1e26; color: #ececf0; }
.stat.image.active { color: #ff6b9d; background: rgba(255, 107, 157, 0.1); }
.stat.video.active { color: #b388ff; background: rgba(179, 136, 255, 0.1); }
.stat.audio.active { color: #5af0ff; background: rgba(90, 240, 255, 0.1); }

.stat-count {
  font-weight: 800;
  font-size: 12px;
}

.stat-label {
  font-weight: 600;
  letter-spacing: 0.5px;
}

.library-toolbar {
  display: flex;
  align-items: center;
  padding: 8px 20px;
  border-bottom: 1px solid #1a1a22;
}

.sort-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.sort-btn {
  background: none;
  border: none;
  color: #50505e;
  font-family: inherit;
  font-size: 10px;
  font-weight: 600;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  letter-spacing: 0.5px;
}

.sort-btn:hover { color: #a0a0b0; }
.sort-btn.active { color: #00ff41; background: rgba(0, 255, 65, 0.1); }

.sort-dir {
  background: none;
  border: none;
  color: #50505e;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  margin-left: 4px;
}

.sort-dir:hover { color: #a0a0b0; }

.library-grid {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
  align-content: start;
}

.media-card {
  border-radius: 10px;
  overflow: hidden;
  background: #16161c;
  border: 1px solid #22222a;
  cursor: pointer;
  transition: all 0.2s;
}

.media-card:hover {
  border-color: #333;
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
}

.media-card.image:hover { border-color: rgba(255, 107, 157, 0.3); }
.media-card.video:hover { border-color: rgba(0, 255, 65, 0.3); }
.media-card.audio:hover { border-color: rgba(90, 240, 255, 0.3); }

.media-thumb {
  position: relative;
  height: 140px;
  background: #0e0e12;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumb-icon {
  color: #50505e;
}

.media-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
  opacity: 0;
  transition: opacity 0.2s;
  color: #fff;
}

.media-card:hover .media-overlay {
  opacity: 1;
}

.media-info {
  padding: 10px 12px;
}

.media-name {
  font-size: 11px;
  font-weight: 600;
  color: #ececf0;
  margin-bottom: 4px;
}

.media-meta {
  display: flex;
  gap: 8px;
  font-size: 9px;
  color: #50505e;
}

.empty-state {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  gap: 12px;
}

.empty-icon { color: #333; }
.empty-title { font-size: 14px; font-weight: 700; color: #50505e; }
.empty-subtitle { font-size: 11px; color: #333; }
</style>
