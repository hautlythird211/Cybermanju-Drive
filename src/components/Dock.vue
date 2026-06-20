<template>
  <div ref="dockContainer" class="dock-container" role="toolbar" aria-label="Application Dock">
    <div class="dock">
      <div
        v-for="(app, idx) in dockApps"
        :key="app.panelType"
        :ref="(el) => { if (el) dockItemRefs[idx] = el as HTMLElement }"
        class="dock-item"
        :class="{ active: isAppActive(app.panelType), open: wm.isOpen(app.panelType) }"
        role="button"
        :tabindex="0"
        :aria-label="app.label"
        :aria-pressed="isAppActive(app.panelType)"
        @click="handleDockClick(app.panelType)"
        @contextmenu.prevent="handleDockContext($event, app.panelType)"
        @mouseenter="onDockItemEnter($event, idx)"
        @mouseleave="onDockItemLeave($event, idx)"
        @keydown.enter="handleDockClick(app.panelType)"
        @keydown.space.prevent="handleDockClick(app.panelType)"
        :title="app.label"
      >
        <div class="dock-icon">
          <Icon :icon="'mdi:' + app.icon" width="20" height="20" class="dock-iconify" />
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
        role="button"
        :tabindex="0"
        :aria-label="win.title + ' (minimized)'"
        @click="wm.restore(win.id)"
        @keydown.enter="wm.restore(win.id)"
        @keydown.space.prevent="wm.restore(win.id)"
        :title="win.title + ' (minimized)'"
      >
        <div class="dock-icon minimized">
          <Icon icon="mdi:window-minimize" width="20" height="20" class="dock-iconify" />
        </div>
        <div class="dock-indicator">
          <div class="indicator-dot muted" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, nextTick } from 'vue'
import gsap from 'gsap'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'
import { useGsapAnimation } from '@/composables/useGsapAnimation'
import type { PanelType } from '@/types'

const anim = useGsapAnimation()
const store = useAppStore()
const wm = useWindowManager()

const dockItemRefs = ref<Record<number, HTMLElement>>({})
const dockContainer = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)
const dockTweens = ref<Map<number, gsap.core.Tween>>(new Map())

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
  { panelType: 'encryption', label: 'Encryption', icon: 'shield-lock-outline', category: 'tools' },
  { panelType: 'compression', label: 'Compression', icon: 'zip-box-outline', category: 'tools' },
  { panelType: 'settings', label: 'Settings', icon: 'cog-outline', category: 'system' },
  { panelType: 'permissions', label: 'Permissions', icon: 'lock-outline', category: 'tools' },
  { panelType: 'trash', label: 'Trash', icon: 'delete-outline', category: 'system' },
  { panelType: 'users', label: 'Users', icon: 'account-group-outline', category: 'system' },
  { panelType: 'accounts', label: 'Accounts', icon: 'account-multiple-outline', category: 'system' },
  { panelType: 'terminal', label: 'Terminal', icon: 'console', category: 'system' },
  { panelType: 'art-maker', label: 'Art Maker', icon: 'palette-outline', category: 'system' },
  { panelType: 'system-monitor', label: 'System Monitor', icon: 'chart-line-variant', category: 'system' },
  { panelType: 'webdash', label: 'Web Dashboard', icon: 'web', category: 'tools' },
  { panelType: 'loose-groups', label: 'Loose Groups', icon: 'shape-outline', category: 'organize' },
  { panelType: 'history', label: 'History', icon: 'history', category: 'system' },
  { panelType: 'task-manager', label: 'Task Manager', icon: 'memory', category: 'system' },
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
    } else if (wm.activeWindow.value?.id === existing.id) {
      wm.minimize(existing.id)
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

// ── GSAP hover magnification ──
let magnifyRaf: number | null = null

function onDockItemEnter(e: MouseEvent, idx: number) {
  if (magnifyRaf) cancelAnimationFrame(magnifyRaf)
  magnifyRaf = requestAnimationFrame(() => {
    const el = dockItemRefs.value[idx]
    if (el) {
      dockTweens.value.get(idx)?.kill()
      const t = gsap.to(el, { scale: 1.35, y: -4, duration: 0.2, ease: 'cubic-bezier(0.22, 1, 0.36, 1)', overwrite: 'auto', force3D: true })
      dockTweens.value.set(idx, t)
    }
    magnifyRaf = null
  })
}

function onDockItemLeave(e: MouseEvent, idx: number) {
  if (magnifyRaf) cancelAnimationFrame(magnifyRaf)
  magnifyRaf = requestAnimationFrame(() => {
    const el = dockItemRefs.value[idx]
    if (el) {
      dockTweens.value.get(idx)?.kill()
      gsap.to(el, { scale: 1, y: 0, duration: 0.2, ease: 'cubic-bezier(0.22, 1, 0.36, 1)', overwrite: 'auto', force3D: true })
      dockTweens.value.delete(idx)
    }
    magnifyRaf = null
  })
}

onMounted(async () => {
  gsapCtx.value = gsap.context(() => {
    const items = Object.values(dockItemRefs.value).filter(Boolean) as HTMLElement[]
    if (items.length > 0) {
      anim.staggerIn(items, { stagger: 0.04, from: 'start', duration: 0.3 })
    }
  })
})

onUnmounted(() => {
  dockTweens.value.forEach(t => t.kill())
  dockTweens.value.clear()
  if (magnifyRaf) cancelAnimationFrame(magnifyRaf)
  gsapCtx.value?.revert()
})
</script>

<style scoped>
.dock-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 60px;
  padding: 0 12px;
  z-index: 50;
  pointer-events: none;
  position: relative;
  overflow: visible;
}

