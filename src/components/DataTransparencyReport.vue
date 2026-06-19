<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Icon } from '@iconify/vue'

const emit = defineEmits<{ (e: 'close'): void }>()

interface SysInfo {
  os_name: string
  os_version: string
  os_arch: string
  hostname: string
  cpu_brand: string
  cpu_cores: number
  total_memory_mb: number
  kernel_version: string
}

const sys = ref<SysInfo | null>(null)
const fingerprint = ref<Record<string, string>>({})
const currentSection = ref(0)

const sections = [
  {
    title: 'DEVICE FINGERPRINT',
    icon: 'mdi:fingerprint',
    description: 'Big Tech sees ALL of this the moment you load their page',
    items: [] as Array<{ label: string; value: string; collected: boolean }>,
  },
  {
    title: 'BROWSER EXPOSURE',
    icon: 'mdi:web',
    description: 'Your browser leaks more than you think',
    items: [] as Array<{ label: string; value: string; collected: boolean }>,
  },
  {
    title: 'NETWORK IDENTITY',
    icon: 'mdi:web-lock',
    description: 'Your network connection reveals everything',
    items: [] as Array<{ label: string; value: string; collected: boolean }>,
  },
  {
    title: 'BEHAVIORAL TRACKING',
    icon: 'mdi:eye-outline',
    description: 'They track HOW you use your device, not just WHAT',
    items: [] as Array<{ label: string; value: string; collected: boolean }>,
  },
  {
    title: 'WHAT CYBERMANJU COLLECTS',
    icon: 'mdi:shield-check',
    description: 'Nothing. Zero. We are open source.',
    items: [] as Array<{ label: string; value: string; collected: boolean }>,
  },
]

function gatherFingerprint() {
  const fp: Record<string, string> = {}
  fp['Screen Resolution'] = `${screen.width}x${screen.height}`
  fp['Color Depth'] = `${screen.colorDepth}-bit`
  fp['Timezone'] = Intl.DateTimeFormat().resolvedOptions().timeZone
  fp['Language'] = navigator.language
  fp['Platform'] = navigator.platform
  fp['Hardware Concurrency'] = `${navigator.hardwareConcurrency || 'unknown'} cores`
  fp['Device Memory'] = `${(navigator as any).deviceMemory || 'unknown'} GB`
  fp['Max Touch Points'] = `${navigator.maxTouchPoints}`
  fp['Cookie Enabled'] = navigator.cookieEnabled ? 'YES' : 'NO'
  fp['Do Not Track'] = navigator.doNotTrack || 'not set'
  fp['PDF Viewer'] = navigator.plugins?.length ? 'YES' : 'unknown'
  fp['WebSocket'] = typeof WebSocket !== 'undefined' ? 'AVAILABLE' : 'BLOCKED'
  fp['WebGL Renderer'] = (() => {
    try {
      const canvas = document.createElement('canvas')
      const gl = canvas.getContext('webgl')
      if (!gl) return 'BLOCKED'
      const ext = gl.getExtension('WEBGL_debug_renderer_info')
      return ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : 'AVAILABLE'
    } catch { return 'BLOCKED' }
  })()
  fingerprint.value = fp
}

