<template>
  <div class="panel-page">
    <div class="panel-card">
      <div class="history-header">
        <div class="history-title-row">
          <div class="panel-title">HISTORY</div>
          <span class="history-count text-muted">{{ history.allEntries.length }} ACTIONS</span>
        </div>
        <div class="history-toolbar">
          <button class="panel-btn" :disabled="!history.canUndo" @click="history.undo()" title="UNDO (Ctrl+Z)">[UNDO]</button>
          <button class="panel-btn" :disabled="!history.canRedo" @click="history.redo()" title="REDO (Ctrl+Shift+Z)">[REDO]</button>
          <button class="panel-btn panel-btn-danger" @click="onClear" title="CLEAR HISTORY">[CLEAR]</button>
        </div>
      </div>
      <p class="panel-hint">ATOMIC ACTION HISTORY — UNDO/REDO ACROSS ALL OPERATIONS. PERSISTED ACROSS RESTARTS.</p>

      <div v-if="history.allEntries.length === 0" class="empty-state">
        <Icon icon="svg-spinners:3-dots-scale-middle" width="18" height="18" class="empty-spinner" />
        <p class="text-muted">NO HISTORY YET</p>
      </div>

      <div v-else class="history-list">
        <div
          v-for="(entry, i) in history.allEntries"
          :key="entry.id"
          class="history-item"
          :class="{ 'history-item-latest': i === 0 }"
        >
          <div class="history-icon-col">
            <div class="history-type-icon" :class="'htype-' + entry.type.split(':')[0]">
              {{ TYPE_ICONS[entry.type] || '[*]' }}
            </div>
          </div>
          <div class="history-info">
            <span class="history-desc">{{ entry.description }}</span>
            <span class="history-meta text-muted">
              {{ formatTime(entry.timestamp) }}
              <template v-if="entry.affectedFileIds.length"> · {{ entry.affectedFileIds.length }} FILE{{ entry.affectedFileIds.length > 1 ? 'S' : '' }}</template>
            </span>
          </div>
          <div class="history-type-tag">{{ entry.type }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useHistoryStore } from '@/stores/history'
import type { HistoryActionType } from '@/types'

const history = useHistoryStore()

const TYPE_ICONS: Partial<Record<HistoryActionType, string>> = {
  'file:delete': '[X]',
  'file:restore': '[R]',
  'file:rename': '[N]',
  'file:create': '[+]',
  'file:move': '[M]',
  'encryption:encrypt': '[E]',
  'encryption:decrypt': '[D]',
  'compression:compress': '[C]',
  'compression:decompress': '[U]',
  'collection:add': '[A]',
  'collection:remove': '[-]',
  'face:rename': '[F]',
  'account:switch': '[S]',
  'user:role': '[P]',
  'share:create': '[L]',
}

function formatTime(ts: number): string {
  const diff = Date.now() - ts
  if (diff < 10000) return 'JUST NOW'
  if (diff < 60000) return `${Math.floor(diff / 1000)}S AGO`
  if (diff < 3600000) return `${Math.floor(diff / 60000)}M AGO`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}H AGO`
  return new Date(ts).toLocaleDateString()
}

function onClear() {
  window.dispatchEvent(new CustomEvent('cybermanju:confirm-clear-history'))
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

.panel-btn:hover:not(:disabled) {
  border-color: #555;
  color: #e0e0e0;
  background: #222;
}

.panel-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.panel-btn-danger:hover:not(:disabled) {
  border-color: #ff5f57;
  color: #ff5f57;
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
.history-header {
  margin-bottom: 6px;
}

.history-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.history-count {
  font-family: 'Courier New', monospace;
  font-size: 9px;
}

.history-toolbar {
  display: flex;
  gap: 4px;
  margin-bottom: 4px;
}

/* ── List ───────────────────────────────────────── */
.history-list {
  display: flex;
  flex-direction: column;
  border: 1px solid #252525;
  border-radius: 6px;
  overflow: hidden;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid #1d1d1d;
  transition: background 0.08s;
}

.history-item:last-child {
  border-bottom: none;
}

.history-item:hover {
  background: #1a1a1a;
}

.history-item-latest {
  border-left: 2px solid rgba(0, 255, 65, 0.3);
  background: rgba(0, 255, 65, 0.03);
}

.history-icon-col {
  width: 24px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.history-type-icon {
  font-size: 9px;
  font-weight: 700;
}

.htype-file { color: #5dade2; }
.htype-encryption { color: #ffb347; }
.htype-compression { color: #58d68d; }
.htype-collection { color: #af7ac5; }
.htype-face { color: #f1948a; }
.htype-account { color: #85c1e9; }
.htype-user { color: #f0b27a; }
.htype-share { color: #82e0aa; }

.history-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.history-desc {
  font-size: 10px;
  font-weight: 600;
  color: #ccc;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-meta {
  font-size: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-type-tag {
  font-family: 'Courier New', monospace;
  font-size: 7px;
  color: #666;
  background: #151515;
  padding: 1px 5px;
  border-radius: 3px;
  flex-shrink: 0;
  letter-spacing: 0.3px;
  border: 1px solid #252525;
}
</style>
