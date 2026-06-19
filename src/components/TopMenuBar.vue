<template>
  <header ref="menuBarRef" class="top-menu-bar">
    <div class="tmb-left">
      <div class="app-logo" @click="store.currentPanel = 'landing'" role="button" tabindex="0" aria-label="Home" @keydown.enter="store.currentPanel = 'landing'" @keydown.space.prevent="store.currentPanel = 'landing'">
        <span class="logo-brand">CYBERMANJU</span>
        <span class="logo-drive">DRIVE</span>
      </div>

      <div class="tmb-path-wrap">
        <span class="tmb-path">{{ store.currentPath }}</span>
        <Icon v-if="store.isLoading" icon="svg-spinners:3-dots-bounce" width="14" height="14" class="tmb-spinner" />
      </div>

      <nav ref="menuRef" class="menu-items" role="menubar" aria-label="Main Menu">
        <div
          v-for="item in menuStructure"
          :key="item.id"
          class="menu-item"
          role="menuitem"
          :aria-expanded="openMenu === item.id"
          :aria-haspopup="true"
          :tabindex="0"
          @click="toggleMenu(item.id)"
          @mouseenter="hoverMenu(item.id)"
          @keydown="onMenuKeydown($event, item.id)"
        >
          <span class="menu-label">{{ item.label }}</span>
          <div v-if="openMenu === item.id" :ref="(el) => { if (el) dropdownRefs[item.id] = el as HTMLElement }" class="menu-dropdown" role="menu">
            <template v-for="sub in item.children" :key="sub.id">
              <div
                v-if="sub.divider"
                class="menu-divider"
                role="separator"
              />
              <div
                v-else
                class="menu-dropdown-item"
                role="menuitem"
                :tabindex="sub.divider ? -1 : 0"
                @click.stop="executeMenuItem(sub)"
                @keydown.enter="executeMenuItem(sub)"
                @keydown.space.prevent="executeMenuItem(sub)"
              >
                <Icon v-if="sub.icon" :icon="'mdi:' + sub.icon" width="14" height="14" class="mdi-icon" />
                <span class="mdi-label">{{ sub.label }}</span>
                <span v-if="sub.shortcut" class="mdi-shortcut">{{ sub.shortcut }}</span>
                <span v-if="sub.checked" class="mdi-check" aria-label="Enabled">[x]</span>
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
          aria-label="Search files"
          @keyup.enter="handleSearch"
          @input="handleSuggest"
          @focus="showSuggestions = store.searchSuggestions.length > 0"
          @blur="hideSuggestions"
        />
        <span v-if="store.isSearching" class="search-cursor">_</span>
      </div>
      <div v-if="showSuggestions && store.searchSuggestions.length > 0" class="search-suggestions">
        <div
          v-for="(suggestion, i) in store.searchSuggestions"
          :key="i"
          class="suggestion-item"
          @mousedown.prevent="applySuggestion(suggestion)"
        >
          <Icon icon="mdi:magnify" width="10" height="10" class="suggestion-icon" />
          <span>{{ suggestion }}</span>
        </div>
      </div>
    </div>

    <div class="tmb-right">
      <span
        class="tmb-sync"
        :class="{ 'tmb-active': isSyncActive }"
        title="SYNC STATUS"
        aria-label="SYNC STATUS"
        role="button"
        tabindex="0"
        @click="wm.open('sync')"
        @keydown.enter="wm.open('sync')"
        @keydown.space.prevent="wm.open('sync')"
      >{{ isSyncActive ? 'SYNC:' + store.syncProgress?.status.toUpperCase() : 'SYNC:IDLE' }}</span>

      <span class="tmb-div">|</span>

      <span class="tmb-info">{{ store.files.length }} FILES</span>
      <template v-if="store.selectedFile">
        <span class="tmb-div">|</span>
        <span class="tmb-info">SEL: {{ store.selectedFile.name }}</span>
      </template>
      <template v-if="store.selectedFile?.encrypted">
        <span class="tmb-div">|</span>
        <span class="tmb-badge">ENC {{ store.selectedFile.encryptionAlgorithm?.toUpperCase() || '' }}</span>
      </template>
      <template v-if="store.selectedFile?.compressionLayers && store.selectedFile.compressionLayers[0] && store.selectedFile.compressionLayers[0] !== 'none'">
        <span class="tmb-div">|</span>
        <span class="tmb-badge">{{ store.selectedFile.compressionLayers[0].toUpperCase() }}</span>
      </template>
      <template v-if="store.selectedFile?.hashBlake3">
        <span class="tmb-div">|</span>
        <span class="tmb-hash">B3:{{ store.selectedFile.hashBlake3.substring(0, 10) }}..</span>
      </template>

      <div class="tmb-separator" />

      <div class="sys-tray">
        <button
          class="tray-icon"
          :class="{ active: store.encryptionStatus.isEncrypted }"
          @click="wm.open('encryption')"
          :title="'Encryption: ' + (store.encryptionStatus.isEncrypted ? 'ON' : 'OFF')"
          :aria-label="'Encryption: ' + (store.encryptionStatus.isEncrypted ? 'ON' : 'OFF')"
        >
          <Icon icon="mdi:lock-outline" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          :class="{ active: store.compressedFiles.length > 0 }"
          @click="wm.open('compression')"
          :title="'Compression: ' + store.compressedFiles.length + ' files'"
          :aria-label="'Compression: ' + store.compressedFiles.length + ' files'"
        >
          <Icon icon="mdi:package-variant-closed" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          :class="{ active: store.activeAccount }"
          @click="wm.open('accounts')"
          :title="store.activeAccount?.name || 'No account'"
          :aria-label="store.activeAccount?.name || 'No account'"
        >
          <Icon icon="mdi:account-circle-outline" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          :class="{ active: store.matrixRainEnabled }"
          @click="store.matrixRainEnabled = !store.matrixRainEnabled"
          :title="store.matrixRainEnabled ? 'GFX:ON' : 'GFX:OFF'"
          :aria-label="store.matrixRainEnabled ? 'GFX:ON' : 'GFX:OFF'"
        >
          <Icon icon="mdi:lightning-bolt-outline" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          :class="{ active: openWindowCountValue > 0 }"
          @click="store.commandPaletteOpen = true"
          title="Command Palette (Ctrl+K)"
          aria-label="Open Command Palette"
        >
          <Icon icon="mdi:code-brackets" width="14" height="14" />
        </button>

        <button
          class="tray-icon"
          @click="store.showLoginPopup = true"
          :title="store.currentUser ? store.currentUser.username : 'Login'"
          :aria-label="store.currentUser ? 'User: ' + store.currentUser.username : 'Login'"
        >
          <Icon icon="mdi:login" width="14" height="14" />
        </button>
      </div>

      <div class="tmb-separator" />

      <span class="tmb-mode">{{ isWebMode() ? 'WEB' : 'TAURI' }}</span>

      <SystemTray />
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import gsap from 'gsap'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'
import { useGsapAnimation } from '@/composables/useGsapAnimation'
import { isWebMode } from '@/composables/useTauri'
import SystemTray from '@/components/SystemTray.vue'

