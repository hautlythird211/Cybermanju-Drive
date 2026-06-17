<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ (e: 'select', mode: string): void }>()

const entries = [
  { label: 'Cybermanju OS', mode: 'normal', description: 'Post-Quantum Encrypted File System' },
  { label: 'Cybermanju OS (Recovery Mode)', mode: 'recovery', description: 'Single-user mode with diagnostics' },
  { label: 'Cybermanju OS (Verbose)', mode: 'verbose', description: 'Detailed boot logging' },
  { label: 'Memory Diagnostics (memtest86+)', mode: 'memtest', description: 'RAM integrity test' },
  { label: 'System Firmware Setup', mode: 'setup', description: 'UEFI BIOS Configuration' },
]

const selectedIdx = ref(0)
const countdown = ref(3)
let countdownTimer: ReturnType<typeof setInterval> | null = null

const autoBootMode = 'normal'

function confirmSelect() {
  emit('select', entries[selectedIdx.value].mode)
}

function decrement() {
  countdown.value--
  if (countdown.value <= 0) {
    emit('select', autoBootMode)
  }
}

function handleKey(e: KeyboardEvent) {
  if (e.key === 'ArrowUp' || e.key === 'k') {
    e.preventDefault()
    selectedIdx.value = (selectedIdx.value - 1 + entries.length) % entries.length
    countdown.value = 3
  } else if (e.key === 'ArrowDown' || e.key === 'j') {
    e.preventDefault()
    selectedIdx.value = (selectedIdx.value + 1) % entries.length
    countdown.value = 3
  } else if (e.key === 'Enter') {
    confirmSelect()
    if (countdownTimer) clearInterval(countdownTimer)
  }
}

onMounted(() => {
  countdownTimer = setInterval(decrement, 1000)
})

onUnmounted(() => {
  if (countdownTimer) clearInterval(countdownTimer)
})
</script>

<template>
  <div class="bootloader" @keydown="handleKey" tabindex="0" autofocus>
    <div class="grub-container">
      <div class="grub-header">
        <div class="grub-title">Cybermanju Boot Loader v1.2</div>
        <div class="grub-subtitle">Use ^ / v to select, Enter to confirm</div>
      </div>
      <div class="grub-menu">
        <div
          v-for="(entry, i) in entries"
          :key="entry.mode"
          class="grub-entry"
          :class="{ 'grub-selected': selectedIdx === i }"
          @click="selectedIdx = i; countdown = 3"
          @dblclick="confirmSelect"
        >
          <div class="grub-entry-main">
            <span class="grub-cursor">{{ selectedIdx === i ? '>' : ' ' }}</span>
            <span class="grub-label" :class="{ 'grub-highlight': selectedIdx === i }">{{ entry.label }}</span>
          </div>
          <div v-if="selectedIdx === i" class="grub-desc">{{ entry.description }}</div>
        </div>
      </div>
      <div class="grub-footer">
        <span class="grub-countdown">Auto-boot in {{ countdown }}s</span>
        <span class="grub-keys">[ENTER=boot] [e=edit] [c=console]</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bootloader {
  position: fixed;
  inset: 0;
  z-index: 99999;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Courier New', 'Fira Code', monospace;
  cursor: default;
}

.grub-container {
  width: 720px;
  max-width: 94vw;
  padding: 24px;
}

.grub-header {
  margin-bottom: 24px;
}

.grub-title {
  font-size: 14px;
  font-weight: 700;
  color: #ccc;
  letter-spacing: 1px;
  margin-bottom: 4px;
}

.grub-subtitle {
  font-size: 10px;
  color: #555;
}

.grub-menu {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.grub-entry {
  padding: 8px 12px;
  border-left: 2px solid transparent;
  cursor: pointer;
  transition: all 0.1s;
}

.grub-entry:hover {
  background: rgba(255, 255, 255, 0.03);
}

.grub-selected {
  background: rgba(0, 255, 65, 0.06);
  border-left-color: #00ff41;
}

.grub-entry-main {
  display: flex;
  align-items: center;
  gap: 8px;
}

.grub-cursor {
  color: #00ff41;
  font-weight: 700;
  width: 10px;
}

.grub-label {
  font-size: 13px;
  color: #888;
  font-weight: 600;
}

.grub-highlight {
  color: #eee;
}

.grub-desc {
  font-size: 10px;
  color: #555;
  margin-left: 18px;
  margin-top: 2px;
}

.grub-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 24px;
  padding-top: 12px;
  border-top: 1px solid #1a1a1a;
}

.grub-countdown {
  font-size: 10px;
  color: #00ff41;
  font-weight: 600;
}

.grub-keys {
  font-size: 9px;
  color: #444;
}
</style>
