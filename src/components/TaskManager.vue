<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import OsPanel from '@/components/ui/OsPanel.vue'
import OsSection from '@/components/ui/OsSection.vue'

interface Process {
  pid: number
  name: string
  cpu: number
  memory: number
  status: 'running' | 'sleeping' | 'stopped' | 'zombie'
  user: string
  started: string
}

const processes = ref<Process[]>([])
const sortKey = ref<'cpu' | 'memory' | 'name' | 'pid'>('cpu')
const sortDir = ref<'asc' | 'desc'>('desc')
const searchQuery = ref('')

const apps = [
  { name: 'cybermanju-shell', cpuBase: 2.1, memBase: 128, user: 'admin' },
  { name: 'file-manager', cpuBase: 0.8, memBase: 64, user: 'admin' },
  { name: 'syncd', cpuBase: 1.5, memBase: 256, user: 'root' },
  { name: 'cryptd', cpuBase: 0.5, memBase: 48, user: 'root' },
  { name: 'watchd', cpuBase: 0.3, memBase: 32, user: 'root' },
  { name: 'indexd', cpuBase: 2.8, memBase: 384, user: 'root' },
  { name: 'faced', cpuBase: 1.2, memBase: 512, user: 'root' },
  { name: 'geod', cpuBase: 0.2, memBase: 24, user: 'root' },
  { name: 'nginx', cpuBase: 0.6, memBase: 16, user: 'www-data' },
  { name: 'postgresql', cpuBase: 3.5, memBase: 768, user: 'postgres' },
  { name: 'redis', cpuBase: 0.4, memBase: 8, user: 'redis' },
  { name: 'web-dashboard', cpuBase: 0.9, memBase: 96, user: 'admin' },
  { name: 'sshd', cpuBase: 0.1, memBase: 4, user: 'root' },
  { name: 'cronie', cpuBase: 0.05, memBase: 2, user: 'root' },
  { name: 'syslog-ng', cpuBase: 0.1, memBase: 8, user: 'root' },
  { name: 'network-manager', cpuBase: 0.3, memBase: 12, user: 'root' },
  { name: 'pulseaudio', cpuBase: 0.2, memBase: 16, user: 'admin' },
  { name: 'Xorg', cpuBase: 1.8, memBase: 64, user: 'root' },
  { name: 'dbus-daemon', cpuBase: 0.1, memBase: 4, user: 'root' },
  { name: 'polkitd', cpuBase: 0.05, memBase: 6, user: 'root' },
  { name: 'udevd', cpuBase: 0.08, memBase: 3, user: 'root' },
  { name: 'systemd-journald', cpuBase: 0.15, memBase: 20, user: 'root' },
  { name: 'systemd-logind', cpuBase: 0.05, memBase: 2, user: 'root' },
  { name: 'upowerd', cpuBase: 0.02, memBase: 1, user: 'root' },
]

let pidCounter = 100
let timer: ReturnType<typeof setInterval> | null = null

function randAround(base: number, variance: number): number {
  return Math.max(0, base + (Math.random() - 0.5) * variance * 2)
}

function spawnProcesses() {
  processes.value = apps.map((app, i) => ({
    pid: 100 + i,
    name: app.name,
    cpu: randAround(app.cpuBase, app.cpuBase * 0.6),
    memory: Math.round(randAround(app.memBase, app.memBase * 0.3)),
    status: 'running' as const,
    user: app.user,
    started: new Date(Date.now() - Math.random() * 86400000).toLocaleTimeString(),
  }))
  pidCounter = 100 + apps.length
}

function updateStats() {
  processes.value = processes.value.map(p => {
    const app = apps.find(a => a.name === p.name)
    if (!app) return p
    return {
      ...p,
      cpu: randAround(app.cpuBase, app.cpuBase * 0.6),
      memory: Math.round(randAround(app.memBase, app.memBase * 0.3)),
      status: Math.random() < 0.98 ? 'running' : (Math.random() < 0.5 ? 'sleeping' : 'stopped'),
    }
  })
}

const sorted = computed(() => {
  let list = [...processes.value]
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter(p => p.name.toLowerCase().includes(q) || p.user.includes(q))
  }
  list.sort((a, b) => {
    let cmp: number
    if (sortKey.value === 'cpu') cmp = a.cpu - b.cpu
    else if (sortKey.value === 'memory') cmp = a.memory - b.memory
    else if (sortKey.value === 'name') cmp = a.name.localeCompare(b.name)
    else cmp = a.pid - b.pid
    return sortDir.value === 'desc' ? -cmp : cmp
  })
  return list
})

function toggleSort(key: 'cpu' | 'memory' | 'name' | 'pid') {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'desc' ? 'asc' : 'desc'
  } else {
    sortKey.value = key
    sortDir.value = 'desc'
  }
}

function killProcess(pid: number) {
  processes.value = processes.value.filter(p => p.pid !== pid)
}