const anim = useGsapAnimation()
const store = useAppStore()
const wm = useWindowManager()
const openWindowCountValue = computed(() => wm.windows.value.filter(w => !w.minimized).length)

const isSyncActive = computed(() =>
  store.syncProgress !== null && store.syncProgress.status !== 'idle' && store.syncProgress.status !== 'done'
)

const gsapCtx = ref<gsap.Context | null>(null)

const openMenu = ref<string | null>(null)
const menuRef = ref<HTMLElement | null>(null)
const menuBarRef = ref<HTMLElement | null>(null)
const dropdownRefs = ref<Record<string, HTMLElement>>({})
const showSuggestions = ref(false)
let suggestTimeout: ReturnType<typeof setTimeout> | null = null

interface MenuItem {
  id: string
  label?: string
  icon?: string
  shortcut?: string
  checked?: boolean
  divider?: true
  action?: () => void
}

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
]; return m; })

async function toggleMenu(id: string) {
  const previouslyOpen = openMenu.value
  if (previouslyOpen && previouslyOpen !== id && dropdownRefs.value[previouslyOpen]) {
    gsapCtx.value?.add(() => {
      anim.dropdownLeave(dropdownRefs.value[previouslyOpen])
    })
  }
  openMenu.value = openMenu.value === id ? null : id
  if (openMenu.value === id) {
    await nextTick()
    if (dropdownRefs.value[id]) {
      gsapCtx.value?.add(() => {
        anim.dropdownEnter(dropdownRefs.value[id])
      })
    }
  }
}

function hoverMenu(id: string) {
  if (openMenu.value !== null) {
    openMenu.value = id
    if (dropdownRefs.value[id]) {
      gsapCtx.value?.add(() => {
        anim.dropdownEnter(dropdownRefs.value[id])
      })
    }
  }
}

const menuIds = computed(() => menuStructure.value.map(m => m.id))

