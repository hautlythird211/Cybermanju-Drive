<template>
  <div class="panel-page">
    <div class="panel-card">
      <div class="trash-header">
        <div class="trash-title-row">
          <div class="panel-title">TRASH</div>
          <span class="trash-count-badge" v-if="store.trashItems.length">{{ store.trashItems.length }}</span>
        </div>
        <div class="trash-toolbar">
          <div class="trash-search">
            <input
              v-model="searchQuery"
              class="trash-search-input"
              placeholder="FILTER..."
              @input="onSearchInput"
            />
          </div>
          <select v-model="sortKey" class="trash-sort-select" title="SORT BY">
            <option value="date-desc">NEWEST</option>
            <option value="date-asc">OLDEST</option>
            <option value="name-asc">A-Z</option>
            <option value="name-desc">Z-A</option>
            <option value="size-desc">LARGEST</option>
            <option value="size-asc">SMALLEST</option>
          </select>
          <div class="trash-btn-group">
            <button
              v-if="selectedIds.length > 0"
              class="panel-btn panel-btn-restore"
              @click="restoreSelected"
              title="RESTORE SELECTED"
            >[RST {{ selectedIds.length }}]</button>
            <button
              v-if="selectedIds.length > 0"
              class="panel-btn panel-btn-danger"
              @click="deleteSelected"
              title="DELETE SELECTED PERMANENTLY"
            >[DEL {{ selectedIds.length }}]</button>
            <button class="panel-btn" @click="onRefresh" title="REFRESH">[R]</button>
            <button
              class="panel-btn panel-btn-danger"
              @click="onEmptyTrash"
              title="EMPTY TRASH"
              :disabled="store.trashItems.length === 0"
            >[EMPTY]</button>
          </div>
        </div>
      </div>
      <p class="panel-hint">DELETED FILES CAN BE RESTORED FROM HERE. SELECT MULTIPLE WITH CHECKBOXES.</p>

      <div v-if="store.trashItems.length === 0" class="empty-state">
        <Icon icon="svg-spinners:6-dots-rotate" width="18" height="18" class="empty-spinner" />
        <p class="text-muted">NO FILES IN TRASH</p>
      </div>

      <template v-else-if="filteredItems.length === 0">
        <div class="empty-state">
          <p class="text-muted">NO FILES MATCH "{{ searchQuery }}"</p>
        </div>
      </template>

      <div v-else class="trash-list">
        <div class="trash-list-header">
          <label class="trash-checkbox-wrapper" title="SELECT ALL">
            <input
              type="checkbox"
              class="trash-checkbox"
              :checked="allSelected"
              @change="toggleSelectAll"
            />
          </label>
          <span class="trash-h-col-name">NAME</span>
          <span class="trash-h-col-size">SIZE</span>
          <span class="trash-h-col-date">DELETED</span>
          <span class="trash-h-col-actions">ACTIONS</span>
        </div>
        <div
          v-for="item in filteredItems"
          :key="item.id"
          class="trash-item"
          :class="{ 'trash-item-selected': selectedIds.includes(item.id) }"
          @contextmenu.prevent="onContextMenu($event, item)"
        >
          <label class="trash-checkbox-wrapper" @click.stop>
            <input
              type="checkbox"
              class="trash-checkbox"
              :checked="selectedIds.includes(item.id)"
              @change="toggleSelect(item.id)"
            />
          </label>
          <div class="trash-icon-group" @click="store.selectFile(item.originalFile.id); wm.open('files')">
            <Icon
              v-if="item.originalFile.fileType !== 'folder'"
              icon="mdi:file-outline"
              width="14"
              height="14"
              class="trash-type-icon"
            />
            <Icon
              v-else
              icon="mdi:folder-outline"
              width="14"
              height="14"
              class="trash-type-icon trash-folder-icon"
            />
          </div>
          <div class="trash-info" @click="store.selectFile(item.originalFile.id); wm.open('files')">
            <span class="trash-name truncate">{{ item.originalFile.name }}</span>
            <span class="trash-meta text-muted">
              {{ getOriginalPath(item) }}
            </span>
          </div>
          <span class="trash-size text-muted">{{ formatSize(item.originalFile.size) }}</span>
          <span class="trash-date text-muted" :title="new Date(item.deletedAt).toLocaleString()">
            {{ relativeTime(item.deletedAt) }}
          </span>
          <div class="trash-item-actions">
            <button
              class="trash-action-btn"
              @click="store.restoreTrashItem(item.originalFile.id)"
              title="RESTORE TO ORIGINAL LOCATION"
            >[RST]</button>
            <button
              class="trash-action-btn danger"
              @click="store.deleteFromTrash(item.originalFile.id)"
              title="DELETE PERMANENTLY"
            >[DEL]</button>
          </div>
        </div>
      </div>

      <div v-if="store.trashItems.length > 0" class="trash-footer">
        <span class="text-muted">{{ filteredItems.length }} / {{ store.trashItems.length }} ITEMS</span>
        <button class="panel-btn panel-btn-danger" @click="onEmptyTrash">[EMPTY ALL]</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'
