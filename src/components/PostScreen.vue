<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ (e: 'complete'): void }>()

const lines = ref<string[]>([])
const showBootMenu = ref(false)
const keyBuffer = ref('')
let timer: ReturnType<typeof setTimeout> | null = null

const POST_SEQUENCE = [
  { text: 'CYBERMANJU UEFI BIOS v2.4.1 (Build 2025-03-15)', delay: 600 },
  { text: 'CPU: Hybrid PQC-NEON @ 2.8GHz, 8C/16T [PASS]', delay: 400 },
  { text: 'CPU Features: AES-NI, SHA-NI, AVX2, AVX-512, VAES, VPCLMULQDQ', delay: 350 },
  { text: 'MEM: Testing 32768MB ECC DDR5-6400...', delay: 300 },
  { text: 'MEM: Channel A: 16384MB [OK]', delay: 250 },
  { text: 'MEM: Channel B: 16384MB [OK]', delay: 250 },
  { text: 'MEM: POST-QUANTUM CRYPTO ZONES: 16 GUARD REGIONS [ACTIVE]', delay: 350 },
  { text: 'PCH: Z890 Chipset — DMI 4.0 x8 [DETECTED]', delay: 300 },
  { text: 'PCI: Bus enumeration... 47 devices found [COMPLETE]', delay: 350 },
  { text: 'PCI: NVMe Controller #1 — Samsung PM9E1 2TB [DETECTED]', delay: 300 },
  { text: 'PCI: NVMe Controller #2 — WD Black SN850X 4TB [DETECTED]', delay: 300 },
  { text: 'PCI: SATA Controller — 6 ports, 2 devices [DETECTED]', delay: 300 },
  { text: 'USB: XHCI Controller #0 at 0xFE800000 (irq 16)', delay: 250 },
  { text: 'USB: 5 hubs, 14 devices enumerated [OK]', delay: 250 },
  { text: 'NET: Intel I226-V 2.5GbE — MAC: 2A:4F:8E:0C:D1:73 [DETECTED]', delay: 300 },
  { text: 'NET: Intel BE201 Wi-Fi 7 + Bluetooth 5.4 [DETECTED]', delay: 300 },
  { text: 'SND: Realtek ALC1220 — HD Audio Codec [INITIALIZED]', delay: 250 },
  { text: 'SND: NVIDIA GPU HDMI/DP Audio Controller [DETECTED]', delay: 250 },
  { text: 'TPM: 2.0 Security Module — firmware v9.1.2 [ACTIVE]', delay: 300 },
  { text: 'RTC: System Clock — 2025-03-15 14:23:07 UTC [SYNCED]', delay: 250 },
  { text: 'ACPI: DSDT loaded — 347 tables [PARSED]', delay: 300 },
  { text: 'ACPI: Thermal zones: CPU=42°C, PCH=38°C, NVMe1=35°C [NOMINAL]', delay: 350 },
  { text: 'SYS: CMOS checksum OK — battery voltage 3.12V [NOMINAL]', delay: 300 },
  { text: 'SYS: SMBIOS v3.7 — Cybermanju Drive MB rev 2.0 [DETECTED]', delay: 300 },
  { text: 'SYS: POST complete — press DEL for Setup, F12 for Boot Menu, any key to boot', delay: 600 },
]

function tack() {
  const el = document.querySelector('.post-terminal-text')
  if (el) el.scrollTop = el.scrollHeight
}

function runPost() {
  let i = 0
  function next() {
    if (i < POST_SEQUENCE.length) {
      const entry = POST_SEQUENCE[i]
      lines.value.push(entry.text)
      tack()
      i++
      if (entry.text.includes('Z890') || entry.text.includes('CMOS') || entry.text.includes('SBY')) {
        timer = setTimeout(next, entry.delay + 200)
      } else if (entry.text.includes('POST complete')) {
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

onMounted(() => {
  runPost()
})

onUnmounted(() => {
  if (timer) clearTimeout(timer)
})
</script>

<template>
  <div class="post-screen" @keydown="keyBuffer = ''" tabindex="0">
    <div class="post-terminal">
      <div class="post-terminal-header">
        <span class="post-motherboard">CYBERMANJU DRIVE MB rev 2.0</span>
        <span class="post-bios-ver">UEFI BIOS v2.4.1</span>
      </div>
      <div class="post-terminal-text">
        <div v-for="(line, i) in lines" :key="i" class="post-line"
          :class="{ 'post-ok': line.includes('[OK]') || line.includes('[ACTIVE]') || line.includes('[COMPLETE]') || line.includes('[NOMINAL]') || line.includes('[INITIALIZED]') || line.includes('[PASS]') || line.includes('[DETECTED]') || line.includes('[SYNCED]') || line.includes('[PARSED]') }">
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
