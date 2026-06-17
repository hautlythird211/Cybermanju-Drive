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
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-glass);
  border-radius: 14px;
  box-shadow: var(--shadow-glass), inset 0 1px 0 rgba(255, 255, 255, 0.05);
  pointer-events: auto;
}

.dock-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 4px 6px;
  cursor: pointer;
  border-radius: var(--radius-lg);
  transition: all var(--transition-fast);
  position: relative;
  min-width: 44px;
  will-change: transform;
  outline: none;
}

.dock-item:focus-visible {
  box-shadow: var(--focus-ring);
}

.dock-item:hover {
  background: rgba(255, 255, 255, 0.06);
  transform: translateY(-2px);
}

.dock-item:active {
  transform: translateY(0px);
}

.dock-item.active {
  background: var(--accent-dim);
}

.dock-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border-radius: var(--radius-lg);
  transition: all var(--transition-fast);
  border: 1px solid var(--border-subtle);
  position: relative;
}

.dock-item:hover .dock-icon {
  background: var(--bg-overlay);
  border-color: var(--border-medium);
}

.dock-item.active .dock-icon {
  background: var(--accent-dim);
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
  background: var(--danger);
  color: #fff;
  font-family: var(--font-mono);
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
  color: var(--text-secondary);
  transition: color var(--transition-normal);
}

.dock-item.active .dock-iconify {
  color: var(--accent);
}

.dock-item:hover .dock-iconify {
  color: var(--text-primary);
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
  background: var(--border-medium);
  transition: all var(--transition-normal);
}

.indicator-dot.active {
  background: var(--accent);
  width: 16px;
  border-radius: 2px;
}

.indicator-dot.muted {
  background: var(--border-subtle);
}

.minimized-item .dock-icon {
  opacity: 0.4;
}

.dock-divider {
  width: 1px;
  height: 28px;
  background: var(--border-glass);
  margin: 0 4px;
}
</style>
