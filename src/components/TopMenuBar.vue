<template>
  <header class="top-menu-bar glass-liquid">
    <div class="tmb-left">
      <div class="app-logo" @click="store.currentPanel = 'landing'">
        <span class="logo-brand">CYBERMANJU</span>
        <span class="logo-drive">DRIVE</span>
      </div>
      <nav class="menu-items" ref="menuRef">
        <div
          v-for="item in menuStructure"
          :key="item.id"
          class="menu-item"
          @click="toggleMenu(item.id)"
          @mouseenter="hoverMenu(item.id)"
        >
          <span class="menu-label">{{ item.label }}</span>
          <div v-if="openMenu === item.id" class="menu-dropdown">
            <template v-for="sub in item.children" :key="sub.id">
              <div
                v-if="sub.divider"
                class="menu-divider"
              />
              <div
                v-else
                class="menu-dropdown-item"
                @click.stop="executeMenuItem(sub)"
              >
                <Icon v-if="sub.icon" :icon="'mdi:' + sub.icon" width="14" height="14" class="mdi-icon" />
                <span class="mdi-label">{{ sub.label }}</span>
                <span v-if="sub.shortcut" class="mdi-shortcut">{{ sub.shortcut }}</span>
                <span v-if="sub.checked" class="mdi-check">[x]</span>
              </div>
            </template>
          </div>
        </div>
      </nav>
    </div>

    <div class="tmb-center">
      <div class="search-wrap" :class="{ searching: store.isSearching }">
        <Icon icon="mdi:magnify" width="11" height="11" class="search-prompt" />
        <input
          v-model="store.searchQuery"
          class="search-input"
          type="text"
          placeholder="TANTIVY_SEARCH..."
          @keyup.enter="handleSearch"
        />
        <span v-if="store.isSearching" class="search-cursor">_</span>
      </div>
    </div>

    <div class="tmb-right">
      <div class="sys-tray">
        <button
          class="tray-icon"
          :class="{ active: store.encryptionStatus.isEncrypted }"
          @click="wm.open('encryption')"
          title="Encryption: {{ store.encryptionStatus.isEncrypted ? 'ON' : 'OFF' }}"
        >
          <Icon icon="mdi:lock-outline" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          :class="{ active: store.compressedFiles.length > 0 }"
          @click="wm.open('compression')"
          title="Compression: {{ store.compressedFiles.length }} files"
        >
          <Icon icon="mdi:package-variant-closed" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          :class="{ active: store.activeAccount }"
          @click="wm.open('accounts')"
          :title="store.activeAccount?.name || 'No account'"
        >
          <Icon icon="mdi:account-circle-outline" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          :class="{ active: store.matrixRainEnabled }"
          @click="store.matrixRainEnabled = !store.matrixRainEnabled"
          title="Toggle background effects"
        >
          <Icon icon="mdi:lightning-bolt-outline" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          :class="{ active: openWindowCountValue > 0 }"
          @click="store.commandPaletteOpen = true"
          title="Command Palette"
        >
          <Icon icon="mdi:code-brackets" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          @click="store.showLoginPopup = true"
          :title="store.currentUser ? store.currentUser.username : 'Login'"
        >
          <Icon icon="mdi:login" width="14" height="14" />
        </button>
      </div>

      <div class="tmb-separator" />

      <div class="clock" @click="openDateInfo">
        <span class="clock-time">{{ timeStr }}</span>
        <span class="clock-date">{{ dateStr }}</span>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'

const store = useAppStore()
const wm = useWindowManager()
const openWindowCountValue = computed(() => wm.windows.value.filter(w => !w.minimized).length)

const timeStr = ref('')
const dateStr = ref('')
let clockTimer: ReturnType<typeof setInterval> | null = null

function updateClock() {
  const now = new Date()
  timeStr.value = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  dateStr.value = now.toLocaleDateString([], { month: 'short', day: 'numeric' })
}

onMounted(() => {
  updateClock()
  clockTimer = setInterval(updateClock, 1000)
})

onUnmounted(() => {
  if (clockTimer) clearInterval(clockTimer)
})

const openMenu = ref<string | null>(null)
const menuRef = ref<HTMLElement | null>(null)

interface MenuItem {
  id: string
  label?: string
  icon?: string
  shortcut?: string
  checked?: boolean
  divider?: true
  action?: () => void
}

const iconifyIcon = (name: string) => `mdi:${name}`

interface MenuGroup {
  id: string
  label: string
  children: MenuItem[]
}