function onMenuKeydown(e: KeyboardEvent, id: string) {
  const idx = menuIds.value.indexOf(id)
  switch (e.key) {
    case 'ArrowRight':
      e.preventDefault()
      if (idx < menuIds.value.length - 1) {
        toggleMenu(menuIds.value[idx + 1])
      }
      break
    case 'ArrowLeft':
      e.preventDefault()
      if (idx > 0) {
        toggleMenu(menuIds.value[idx - 1])
      }
      break
    case 'ArrowDown':
      e.preventDefault()
      if (openMenu.value !== id) {
        toggleMenu(id)
      }
      break
    case 'Escape':
      e.preventDefault()
      openMenu.value = null
      break
  }
}

function executeMenuItem(item: any) {
  const keys = Object.keys(dropdownRefs.value)
  keys.forEach(k => {
    if (dropdownRefs.value[k]) {
      gsapCtx.value?.add(() => {
        anim.dropdownLeave(dropdownRefs.value[k])
      })
    }
  })
  openMenu.value = null
  item.action?.()
}

function handleSearch() {
  if (store.searchQuery.trim()) {
    store.searchFiles(store.searchQuery)
    wm.open('search')
  }
}

function handleSuggest() {
  if (suggestTimeout) clearTimeout(suggestTimeout)
  suggestTimeout = setTimeout(() => {
    store.fetchSuggestions(store.searchQuery)
    showSuggestions.value = store.searchSuggestions.length > 0
  }, 200)
}

function hideSuggestions() {
  setTimeout(() => {
    showSuggestions.value = false
  }, 150)
}

function applySuggestion(suggestion: string) {
  store.searchQuery = suggestion
  store.searchFiles(suggestion)
  showSuggestions.value = false
  wm.open('search')
}

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    openMenu.value = null
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  gsapCtx.value = gsap.context(() => {
    if (menuBarRef.value) {
      anim.fadeIn(menuBarRef.value, { from: { y: -4, opacity: 0 } })
    }
  })
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  gsapCtx.value?.revert()
})
</script>

<style scoped>
.top-menu-bar {
  display: flex;
  align-items: center;
  height: 32px;
  padding: 0 8px;
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border-bottom: 1px solid var(--border-glass);
  z-index: 100;
  position: relative;
  gap: 8px;
  -webkit-app-region: drag;
  user-select: none;
  isolation: isolate;
  overflow: hidden;
}

/* Velvet texture overlay */
.top-menu-bar::before {
  content: '';
  position: absolute;
  inset: 0;
  z-index: -1;
  background:
    repeating-radial-gradient(circle at 50% 50%, transparent 0, rgba(255,255,255,0.015) 1px, transparent 2px),
    repeating-conic-gradient(rgba(255,255,255,0.008) 0% 25%, transparent 0% 50%) 0 0 / 4px 4px,
    radial-gradient(ellipse at 30% 0%, rgba(0,255,65,0.04) 0%, transparent 50%),
    radial-gradient(ellipse at 70% 100%, rgba(90,240,255,0.03) 0%, transparent 50%);
  pointer-events: none;
  mix-blend-mode: screen;
}

/* Bottom psychedelic glow line */
.top-menu-bar::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg,
    transparent 0%,
    rgba(0, 255, 65, 0.25) 20%,
    rgba(90, 240, 255, 0.2) 40%,
    rgba(179, 136, 255, 0.2) 60%,
    rgba(255, 107, 157, 0.15) 80%,
    transparent 100%);
  background-size: 200% 100%;
  animation: velvet-shimmer 6s ease-in-out infinite;
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
  border-right: 1px solid var(--border-subtle);
  margin-right: 4px;
  outline: none;
}

.app-logo:focus-visible {
  box-shadow: var(--focus-ring);
  border-radius: var(--radius-sm);
}

.app-logo:hover .logo-brand {
  color: var(--accent);
  text-shadow: 0 0 12px rgba(0, 255, 65, 0.4);
}

.logo-brand {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 1.5px;
  color: var(--text-primary);
  transition: all var(--transition-fast);
}

.logo-drive {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}

/* ── Path display ── */
.tmb-path-wrap {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  border-right: 1px solid var(--border-subtle);
  padding-right: 8px;
  margin-right: 4px;
}

.tmb-path {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  font-weight: 600;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
}

.tmb-spinner {
  display: inline-flex;
  vertical-align: middle;
  margin-left: 6px;
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
  border-radius: var(--radius-sm);
  outline: none;
  transition: all var(--duration-fast) cubic-bezier(0.22, 1, 0.36, 1);
}

