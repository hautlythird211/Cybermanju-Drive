import { ref, computed, markRaw, type Component } from 'vue'
import type { PanelType } from '@/types'
import { MODULE_METADATA } from '@/types'
import FileExplorer from '@/components/FileExplorer.vue'
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
import WebBrowserPanel from '@/components/WebBrowserPanel.vue'
import BookWriter from '@/components/BookWriter.vue'
import NotesPanel from '@/components/NotesPanel.vue'
import PluginCreator from '@/components/PluginCreator.vue'
import ArtMaker from '@/components/ArtMaker.vue'

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
  maximized: boolean
  previousBounds: { x: number; y: number; width: number; height: number }
  zIndex: number
  component: Component | null
  props?: Record<string, unknown>
  screenX: number
  screenY: number
  tileSlot: number
  animState: 'idle' | 'entering' | 'exiting'
}

export interface TileRect {
  x: number
  y: number
  width: number
  height: number
}



const inlinePanels: PanelType[] = [
  'search', 'trash', 'activity', 'favorites', 'recent',
  'loose-groups', 'style', 'history'
]

const panelComponentMap: Record<string, Component> = {
  files: FileExplorer,
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
  browser: WebBrowserPanel,
  book: BookWriter,
  notes: NotesPanel,
  plugins: PluginCreator,
  'art-maker': ArtMaker,
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
const currentScreen = ref({ x: 0, y: 0 })
const lastContainerSize = ref({ width: 1200, height: 800 })

const MAX_Z_INDEX = 9999

export function useWindowManager() {
  const activeWindow = computed(() => {
    if (windowFocusHistory.value.length === 0) return null
    const id = windowFocusHistory.value[windowFocusHistory.value.length - 1]
    return windows.value.find(w => w.id === id) || null
  })

  function allocateTile(screenX: number, screenY: number): number {
    const taken = windows.value
      .filter(w => w.screenX === screenX && w.screenY === screenY && w.animState !== 'exiting')
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

  function getTileRect(
    screenX: number, screenY: number, slot: number,
    containerWidth: number, containerHeight: number,
    windowCount: number
  ): TileRect {
    if (windowCount <= 1) {
      return { x: 0, y: 0, width: containerWidth, height: containerHeight }
    }
    if (windowCount === 2) {
      const w = containerWidth / 2
      return { x: slot * w, y: 0, width: w, height: containerHeight }
    }
    if (windowCount === 3) {
      const w = containerWidth / 3
      return { x: slot * w, y: 0, width: w, height: containerHeight }
    }
    const halfW = containerWidth / 2
    const halfH = containerHeight / 2
    const col = slot % 2
    const row = Math.floor(slot / 2)
    return { x: col * halfW, y: row * halfH, width: halfW, height: halfH }
  }

  function getWindowsForScreen(sx: number, sy: number): WindowState[] {
    return windows.value.filter(w => w.screenX === sx && w.screenY === sy && !w.minimized)
  }

  function getAllScreens(): Array<{ x: number; y: number; windows: WindowState[] }> {
    const screenMap = new Map<string, { x: number; y: number; windows: WindowState[] }>()
    for (const w of windows.value.filter(w => !w.minimized)) {
      const key = `${w.screenX},${w.screenY}`
      if (!screenMap.has(key)) {
        screenMap.set(key, { x: w.screenX, y: w.screenY, windows: [] })
      }
      screenMap.get(key)!.windows.push(w)
    }
    const screens = Array.from(screenMap.values())
    screens.sort((a, b) => a.x - b.x || a.y - b.y)
    if (screens.length === 0) {
      screens.push({ x: 0, y: 0, windows: [] })
    }
    return screens
  }

  function getCurrentScreenWindows(): WindowState[] {
    return getWindowsForScreen(currentScreen.value.x, currentScreen.value.y)
  }

  function open(panelType: PanelType, props?: Record<string, unknown>) {
    const existing = windows.value.find(
      w => w.panelType === panelType && w.animState !== 'exiting'
    )
    if (existing) {
      if (existing.minimized) {
        restore(existing.id)
      } else {
        focus(existing.id)
      }
      return existing.id
    }

    const meta = MODULE_METADATA[panelType] || { label: panelType.toUpperCase(), icon: '[*]' }
    const id = `win-${++windowCounter}`
    const comp = getComponent(panelType)

    const screen = findNextScreen()
    const tileSlot = allocateTile(screen.x, screen.y)

    if (tileSlot < 0) return null

    const resolvedProps = { ...(props || {}) }
    if (inlinePanels.includes(panelType)) {
      resolvedProps.panelType = panelType
    }

    const countOnScreen = getWindowsForScreen(screen.x, screen.y).length + 1
    const tile = getTileRect(screen.x, screen.y, tileSlot, lastContainerSize.value.width, lastContainerSize.value.height, countOnScreen)

    const win: WindowState = {
      id,
      panelType,
      title: meta.label,
      icon: meta.icon,
      x: tile.x,
      y: tile.y,
      width: tile.width,
      height: tile.height,
      minimized: false,
      maximized: false,
      previousBounds: { x: tile.x, y: tile.y, width: tile.width, height: tile.height },
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
    const idx = windows.value.findIndex(w => w.id === id)
    if (idx === -1) return
    const win = windows.value[idx]
    win.animState = 'exiting'
    windowFocusHistory.value = windowFocusHistory.value.filter(w => w !== id)
    setTimeout(() => {
      windows.value = windows.value.filter(w => w.id !== id)
    }, 300)
  }

  function minimize(id: string) {
    const win = windows.value.find(w => w.id === id)
    if (win) {
      win.animState = 'exiting'
      windowFocusHistory.value = windowFocusHistory.value.filter(w => w !== id)
      setTimeout(() => {
        win.minimized = true
        win.animState = 'idle'
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
      if (nextZIndex.value > MAX_Z_INDEX) {
        const minZ = Math.min(...windows.value.map(w => w.zIndex))
        const shift = minZ - 1
        windows.value.forEach(w => { w.zIndex -= shift })
        nextZIndex.value -= shift
      }
      win.zIndex = nextZIndex.value++
      windowFocusHistory.value = windowFocusHistory.value.filter(w => w !== id)
      windowFocusHistory.value.push(id)
      currentScreen.value = { x: win.screenX, y: win.screenY }
    }
  }

  function toggle(panelType: PanelType, props?: Record<string, unknown>) {
    const existing = windows.value.find(
      w => w.panelType === panelType && w.animState !== 'exiting'
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
    windowFocusHistory.value = []
    setTimeout(() => {
      windows.value = []
    }, 300)
  }

  function minimizeAll() {
    windows.value.forEach(w => {
      w.animState = 'exiting'
      windowFocusHistory.value = []
      setTimeout(() => {
        w.minimized = true
        w.animState = 'idle'
      }, 300)
    })
  }

  function maximize(id: string) {
    const win = windows.value.find(w => w.id === id)
    if (!win) return
    if (win.maximized) {
      win.x = win.previousBounds.x
      win.y = win.previousBounds.y
      win.width = win.previousBounds.width
      win.height = win.previousBounds.height
      win.maximized = false
    } else {
      win.previousBounds = { x: win.x, y: win.y, width: win.width, height: win.height }
      win.x = 0
      win.y = 0
      win.width = window.innerWidth
      win.height = window.innerHeight
      win.maximized = true
    }
  }

  function updatePosition(id: string, x: number, y: number) {
    const win = windows.value.find(w => w.id === id)
    if (win && !win.maximized) {
      win.x = x
      win.y = y
    }
  }

  function updateSize(id: string, width: number, height: number) {
    const win = windows.value.find(w => w.id === id)
    if (win && !win.maximized) {
      win.width = Math.max(320, width)
      win.height = Math.max(240, height)
    }
  }

  function updateTilePosition(id: string, containerWidth: number, containerHeight: number) {
    const win = windows.value.find(w => w.id === id)
    if (!win || win.maximized) return
    const tile = getTileRect(win.screenX, win.screenY, win.tileSlot, containerWidth, containerHeight)
    win.x = tile.x
    win.y = tile.y
    win.width = tile.width
    win.height = tile.height
  }

  function retileAll(containerWidth: number, containerHeight: number) {
    lastContainerSize.value = { width: containerWidth, height: containerHeight }
    for (const win of windows.value) {
      if (win.minimized || win.maximized || win.animState === 'exiting') continue
      const tile = getTileRect(win.screenX, win.screenY, win.tileSlot, containerWidth, containerHeight)
      win.x = tile.x
      win.y = tile.y
      win.width = tile.width
      win.height = tile.height
    }
  }

  function arrangeWindows() {
    const { width, height } = lastContainerSize.value
    retileAll(width, height)
  }

  const openWindowCount = computed(() =>
    windows.value.filter(w => !w.minimized && w.animState !== 'exiting').length
  )

  const isOpen = (panelType: PanelType) =>
    windows.value.some(w => w.panelType === panelType && w.animState !== 'exiting')

  return {
    windows,
    activeWindow,
    nextZIndex,
    currentScreen,
    open,
    close,
    minimize,
    maximize,
    restore,
    focus,
    toggle,
    closeAll,
    minimizeAll,
    updatePosition,
    updateSize,
    updateTilePosition,
    retileAll,
    arrangeWindows,
    getTileRect,
    getWindowsForScreen,
    getAllScreens,
    getCurrentScreenWindows,
    openWindowCount,
    isOpen,
    inlinePanels,
    TILE_GAP,
  }
}

export type WindowManager = ReturnType<typeof useWindowManager>