import { useContextMenu } from '@/composables/useContextMenu'
import type { TrashItem } from '@/types'

const store = useAppStore()
const wm = useWindowManager()
const ctx = useContextMenu()

const searchQuery = ref('')
const sortKey = ref<string>('date-desc')
const selectedIds = ref<string[]>([])

const filteredItems = computed(() => {
  let items = [...store.trashItems]

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    items = items.filter(i =>
      i.originalFile.name.toLowerCase().includes(q) ||
      (i.restorePath && i.restorePath.toLowerCase().includes(q))
    )
  }

  const sf = sortKey.value
  items.sort((a, b) => {
    switch (sf) {
      case 'date-asc': return new Date(a.deletedAt).getTime() - new Date(b.deletedAt).getTime()
      case 'name-asc': return a.originalFile.name.localeCompare(b.originalFile.name)
      case 'name-desc': return b.originalFile.name.localeCompare(a.originalFile.name)
      case 'size-desc': return (b.originalFile.size || 0) - (a.originalFile.size || 0)
      case 'size-asc': return (a.originalFile.size || 0) - (b.originalFile.size || 0)
      default: return new Date(b.deletedAt).getTime() - new Date(a.deletedAt).getTime()
    }
  })

  return items
})

const allSelected = computed(() =>
  filteredItems.value.length > 0 && selectedIds.value.length === filteredItems.value.length
)

function onSearchInput() {
  selectedIds.value = []
}

function toggleSelect(id: string) {
  const idx = selectedIds.value.indexOf(id)
  if (idx === -1) selectedIds.value.push(id)
  else selectedIds.value.splice(idx, 1)
}

function toggleSelectAll() {
  if (allSelected.value) {
    selectedIds.value = []
  } else {
    selectedIds.value = filteredItems.value.map(i => i.id)
  }
}

async function restoreSelected() {
  for (const id of selectedIds.value) {
    const item = store.trashItems.find(i => i.id === id)
    if (item) await store.restoreTrashItem(item.originalFile.id)
  }
  selectedIds.value = []
}

async function deleteSelected() {
  for (const id of selectedIds.value) {
    const item = store.trashItems.find(i => i.id === id)
    if (item) await store.deleteFromTrash(item.originalFile.id)
  }
  selectedIds.value = []
}

async function onRefresh() {
  await store.fetchTrashItems()
  selectedIds.value = []
}

async function onEmptyTrash() {
  // Use the ConfirmDialog mechanism — fire event, App.vue picks it up
  window.dispatchEvent(new CustomEvent('cybermanju:confirm-empty-trash'))
}

function getOriginalPath(item: TrashItem): string {
  return item.restorePath ? `[${item.restorePath}]` : '[UNKNOWN]'
}

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let size = bytes
  while (size >= 1024 && i < units.length - 1) { size /= 1024; i++ }
  return `${size.toFixed(1)} ${units[i]}`
}

