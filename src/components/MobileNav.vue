<template>
  <nav class="mobile-nav glass" role="navigation" aria-label="MOBILE NAVIGATION">
    <button class="mn-btn" :class="{ active: store.currentPanel === 'files' }" @click="store.currentPanel = 'files'" aria-label="FILES"><Icon icon="mdi:folder-outline" width="18" height="18" /></button>
    <button class="mn-btn" :class="{ active: store.currentPanel === 'search' }" @click="store.currentPanel = 'search'; store.searchQuery ? null : null" aria-label="SEARCH"><Icon icon="mdi:magnify" width="18" height="18" /></button>
    <button class="mn-btn" :class="{ active: store.currentPanel === 'favorites' }" @click="store.currentPanel = 'favorites'" aria-label="FAVORITES"><Icon icon="mdi:star-outline" width="18" height="18" /></button>
    <button class="mn-btn" :class="{ active: store.currentPanel === 'recent' }" @click="store.currentPanel = 'recent'" aria-label="RECENT"><Icon icon="mdi:clock-outline" width="18" height="18" /></button>
    <button class="mn-btn" :class="{ active: store.currentPanel === 'settings' }" @click="store.currentPanel = 'settings'" aria-label="SETTINGS"><Icon icon="mdi:cog-outline" width="18" height="18" /></button>
  </nav>
</template>

<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
const store = useAppStore()
</script>

<style scoped>
.mobile-nav {
  display: none;
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 48px;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  z-index: 100;
  font-family: var(--font-mono);
}

.mn-btn {
  flex: 1;
  background: transparent;
  border: none;
  border-right: 1px solid rgba(255,255,255,0.06);
  color: rgba(255,255,255,0.35);
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 44px;
  transition: all 0.15s;
}

.mn-btn:last-child {
  border-right: none;
}

.mn-btn:hover {
  color: rgba(255,255,255,0.7);
  background: rgba(255,255,255,0.04);
}

.mn-btn.active {
  color: var(--text-accent);
  background: rgba(var(--accent-rgb), 0.08);
}

@media (max-width: 768px) {
  .mobile-nav {
    display: flex;
  }
}
</style>
