<template>
  <div class="duplicates-panel">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-col">[=]</span>
        <h2 class="panel-title">DUPLICATE FILES</h2>
      </div>
      <button class="bw-btn-sm" @click="store.fetchDuplicates()">
        <Icon icon="mdi:refresh" width="14" height="14" />
        [REFRESH]
      </button>
    </div>

    <div v-if="store.isLoadingDuplicates" class="loading-state">
      <Icon icon="svg-spinners:blocks-wave" width="24" height="24" />
      <span>Scanning for duplicates...</span>
    </div>

    <div v-else-if="store.duplicateGroups.length === 0" class="empty-state">
      <Icon icon="mdi:check-circle-outline" width="32" height="32" style="color:#00ff41" />
      <span>NO DUPLICATES FOUND</span>
      <span class="text-muted">All files are unique</span>
    </div>

    <div v-else class="groups-list">
      <div class="summary-bar">
        <span class="summary-text">{{ store.duplicateGroups.length }} GROUPS</span>
        <span class="summary-text">{{ totalDuplicateFiles }} FILES</span>
        <span class="summary-text">{{ formatBytes(totalWastedSpace) }} WASTED</span>
      </div>

      <div v-for="(group, gi) in store.duplicateGroups" :key="gi" class="dup-group">
        <div class="group-header">
          <Icon icon="mdi:content-copy" width="12" height="12" class="group-icon" />
          <span class="group-label">GROUP {{ gi + 1 }}</span>
          <span class="group-count">{{ group.length }} COPIES</span>
        </div>

        <div class="group-files">
          <div
            v-for="file in group"
            :key="file.id"
            class="dup-file"
            @click="selectFile(file)"
          >
            <Icon :icon="fileIcon(file)" width="14" height="14" class="file-icon" />
            <div class="file-info">
              <span class="file-name">{{ file.name }}</span>
              <span class="file-meta text-muted">{{ file.path || '/' }} | {{ formatBytes(file.sizeBytes) }}</span>
            </div>
            <span v-if="file.accountId" class="file-account">{{ file.accountId }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import type { FileNode } from '@/types'

const store = useAppStore()

onMounted(() => {
  if (store.duplicateGroups.length === 0) {
    store.fetchDuplicates()
  }
})

const totalDuplicateFiles = computed(() =>
  store.duplicateGroups.reduce((sum, group) => sum + group.length, 0)
)

const totalWastedSpace = computed(() =>
  store.duplicateGroups.reduce((sum, group) => {
    if (group.length < 2) return sum
    const fileSize = group[0].sizeBytes || 0
    return sum + fileSize * (group.length - 1)
  }, 0)
)

function fileIcon(file: FileNode): string {
  const ext = file.name.split('.').pop()?.toLowerCase() || ''
  const iconMap: Record<string, string> = {
    jpg: 'mdi:file-image', jpeg: 'mdi:file-image', png: 'mdi:file-image',
    gif: 'mdi:file-image', webp: 'mdi:file-image', svg: 'mdi:file-image',
    mp4: 'mdi:file-video', avi: 'mdi:file-video', mkv: 'mdi:file-video',
    mp3: 'mdi:file-music', wav: 'mdi:file-music', flac: 'mdi:file-music',
    pdf: 'mdi:file-document', doc: 'mdi:file-document', docx: 'mdi:file-document',
    txt: 'mdi:file-document', md: 'mdi:file-document',
    zip: 'mdi:file-archive', tar: 'mdi:file-archive', gz: 'mdi:file-archive',
    rs: 'mdi:language-rust', js: 'mdi:language-javascript', ts: 'mdi:language-typescript',
    py: 'mdi:language-python', go: 'mdi:language-go',
  }
  return iconMap[ext] || 'mdi:file-outline'
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

function selectFile(file: FileNode) {
  store.selectFile(file.id)
}
</script>

<style scoped>
.duplicates-panel {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
  font-family: 'Courier New', monospace;
  background: #0a0a0a;
  color: #00ff41;
}

.duplicates-panel::-webkit-scrollbar { width: 4px; }
.duplicates-panel::-webkit-scrollbar-track { background: #0a0a0a; }
.duplicates-panel::-webkit-scrollbar-thumb { background: #00ff41; }

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
  padding-bottom: 12px;
  border-bottom: 1px solid #1a1a1a;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-col { color: #00ff41; font-weight: 700; }

.panel-title {
  font-size: 13px;
  font-weight: 800;
  color: #e0e0e0;
  letter-spacing: 2px;
  margin: 0;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 40px;
  color: #555;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 40px;
  color: #00ff41;
}

.text-muted { color: #555; font-size: 10px; }

.summary-bar {
  display: flex;
  gap: 16px;
  padding: 10px 12px;
  background: rgba(0, 255, 65, 0.05);
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  margin-bottom: 16px;
}

.summary-text {
  font-size: 10px;
  font-weight: 700;
  color: #00ff41;
  letter-spacing: 1px;
}

.groups-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.dup-group {
  background: #0d0d0d;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: rgba(255, 95, 87, 0.08);
  border-bottom: 1px solid #1a1a1a;
}

.group-icon { color: #ff5f57; }

.group-label {
  font-size: 10px;
  font-weight: 700;
  color: #ff5f57;
  letter-spacing: 1px;
}

.group-count {
  font-size: 9px;
  color: #555;
  margin-left: auto;
}

.group-files {
  display: flex;
  flex-direction: column;
}

.dup-file {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  border-bottom: 1px solid #1a1a1a;
  cursor: pointer;
  transition: background 0.1s;
}

.dup-file:last-child { border-bottom: none; }
.dup-file:hover { background: rgba(0, 255, 65, 0.02); }

.file-icon { color: #888; flex-shrink: 0; }

.file-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.file-name {
  font-size: 11px;
  color: #ccc;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  font-size: 9px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-account {
  font-size: 8px;
  color: #555;
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
}
</style>
