<template>
  <div class="face-panel">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-face">[+]</span>
        <h2 class="panel-title">FACE GROUPING</h2>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[SCAN] DETECT FACES</h3>
      <button class="bw-btn" style="width:100%;" @click="handleBatchDetect">[BATCH DETECT]</button>
    </div>

    <div class="section" v-if="lastResult">
      <h3 class="section-title">[STATS] LAST SCAN</h3>
      <div class="stats-card">
        <div class="stat-row"><span class="stat-key text-muted">CLUSTERS</span><span class="stat-value">{{ lastResult.clustersCreated }}</span></div>
        <div class="stat-row"><span class="stat-key text-muted">FACES</span><span class="stat-value">{{ lastResult.totalFaces }}</span></div>
        <div class="stat-row"><span class="stat-key text-muted">COHESION</span><span class="stat-value">{{ lastResult.avgCohesion.toFixed(3) }}</span></div>
        <div class="stat-row"><span class="stat-key text-muted">STRATEGY</span><span class="stat-value">{{ lastResult.strategyUsed }}</span></div>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[GROUP] PEOPLE ({{ faceGroups.length }})</h3>
      <div class="group-list">
        <div v-for="group in faceGroups" :key="group.id" class="group-card">
          <div class="group-header">
            <div class="avatar">++</div>
            <div class="group-info">
              <span class="group-name">{{ group.name }}</span>
              <span class="group-meta text-muted">{{ group.fileIds.length }} FILES</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const faceGroups = computed(() => store.faceGroups)
const lastResult = ref<{ clustersCreated: number; totalFaces: number; noiseFaces: number; avgCohesion: number; strategyUsed: string } | null>(null)

async function handleBatchDetect() {
  const result = await store.detectFacesBatch()
  if (result) lastResult.value = result
}
</script>

<style scoped>
.face-panel {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
  margin-bottom: 16px;
}

.header-left { display: flex; align-items: center; gap: 8px; }
.icon-face { font-size: 14px; color: var(--text-accent); }
.panel-title { font-size: 13px; font-weight: 700; letter-spacing: 1px; margin: 0; color: var(--text-primary); }

.section { margin-bottom: 16px; }

.section-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0 0 8px;
  font-family: var(--font-mono);
}

.bw-btn {
  padding: 7px 14px;
  background: var(--accent);
  color: var(--text-inverse);
  border: 1px solid var(--accent);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.bw-btn:hover { background: #00cc35; border-color: #00cc35; }

.stats-card {
  border: 1px solid var(--border-glass);
  padding: 12px;
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

.stat-key { font-size: 10px; color: var(--text-muted); }
.stat-value { font-size: 11px; font-weight: 600; color: var(--text-primary); font-family: var(--font-mono); }

.group-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-card {
  border: 1px solid var(--border-glass);
  padding: 12px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.group-card:hover {
  border-color: var(--border-accent);
  background: var(--accent-dim);
}

.group-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.avatar {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-accent);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 10px;
  color: var(--text-accent);
  background: var(--accent-dim);
  border-radius: var(--radius-sm);
}

.group-info { flex: 1; min-width: 0; }
.group-name { font-size: 12px; font-weight: 600; color: var(--text-primary); }
.group-meta { font-size: 10px; color: var(--text-muted); }

.text-muted { color: var(--text-muted) !important; }
</style>
