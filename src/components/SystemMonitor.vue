<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import OsPanel from '@/components/ui/OsPanel.vue'
import OsSection from '@/components/ui/OsSection.vue'

const cpuUsage = ref(0)
const cpuTemp = ref(42)
const ramUsed = ref(0)
const ramTotal = 32768
const ramPercent = ref(0)
const diskRead = ref(0)
const diskWrite = ref(0)
const netRx = ref(0)
const netTx = ref(0)
const uptime = ref(0)
const processes = ref(142)
const gpuUsage = ref(0)
const gpuTemp = ref(38)

let timer: ReturnType<typeof setInterval> | null = null

function randAround(base: number, variance: number): number {
  return base + (Math.random() - 0.5) * variance * 2
}

function formatBytes(val: number): string {
  if (val < 1024) return `${val.toFixed(0)} B/s`
  if (val < 1048576) return `${(val / 1024).toFixed(1)} KB/s`
  return `${(val / 1048576).toFixed(1)} MB/s`
}

function formatUptime(s: number): string {
  const d = Math.floor(s / 86400)
  const h = Math.floor((s % 86400) / 3600)
  const m = Math.floor((s % 3600) / 60)
  return `${d}d ${h}h ${m}m`
}

function tick() {
  uptime.value++
  cpuUsage.value = Math.min(100, Math.max(0, randAround(23, 18)))
  cpuTemp.value = Math.min(90, Math.max(30, randAround(45, 8)))
  ramUsed.value = Math.min(ramTotal, Math.max(0, randAround(8192, 4096)))
  ramPercent.value = (ramUsed.value / ramTotal) * 100
  diskRead.value = Math.max(0, randAround(50 * 1048576, 40 * 1048576))
  diskWrite.value = Math.max(0, randAround(20 * 1048576, 15 * 1048576))
  netRx.value = Math.max(0, randAround(5 * 1048576, 4 * 1048576))
  netTx.value = Math.max(0, randAround(2 * 1048576, 1.5 * 1048576))
  gpuUsage.value = Math.min(100, Math.max(0, randAround(15, 12)))
  gpuTemp.value = Math.min(85, Math.max(30, randAround(42, 6)))
}