function buildSections(s: SysInfo | null) {
  const deviceItems = [
    { label: 'Operating System', value: s ? `${s.os_name} ${s.os_version}` : 'Detectable', collected: true },
    { label: 'CPU Architecture', value: s?.os_arch || 'Detectable', collected: true },
    { label: 'CPU Model', value: s?.cpu_brand || 'Detectable', collected: true },
    { label: 'CPU Cores', value: s ? `${s.cpu_cores}` : 'Detectable', collected: true },
    { label: 'Total RAM', value: s ? `${s.total_memory_mb} MB` : 'Detectable', collected: true },
    { label: 'GPU / WebGL Renderer', value: fingerprint.value['WebGL Renderer'] || 'Detectable', collected: true },
    { label: 'Screen Resolution', value: fingerprint.value['Screen Resolution'] || 'Detectable', collected: true },
    { label: 'Color Depth', value: fingerprint.value['Color Depth'] || 'Detectable', collected: true },
    { label: 'Kernel Version', value: s?.kernel_version || 'Detectable', collected: true },
    { label: 'Hostname', value: s?.hostname || 'Detectable', collected: true },
  ]

  const browserItems = [
    { label: 'User Agent', value: navigator.userAgent.slice(0, 60) + '...', collected: true },
    { label: 'Language Preference', value: navigator.language, collected: true },
    { label: 'Timezone', value: fingerprint.value['Timezone'] || 'Detectable', collected: true },
    { label: 'Cookie Policy', value: navigator.cookieEnabled ? 'ENABLED (tracking ON)' : 'DISABLED', collected: true },
    { label: 'Do Not Track', value: fingerprint.value['Do Not Track'] || 'IGNORED', collected: true },
    { label: 'Installed Plugins', value: `${navigator.plugins?.length || 0} detected`, collected: true },
    { label: 'Font Fingerprint', value: 'Unique combo (200+ fonts)', collected: true },
    { label: 'Canvas Fingerprint', value: 'Unique hash per device', collected: true },
    { label: 'Audio Context', value: 'Unique hardware signature', collected: true },
  ]

  const networkItems = [
    { label: 'IP Address', value: 'Visible to every server', collected: true },
    { label: 'ISP / Carrier', value: 'Traced from IP range', collected: true },
    { label: 'Approximate Location', value: 'City-level from IP', collected: true },
    { label: 'DNS Resolver', value: 'Logged by resolver', collected: true },
    { label: 'Connection Type', value: 'WiFi / Cellular / Ethernet', collected: true },
    { label: 'WebRTC Leak', value: 'Can expose real IP', collected: true },
  ]

  const behavioralItems = [
    { label: 'Mouse Movement Patterns', value: 'Unique behavioral biometric', collected: true },
    { label: 'Keystroke Dynamics', value: 'Typing speed & rhythm', collected: true },
    { label: 'Scroll Behavior', value: 'Speed, direction, patterns', collected: true },
    { label: 'Click Heatmap', value: 'Where you click & how often', collected: true },
    { label: 'Time on Page', value: 'Exact duration per element', collected: true },
    { label: 'Focus/Blur Events', value: 'When you switch tabs', collected: true },
    { label: 'Clipboard Access', value: 'What you copy/paste', collected: true },
    { label: 'Battery Status', value: 'Charging state & level', collected: true },
    { label: 'Camera/Mic Status', value: 'Permission state tracked', collected: true },
    { label: 'Geolocation', value: 'Precise GPS if permitted', collected: true },
  ]

  const cyItems = [
    { label: 'Personal Data', value: 'NEVER collected', collected: false },
    { label: 'Device Fingerprint', value: 'NEVER collected', collected: false },
    { label: 'Behavioral Tracking', value: 'NEVER collected', collected: false },
    { label: 'Analytics / Telemetry', value: 'NONE — zero analytics', collected: false },
    { label: 'Third-Party Trackers', value: 'NONE — zero trackers', collected: false },
    { label: 'Cookies', value: 'NONE — no tracking cookies', collected: false },
    { label: 'Ad Profiles', value: 'NONE — no ads, ever', collected: false },
    { label: 'Data Selling', value: 'NEVER — we don\'t sell data', collected: false },
    { label: 'Source Code', value: '100% open source on GitHub', collected: false },
    { label: 'Encryption', value: 'Post-quantum (Kyber + Dilithium)', collected: false },
  ]

  sections[0].items = deviceItems
  sections[1].items = browserItems
  sections[2].items = networkItems
  sections[3].items = behavioralItems
  sections[4].items = cyItems
}

function nextSection() {
  if (currentSection.value < sections.length - 1) {
    currentSection.value++
  } else {
    emit('close')
  }
}

function prevSection() {
  if (currentSection.value > 0) {
    currentSection.value--
  }
}

onMounted(async () => {
  gatherFingerprint()
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    sys.value = await invoke<SysInfo>('get_system_info')
  } catch {}
  buildSections(sys.value)
})
</script>

<template>
  <div class="transparency-overlay" @click.self="emit('close')">
    <div class="transparency-panel">
      <div class="tp-header">
        <div class="tp-title">
          <Icon icon="mdi:eye-outline" width="18" height="18" />
          <span>DATA EXPOSURE REPORT</span>
        </div>
        <button class="tp-close" @click="emit('close')" aria-label="Close">[X]</button>
      </div>

      <div class="tp-banner">
        <Icon icon="mdi:alert-circle-outline" width="16" height="16" />
        <span>Every website you visit collects this data. Cybermanju collects NONE of it.</span>
      </div>

      <div class="tp-section-nav">
        <button
          v-for="(sec, i) in sections"
          :key="i"
          class="tp-nav-btn"
          :class="{ active: currentSection === i }"
          @click="currentSection = i"
        >
          <Icon :icon="sec.icon" width="14" height="14" />
          <span class="tp-nav-label">{{ sec.title }}</span>
        </button>
      </div>

      <div class="tp-content">
        <div class="tp-section-header">
          <Icon :icon="sections[currentSection].icon" width="20" height="20" />
          <div>
            <div class="tp-section-title">{{ sections[currentSection].title }}</div>
            <div class="tp-section-desc">{{ sections[currentSection].description }}</div>
          </div>
        </div>

        <div class="tp-items">
          <div
            v-for="(item, i) in sections[currentSection].items"
            :key="i"
            class="tp-item"
            :class="{ exposed: item.collected, safe: !item.collected }"
          >
            <div class="tp-item-icon">
              <Icon
                :icon="item.collected ? 'mdi:alert-circle' : 'mdi:check-circle'"
                width="14"
                height="14"
              />
            </div>
            <div class="tp-item-info">
              <div class="tp-item-label">{{ item.label }}</div>
              <div class="tp-item-value">{{ item.value }}</div>
            </div>
            <div class="tp-item-badge" :class="item.collected ? 'badge-danger' : 'badge-safe'">
              {{ item.collected ? 'COLLECTED' : 'NOT COLLECTED' }}
            </div>
          </div>
        </div>
      </div>

      <div class="tp-footer">
        <button class="tp-btn" :disabled="currentSection === 0" @click="prevSection">
          [ PREV ]
        </button>
        <div class="tp-progress">
          <span v-for="(_, i) in sections" :key="i" class="tp-dot" :class="{ active: currentSection === i }" />
        </div>
        <button class="tp-btn tp-btn-primary" @click="nextSection">
          {{ currentSection === sections.length - 1 ? '[ GOT IT ]' : '[ NEXT ]' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.transparency-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(8px);
  font-family: 'Courier New', monospace;
}

.transparency-panel {
  width: 580px;
  max-width: 94vw;
  max-height: 85vh;
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 0 80px rgba(255, 50, 50, 0.08);
}

.tp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #1a1a1a;
}

