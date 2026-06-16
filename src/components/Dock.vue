<template>
  <div class="dock-container">
    <div class="dock">
      <div
        v-for="app in dockApps"
        :key="app.panelType"
        class="dock-item"
        :class="{ active: isAppActive(app.panelType), open: wm.isOpen(app.panelType) }"
        @click="handleDockClick(app.panelType)"
        @contextmenu.prevent="handleDockContext($event, app.panelType)"
        :title="app.label"
      >
        <div class="dock-icon">
          <Icon :icon="'mdi:' + app.icon" width="18" height="18" class="dock-iconify" />
          <span
            v-if="app.panelType === 'trash' && store.trashCount > 0"
            class="dock-badge"
          >{{ store.trashCount > 99 ? '99+' : store.trashCount }}</span>
        </div>
        <div class="dock-indicator" v-if="wm.isOpen(app.panelType)">
          <div class="indicator-dot" :class="{ active: isAppActive(app.panelType) }" />
        </div>
      </div>

      <div class="dock-divider" />

      <div
        v-for="win in minimizedWindows"
        :key="win.id"
        class="dock-item minimized-item"
        @click="wm.restore(win.id)"
        :title="win.title + ' (minimized)'"
      >
        <div class="dock-icon minimized">
          <Icon icon="mdi:window-minimize" width="18" height="18" class="dock-iconify" />
        </div>
        <div class="dock-indicator">
          <div class="indicator-dot muted" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'
import type { PanelType } from '@/types'

const store = useAppStore()
const wm = useWindowManager()

interface DockApp {
  panelType: PanelType
  label: string
  icon: string
  category: string
}

const dockApps = computed<DockApp[]>(() => [
  { panelType: 'files', label: 'File Browser', icon: 'folder-outline', category: 'core' },
  { panelType: 'search', label: 'Search', icon: 'magnify', category: 'core' },
  { panelType: 'collections', label: 'Collections', icon: 'bookmark-multiple-outline', category: 'organize' },
  { panelType: 'faces', label: 'People', icon: 'face-man-outline', category: 'organize' },
  { panelType: 'map', label: 'Map', icon: 'map-outline', category: 'tools' },
  { panelType: 'code', label: 'Code', icon: 'code-tags', category: 'tools' },
  { panelType: 'sync', label: 'Sync', icon: 'sync', category: 'tools' },
  { panelType: 'transfer', label: 'Transfer', icon: 'transfer', category: 'tools' },
  { panelType: 'import', label: 'Import', icon: 'file-import-outline', category: 'tools' },
  { panelType: 'storage', label: 'Storage', icon: 'harddisk', category: 'tools' },
  { panelType: 'settings', label: 'Settings', icon: 'cog-outline', category: 'system' },
  { panelType: 'history', label: 'History', icon: 'history', category: 'system' },
  { panelType: 'trash', label: 'Trash', icon: 'delete-outline', category: 'system' },
  { panelType: 'users', label: 'Users', icon: 'account-group-outline', category: 'system' },
  { panelType: 'accounts', label: 'Accounts', icon: 'account-outline', category: 'system' },
])

const minimizedWindows = computed(() =>
  wm.windows.value.filter(w => w.minimized)
)

function isAppActive(panelType: PanelType): boolean {
  return wm.windows.value.some(
    w => w.panelType === panelType && !w.minimized
  )
}

function handleDockClick(panelType: PanelType) {
  const existing = wm.windows.value.find(w => w.panelType === panelType)
  if (existing) {
    if (existing.minimized) {
      wm.restore(existing.id)
    } else {
      wm.focus(existing.id)
    }
  } else {
    wm.open(panelType)
  }
}

function handleDockContext(e: MouseEvent, panelType: PanelType) {
  const existing = wm.windows.value.find(w => w.panelType === panelType)
  if (existing) {
    const items = [
      { label: 'Focus', action: () => wm.focus(existing.id) },
      { label: 'Minimize', action: () => wm.minimize(existing.id) },
      { label: 'Close', action: () => wm.close(existing.id) },
    ]
    window.dispatchEvent(new CustomEvent('cybermanju:dock-context', {
      detail: { x: e.clientX, y: e.clientY, items },
    }))
  }
}
</script>

<style scoped>
.dock-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 60px;
  padding: 0 16px;
  background: transparent;
  z-index: 50;
  pointer-events: none;
}

.dock {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  background: rgba(15, 15, 15, 0.5);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
  pointer-events: auto;
}

.dock-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 4px 6px;
  cursor: pointer;
  border-radius: 8px;
  transition: all 0.12s;
  position: relative;
  min-width: 44px;
}

.dock-item:hover {
  background: rgba(255, 255, 255, 0.06);
  transform: translateY(-2px);
}

.dock-item:active {
  transform: translateY(0px);
}

.dock-item.active {
  background: rgba(0, 255, 65, 0.06);
}

.dock-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1a1a1a;
  border-radius: 8px;
  transition: all 0.12s;
  border: 1px solid #2a2a2a;
  position: relative;
}

.dock-item:hover .dock-icon {
  background: #222;
  border-color: #3a3a3a;
}

.dock-item.active .dock-icon {
  background: rgba(0, 255, 65, 0.1);
  border-color: rgba(0, 255, 65, 0.2);
}

.dock-item.active:hover .dock-icon {
  background: rgba(0, 255, 65, 0.15);
  border-color: rgba(0, 255, 65, 0.3);
}

.dock-badge {
  position: absolute;
  top: -4px;
  right: -6px;
  background: #ff5f57;
  color: #fff;
  font-family: 'Courier New', monospace;
  font-size: 8px;
  font-weight: 700;
  min-width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  padding: 0 3px;
  border: 1px solid rgba(0, 0, 0, 0.2);
  pointer-events: none;
}

.dock-iconify {
  color: #ccc;
  transition: color 0.15s;
}

.dock-item.active .dock-iconify {
  color: #00ff41;
}

.dock-item:hover .dock-iconify {
  color: #fff;
}

.dock-icon.minimized {
  opacity: 0.5;
}

.dock-indicator {
  height: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.indicator-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: #444;
  transition: all 0.15s;
}

.indicator-dot.active {
  background: #00ff41;
  width: 16px;
  border-radius: 2px;
}

.indicator-dot.muted {
  background: #333;
}

.minimized-item .dock-icon {
  opacity: 0.4;
}

.dock-divider {
  width: 1px;
  height: 28px;
  background: rgba(255, 255, 255, 0.08);
  margin: 0 4px;
}
</style>
