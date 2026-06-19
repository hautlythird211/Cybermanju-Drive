<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ (e: 'complete'): void }>()

const lines = ref<string[]>([])
const showBootMenu = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

interface SystemInfo {
  os_name: string
  os_version: string
  os_arch: string
  hostname: string
  cpu_brand: string
  cpu_cores: number
  cpu_threads: number
  total_memory_mb: number
  used_memory_mb: number
  total_disk_gb: number
  used_disk_gb: number
  kernel_version: string
  uptime_seconds: number
}

const sys = ref<SystemInfo | null>(null)

function buildPostSequence(s: SystemInfo): Array<{ text: string; delay: number }> {
  const memChannels = Math.max(2, Math.ceil(s.cpu_cores / 4))
  const memPerChannel = Math.round(s.total_memory_mb / memChannels)
  const now = new Date()
  const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')} ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}`

  return [
    { text: `CYBERMANJU UEFI BIOS v2.4.1 (Build ${dateStr})`, delay: 600 },
    { text: `CPU: ${s.cpu_brand} — ${s.cpu_cores}C/${s.cpu_threads}T [PASS]`, delay: 400 },
    { text: `CPU Features: AES-NI, SHA-NI, AVX2, AVX-512, VAES, VPCLMULQDQ`, delay: 350 },
    { text: `MEM: Testing ${s.total_memory_mb}MB DDR5...`, delay: 300 },
    ...Array.from({ length: memChannels }, (_, i) => ({
      text: `MEM: Channel ${String.fromCharCode(65 + i)}: ${memPerChannel}MB [OK]`,
      delay: 250,
    })),
    { text: `MEM: POST-QUANTUM CRYPTO ZONES: ${Math.max(4, Math.floor(s.cpu_cores / 2))} GUARD REGIONS [ACTIVE]`, delay: 350 },
    { text: `PCH: Chipset — DMI 4.0 x8 [DETECTED]`, delay: 300 },
    { text: `PCI: Bus enumeration... devices found [COMPLETE]`, delay: 350 },
    { text: `USB: XHCI Controller at 0xFE800000 (irq 16)`, delay: 250 },
    { text: `USB: hubs, devices enumerated [OK]`, delay: 250 },
    { text: `NET: Network interface [DETECTED]`, delay: 300 },
    { text: `NET: Wireless adapter [DETECTED]`, delay: 300 },
    { text: `SND: Audio Controller [INITIALIZED]`, delay: 250 },
    { text: `TPM: 2.0 Security Module [ACTIVE]`, delay: 300 },
    { text: `RTC: System Clock — ${dateStr} UTC [SYNCED]`, delay: 250 },
    { text: `ACPI: DSDT loaded [PARSED]`, delay: 300 },
    { text: `SYS: CMOS checksum OK [NOMINAL]`, delay: 300 },
    { text: `SYS: Kernel ${s.kernel_version} (${s.os_arch}) [DETECTED]`, delay: 300 },
    { text: `SYS: ${s.os_name} ${s.os_version} — ${s.hostname}`, delay: 300 },
    { text: `SYS: Disk ${(s.total_disk_gb).toFixed(0)}GB — ${s.used_disk_gb.toFixed(0)}GB used [MOUNTED]`, delay: 300 },
    { text: `SYS: POST complete — press any key to boot Cybermanju OS`, delay: 600 },
  ]
}

function tack() {
  const el = document.querySelector('.post-terminal-text')
  if (el) el.scrollTop = el.scrollHeight
}

function runPost() {
  const seq = buildPostSequence(sys.value || {
    os_name: 'Unknown', os_version: '', os_arch: 'unknown', hostname: 'localhost',
    cpu_brand: 'Unknown CPU', cpu_cores: 1, cpu_threads: 1,
    total_memory_mb: 0, used_memory_mb: 0,
    total_disk_gb: 0, used_disk_gb: 0,
    kernel_version: 'unknown', uptime_seconds: 0,
  })
  let i = 0
  function next() {
    if (i < seq.length) {
      const entry = seq[i]
      lines.value.push(entry.text)
      tack()
      i++
      if (entry.text.includes('POST complete')) {
        timer = setTimeout(() => {
          showBootMenu.value = true
          emit('complete')
        }, 1500)
      } else {
        timer = setTimeout(next, entry.delay + Math.random() * 80)
      }
    }
  }
  next()
}

onMounted(async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    sys.value = await invoke<SystemInfo>('get_system_info')
  } catch {
    // Fallback: run with placeholder data if Tauri invoke fails
  }
  runPost()
})

onUnmounted(() => {
  if (timer) clearTimeout(timer)
})
</script>

<template>
  <div class="post-screen" tabindex="0">
    <div class="post-terminal">
      <div class="post-terminal-header">
        <span class="post-motherboard">CYBERMANJU DRIVE MB rev 2.0</span>
        <span class="post-bios-ver">UEFI BIOS v2.4.1</span>
      </div>
      <div class="post-terminal-text">
        <div v-for="(line, i) in lines" :key="i" class="post-line"
          :class="{ 'post-ok': line.includes('[OK]') || line.includes('[ACTIVE]') || line.includes('[COMPLETE]') || line.includes('[NOMINAL]') || line.includes('[INITIALIZED]') || line.includes('[PASS]') || line.includes('[DETECTED]') || line.includes('[SYNCED]') || line.includes('[PARSED]') || line.includes('[MOUNTED]') }">
          <span class="post-bracket">{{ '[' + i.toString().padStart(3, '0') + ']' }}</span>
          {{ line }}
        </div>
      </div>
      <div v-if="showBootMenu" class="post-hint">
        <span class="hint-blink">Press any key to boot Cybermanju OS...</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.post-screen {
  position: fixed;
  inset: 0;
  z-index: 99999;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Courier New', 'Fira Code', monospace;
  overflow: hidden;
  cursor: default;
}

.post-terminal {
  width: 820px;
  max-width: 96vw;
  background: #000;
  border: 1px solid #111;
  overflow: hidden;
}

.post-terminal-header {
  display: flex;
  justify-content: space-between;
  padding: 4px 12px;
  background: #080808;
  border-bottom: 1px solid #111;
  font-size: 11px;
}

.post-motherboard {
  color: #aaa;
  letter-spacing: 1px;
}

.post-bios-ver {
  color: #666;
}

.post-terminal-text {
  height: 440px;
  max-height: 60vh;
  overflow-y: auto;
  padding: 8px 12px;
  background: #000;
}

.post-terminal-text::-webkit-scrollbar { width: 4px; }
.post-terminal-text::-webkit-scrollbar-track { background: transparent; }
.post-terminal-text::-webkit-scrollbar-thumb { background: #1a1a1a; border-radius: 2px; }

.post-line {
  font-size: 11px;
  line-height: 1.65;
  color: #888;
  white-space: pre-wrap;
  word-break: break-all;
  font-weight: 500;
}

.post-ok {
  color: #bbb;
}

.post-bracket {
  color: #333;
  margin-right: 8px;
  font-size: 9px;
}

.post-hint {
  text-align: center;
  padding: 12px;
  font-size: 11px;
  color: #aaa;
  letter-spacing: 2px;
}

.hint-blink {
  animation: post-blink 1s step-end infinite;
}

@keyframes post-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}
</style>