.tp-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 800;
  color: #ff5f57;
  letter-spacing: 2px;
}

.tp-close {
  background: none;
  border: 1px solid #333;
  color: #888;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  padding: 2px 8px;
  cursor: pointer;
  border-radius: 4px;
}

.tp-close:hover {
  border-color: #ff5f57;
  color: #ff5f57;
}

.tp-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: rgba(255, 95, 87, 0.08);
  border-bottom: 1px solid #1a1a1a;
  font-size: 10px;
  color: #ff5f57;
  letter-spacing: 0.5px;
}

.tp-section-nav {
  display: flex;
  gap: 2px;
  padding: 8px 12px;
  border-bottom: 1px solid #1a1a1a;
  overflow-x: auto;
}

.tp-nav-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: #666;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.5px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s;
}

.tp-nav-btn:hover {
  color: #aaa;
  background: #111;
}

.tp-nav-btn.active {
  color: #00ff41;
  border-color: #00ff41;
  background: rgba(0, 255, 65, 0.05);
}

.tp-nav-label {
  display: none;
}

@media (min-width: 600px) {
  .tp-nav-label { display: inline; }
}

.tp-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.tp-content::-webkit-scrollbar { width: 4px; }
.tp-content::-webkit-scrollbar-track { background: transparent; }
.tp-content::-webkit-scrollbar-thumb { background: #1a1a1a; border-radius: 2px; }

.tp-section-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  color: #00ff41;
}

.tp-section-title {
  font-size: 13px;
  font-weight: 800;
  color: #e0e0e0;
  letter-spacing: 1px;
}

.tp-section-desc {
  font-size: 10px;
  color: #666;
  margin-top: 2px;
}

.tp-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tp-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  background: #0d0d0d;
}

.tp-item.exposed {
  border-color: rgba(255, 95, 87, 0.2);
}

.tp-item.safe {
  border-color: rgba(0, 255, 65, 0.2);
  background: rgba(0, 255, 65, 0.02);
}

.tp-item-icon {
  flex-shrink: 0;
}

.tp-item.exposed .tp-item-icon { color: #ff5f57; }
.tp-item.safe .tp-item-icon { color: #00ff41; }

.tp-item-info {
  flex: 1;
  min-width: 0;
}

.tp-item-label {
  font-size: 10px;
  font-weight: 700;
  color: #ccc;
  letter-spacing: 0.5px;
}

.tp-item-value {
  font-size: 9px;
  color: #666;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tp-item-badge {
  flex-shrink: 0;
  font-size: 8px;
  font-weight: 700;
  letter-spacing: 0.5px;
  padding: 2px 6px;
  border-radius: 3px;
}

.badge-danger {
  color: #ff5f57;
  background: rgba(255, 95, 87, 0.1);
  border: 1px solid rgba(255, 95, 87, 0.2);
}

.badge-safe {
  color: #00ff41;
  background: rgba(0, 255, 65, 0.1);
  border: 1px solid rgba(0, 255, 65, 0.2);
}

.tp-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid #1a1a1a;
}

.tp-btn {
  background: transparent;
  border: 1px solid #333;
  border-radius: 4px;
  color: #888;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 6px 14px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.tp-btn:hover:not(:disabled) {
  border-color: #555;
  color: #e0e0e0;
}

.tp-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.tp-btn-primary {
  border-color: #00ff41;
  color: #00ff41;
}

.tp-btn-primary:hover {
  background: rgba(0, 255, 65, 0.1);
}

.tp-progress {
  display: flex;
  gap: 6px;
}

.tp-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #333;
  transition: all 0.2s;
}

.tp-dot.active {
  background: #00ff41;
  box-shadow: 0 0 8px rgba(0, 255, 65, 0.4);
}
</style>