.dock {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 14px;
  background: rgba(8, 8, 10, 0.6);
  backdrop-filter: blur(50px) saturate(1.8);
  -webkit-backdrop-filter: blur(50px) saturate(1.8);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 22px;
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.25),
    0 0 0 1px rgba(255, 255, 255, 0.02),
    inset 0 1px 0 rgba(255, 255, 255, 0.06);
  pointer-events: auto;
  position: relative;
  isolation: isolate;
}

/* Subtle top highlight */
.dock::before {
  content: '';
  position: absolute;
  top: 0;
  left: 15%;
  right: 15%;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
  border-radius: 22px 22px 0 0;
  pointer-events: none;
}

/* Cyberpunk accent glow */
.dock::after {
  content: '';
  position: absolute;
  inset: -1px;
  border-radius: 23px;
  background: linear-gradient(135deg,
    rgba(0, 255, 65, 0.04),
    transparent 30%,
    transparent 70%,
    rgba(0, 255, 65, 0.02));
  pointer-events: none;
  z-index: -1;
}

.dock-item {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  width: 44px;
  height: 44px;
  padding: 0;
  cursor: pointer;
  border-radius: 12px;
  position: relative;
  will-change: transform;
  outline: none;
  transition: background 0.2s ease, transform 0.15s cubic-bezier(0.22, 1, 0.36, 1);
}

.dock-item:focus-visible {
  box-shadow: 0 0 0 2px rgba(0, 255, 65, 0.25);
}

.dock-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.dock-item:active {
  transform: scale(0.92) !important;
}

.dock-item.active {
  background: rgba(255, 255, 255, 0.08);
}

.dock-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.dock-iconify {
  color: rgba(255, 255, 255, 0.6);
  transition: color 0.2s ease, filter 0.2s ease;
}

.dock-item.active .dock-iconify {
  color: rgba(255, 255, 255, 0.9);
  filter: drop-shadow(0 0 6px rgba(0, 255, 65, 0.15));
}

.dock-item:hover .dock-iconify {
  color: rgba(255, 255, 255, 0.85);
}

.dock-icon.minimized {
  opacity: 0.3;
}

.dock-badge {
  position: absolute;
  top: -2px;
  right: -4px;
  background: #ff453a;
  color: #fff;
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 700;
  min-width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  padding: 0 4px;
  border: 1.5px solid rgba(0, 0, 0, 0.3);
  pointer-events: none;
  box-shadow: 0 2px 8px rgba(255, 69, 58, 0.3);
}

.dock-indicator {
  position: absolute;
  bottom: 2px;
  left: 50%;
  transform: translateX(-50%);
}

.indicator-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.15);
  transition: all 0.25s cubic-bezier(0.22, 1, 0.36, 1);
}

.indicator-dot.active {
  background: var(--accent);
  width: 16px;
  border-radius: 2px;
  height: 3px;
  box-shadow: 0 0 8px rgba(0, 255, 65, 0.2);
}

.indicator-dot.muted {
  background: rgba(255, 255, 255, 0.06);
}

.minimized-item .dock-icon {
  opacity: 0.25;
}

.dock-divider {
  width: 1px;
  height: 24px;
  background: rgba(255, 255, 255, 0.08);
  margin: 0 4px;
}

.dock-label {
  display: none;
}
</style>