function relativeTime(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 1) return 'JUST NOW'
  if (mins < 60) return `${mins}M AGO`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs}H AGO`
  const days = Math.floor(hrs / 24)
  if (days < 30) return `${days}D AGO`
  return new Date(dateStr).toLocaleDateString()
}

function onContextMenu(e: MouseEvent, item: TrashItem) {
  ctx.open(e, 'trash_item', {
    restore: () => store.restoreTrashItem(item.originalFile.id),
    deletePermanently: () => store.deleteFromTrash(item.originalFile.id),
  })
}
</script>

<style scoped>
.panel-page {
  padding: 12px;
  height: 100%;
  overflow-y: auto;
}

.panel-card {
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 8px;
  padding: 16px;
}

.panel-title {
  font-family: 'Courier New', monospace;
  font-size: 11px;
  font-weight: 700;
  color: #e0e0e0;
  letter-spacing: 1px;
}

.panel-hint {
  font-family: 'Courier New', monospace;
  font-size: 9px;
  color: #555;
  margin-bottom: 12px;
}

.panel-btn {
  background: transparent;
  border: 1px solid #333;
  color: #999;
  padding: 2px 8px;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  font-weight: 700;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.1s;
}

.panel-btn:hover {
  border-color: #555;
  color: #e0e0e0;
  background: #222;
}

.panel-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.panel-btn-danger:hover {
  border-color: #ff5f57;
  color: #ff5f57;
}

.panel-btn-restore:hover {
  border-color: #00ff41;
  color: #00ff41;
}

.text-muted {
  color: #555 !important;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 36px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 10px;
}

.empty-spinner {
  opacity: 0.5;
}

/* ── Header ─────────────────────────────────────── */
.trash-header {
  margin-bottom: 6px;
}

.trash-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.trash-count-badge {
  background: rgba(0, 255, 65, 0.1);
  border: 1px solid rgba(0, 255, 65, 0.2);
  color: #00ff41;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  font-weight: 700;
  padding: 0 6px;
  border-radius: 3px;
  line-height: 16px;
}

/* ── Toolbar ────────────────────────────────────── */
.trash-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 4px;
}

.trash-search {
  flex: 1;
  min-width: 120px;
}

.trash-search-input {
  width: 100%;
  background: #111;
  border: 1px solid #333;
  color: #ccc;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  padding: 3px 6px;
  border-radius: 4px;
  outline: none;
  box-sizing: border-box;
}

.trash-search-input:focus {
  border-color: #555;
}

.trash-sort-select {
  background: #111;
  border: 1px solid #333;
  color: #ccc;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  padding: 3px 4px;
  cursor: pointer;
  appearance: none;
  border-radius: 4px;
}

.trash-btn-group {
  display: flex;
  gap: 4px;
}

/* ── List ───────────────────────────────────────── */
.trash-list {
  display: flex;
  flex-direction: column;
  border: 1px solid #252525;
  border-radius: 6px;
  overflow: hidden;
}

.trash-list-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  background: #151515;
  border-bottom: 1px solid #252525;
  font-family: 'Courier New', monospace;
  font-size: 8px;
  font-weight: 700;
  color: #666;
  letter-spacing: 0.5px;
}

.trash-h-col-name {
  flex: 1;
  min-width: 0;
}

.trash-h-col-size {
  width: 72px;
  text-align: right;
  flex-shrink: 0;
}

.trash-h-col-date {
  width: 72px;
  text-align: right;
  flex-shrink: 0;
}

.trash-h-col-actions {
  width: 80px;
  text-align: right;
  flex-shrink: 0;
}

/* ── Item ───────────────────────────────────────── */
.trash-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid #1d1d1d;
  cursor: default;
  transition: background 0.08s;
}

.trash-item:last-child {
  border-bottom: none;
}

.trash-item:hover {
  background: #1a1a1a;
}

.trash-item-selected {
  background: rgba(0, 255, 65, 0.04) !important;
  border-left: 2px solid rgba(0, 255, 65, 0.3);
}

.trash-checkbox-wrapper {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.trash-checkbox {
  appearance: none;
  width: 12px;
  height: 12px;
  border: 1px solid #555;
  background: #111;
  cursor: pointer;
  border-radius: 2px;
  margin: 0;
  flex-shrink: 0;
}

.trash-checkbox:checked {
  background: #00ff41;
  border-color: #00ff41;
}

.trash-icon-group {
  width: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  cursor: pointer;
}

.trash-type-icon {
  color: #666;
}

.trash-type-icon.trash-folder-icon {
  color: #888;
}

.trash-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
  cursor: pointer;
}

.trash-name {
  font-size: 10px;
  font-weight: 600;
  color: #ccc;
}

.trash-meta {
  font-size: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trash-size {
  width: 72px;
  text-align: right;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trash-date {
  width: 72px;
  text-align: right;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trash-item-actions {
  width: 80px;
  display: flex;
  gap: 4px;
  justify-content: flex-end;
  flex-shrink: 0;
}

.trash-action-btn {
  background: transparent;
  border: 1px solid #333;
  color: #888;
  padding: 1px 6px;
  font-family: 'Courier New', monospace;
  font-size: 8px;
  font-weight: 700;
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.1s;
}

.trash-action-btn:hover {
  border-color: #555;
  color: #e0e0e0;
}

.trash-action-btn.danger:hover {
  border-color: #ff5f57;
  color: #ff5f57;
}

/* ── Footer ─────────────────────────────────────── */
.trash-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 4px 0;
  font-family: 'Courier New', monospace;
  font-size: 9px;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