onMounted(() => {
  spawnProcesses()
  timer = setInterval(updateStats, 2000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div class="task-manager">
    <OsPanel variant="glass" padding="md">
      <OsSection title="TASK MANAGER" icon="mdi:memory" variant="neon" spaced>
        <div class="tm-toolbar">
          <div class="tm-search">
            <span class="tm-search-icon">🔍</span>
            <input v-model="searchQuery" class="tm-search-input" placeholder="Filter processes..." />
          </div>
          <div class="tm-count">{{ sorted.length }} processes</div>
        </div>
        <div class="tm-table-wrap">
          <table class="tm-table">
            <thead>
              <tr>
                <th class="th-sort" @click="toggleSort('pid')">PID <span v-if="sortKey === 'pid'">{{ sortDir === 'desc' ? '↓' : '↑' }}</span></th>
                <th class="th-sort" @click="toggleSort('name')">NAME <span v-if="sortKey === 'name'">{{ sortDir === 'desc' ? '↓' : '↑' }}</span></th>
                <th class="th-sort" @click="toggleSort('cpu')">CPU% <span v-if="sortKey === 'cpu'">{{ sortDir === 'desc' ? '↓' : '↑' }}</span></th>
                <th class="th-sort" @click="toggleSort('memory')">MEM <span v-if="sortKey === 'memory'">{{ sortDir === 'desc' ? '↓' : '↑' }}</span></th>
                <th>STATUS</th>
                <th>USER</th>
                <th>ACTION</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="p in sorted" :key="p.pid" class="tm-row">
                <td class="tm-pid">{{ p.pid }}</td>
                <td class="tm-name">{{ p.name }}</td>
                <td class="tm-cpu">
                  <div class="tm-bar-wrap">
                    <div class="tm-bar tm-cpu-bar" :style="{ width: Math.min(p.cpu * 10, 100) + '%' }"></div>
                  </div>
                  <span>{{ p.cpu.toFixed(1) }}</span>
                </td>
                <td class="tm-mem">
                  <div class="tm-bar-wrap">
                    <div class="tm-bar tm-mem-bar" :style="{ width: Math.min(p.memory / 10, 100) + '%' }"></div>
                  </div>
                  <span>{{ p.memory }} MB</span>
                </td>
                <td class="tm-status">
                  <span class="status-dot" :class="'status--' + p.status"></span>
                  {{ p.status }}
                </td>
                <td class="tm-user">{{ p.user }}</td>
                <td class="tm-action">
                  <button class="tm-kill-btn" @click="killProcess(p.pid)" :disabled="p.name === 'systemd-journald' || p.name === 'cybermanju-shell'">KILL</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </OsSection>
    </OsPanel>
  </div>
</template>

<style scoped>
.task-manager {
  width: 100%;
}

.tm-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.tm-search {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(0,0,0,0.3);
  border: 1px solid var(--border-subtle);
  border-radius: 6px;
  padding: 6px 10px;
}

.tm-search-icon {
  font-size: 11px;
  opacity: 0.5;
}

.tm-search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
}

.tm-count {
  font-size: 10px;
  color: var(--text-muted);
  letter-spacing: 1px;
}

.tm-table-wrap {
  max-height: 400px;
  overflow-y: auto;
  contain: paint;
}

.tm-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 10px;
}

.tm-table th {
  text-align: left;
  padding: 8px 8px;
  color: var(--text-muted);
  font-weight: 700;
  letter-spacing: 1px;
  border-bottom: 1px solid var(--border-subtle);
  font-size: 9px;
  position: sticky;
  top: 0;
  background: var(--bg-surface);
}

.th-sort {
  cursor: pointer;
  user-select: none;
}

.th-sort:hover {
  color: var(--text-secondary);
}

.tm-table td {
  padding: 6px 8px;
  border-bottom: 1px solid rgba(255,255,255,0.02);
}

.tm-row:hover {
  background: rgba(255,255,255,0.02);
}

.tm-pid {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 9px;
  width: 40px;
}

.tm-name {
  color: var(--text-primary);
  font-weight: 600;
  font-size: 10px;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tm-cpu, .tm-mem {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  min-width: 80px;
}

.tm-bar-wrap {
  flex: 1;
  height: 4px;
  background: var(--bg-surface);
  border-radius: 2px;
  overflow: hidden;
  min-width: 30px;
}

.tm-bar {
  height: 100%;
  border-radius: 2px;
  transition: width 0.5s cubic-bezier(0.22, 1, 0.36, 1);
}

.tm-cpu-bar {
  background: linear-gradient(90deg, var(--accent), var(--accent-dim));
  box-shadow: 0 0 4px var(--accent-glow);
}

.tm-mem-bar {
  background: linear-gradient(90deg, #5af0ff, #00d4ff);
  box-shadow: 0 0 4px rgba(90, 240, 255, 0.3);
}

.tm-status {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status--running { background: var(--accent); box-shadow: 0 0 4px var(--accent-glow); }
.status--sleeping { background: #febc2e; box-shadow: 0 0 4px rgba(254, 188, 46, 0.5); }
.status--stopped { background: #ff5f57; box-shadow: 0 0 4px rgba(255, 95, 87, 0.5); }
.status--zombie { background: #555; }

.tm-user {
  color: var(--text-muted);
  font-size: 9px;
}

.tm-kill-btn {
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 4px;
  color: #555;
  font-family: var(--font-mono);
  font-size: 8px;
  font-weight: 600;
  padding: 3px 10px;
  cursor: pointer;
  letter-spacing: 1px;
}

.tm-kill-btn:hover:not(:disabled) {
  border-color: rgba(255, 95, 87, 0.3);
  color: #ff5f57;
}

.tm-kill-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
</style>
