<template>
  <div class="activity-panel">
    <div class="panel-card">
      <div class="panel-header">
        <div class="panel-title">ACTIVITY LOG</div>
        <button class="panel-btn" @click="store.fetchAuditLog()" title="REFRESH">[R]</button>
      </div>
      <p class="panel-hint">FILE OPERATIONS TIMELINE.</p>
      <div v-if="store.auditLog.length === 0" class="empty-state">
        <Icon icon="svg-spinners:clock" width="18" height="18" class="empty-spinner" />
        <p class="text-muted">NO RECENT ACTIVITY</p>
      </div>
      <div v-else class="activity-list">
        <div v-for="entry in store.auditLog" :key="entry.id" class="activity-item">
          <span class="activity-icon" :class="iconClass(entry.action)">{{ iconLabel(entry.action) }}</span>
          <div class="activity-body">
            <div class="activity-header">
              <span class="activity-action" :class="actionClass(entry.action)">{{ entry.action.toUpperCase() }}</span>
              <span class="activity-entity text-muted">{{ entry.entityType }}</span>
            </div>
            <div class="activity-details">
              <span class="activity-filename" v-if="entry.details?.fileName">{{ entry.details.fileName }}</span>
              <span class="activity-timestamp text-muted">{{ formatTime(entry.timestamp) }}</span>
            </div>
            <div v-if="entry.details && Object.keys(entry.details).length > 1" class="activity-extra text-muted">
              {{ formatDetails(entry.details) }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'

const store = useAppStore()

onMounted(() => {
  store.fetchAuditLog()
})

const actionIcons: Record<string, string> = {
  encrypt: '[@]',
  decrypt: '[D]',
  delete: '[X]',
  sync: '[S]',
  batch: '[B]',
}

const actionColors: Record<string, string> = {
  encrypt: 'encrypt',
  decrypt: 'decrypt',
  delete: 'delete',
  sync: 'sync',
  batch: 'batch',
}

function iconClass(action: string): string {
  return actionColors[action.toLowerCase()] || ''
}

function iconLabel(action: string): string {
  const key = action.toLowerCase()
  const prefixes = Object.keys(actionIcons).filter(k => key.includes(k))
  if (prefixes.length > 0) return actionIcons[prefixes[0]] || '[?]'
  return '[?]'
}

function actionClass(action: string): string {
  return actionColors[action.toLowerCase()] || ''
}

function formatTime(ts: string): string {
  if (!ts) return ''
  return new Date(ts).toLocaleString()
}

function formatDetails(details: Record<string, unknown>): string {
  const entries = Object.entries(details).filter(([k]) => k !== 'fileName')
  if (entries.length === 0) return ''
  return entries.map(([k, v]) => `${k}: ${String(v).substring(0, 30)}`).join(', ')
}
</script>

<style scoped>
.activity-panel {
  padding: 12px;
}

.panel-card {
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 8px;
  padding: 16px;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.panel-title {
  font-family: 'Courier New', monospace;
  font-size: 11px;
  font-weight: 700;
  color: #e0e0e0;
  letter-spacing: 1px;
  margin-bottom: 8px;
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

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 24px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  height: 80px;
}

.empty-spinner {
  opacity: 0.5;
}

.text-muted {
  color: #555 !important;
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.activity-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid transparent;
  border-radius: 4px;
  transition: border-color 0.1s;
}

.activity-item:hover {
  border-color: #2a2a2a;
  background: #111;
}

.activity-icon {
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #333;
  border-radius: 3px;
}

.activity-icon.encrypt { color: #ff6b6b; border-color: #ff6b6b; }
.activity-icon.decrypt { color: #51cf66; border-color: #51cf66; }
.activity-icon.delete { color: #ff5f57; border-color: #ff5f57; }
.activity-icon.sync { color: #5c7cfa; border-color: #5c7cfa; }
.activity-icon.batch { color: #fcc419; border-color: #fcc419; }

.activity-body {
  flex: 1;
  min-width: 0;
}

.activity-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
}

.activity-action {
  font-family: 'Courier New', monospace;
  font-size: 9px;
  font-weight: 700;
}

.activity-action.encrypt { color: #ff6b6b; }
.activity-action.decrypt { color: #51cf66; }
.activity-action.delete { color: #ff5f57; }
.activity-action.sync { color: #5c7cfa; }
.activity-action.batch { color: #fcc419; }

.activity-entity {
  font-size: 8px;
  font-family: 'Courier New', monospace;
  text-transform: uppercase;
}

.activity-details {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 9px;
}

.activity-filename {
  font-family: 'Courier New', monospace;
  color: #ccc;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.activity-timestamp {
  font-size: 8px;
  flex-shrink: 0;
}

.activity-extra {
  font-size: 8px;
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
