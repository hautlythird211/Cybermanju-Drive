<template>
  <div class="compression-panel">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-compress">[$]</span>
        <h2 class="panel-title">COMPRESSION ENGINE</h2>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[CMP] COMPRESSION ALGORITHMS</h3>
      <div class="algo-list">
        <button
          v-for="(info, type) in COMPRESSION_INFO"
          :key="type"
          class="algo-btn"
          :class="{ selected: selectedAlgo === type }"
          @click="selectedAlgo = type as CompressionType"
        >
          <div class="algo-header">
            <span class="algo-name">{{ info.name }}</span>
            <span class="algo-speed">{{ info.speed }}</span>
          </div>
          <span class="algo-desc text-muted">{{ info.description }}</span>
        </button>
      </div>
    </div>

    <div class="section" v-if="selectedFile">
      <h3 class="section-title">[FILE] SELECTED FILE</h3>
      <p class="selected-file-name">{{ selectedFile.name }}</p>
      <button class="compress-btn" @click="handleCompress">[COMPRESS]</button>
    </div>

    <div class="section" v-if="compressionStats">
      <h3 class="section-title"><Icon icon="svg-spinners:bars-scale-fade" width="12" height="12" class="section-spinner" /> [STATS] RESULTS</h3>
      <div class="stats-card">
        <div class="stat-row">
          <span class="stat-key text-muted">ORIGINAL</span>
          <span class="stat-value">{{ formatSize(compressionStats.originalSize) }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-key text-muted">COMPRESSED</span>
          <span class="stat-value">{{ formatSize(compressionStats.compressedSize) }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-key text-muted">RATIO</span>
          <span class="stat-value">{{ (compressionStats.ratio * 100).toFixed(1) }}%</span>
        </div>
        <div class="stat-row">
          <span class="stat-key text-muted">DURATION</span>
          <span class="stat-value">{{ compressionStats.durationMs }}ms</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import type { CompressionType } from '@/types'
import { COMPRESSION_INFO } from '@/types'

const store = useAppStore()
const emit = defineEmits<{ close: [] }>()

const selectedFile = computed(() => store.selectedFile)
const compressionStats = computed(() => store.compressionStats)
const selectedAlgo = ref<CompressionType>('zstd')

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const k = 1024
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + units[i]
}

async function handleCompress() {
  if (!store.selectedFileId) return
  await store.compressFile(store.selectedFileId, selectedAlgo.value)
}
</script>

<style scoped>
.compression-panel {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}
.compression-panel::-webkit-scrollbar { width: 4px; }
.compression-panel::-webkit-scrollbar-track { background: transparent; }
.compression-panel::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 2px; }

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-compress {
  font-size: 14px;
  color: var(--text-accent);
}

.panel-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--text-primary);
  margin: 0;
  font-family: var(--font-mono);
}

.section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-glass);
  font-family: var(--font-mono);
}

.algo-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.algo-btn {
  background: var(--bg-glass-light);
  border: 1px solid var(--border-glass);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  padding: 10px 12px;
  cursor: pointer;
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.algo-btn:hover,
.algo-btn.selected {
  background: var(--accent-dim);
  border-color: var(--border-accent);
}

.algo-btn:hover .algo-desc,
.algo-btn.selected .algo-desc {
  color: var(--text-accent) !important;
}

.algo-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.algo-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.algo-speed {
  font-size: 9px;
  border: 1px solid var(--border-medium);
  padding: 1px 6px;
  opacity: 0.7;
  color: var(--text-muted);
  border-radius: var(--radius-sm);
}

.algo-desc {
  font-size: 10px;
  line-height: 1.4;
  color: var(--text-muted);
}

.selected-file-name {
  font-size: 11px;
  color: var(--text-primary);
  background: var(--bg-elevated);
  padding: 6px 10px;
  border: 1px solid var(--border-subtle);
  word-break: break-all;
  margin: 0;
  border-radius: var(--radius-sm);
}

.compress-btn {
  background: var(--accent);
  color: var(--text-inverse);
  border: 1px solid var(--accent);
  padding: 8px 16px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--font-mono);
  width: 100%;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.compress-btn:hover {
  background: #00cc35;
  border-color: #00cc35;
}

.stats-card {
  border: 1px solid var(--border-glass);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-md);
}

.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-key {
  font-size: 10px;
  color: var(--text-muted);
}

.stat-value {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
  font-family: var(--font-mono);
}

.text-muted { color: var(--text-muted) !important; }
</style>