.menu-item:focus-visible {
  box-shadow: var(--focus-ring);
}

.menu-item:hover {
  background: rgba(0, 255, 65, 0.06);
}

.menu-item:active {
  transform: scale(0.97);
}

.menu-label {
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
  color: var(--text-secondary);
  letter-spacing: 0.2px;
  font-weight: 500;
}

.menu-item:hover .menu-label {
  color: var(--text-primary);
}

.menu-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 220px;
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-lg);
  padding: 4px;
  box-shadow: var(--shadow-elevated), var(--glow-accent);
  z-index: 200;
  will-change: transform, opacity;
}

/* Dropdown velvet texture */
.menu-dropdown::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background:
    repeating-radial-gradient(circle at 50% 50%, transparent 0, rgba(255,255,255,0.01) 1px, transparent 2px),
    radial-gradient(ellipse at 50% 0%, rgba(0,255,65,0.03) 0%, transparent 60%);
  pointer-events: none;
}

.menu-dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast) cubic-bezier(0.22, 1, 0.36, 1);
  outline: none;
  position: relative;
}

.menu-dropdown-item:hover {
  background: rgba(0, 255, 65, 0.08);
  color: var(--text-primary);
  padding-left: 12px;
}

.menu-dropdown-item:focus-visible {
  box-shadow: var(--focus-ring);
}

.mdi-icon {
  width: 18px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.mdi-label {
  flex: 1;
}

.mdi-shortcut {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  margin-left: auto;
}

.mdi-check {
  color: var(--accent);
  font-size: var(--font-size-xs);
}

.menu-divider {
  height: 1px;
  background: var(--border-subtle);
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
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-md);
  padding: 0 8px;
  height: 22px;
  transition: all var(--transition-normal);
}

.search-wrap:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 8px rgba(0, 255, 65, 0.1);
}

.search-prompt {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  margin-right: 4px;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  height: 100%;
  outline: none;
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-cursor {
  color: var(--accent);
  animation: blink 0.8s step-end infinite;
  font-size: var(--font-size-sm);
}

@keyframes blink {
  50% { opacity: 0; }
}

.search-suggestions {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-md);
  max-height: 200px;
  overflow-y: auto;
  z-index: 1000;
}

.suggestion-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  transition: background var(--transition-fast);
}

.suggestion-item:hover {
  background: var(--bg-hover);
}

.suggestion-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.tmb-right {
  display: flex;
  align-items: center;
  gap: 6px;
  -webkit-app-region: no-drag;
  flex-shrink: 0;
}

/* ── Status indicators (merged from StatusBar) ── */
.tmb-sync {
  cursor: pointer;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  padding: 2px 4px;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  outline: none;
  white-space: nowrap;
}

.tmb-sync:hover {
  color: var(--text-primary);
  background: rgba(0, 255, 65, 0.06);
}

.tmb-sync:focus-visible {
  box-shadow: var(--focus-ring);
}

.tmb-active {
  color: var(--accent);
  font-weight: 700;
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.3);
}

.tmb-info {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  white-space: nowrap;
}

.tmb-badge {
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: var(--font-size-xs);
  color: var(--accent);
  border: 1px solid var(--accent);
  padding: 0 4px;
  white-space: nowrap;
}

.tmb-hash {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  white-space: nowrap;
}

.tmb-div {
  color: var(--text-muted);
  opacity: 0.5;
}

.tmb-mode {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  letter-spacing: 0.5px;
  white-space: nowrap;
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
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  background: transparent;
  border: none;
}

.tray-icon:hover {
  color: var(--text-primary);
  background: rgba(0, 255, 65, 0.06);
}

.tray-icon.active {
  color: var(--accent);
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.4);
}

.tmb-separator {
  width: 1px;
  height: 16px;
  background: var(--border-subtle);
  flex-shrink: 0;
}

@keyframes velvet-shimmer {
  0% { background-position: 200% 0; }
  50% { background-position: 0% 0; }
  100% { background-position: -200% 0; }
}

/* ── Mobile responsive ── */
@media (max-width: 900px) {
  .tmb-info,
  .tmb-badge,
  .tmb-hash,
  .tmb-div {
    display: none;
  }
  .tmb-path {
    max-width: 100px;
  }
}

@media (max-width: 600px) {
  .tmb-path-wrap {
    display: none;
  }
  .tmb-sync {
    display: none;
  }
  .tmb-mode {
    display: none;
  }
  .tmb-center {
    max-width: 200px;
  }
}
</style>