onMounted(() => {
  timer = setInterval(tick, 1000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div class="system-monitor">
    <OsPanel variant="glass" padding="md">
      <OsSection title="SYSTEM MONITOR" icon="mdi:chart-line-variant" variant="neon" spaced>
        <div class="monitor-grid">
          <!-- CPU -->
          <div class="monitor-card">
            <div class="monitor-header">
              <span class="monitor-icon">⚡</span>
              <span class="monitor-label">CPU</span>
              <span class="monitor-value">{{ cpuUsage.toFixed(1) }}%</span>
            </div>
            <div class="monitor-track">
              <div class="monitor-fill cpu-fill" :style="{ width: cpuUsage + '%' }"></div>
            </div>
            <div class="monitor-detail">
              <span>8C/16T @ 2.8GHz</span>
              <span>{{ cpuTemp.toFixed(0) }}°C</span>
            </div>
          </div>

          <!-- RAM -->
          <div class="monitor-card">
            <div class="monitor-header">
              <span class="monitor-icon">🧠</span>
              <span class="monitor-label">RAM</span>
              <span class="monitor-value">{{ (ramUsed / 1024).toFixed(0) }} / {{ (ramTotal / 1024).toFixed(0) }} GB</span>
            </div>
            <div class="monitor-track">
              <div class="monitor-fill ram-fill" :style="{ width: ramPercent + '%' }"></div>
            </div>
            <div class="monitor-detail">
              <span>DDR5-6400 ECC</span>
              <span>{{ ramPercent.toFixed(0) }}%</span>
            </div>
          </div>

          <!-- Disk -->
          <div class="monitor-card">
            <div class="monitor-header">
              <span class="monitor-icon">💾</span>
              <span class="monitor-label">DISK I/O</span>
            </div>
            <div class="monitor-io">
              <div class="io-row">
                <span class="io-label">READ</span>
                <div class="io-track"><div class="io-fill io-read" :style="{ width: Math.min(100, diskRead / (100 * 1048576) * 100) + '%' }"></div></div>
                <span class="io-val">{{ formatBytes(diskRead) }}</span>
              </div>
              <div class="io-row">
                <span class="io-label">WRITE</span>
                <div class="io-track"><div class="io-fill io-write" :style="{ width: Math.min(100, diskWrite / (100 * 1048576) * 100) + '%' }"></div></div>
                <span class="io-val">{{ formatBytes(diskWrite) }}</span>
              </div>
            </div>
          </div>

          <!-- Network -->
          <div class="monitor-card">
            <div class="monitor-header">
              <span class="monitor-icon">🌐</span>
              <span class="monitor-label">NETWORK</span>
            </div>
            <div class="monitor-io">
              <div class="io-row">
                <span class="io-label">RX</span>
                <div class="io-track"><div class="io-fill io-rx" :style="{ width: Math.min(100, netRx / (50 * 1048576) * 100) + '%' }"></div></div>
                <span class="io-val">{{ formatBytes(netRx) }}</span>
              </div>
              <div class="io-row">
                <span class="io-label">TX</span>
                <div class="io-track"><div class="io-fill io-tx" :style="{ width: Math.min(100, netTx / (50 * 1048576) * 100) + '%' }"></div></div>
                <span class="io-val">{{ formatBytes(netTx) }}</span>
              </div>
            </div>
          </div>

          <!-- GPU -->
          <div class="monitor-card">
            <div class="monitor-header">
              <span class="monitor-icon">🎮</span>
              <span class="monitor-label">GPU</span>
              <span class="monitor-value">{{ gpuUsage.toFixed(0) }}%</span>
            </div>
            <div class="monitor-track">
              <div class="monitor-fill gpu-fill" :style="{ width: gpuUsage + '%' }"></div>
            </div>
            <div class="monitor-detail">
              <span>NVIDIA RTX 5090</span>
              <span>{{ gpuTemp.toFixed(0) }}°C</span>
            </div>
          </div>

          <!-- Uptime / Processes -->
          <div class="monitor-card">
            <div class="monitor-header">
              <span class="monitor-icon">⏱</span>
              <span class="monitor-label">SYSTEM</span>
            </div>
            <div class="monitor-stat-grid">
              <div class="monitor-stat">
                <span class="stat-num">{{ formatUptime(uptime) }}</span>
                <span class="stat-lbl">UPTIME</span>
              </div>
              <div class="monitor-stat">
                <span class="stat-num">{{ processes }}</span>
                <span class="stat-lbl">PROCESSES</span>
              </div>
            </div>
          </div>
        </div>
      </OsSection>
    </OsPanel>
  </div>
</template>

<style scoped>
.system-monitor {
  width: 100%;
}

.monitor-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.monitor-card {
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  contain: layout style;
}

.monitor-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.monitor-icon {
  font-size: 14px;
  opacity: 0.7;
}

.monitor-label {
  flex: 1;
  font-size: 10px;
  font-weight: 700;
  color: #888;
  letter-spacing: 2px;
}

.monitor-value {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-accent);
  font-family: var(--font-mono);
}

.monitor-track {
  height: 4px;
  background: var(--bg-surface);
  border-radius: 2px;
  overflow: hidden;
}

.monitor-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.5s cubic-bezier(0.22, 1, 0.36, 1);
}

.cpu-fill { background: linear-gradient(90deg, var(--accent), var(--accent-dim)); box-shadow: 0 0 6px var(--accent-glow); }
.ram-fill { background: linear-gradient(90deg, #5af0ff, #00d4ff); box-shadow: 0 0 6px rgba(90, 240, 255, 0.3); }
.gpu-fill { background: linear-gradient(90deg, #ff6b9d, #ff3b6f); box-shadow: 0 0 6px rgba(255, 107, 157, 0.3); }

.monitor-detail {
  display: flex;
  justify-content: space-between;
  font-size: 8px;
  color: var(--text-muted);
  letter-spacing: 1px;
}

.monitor-io {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.io-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.io-label {
  font-size: 9px;
  font-weight: 700;
  color: var(--text-muted);
  width: 40px;
  letter-spacing: 1px;
}

.io-track {
  flex: 1;
  height: 3px;
  background: var(--bg-surface);
  border-radius: 2px;
  overflow: hidden;
}

.io-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.5s cubic-bezier(0.22, 1, 0.36, 1);
}

.io-read { background: #5af0ff; box-shadow: 0 0 4px rgba(90, 240, 255, 0.3); }
.io-write { background: #ff6b9d; box-shadow: 0 0 4px rgba(255, 107, 157, 0.3); }
.io-rx { background: var(--accent); box-shadow: 0 0 4px var(--accent-glow); }
.io-tx { background: #b388ff; box-shadow: 0 0 4px rgba(179, 136, 255, 0.3); }

.io-val {
  font-size: 9px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  width: 60px;
  text-align: right;
}

.monitor-stat-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.monitor-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.stat-num {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-accent);
  font-family: var(--font-mono);
  text-shadow: 0 0 6px var(--accent-glow);
}

.stat-lbl {
  font-size: 7px;
  color: #444;
  letter-spacing: 2px;
  margin-top: 2px;
}

@media (max-width: 640px) {
  .monitor-grid {
    grid-template-columns: 1fr;
  }
}
</style>
