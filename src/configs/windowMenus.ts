import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'
import { applyPreset, randomizeSettings, DEFAULT_SETTINGS } from '@/configs/artMaker'
import type { PanelType } from '@/types'

export interface MenuItem {
  id: string
  label?: string
  icon?: string
  shortcut?: string
  checked?: boolean
  divider?: true
  action?: () => void
}

export interface MenuGroup {
  id: string
  label: string
  children: MenuItem[]
}

const store = useAppStore()
const wm = useWindowManager()

const DEFAULT_MENUS: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'new-folder', label: 'New Folder', icon: 'folder-plus-outline', shortcut: 'Ctrl+N', action: () => { store.createFolderPromptOpen = true } },
      { id: 'upload', label: 'Upload Files', icon: 'upload-outline', action: () => { window.dispatchEvent(new CustomEvent('cybermanju:upload')) } },
      { id: 'div1', divider: true },
      { id: 'open-terminal', label: 'Open Terminal', icon: 'console', action: () => { wm.open('terminal') } },
      { id: 'div2', divider: true },
      { id: 'settings', label: 'Settings', icon: 'cog-outline', shortcut: 'Ctrl+,', action: () => { wm.open('settings') } },
      { id: 'quit', label: 'Quit', icon: 'exit-to-app', action: () => {} },
    ],
  },
  {
    id: 'edit',
    label: 'Edit',
    children: [
      { id: 'cut', label: 'Cut', icon: 'content-cut', shortcut: 'Ctrl+X', action: () => {} },
      { id: 'copy', label: 'Copy', icon: 'content-copy', shortcut: 'Ctrl+C', action: () => {} },
      { id: 'paste', label: 'Paste', icon: 'content-paste', shortcut: 'Ctrl+V', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'select-all', label: 'Select All', icon: 'select-all', shortcut: 'Ctrl+A', action: () => { store.selectedFileIds = store.files.map(f => f.id) } },
      { id: 'deselect', label: 'Deselect', icon: 'select-off', action: () => { store.selectedFileIds = [] } },
    ],
  },
  {
    id: 'view',
    label: 'View',
    children: [
      { id: 'file-browser', label: 'File Browser', icon: 'folder-outline', shortcut: 'Ctrl+1', action: () => { wm.open('files') } },
      { id: 'collections', label: 'Collections', icon: 'bookmark-multiple-outline', action: () => { wm.open('collections') } },
      { id: 'people', label: 'People (Faces)', icon: 'face-man-outline', action: () => { wm.open('faces') } },
      { id: 'map', label: 'Map View', icon: 'map-outline', action: () => { wm.open('map') } },
      { id: 'code', label: 'Code Intelligence', icon: 'code-tags', action: () => { wm.open('code') } },
      { id: 'div1', divider: true },
      { id: 'search', label: 'Search', icon: 'magnify', shortcut: 'Ctrl+F', action: () => { store.searchQuery = ''; store.currentPanel = 'search' } },
      { id: 'storage', label: 'Storage Dashboard', icon: 'harddisk', action: () => { wm.open('storage') } },
      { id: 'sync-panel', label: 'Sync Panel', icon: 'sync', action: () => { wm.open('sync') } },
      { id: 'transfer-panel', label: 'Transfer Manager', icon: 'transfer', action: () => { wm.open('transfer') } },
      { id: 'import-panel', label: 'Import Manager', icon: 'file-import-outline', action: () => { wm.open('import') } },
      { id: 'div2', divider: true },
      { id: 'minimize-all', label: 'Minimize All', icon: 'window-minimize', action: () => wm.minimizeAll() },
      { id: 'close-all', label: 'Close All Windows', icon: 'close-box-multiple-outline', action: () => wm.closeAll() },
    ],
  },
  {
    id: 'tools',
    label: 'Tools',
    children: [
      { id: 'trash', label: 'Trash', icon: 'delete-outline', action: () => { wm.open('trash'); store.fetchTrashItems() } },
      { id: 'activity', label: 'Activity Log', icon: 'history', action: () => { wm.open('activity'); store.fetchAuditLog() } },
      { id: 'favorites', label: 'Favorites', icon: 'star-outline', action: () => { wm.open('favorites') } },
      { id: 'recent', label: 'Recent Files', icon: 'clock-outline', action: () => { wm.open('recent') } },
      { id: 'div1', divider: true },
      { id: 'accounts', label: 'Account Manager', icon: 'account-outline', action: () => { wm.open('accounts') } },
      { id: 'users', label: 'User Management', icon: 'account-group-outline', action: () => { wm.open('users'); store.fetchUsers() } },
      { id: 'div2', divider: true },
      { id: 'command-palette', label: 'Command Palette', icon: 'keyboard', shortcut: 'Ctrl+K', action: () => { store.commandPaletteOpen = true } },
      { id: 'keyboard-shortcuts', label: 'Keyboard Shortcuts', icon: 'keyboard-settings-outline', action: () => { store.showShortcutsHelp = true } },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About Cybermanju Drive', icon: 'information-outline', action: () => {} },
      { id: 'docs', label: 'Documentation', icon: 'file-document-outline', action: () => { window.open('https://github.com/hautlythird211/Cybermanju-Drive', '_blank') } },
      { id: 'div1', divider: true },
      { id: 'matrix', label: 'Toggle Matrix Rain', icon: 'lightning-bolt-outline', checked: store.matrixRainEnabled, action: () => { store.matrixRainEnabled = !store.matrixRainEnabled } },
    ],
  },
]

