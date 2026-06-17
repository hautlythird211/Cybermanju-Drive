import { ref, computed, markRaw, type Component } from 'vue'
import type { PanelType } from '@/types'
import { MODULE_METADATA } from '@/types'
import FileGrid from '@/components/FileGrid.vue'
import CollectionsPanel from '@/components/CollectionsPanel.vue'
import FaceGroupingPanel from '@/components/FaceGroupingPanel.vue'
import MapView from '@/components/MapView.vue'
import CodeIntelligencePanel from '@/components/CodeIntelligencePanel.vue'
import UserManagementPanel from '@/components/UserManagementPanel.vue'
import WebDashboardPanel from '@/components/WebDashboardPanel.vue'
import SyncPanel from '@/components/SyncPanel.vue'
import SettingsPage from '@/components/SettingsPage.vue'
import StorageDashboard from '@/components/StorageDashboard.vue'
import EncryptionPanel from '@/components/EncryptionPanel.vue'
import CompressionPanel from '@/components/CompressionPanel.vue'
import AccountsPanel from '@/components/AccountsPanel.vue'
import FilePermissionsPanel from '@/components/FilePermissionsPanel.vue'
import FilePreview from '@/components/FilePreview.vue'
import ImportWindow from '@/components/ImportWindow.vue'
import TransferWindow from '@/components/TransferWindow.vue'
import WindowContent from '@/components/WindowContent.vue'
import SystemMonitor from '@/components/SystemMonitor.vue'
import TaskManager from '@/components/TaskManager.vue'
import Terminal from '@/components/Terminal.vue'

export interface WindowState {
  id: string
  panelType: PanelType
  title: string
  icon: string
  x: number
  y: number
  width: number
  height: number
  minimized: boolean
  zIndex: number
  component: Component | null
  props?: Record<string, unknown>
  screenX: number
  screenY: number
  tileSlot: number
  animState: 'idle' | 'entering' | 'exiting'
}

type SizeMap = { [K in PanelType]?: { width: number; height: number } } & {
  permissions?: { width: number; height: number }
}
const defaultSizes: SizeMap = {
  files: { width: 620, height: 440 },
  search: { width: 440, height: 380 },
  collections: { width: 380, height: 340 },
  faces: { width: 420, height: 360 },
  map: { width: 520, height: 400 },
  code: { width: 480, height: 400 },
  users: { width: 400, height: 360 },
  sync: { width: 440, height: 360 },
  settings: { width: 440, height: 420 },
  trash: { width: 400, height: 340 },
  activity: { width: 400, height: 340 },
  favorites: { width: 360, height: 300 },
  recent: { width: 360, height: 300 },
  accounts: { width: 380, height: 340 },
  'loose-groups': { width: 360, height: 320 },
  style: { width: 360, height: 300 },
  storage: { width: 440, height: 380 },
  dashboard: { width: 460, height: 380 },
  webdash: { width: 480, height: 400 },
  encryption: { width: 380, height: 340 },
  compression: { width: 380, height: 340 },
  permissions: { width: 360, height: 320 },
  preview: { width: 380, height: 420 },
  import: { width: 480, height: 400 },
  transfer: { width: 420, height: 460 },
  'system-monitor': { width: 520, height: 400 },
  'task-manager': { width: 540, height: 420 },
  terminal: { width: 480, height: 360 },
}

const inlinePanels: PanelType[] = [
  'search', 'trash', 'activity', 'favorites', 'recent',
  'loose-groups', 'style'
]

const panelComponentMap: Record<string, Component> = {
  files: FileGrid,
  collections: CollectionsPanel,
  faces: FaceGroupingPanel,
  map: MapView,
  code: CodeIntelligencePanel,
  users: UserManagementPanel,
  dashboard: WebDashboardPanel,
  sync: SyncPanel,
  settings: SettingsPage,
  storage: StorageDashboard,
  encryption: EncryptionPanel,
  compression: CompressionPanel,
  permissions: FilePermissionsPanel,
  preview: FilePreview,
  import: ImportWindow,
  transfer: TransferWindow,
  webdash: WebDashboardPanel,
  'system-monitor': SystemMonitor,
  'task-manager': TaskManager,
  terminal: Terminal,
  accounts: AccountsPanel,
}

function getComponent(panelType: PanelType): Component | null {
  if (panelComponentMap[panelType]) return markRaw(panelComponentMap[panelType])
  if (inlinePanels.includes(panelType)) return markRaw(WindowContent)
  return null
}

let windowCounter = 0

const windows = ref<WindowState[]>([])
const nextZIndex = ref(10)
const windowFocusHistory = ref<string[]>([])

