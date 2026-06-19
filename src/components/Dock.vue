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
        role="button"
        :tabindex="0"
        :aria-label="win.title + ' (minimized)'"
        @click="wm.restore(win.id)"
        @keydown.enter="wm.restore(win.id)"
        @keydown.space.prevent="wm.restore(win.id)"
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

// ── GSAP hover magnification with RAF throttling ──
let magnifyRaf: number | null = null

function onDockItemEnter(e: MouseEvent, idx: number) {
  if (magnifyRaf) cancelAnimationFrame(magnifyRaf)
  magnifyRaf = requestAnimationFrame(() => {
    const el = dockItemRefs.value[idx]
    if (el) {
      dockTweens.value.get(idx)?.kill()
      const t = gsap.to(el, { scale: 1.4, duration: 0.2, ease: 'cubic-bezier(0.22, 1, 0.36, 1)', overwrite: 'auto', force3D: true })
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
      gsap.to(el, { scale: 1, duration: 0.2, ease: 'cubic-bezier(0.22, 1, 0.36, 1)', overwrite: 'auto', force3D: true })
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
  height: 48px;
  padding: 0 12px;
  z-index: 50;
  pointer-events: none;
  position: relative;
  overflow: visible;
}

.dock {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 10px;
  background: rgba(8, 8, 12, 0.88);
  backdrop-filter: blur(28px);
  -webkit-backdrop-filter: blur(28px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  box-shadow:
    0 4px 28px rgba(0, 0, 0, 0.5),
    0 0 1px rgba(0, 255, 65, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.04);
  pointer-events: auto;
  position: relative;
  isolation: isolate;
}

/* Velvet texture overlay */
.dock::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  z-index: -1;
  background:
    repeating-radial-gradient(circle at 50% 50%, transparent 0, rgba(255,255,255,0.018) 1px, transparent 2px),
    repeating-conic-gradient(rgba(255,255,255,0.01) 0% 25%, transparent 0% 50%) 0 0 / 3px 3px,
    radial-gradient(ellipse at 50% 0%, rgba(0,255,65,0.05) 0%, transparent 50%),
    radial-gradient(ellipse at 20% 100%, rgba(90,240,255,0.04) 0%, transparent 40%),
    radial-gradient(ellipse at 80% 100%, rgba(179,136,255,0.03) 0%, transparent 40%);
  pointer-events: none;
  mix-blend-mode: screen;
}

/* Bottom psychedelic glow line */
.dock::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 10%;
  right: 10%;
  height: 1px;
  background: linear-gradient(90deg,
    transparent 0%,
    rgba(0, 255, 65, 0.2) 25%,
    rgba(90, 240, 255, 0.25) 50%,
    rgba(179, 136, 255, 0.2) 75%,
    transparent 100%);
  background-size: 200% 100%;
  animation: dock-shimmer 5s ease-in-out infinite;
  border-radius: 0 0 14px 14px;
}

.dock-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  padding: 0;
  cursor: pointer;
  border-radius: 8px;
  position: relative;
  will-change: transform;
  outline: none;
  transition: all 0.15s cubic-bezier(0.22, 1, 0.36, 1);
}

.dock-item:focus-visible {
  box-shadow: 0 0 0 2px rgba(0, 255, 65, 0.4);
}

.dock-item:hover {
  transform: translateY(-1px);
  background: rgba(0, 255, 65, 0.05);
}

.dock-item:active {
  transform: scale(0.92);
}

.dock-item.active {
  background: rgba(0, 255, 65, 0.08);
}

.dock-icon {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.dock-iconify {
  color: rgba(255, 255, 255, 0.55);
  transition: all 0.15s;
}

.dock-item.active .dock-iconify {
  color: rgba(0, 255, 65, 0.9);
  filter: drop-shadow(0 0 6px rgba(0, 255, 65, 0.3));
}

.dock-item:hover .dock-iconify {
  color: rgba(255, 255, 255, 0.85);
}

.dock-icon.minimized {
  opacity: 0.35;
}

.dock-badge {
  position: absolute;
  top: -3px;
  right: -5px;
  background: #ff5f57;
  color: #fff;
  font-family: var(--font-mono);
  font-size: 7px;
  font-weight: 700;
  min-width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  padding: 0 3px;
  border: 1.5px solid rgba(0, 0, 0, 0.4);
  pointer-events: none;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
}

.dock-indicator {
  position: absolute;
  bottom: -2px;
  left: 50%;
  transform: translateX(-50%);
}

.indicator-dot {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  transition: all 0.2s;
}

.indicator-dot.active {
  background: rgba(0, 255, 65, 0.7);
  width: 12px;
  border-radius: 2px;
  height: 2.5px;
  box-shadow: 0 0 8px rgba(0, 255, 65, 0.3);
}

.indicator-dot.muted {
  background: rgba(255, 255, 255, 0.1);
}

.minimized-item .dock-icon {
  opacity: 0.3;
}

.dock-divider {
  width: 1px;
  height: 20px;
  background: rgba(255, 255, 255, 0.06);
  margin: 0 4px;
}

.dock-label {
  display: none;
}

@keyframes dock-shimmer {
  0% { background-position: 200% 0; }
  50% { background-position: 0% 0; }
  100% { background-position: -200% 0; }
}
</style>