const TERMINAL_MENUS: MenuGroup[] = [
  {
    id: 'shell',
    label: 'Shell',
    children: [
      { id: 'new-terminal', label: 'New Terminal', icon: 'console', action: () => { wm.open('terminal') } },
      { id: 'div1', divider: true },
      { id: 'close', label: 'Close', icon: 'close', shortcut: 'Ctrl+W', action: () => { if (wm.activeWindow.value) wm.close(wm.activeWindow.value.id) } },
    ],
  },
  {
    id: 'edit',
    label: 'Edit',
    children: [
      { id: 'copy', label: 'Copy', icon: 'content-copy', shortcut: 'Ctrl+Shift+C', action: () => {} },
      { id: 'paste', label: 'Paste', icon: 'content-paste', shortcut: 'Ctrl+Shift+V', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'select-all', label: 'Select All', icon: 'select-all', action: () => {} },
    ],
  },
  {
    id: 'view',
    label: 'View',
    children: [
      { id: 'clear', label: 'Clear Screen', icon: 'eraser', action: () => {} },
      { id: 'find', label: 'Find', icon: 'magnify', shortcut: 'Ctrl+F', action: () => {} },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About Terminal', icon: 'information-outline', action: () => {} },
      { id: 'docs', label: 'Documentation', icon: 'file-document-outline', action: () => { window.open('https://github.com/hautlythird211/Cybermanju-Drive', '_blank') } },
    ],
  },
]

const BOOK_MENUS: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'new', label: 'New Book', icon: 'file-plus-outline', action: () => {} },
      { id: 'open', label: 'Open', icon: 'folder-open-outline', shortcut: 'Ctrl+O', action: () => {} },
      { id: 'save', label: 'Save', icon: 'content-save-outline', shortcut: 'Ctrl+S', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'export', label: 'Export', icon: 'export-variant', action: () => {} },
      { id: 'div2', divider: true },
      { id: 'close', label: 'Close', icon: 'close', shortcut: 'Ctrl+W', action: () => { if (wm.activeWindow.value) wm.close(wm.activeWindow.value.id) } },
    ],
  },
  {
    id: 'edit',
    label: 'Edit',
    children: [
      { id: 'undo', label: 'Undo', icon: 'undo-variant', shortcut: 'Ctrl+Z', action: () => {} },
      { id: 'redo', label: 'Redo', icon: 'redo-variant', shortcut: 'Ctrl+Shift+Z', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'cut', label: 'Cut', icon: 'content-cut', shortcut: 'Ctrl+X', action: () => {} },
      { id: 'copy', label: 'Copy', icon: 'content-copy', shortcut: 'Ctrl+C', action: () => {} },
      { id: 'paste', label: 'Paste', icon: 'content-paste', shortcut: 'Ctrl+V', action: () => {} },
      { id: 'div2', divider: true },
      { id: 'select-all', label: 'Select All', icon: 'select-all', shortcut: 'Ctrl+A', action: () => {} },
    ],
  },
  {
    id: 'view',
    label: 'View',
    children: [
      { id: 'zoom-in', label: 'Zoom In', icon: 'magnify-plus-outline', shortcut: 'Ctrl+=', action: () => {} },
      { id: 'zoom-out', label: 'Zoom Out', icon: 'magnify-minus-outline', shortcut: 'Ctrl+-', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'fullscreen', label: 'Fullscreen', icon: 'fullscreen', shortcut: 'F11', action: () => {} },
    ],
  },
  {
    id: 'tools',
    label: 'Tools',
    children: [
      { id: 'word-count', label: 'Word Count', icon: 'counter', action: () => {} },
      { id: 'spell-check', label: 'Spell Check', icon: 'spellcheck', action: () => {} },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About Book Writer', icon: 'information-outline', action: () => {} },
    ],
  },
]

