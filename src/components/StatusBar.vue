<template>
  <footer ref="statusRef" class="statusbar" role="status" aria-live="polite">
    <div class="sb-left">
      <span class="sb-path">{{ store.currentPath }}</span>
      <Icon v-if="store.isLoading" icon="svg-spinners:3-dots-bounce" width="14" height="14" class="sb-spinner" />
    </div>

    <div class="sb-center">
      <span class="sb-item">{{ store.files.length }} FILES</span>
      <template v-if="store.selectedFile">
        <span class="sb-div">|</span>
        <span class="sb-item">SEL: {{ store.selectedFile.name }}</span>
      </template>

      <template v-if="store.selectedFile?.encrypted">
        <span class="sb-div">|</span>
        <span class="sb-badge">ENC {{ store.selectedFile.encryptionAlgorithm?.toUpperCase() || '' }}</span>
      </template>

      <template v-if="store.selectedFile?.compressionLayers && store.selectedFile.compressionLayers[0] && store.selectedFile.compressionLayers[0] !== 'none'">
        <span class="sb-div">|</span>
        <span class="sb-badge">{{ store.selectedFile.compressionLayers[0].toUpperCase() }}</span>
      </template>

      <template v-if="store.selectedFile?.hashBlake3">
        <span class="sb-div">|</span>
        <span class="sb-hash">B3:{{ store.selectedFile.hashBlake3.substring(0, 10) }}..</span>
      </template>
    </div>

    <div class="sb-right">
      <span
        class="sb-clickable sync-icon"
        :class="{ 'sb-active': isSyncActive }"
        title="SYNC STATUS"
        aria-label="SYNC STATUS"
        role="button"
        tabindex="0"
        @keydown.enter="store.commandPaletteOpen = true"
        @keydown.space.prevent="store.commandPaletteOpen = true"
      >{{ isSyncActive ? 'SYNC:' + store.syncProgress?.status.toUpperCase() : 'SYNC:IDLE' }}</span>
      <span class="sb-div">|</span>
      <span
        class="sb-clickable"
        :class="{ 'sb-active': store.matrixRainEnabled }"
        @click="store.matrixRainEnabled = !store.matrixRainEnabled"
        title="TOGGLE MATRIX RAIN"
        aria-label="TOGGLE MATRIX RAIN BACKGROUND"
        role="button"
        tabindex="0"
        @keydown.enter="store.matrixRainEnabled = !store.matrixRainEnabled"
        @keydown.space.prevent="store.matrixRainEnabled = !store.matrixRainEnabled"
      >{{ store.matrixRainEnabled ? 'GFX:ON' : 'GFX:OFF' }}</span>
      <span class="sb-div">|</span>
      <span class="sb-clickable" @click="store.commandPaletteOpen = true" title="COMMAND PALETTE (CTRL+K)" aria-label="OPEN COMMAND PALETTE" role="button" tabindex="0" @keydown.enter="store.commandPaletteOpen = true" @keydown.space.prevent="store.commandPaletteOpen = true">CMD+K</span>
      <span class="sb-div">|</span>
      <span class="sb-tech">{{ isWebMode() ? 'WEB MODE' : 'TAURI MODE' }}</span>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { isWebMode } from '@/composables/useTauri'

const store = useAppStore()
const isSyncActive = computed(() =>
  store.syncProgress !== null && store.syncProgress.status !== 'idle' && store.syncProgress.status !== 'done'
)
</script>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  height: 24px;
  padding: 0 8px;
  gap: 6px;
  background: var(--bg-elevated);
  border-top: 1px solid var(--border-subtle);
  font-size: var(--font-size-xs);
  overflow: hidden;
  z-index: 10;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.sb-left {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.sb-path {
  font-size: var(--font-size-xs);
  font-weight: 600;
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
}

.sb-spinner {
  display: inline-flex;
  vertical-align: middle;
  margin-left: 8px;
}

.sb-center {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.sb-item {
  white-space: nowrap;
  color: var(--text-muted);
}

.sb-div {
  color: var(--text-muted);
  opacity: 0.5;
}

.sb-badge {
  font-weight: 700;
  font-size: var(--font-size-xs);
  color: var(--accent);
  border: 1px solid var(--accent);
  padding: 0 4px;
}

.sb-hash {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
}

.sb-right {
  margin-left: auto;
  display: flex;
  align-items: center;
}

.sb-tech {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  letter-spacing: 0.5px;
}

.sb-clickable {
  cursor: pointer;
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  padding: 2px 4px;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  outline: none;
}

.sb-clickable:hover {
  color: var(--text-primary);
  background: var(--bg-overlay);
}

.sb-clickable:focus-visible {
  box-shadow: var(--focus-ring);
}

.sb-active {
  color: var(--accent);
  font-weight: 700;
}

@media (max-width: 768px) {
  .sb-center {
    display: none;
  }
  .sb-path {
    max-width: 120px;
  }
}
</style>