export function useWindowManager() {
  const activeWindow = computed(() => {
    if (windowFocusHistory.value.length === 0) return null
    const id = windowFocusHistory.value[windowFocusHistory.value.length - 1]
    return windows.value.find(w => w.id === id) || null
  })

  const currentScreen = ref({ x: 0, y: 0 })

  function allocateTile(screenX: number, screenY: number): number {
    const taken = windows.value
      .filter(w => w.screenX === screenX && w.screenY === screenY)
      .map(w => w.tileSlot)
    for (let slot = 0; slot < 4; slot++) {
      if (!taken.includes(slot)) return slot
    }
    return -1
  }

  function findNextScreen(): { x: number; y: number } {
    const cx = currentScreen.value.x
    const cy = currentScreen.value.y
    if (allocateTile(cx, cy) >= 0) return { x: cx, y: cy }
    const dirs = [{ x: 1, y: 0 }, { x: -1, y: 0 }, { x: 0, y: 1 }, { x: 0, y: -1 }]
    for (const d of dirs) {
      const nx = cx + d.x
      const ny = cy + d.y
      if (allocateTile(nx, ny) >= 0) return { x: nx, y: ny }
    }
    return { x: cx + 1, y: cy }
  }

  function open(panelType: PanelType, props?: Record<string, unknown>) {
    const existing = windows.value.find(
      w => w.panelType === panelType && !w.minimized
    )
    if (existing) {
      focus(existing.id)
      return existing.id
    }

    const meta = MODULE_METADATA[panelType] || { label: panelType.toUpperCase(), icon: '[*]' }
    const size = defaultSizes[panelType] || { width: 520, height: 440 }
    const id = `win-${++windowCounter}`
    const comp = getComponent(panelType)

    const screen = findNextScreen()
    const tileSlot = allocateTile(screen.x, screen.y)

    const resolvedProps = { ...(props || {}) }
    if (inlinePanels.includes(panelType)) {
      resolvedProps.panelType = panelType
    }

    const win: WindowState = {
      id,
      panelType,
      title: meta.label,
      icon: meta.icon,
      x: 0,
      y: 0,
      width: size.width,
      height: size.height,
      minimized: false,
      zIndex: nextZIndex.value++,
      component: comp,
      props: Object.keys(resolvedProps).length > 0 ? resolvedProps : undefined,
      screenX: screen.x,
      screenY: screen.y,
      tileSlot,
      animState: 'entering',
    }
    windows.value.push(win)
    windowFocusHistory.value = windowFocusHistory.value.filter(w => w !== id)
    windowFocusHistory.value.push(id)
    currentScreen.value = { x: screen.x, y: screen.y }
    return id
  }

  function close(id: string) {
    const win = windows.value.find(w => w.id === id)
    if (win) {
      win.animState = 'exiting'
      setTimeout(() => {
        windows.value = windows.value.filter(w => w.id !== id)
        windowFocusHistory.value = windowFocusHistory.value.filter(w => w !== id)
      }, 300)
    }
  }

  function minimize(id: string) {
    const win = windows.value.find(w => w.id === id)
    if (win) {
      win.animState = 'exiting'
      setTimeout(() => {
        win.minimized = true
        win.animState = 'idle'
        windowFocusHistory.value = windowFocusHistory.value.filter(w => w !== id)
      }, 300)
    }
  }

  function restore(id: string) {
    const win = windows.value.find(w => w.id === id)
    if (win) {
      win.minimized = false
      win.animState = 'entering'
      setTimeout(() => {
        win.animState = 'idle'
      }, 350)
      focus(id)
    }
  }

  function focus(id: string) {
    const win = windows.value.find(w => w.id === id)
    if (win && !win.minimized) {
      win.zIndex = nextZIndex.value++
      windowFocusHistory.value = windowFocusHistory.value.filter(w => w !== id)
      windowFocusHistory.value.push(id)
      currentScreen.value = { x: win.screenX, y: win.screenY }
    }
  }

  function toggle(panelType: PanelType, props?: Record<string, unknown>) {
    const existing = windows.value.find(
      w => w.panelType === panelType
    )
    if (existing) {
      if (existing.minimized) {
        restore(existing.id)
      } else if (activeWindow.value?.id === existing.id) {
        minimize(existing.id)
      } else {
        close(existing.id)
      }
    } else {
      open(panelType, props)
    }
  }

  function closeAll() {
    windows.value.forEach(w => { w.animState = 'exiting' })
    setTimeout(() => {
      windows.value = []
      windowFocusHistory.value = []
    }, 300)
  }

  function minimizeAll() {
    windows.value.forEach(w => {
      w.animState = 'exiting'
      setTimeout(() => {
        w.minimized = true
        w.animState = 'idle'
      }, 300)
    })
    windowFocusHistory.value = []
  }

  function updatePosition(id: string, x: number, y: number) {
    const win = windows.value.find(w => w.id === id)
    if (win) {
      win.x = x
      win.y = y
    }
  }

  function updateSize(id: string, width: number, height: number) {
    const win = windows.value.find(w => w.id === id)
    if (win) {
      win.width = Math.max(320, width)
      win.height = Math.max(240, height)
    }
  }

  const openWindowCount = computed(() =>
    windows.value.filter(w => !w.minimized).length
  )

  const isOpen = (panelType: PanelType) =>
    windows.value.some(w => w.panelType === panelType)

  return {
    windows,
    activeWindow,
    nextZIndex,
    currentScreen,
    open,
    close,
    minimize,
    restore,
    focus,
    toggle,
    closeAll,
    minimizeAll,
    updatePosition,
    updateSize,
    openWindowCount,
    isOpen,
    inlinePanels,
  }
}

export type WindowManager = ReturnType<typeof useWindowManager>