const BROWSER_MENUS: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'new-tab', label: 'New Tab', icon: 'tab-plus', shortcut: 'Ctrl+T', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'close', label: 'Close', icon: 'close', shortcut: 'Ctrl+W', action: () => { if (wm.activeWindow.value) wm.close(wm.activeWindow.value.id) } },
    ],
  },
  {
    id: 'edit',
    label: 'Edit',
    children: [
      { id: 'cut', label: 'Cut', icon: 'content-cut', shortcut: 'Ctrl+X', action: () => {} },
      { id: 'copy', label: 'Copy', icon: 'content-copy', shortcut: 'Ctrl+C', action: () => {} },
      { id: 'paste', label: 'Paste', icon: 'content-paste', shortcut: 'Ctrl+V', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'select-all', label: 'Select All', icon: 'select-all', shortcut: 'Ctrl+A', action: () => {} },
      { id: 'find', label: 'Find', icon: 'magnify', shortcut: 'Ctrl+F', action: () => {} },
    ],
  },
  {
    id: 'view',
    label: 'View',
    children: [
      { id: 'reload', label: 'Reload', icon: 'reload', shortcut: 'Ctrl+R', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'zoom-in', label: 'Zoom In', icon: 'magnify-plus-outline', shortcut: 'Ctrl+=', action: () => {} },
      { id: 'zoom-out', label: 'Zoom Out', icon: 'magnify-minus-outline', shortcut: 'Ctrl+-', action: () => {} },
      { id: 'fullscreen', label: 'Fullscreen', icon: 'fullscreen', shortcut: 'F11', action: () => {} },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About Browser', icon: 'information-outline', action: () => {} },
    ],
  },
]

const NOTES_MENUS: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'new-note', label: 'New Note', icon: 'file-plus-outline', shortcut: 'Ctrl+N', action: () => {} },
      { id: 'save', label: 'Save', icon: 'content-save-outline', shortcut: 'Ctrl+S', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'close', label: 'Close', icon: 'close', shortcut: 'Ctrl+W', action: () => { if (wm.activeWindow.value) wm.close(wm.activeWindow.value.id) } },
    ],
  },
  {
    id: 'edit',
    label: 'Edit',
    children: [
      { id: 'cut', label: 'Cut', icon: 'content-cut', shortcut: 'Ctrl+X', action: () => {} },
      { id: 'copy', label: 'Copy', icon: 'content-copy', shortcut: 'Ctrl+C', action: () => {} },
      { id: 'paste', label: 'Paste', icon: 'content-paste', shortcut: 'Ctrl+V', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'select-all', label: 'Select All', icon: 'select-all', shortcut: 'Ctrl+A', action: () => {} },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About Notes', icon: 'information-outline', action: () => {} },
    ],
  },
]

const SETTINGS_MENUS: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'close', label: 'Close', icon: 'close', shortcut: 'Ctrl+W', action: () => { if (wm.activeWindow.value) wm.close(wm.activeWindow.value.id) } },
    ],
  },
  {
    id: 'edit',
    label: 'Edit',
    children: [
      { id: 'copy', label: 'Copy', icon: 'content-copy', shortcut: 'Ctrl+C', action: () => {} },
      { id: 'paste', label: 'Paste', icon: 'content-paste', shortcut: 'Ctrl+V', action: () => {} },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About Settings', icon: 'information-outline', action: () => {} },
    ],
  },
]

const FILES_MENUS: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'new-folder', label: 'New Folder', icon: 'folder-plus-outline', shortcut: 'Ctrl+N', action: () => { store.createFolderPromptOpen = true } },
      { id: 'upload', label: 'Upload Files', icon: 'upload-outline', action: () => { window.dispatchEvent(new CustomEvent('cybermanju:upload')) } },
      { id: 'div1', divider: true },
      { id: 'rename', label: 'Rename', icon: 'rename-box', action: () => {} },
      { id: 'delete', label: 'Move to Trash', icon: 'delete-outline', shortcut: 'Del', action: () => { if (store.selectedFileId) { store.deleteFile(store.selectedFileId) } } },
      { id: 'div2', divider: true },
      { id: 'open-terminal', label: 'Open Terminal', icon: 'console', action: () => { wm.open('terminal') } },
      { id: 'div3', divider: true },
      { id: 'close', label: 'Close', icon: 'close', shortcut: 'Ctrl+W', action: () => { if (wm.activeWindow.value) wm.close(wm.activeWindow.value.id) } },
    ],
  },
  {
    id: 'edit',
    label: 'Edit',
    children: [
      { id: 'cut', label: 'Cut', icon: 'content-cut', shortcut: 'Ctrl+X', action: () => {} },
      { id: 'copy', label: 'Copy File', icon: 'content-copy', shortcut: 'Ctrl+C', action: () => {} },
      { id: 'paste', label: 'Paste File', icon: 'content-paste', shortcut: 'Ctrl+V', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'select-all', label: 'Select All', icon: 'select-all', shortcut: 'Ctrl+A', action: () => { store.selectedFileIds = store.files.map(f => f.id) } },
      { id: 'deselect', label: 'Deselect', icon: 'select-off', action: () => { store.selectedFileIds = [] } },
    ],
  },
  {
    id: 'view',
    label: 'View',
    children: [
      { id: 'refresh', label: 'Refresh', icon: 'refresh', shortcut: 'F5', action: () => { store.fetchFiles() } },
      { id: 'div1', divider: true },
      { id: 'sort-name', label: 'Sort by Name', icon: 'sort-alphabetical-ascending', action: () => {} },
      { id: 'sort-date', label: 'Sort by Date', icon: 'sort-calendar-ascending', action: () => {} },
      { id: 'sort-size', label: 'Sort by Size', icon: 'sort-numeric-ascending', action: () => {} },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About File Browser', icon: 'information-outline', action: () => {} },
    ],
  },
]

