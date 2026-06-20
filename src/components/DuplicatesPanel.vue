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
      <Icon icon="mdi:check-circle-outline" width="32" height="32" style="color:var(--text-accent)" />
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
  padding: 20px;
  height: 100%;
  overflow-y: auto;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}

.duplicates-panel::-webkit-scrollbar { width: 4px; }
.duplicates-panel::-webkit-scrollbar-track { background: transparent; }
.duplicates-panel::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 2px; }

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-col { color: var(--text-accent); font-weight: 700; }

.panel-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 1px;
  margin: 0;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 40px;
  color: var(--text-muted);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 40px;
  color: var(--text-accent);
}

.text-muted { color: var(--text-muted); font-size: 10px; }

.summary-bar {
  display: flex;
  gap: 16px;
  padding: 12px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-md);
  margin-bottom: 16px;
}

.summary-text {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-accent);
  letter-spacing: 1px;
}

.groups-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.dup-group {
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: rgba(255, 95, 87, 0.08);
  border-bottom: 1px solid var(--border-glass);
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
  color: var(--text-muted);
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
  border-bottom: 1px solid var(--border-glass);
  cursor: pointer;
  transition: background 0.1s;
}

.dup-file:last-child { border-bottom: none; }
.dup-file:hover { background: var(--accent-dim); }

.file-icon { color: var(--text-muted); flex-shrink: 0; }

.file-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.file-name {
  font-size: 11px;
  color: var(--text-primary);
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
  color: var(--text-muted);
  padding: 2px 6px;
  background: var(--bg-surface);
  border-radius: 3px;
}
</style>