const menuStructure = computed(() => { const m: MenuGroup[] = [
  {
    id: 'file',
    label: 'File',
    children: [
      { id: 'new-folder', label: 'New Folder', icon: 'folder-plus-outline', shortcut: 'Ctrl+N', action: () => { store.createFolderPromptOpen = true } },
      { id: 'upload', label: 'Upload Files', icon: 'upload-outline', action: () => { window.dispatchEvent(new CustomEvent('cybermanju:upload')) } },
      { id: 'div1', divider: true },
      { id: 'open-terminal', label: 'Open Terminal', icon: 'console', action: () => { store.currentPanel = 'landing' } },
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
]; return m; })

function toggleMenu(id: string) {
  openMenu.value = openMenu.value === id ? null : id
}

function hoverMenu(id: string) {
  if (openMenu.value !== null) {
    openMenu.value = id
  }
}

function executeMenuItem(item: any) {
  openMenu.value = null
  item.action?.()
}

function handleSearch() {
  if (store.searchQuery.trim()) {
    store.searchFiles(store.searchQuery)
    wm.open('search')
  }
}

function openDateInfo() {
  wm.open('settings')
}

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    openMenu.value = null
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.top-menu-bar {
  display: flex;
  align-items: center;
  height: 32px;
  padding: 0 8px;
  background: rgba(17, 17, 17, 0.55);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  z-index: 100;
  position: relative;
  gap: 8px;
  -webkit-app-region: drag;
  user-select: none;
}

.tmb-left {
  display: flex;
  align-items: center;
  gap: 4px;
  -webkit-app-region: no-drag;
}

.app-logo {
  display: flex;
  align-items: baseline;
  gap: 3px;
  padding: 0 8px;
  cursor: pointer;
  border-right: 1px solid #222;
  margin-right: 4px;
}

.app-logo:hover .logo-brand {
  color: #00ff41;
}

.logo-brand {
  font-family: 'Courier New', monospace;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 1.5px;
  color: #e0e0e0;
  transition: color 0.15s;
}

.logo-drive {
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 600;
  color: #555;
  letter-spacing: 0.5px;
}

.menu-items {
  display: flex;
  align-items: center;
  gap: 0;
}

.menu-item {
  position: relative;
  padding: 4px 10px;
  cursor: pointer;
  border-radius: 4px;
}

.menu-item:hover {
  background: #1a1a1a;
}

.menu-label {
  font-family: 'Courier New', monospace;
  font-size: 11px;
  color: #aaa;
  letter-spacing: 0.2px;
  font-weight: 500;
}

.menu-item:hover .menu-label {
  color: #e0e0e0;
}

.menu-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 220px;
  background: rgba(24, 24, 24, 0.7);
  backdrop-filter: blur(28px);
  -webkit-backdrop-filter: blur(28px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 4px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
  z-index: 200;
}

.menu-dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  color: #ccc;
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.1s;
}

.menu-dropdown-item:hover {
  background: #222;
  color: #fff;
}

.mdi-icon {
  width: 18px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #666;
}

.mdi-label {
  flex: 1;
}

.mdi-shortcut {
  font-size: 9px;
  color: #555;
  margin-left: auto;
}

.mdi-check {
  color: #00ff41;
  font-size: 9px;
}

.menu-divider {
  height: 1px;
  background: #2a2a2a;
  margin: 4px 6px;
}

.tmb-center {
  flex: 1;
  display: flex;
  justify-content: center;
  max-width: 360px;
  margin: 0 auto;
}

.search-wrap {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 6px;
  padding: 0 8px;
  height: 22px;
  transition: border-color 0.15s;
}

.search-wrap:focus-within {
  border-color: #00ff41;
}

.search-prompt {
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  margin-right: 4px;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  height: 100%;
  outline: none;
}

.search-input::placeholder {
  color: #444;
}

.search-cursor {
  color: #00ff41;
  animation: blink 0.8s step-end infinite;
  font-size: 10px;
}

@keyframes blink {
  50% { opacity: 0; }
}

.tmb-right {
  display: flex;
  align-items: center;
  gap: 6px;
  -webkit-app-region: no-drag;
}

.sys-tray {
  display: flex;
  align-items: center;
  gap: 2px;
}

.tray-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 22px;
  color: #666;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.1s;
  background: transparent;
  border: none;
}

.tray-icon:hover {
  color: #e0e0e0;
  background: #1a1a1a;
}

.tray-icon.active {
  color: #00ff41;
}

.tmb-separator {
  width: 1px;
  height: 16px;
  background: #222;
  flex-shrink: 0;
}

.clock {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  padding: 0 6px;
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.1s;
}

.clock:hover {
  background: #1a1a1a;
}

.clock-time {
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 600;
  color: #ccc;
  line-height: 1.2;
}

.clock-date {
  font-family: 'Courier New', monospace;
  font-size: 8px;
  color: #555;
  line-height: 1.2;
}
</style>