const MAP_MENUS: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'close', label: 'Close', icon: 'close', shortcut: 'Ctrl+W', action: () => { if (wm.activeWindow.value) wm.close(wm.activeWindow.value.id) } },
    ],
  },
  {
    id: 'view',
    label: 'View',
    children: [
      { id: 'zoom-in', label: 'Zoom In', icon: 'magnify-plus-outline', action: () => {} },
      { id: 'zoom-out', label: 'Zoom Out', icon: 'magnify-minus-outline', action: () => {} },
      { id: 'div1', divider: true },
      { id: 'satellite', label: 'Satellite View', icon: 'satellite-variant', action: () => {} },
      { id: 'terrain', label: 'Terrain View', icon: 'terrain', action: () => {} },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About Map View', icon: 'information-outline', action: () => {} },
    ],
  },
]

const ART_MAKER_MENUS: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'close', label: 'Close', icon: 'close', shortcut: 'Ctrl+W', action: () => { if (wm.activeWindow.value) wm.close(wm.activeWindow.value.id) } },
    ],
  },
  {
    id: 'presets',
    label: 'Presets',
    children: [
      { id: 'psyc', label: 'Psychedelic', icon: 'palette-swatch-outline', action: () => { store.artSettings = applyPreset(store.artSettings, 'psychedelic') } },
      { id: 'dark', label: 'Dark Ambient', icon: 'weather-night', action: () => { store.artSettings = applyPreset(store.artSettings, 'darkAmbient') } },
      { id: 'matrix', label: 'Matrix Mode', icon: 'lightning-bolt-outline', action: () => { store.artSettings = applyPreset(store.artSettings, 'matrixMode') } },
      { id: 'glitch', label: 'Glitch Core', icon: 'lightning-bolt', action: () => { store.artSettings = applyPreset(store.artSettings, 'glitchCore') } },
      { id: 'heaven', label: 'Heavenly', icon: 'weather-partly-cloudy', action: () => { store.artSettings = applyPreset(store.artSettings, 'heavenly') } },
      { id: 'neuralDream', label: 'Neural Dream', icon: 'brain', action: () => { store.artSettings = applyPreset(store.artSettings, 'neuralDream') } },
      { id: 'cyberpunk', label: 'Cyberpunk', icon: 'hexagon-multiple-outline', action: () => { store.artSettings = applyPreset(store.artSettings, 'cyberpunk') } },
      { id: 'auroraBorealis', label: 'Aurora Borealis', icon: 'weather-night', action: () => { store.artSettings = applyPreset(store.artSettings, 'auroraBorealis') } },
      { id: 'randomize', label: 'Randomize', icon: 'shuffle-variant', action: () => { store.artSettings = randomizeSettings(store.artSettings) } },
      { id: 'reset', label: 'Reset Defaults', icon: 'restore', action: () => { store.artSettings = { ...DEFAULT_SETTINGS, layers: { ...DEFAULT_SETTINGS.layers } } } },
    ],
  },
  {
    id: 'view',
    label: 'View',
    children: [
      { id: 'toggle-matrix', label: 'Toggle Matrix Rain', icon: 'lightning-bolt-outline', checked: store.matrixRainEnabled, action: () => { store.matrixRainEnabled = !store.matrixRainEnabled } },
    ],
  },
  {
    id: 'help',
    label: 'Help',
    children: [
      { id: 'about', label: 'About Art Maker', icon: 'information-outline', action: () => {} },
    ],
  },
]

export const PANEL_MENUS: Partial<Record<PanelType, MenuGroup[]>> = {
  files: FILES_MENUS,
  terminal: TERMINAL_MENUS,
  book: BOOK_MENUS,
  browser: BROWSER_MENUS,
  notes: NOTES_MENUS,
  settings: SETTINGS_MENUS,
  map: MAP_MENUS,
  'art-maker': ART_MAKER_MENUS,
}

export function getMenusForPanel(panelType: PanelType | null): MenuGroup[] {
  if (panelType && PANEL_MENUS[panelType]) {
    return PANEL_MENUS[panelType]!
  }
  return DEFAULT_MENUS
}
