<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@/composables/useTauri'
import { Icon } from '@iconify/vue'

interface SearchResult {
  title: string
  url: string
  snippet: string
}

const STORAGE_KEYS = {
  BOOKMARKS: 'cybermanju_web_bookmarks',
  HISTORY: 'cybermanju_web_history',
} as const

interface BrowserTab {
  id: string
  title: string
  url: string
  loading: boolean
  history: string[]
  historyIndex: number
}

const currentUrl = ref('')
const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const isLoading = ref(false)
const activeView = ref<'search' | 'page' | 'start'>('start')
const mode = ref<'web' | 'local' | 'hybrid'>('web')

function loadPersisted<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key)
    return raw ? JSON.parse(raw) : fallback
  } catch { return fallback }
}

const tabs = ref<BrowserTab[]>([
  { id: 'tab-0', title: 'New Tab', url: 'about:start', loading: false, history: [], historyIndex: -1 }
])
const activeTabId = ref('tab-0')

const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value))
const pageContent = ref('')
const historyList = ref<{ url: string; title: string; timestamp: number }[]>(loadPersisted(STORAGE_KEYS.HISTORY, []))
const showHistory = ref(false)
const showBookmarks = ref(false)
const bookmarks = ref<{ url: string; title: string }[]>(loadPersisted(STORAGE_KEYS.BOOKMARKS, []))

const theme = ref<'cyber' | 'psych' | 'vintage'>('cyber')
const fontScale = ref(1)

function switchTab(id: string) {
  activeTabId.value = id
  const tab = tabs.value.find(t => t.id === id)
  if (tab) {
    currentUrl.value = tab.url
    activeView.value = tab.url === 'about:start' ? 'start' : 'page'
  }
}

function closeTab(id: string) {
  const idx = tabs.value.findIndex(t => t.id === id)
  if (tabs.value.length > 1) {
    tabs.value = tabs.value.filter(t => t.id !== id)
    if (activeTabId.value === id) {
      const newIdx = Math.min(idx, tabs.value.length - 1)
      activeTabId.value = tabs.value[newIdx].id
      switchTab(activeTabId.value)
    }
  }
}

function newTab() {
  const id = `tab-${Date.now()}`
  tabs.value.push({ id, title: 'New Tab', url: 'about:start', loading: false, history: [], historyIndex: -1 })
  activeTabId.value = id
  activeView.value = 'start'
  currentUrl.value = ''
  searchResults.value = []
}

function goToUrl(url: string) {
  if (!url.trim()) return
  let finalUrl = url.trim()
  if (!finalUrl.startsWith('http://') && !finalUrl.startsWith('https://') && !finalUrl.startsWith('about:')) {
    finalUrl = `https://${finalUrl}`
  }
  currentUrl.value = finalUrl
  const tab = tabs.value.find(t => t.id === activeTabId.value)
  if (tab) {
    if (tab.historyIndex < tab.history.length - 1) {
      tab.history = tab.history.slice(0, tab.historyIndex + 1)
    }
    tab.history.push(finalUrl)
    tab.historyIndex = tab.history.length - 1
    tab.url = finalUrl
    tab.title = finalUrl
    tab.loading = true
  }
  activeView.value = 'page'
  loadPage(finalUrl)
}

function doSearch(query: string) {
  if (!query.trim()) return
  searchQuery.value = query
  isLoading.value = true
  activeView.value = 'search'

  const tab = tabs.value.find(t => t.id === activeTabId.value)
  if (tab) {
    const searchUrl = `search:${query}`
    if (tab.historyIndex < tab.history.length - 1) {
      tab.history = tab.history.slice(0, tab.historyIndex + 1)
    }
    tab.history.push(searchUrl)
    tab.historyIndex = tab.history.length - 1
    tab.title = `${query} - Search`
    tab.url = searchUrl
  }

  if (mode.value === 'local') {
    localSearch(query)
  } else if (mode.value === 'hybrid') {
    hybridSearch(query)
  } else {
    webSearch(query)
  }
}

async function webSearch(query: string) {
  try {
    const results = await invoke<SearchResult[]>('web_search', { query, limit: 20 })
    searchResults.value = results || []
  } catch {
    const resp = await fetch(`https://lite.duckduckgo.com/lite/?q=${encodeURIComponent(query)}`)
    const html = await resp.text()
    const parser = new DOMParser()
    const doc = parser.parseFromString(html, 'text/html')
    const rows = doc.querySelectorAll('table.result')
    searchResults.value = Array.from(rows).map(row => {
      const link = row.querySelector('a.result-link')
      const snippet = row.querySelector('td.result-snippet')
      return {
        title: link?.textContent?.trim() || '',
        url: link?.getAttribute('href') || '',
        snippet: snippet?.textContent?.trim() || '',
      }
    }).filter(r => r.title)
  }
  isLoading.value = false
  addHistory(query, `search:${query}`)
}

async function localSearch(query: string) {
  try {
    const results = await invoke<any[]>('search_files_paginated', { query, limit: 20, offset: 0 })
    if (results) {
      searchResults.value = results.map(r => ({
        title: r.fileName || r.file_name || 'File',
        url: `cybermanju://file/${r.fileId || r.file_id}`,
        snippet: r.snippet || '',
      }))
    }
  } catch {
    searchResults.value = []
  }
  isLoading.value = false
}

async function hybridSearch(query: string) {
  isLoading.value = true
  try {
    const local = await invoke<any[]>('search_files_paginated', { query, limit: 10, offset: 0 })
    if (local && local.length > 0) {
      searchResults.value = local.map(r => ({
        title: r.fileName || r.file_name || 'File',
        url: `cybermanju://file/${r.fileId || r.file_id}`,
        snippet: r.snippet || '',
      }))
      isLoading.value = false
      return
    }
  } catch {}
  await webSearch(query)
}

async function loadPage(url: string) {
  if (url.startsWith('about:')) {
    pageContent.value = ''
    return
  }
  try {
    const html = await invoke<string>('fetch_page', { url })
    pageContent.value = html
    const tab = tabs.value.find(t => t.id === activeTabId.value)
    if (tab) {
      const titleMatch = html.match(/<title>([^<]*)<\/title>/i)
      tab.title = titleMatch ? titleMatch[1] : url
      tab.loading = false
    }
    addHistory(url, tab?.title || url)
  } catch {
    pageContent.value = `<div class="error-page"><span class="error-icon">⚠</span><p>Failed to load ${url}</p></div>`
    const tab = tabs.value.find(t => t.id === activeTabId.value)
    if (tab) tab.loading = false
  }
}

function addHistory(url: string, title: string) {
  historyList.value.unshift({ url, title, timestamp: Date.now() })
  if (historyList.value.length > 100) historyList.value = historyList.value.slice(0, 100)
}

function addBookmark() {
  const tab = activeTab.value
  if (tab && tab.url !== 'about:start') {
    bookmarks.value.push({ url: tab.url, title: tab.title })
  }
}

function removeBookmark(url: string) {
  bookmarks.value = bookmarks.value.filter(b => b.url !== url)
}

function isBookmarked(url: string): boolean {
  return bookmarks.value.some(b => b.url === url)
}

function goBack() {
  const tab = tabs.value.find(t => t.id === activeTabId.value)
  if (tab && tab.historyIndex > 0) {
    tab.historyIndex--
    const url = tab.history[tab.historyIndex]
    currentUrl.value = url
    tab.url = url
    tab.loading = true
    if (url.startsWith('search:')) {
      const q = url.slice(7)
      searchQuery.value = q
      activeView.value = 'search'
      doSearch(q)
    } else {
      activeView.value = 'page'
      loadPage(url)
    }
  }
}

function goForward() {
  const tab = tabs.value.find(t => t.id === activeTabId.value)
  if (tab && tab.historyIndex < tab.history.length - 1) {
    tab.historyIndex++
    const url = tab.history[tab.historyIndex]
    currentUrl.value = url
    tab.url = url
    tab.loading = true
    if (url.startsWith('search:')) {
      const q = url.slice(7)
      searchQuery.value = q
      activeView.value = 'search'
      doSearch(q)
    } else {
      activeView.value = 'page'
      loadPage(url)
    }
  }
}

function refreshPage() {
  if (currentUrl.value) goToUrl(currentUrl.value)
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    const input = e.target as HTMLInputElement
    const val = input.value.trim()
    const isUrl = val.includes('.') || val.startsWith('http://') || val.startsWith('https://')
    if (isUrl) {
      goToUrl(val)
    } else {
      doSearch(val)
    }
  }
}

function persistBookmarks() {
  localStorage.setItem(STORAGE_KEYS.BOOKMARKS, JSON.stringify(bookmarks.value))
}

function persistHistory() {
  localStorage.setItem(STORAGE_KEYS.HISTORY, JSON.stringify(historyList.value.slice(0, 100)))
}

watch(bookmarks, persistBookmarks, { deep: true })
watch(historyList, persistHistory, { deep: true })

function exportBookmarks() {
  const data = JSON.stringify(bookmarks.value, null, 2)
  const blob = new Blob([data], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `cybermanju-bookmarks-${Date.now()}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function importBookmarks() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = async () => {
    const file = input.files?.[0]
    if (!file) return
    try {
      const text = await file.text()
      const parsed = JSON.parse(text)
      if (Array.isArray(parsed)) {
        bookmarks.value = parsed.filter(b => b.url && b.title)
      }
    } catch (e) {
      console.error('Failed to import bookmarks:', e)
    }
  }
  input.click()
}

function clearHistory() {
  historyList.value = []
}

onMounted(() => {
  addHistory('about:start', 'New Tab')
})

const themeStyles = computed(() => {
  switch (theme.value) {
    case 'psych': return {
      bg: 'linear-gradient(135deg, #ff00ff33, #00ffff33, #ffff0033)',
      accent: '#ff00ff',
      font: "var(--font-mono)",
      glow: '0 0 20px rgba(255,0,255,0.5)',
    }
    case 'vintage': return {
      bg: 'linear-gradient(135deg, #2a1a0a, #1a0a00)',
      accent: '#ff9933',
      font: "'Georgia', serif",
      glow: '0 0 15px rgba(255,153,51,0.3)',
    }
    default: return {
      bg: 'linear-gradient(135deg, #0a000d, #100520, #0d0015)',
      accent: '#b388ff',
      font: "var(--font-mono)",
      glow: '0 0 20px rgba(179,136,255,0.4)',
    }
  }
})
</script>

<template>
  <div class="web-browser" :style="{ '--accent': themeStyles.accent, '--bg': themeStyles.bg, '--font': themeStyles.font, '--glow': themeStyles.glow }">
    <div class="browser-titlebar">
      <div class="tab-bar">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="tab-item"
          :class="{ active: tab.id === activeTabId }"
          @click="switchTab(tab.id)"
        >
          <Icon v-if="tab.loading" icon="svg-spinners:ring-resize" width="10" height="10" />
          <span class="tab-title truncate">{{ tab.title }}</span>
          <span class="tab-close" @click.stop="closeTab(tab.id)">×</span>
        </div>
        <div class="tab-new" @click="newTab" title="New Tab">+</div>
      </div>
    </div>

    <div class="browser-toolbar">
      <div class="nav-buttons">
        <button class="tool-btn" @click="goBack" title="Back">◀</button>
        <button class="tool-btn" @click="goForward" title="Forward">▶</button>
        <button class="tool-btn" @click="refreshPage" title="Refresh">⟳</button>
      </div>
      <div class="url-bar">
        <div class="search-mode-selector">
          <button
            v-for="m in (['web', 'local', 'hybrid'] as const)"
            :key="m"
            class="mode-btn"
            :class="{ active: mode === m }"
            @click="mode = m"
          >{{ m === 'web' ? '🌐' : m === 'local' ? '📁' : '⚡' }}</button>
        </div>
        <input
          v-model="currentUrl"
          class="url-input"
          :placeholder="mode === 'local' ? 'Search local files...' : 'Search DuckDuckGo or enter URL...'"
          @keydown="handleKeydown"
          spellcheck="false"
        />
        <button class="tool-btn" @click="addBookmark" :title="isBookmarked(currentUrl) ? 'Remove Bookmark' : 'Add Bookmark'">
          {{ isBookmarked(currentUrl) ? '★' : '☆' }}
        </button>
      </div>
      <div class="view-controls">
        <button class="tool-btn" @click="showHistory = !showHistory" title="History">📋</button>
        <button class="tool-btn" @click="showBookmarks = !showBookmarks" title="Bookmarks">🔖</button>
        <button class="tool-btn" @click="theme = theme === 'cyber' ? 'psych' : theme === 'psych' ? 'vintage' : 'cyber'" title="Theme">
          {{ theme === 'cyber' ? '💜' : theme === 'psych' ? '🌈' : '🕰️' }}
        </button>
      </div>
    </div>

    <div class="browser-content">

      <!-- History Panel -->
      <div v-if="showHistory" class="side-panel">
        <div class="side-panel-header">
          <span>HISTORY</span>
          <div class="side-header-actions">
            <button class="tool-btn" @click.stop="clearHistory" title="Clear History">🗑</button>
            <button class="tool-btn" @click="showHistory = false">×</button>
          </div>
        </div>
        <div v-for="h in historyList.slice(0, 30)" :key="h.timestamp" class="side-item" @click="goToUrl(h.url)">
          <span class="side-item-title truncate">{{ h.title }}</span>
          <span class="side-item-url truncate">{{ h.url }}</span>
        </div>
      </div>

      <!-- Bookmarks Panel -->
      <div v-if="showBookmarks" class="side-panel">
        <div class="side-panel-header">
          <span>BOOKMARKS</span>
          <div class="side-header-actions">
            <button class="tool-btn" @click.stop="exportBookmarks" title="Export Bookmarks">↓</button>
            <button class="tool-btn" @click.stop="importBookmarks" title="Import Bookmarks">↑</button>
            <button class="tool-btn" @click="showBookmarks = false">×</button>
          </div>
        </div>
        <div v-for="bm in bookmarks" :key="bm.url" class="side-item">
          <div class="side-item-content" @click="goToUrl(bm.url)">
            <span class="side-item-title truncate">{{ bm.title }}</span>
            <span class="side-item-url truncate">{{ bm.url }}</span>
          </div>
          <button class="tool-btn side-remove" @click.stop="removeBookmark(bm.url)">×</button>
        </div>
        <div v-if="bookmarks.length === 0" class="side-empty">NO BOOKMARKS</div>
      </div>

      <!-- Start Page -->
      <div v-if="activeView === 'start'" class="start-page">
        <div class="start-logo">🌐</div>
        <h1 class="start-title">CYBERMANJU <span class="accent">WEB</span></h1>
        <p class="start-subtitle">Futuristic Search Engine — DuckDuckGo + Tantivy</p>
        <div class="start-search-box">
          <input
            v-model="searchQuery"
            class="start-input"
            placeholder="Search the web or your files..."
            @keydown="handleKeydown"
            spellcheck="false"
            ref="startInput"
          />
          <button class="start-go-btn" @click="doSearch(searchQuery)">SEARCH</button>
        </div>
        <div class="start-shortcuts">
          <div class="shortcut-grid">
            <div v-for="bm in bookmarks.slice(0, 8)" :key="bm.url" class="shortcut-card" @click="goToUrl(bm.url)">
              <span class="shortcut-icon">🔗</span>
              <span class="shortcut-label truncate">{{ bm.title }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Search Results -->
      <div v-if="activeView === 'search'" class="search-page">
        <div class="search-stats">
          <span class="text-muted">{{ searchResults.length }} RESULTS FOR "<strong>{{ searchQuery }}</strong>"</span>
          <span class="search-mode-badge">{{ mode.toUpperCase() }}</span>
        </div>
        <div v-if="isLoading" class="search-loading">
          <Icon icon="svg-spinners:ring-resize" width="24" height="24" />
          <span class="text-muted">SEARCHING...</span>
        </div>
        <div v-else class="search-results-grid">
          <div
            v-for="(result, i) in searchResults"
            :key="i"
            class="result-block"
            :style="{ '--i': i }"
            @click="goToUrl(result.url)"
          >
            <div class="result-title">{{ result.title }}</div>
            <div class="result-url text-muted">{{ result.url }}</div>
            <div class="result-snippet">{{ result.snippet }}</div>
          </div>
          <div v-if="searchResults.length === 0" class="search-empty">
            <p>NO RESULTS FOUND</p>
            <button class="web-fallback-btn" @click="webSearch(searchQuery)">SEARCH ON DUCKDUCKGO →</button>
          </div>
        </div>
      </div>

      <!-- Page Rendering -->
      <div v-if="activeView === 'page'" class="page-view">
        <div v-if="pageContent" class="page-render">
          <iframe
            v-if="currentUrl.startsWith('http')"
            :src="currentUrl"
            class="page-iframe"
            sandbox="allow-same-origin allow-scripts allow-popups allow-forms"
            loading="lazy"
          />
          <div v-else class="page-text" v-html="pageContent" />
        </div>
        <div v-else class="page-empty">
          <Icon icon="svg-spinners:ring-resize" width="24" height="24" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.web-browser {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: transparent;
  font-family: var(--font-mono);
  color: var(--text-primary);
  overflow: hidden;
}

.browser-titlebar {
  flex-shrink: 0;
  background: var(--bg-glass-light);
  border-bottom: 1px solid var(--border-glass);
}

.tab-bar {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 4px 0;
  overflow-x: auto;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  background: var(--overlay-light);
  border: 1px solid var(--border-glass);
  border-bottom: none;
  border-radius: 6px 6px 0 0;
  font-size: 10px;
  cursor: pointer;
  min-width: 80px;
  max-width: 180px;
  color: var(--text-muted);
  transition: all 0.15s;
}

.tab-item:hover { background: var(--overlay-medium); color: var(--text-secondary); }
.tab-item.active { background: var(--purple-dim); color: var(--accent); border-color: var(--border-glass-hover); }
.tab-title { flex: 1; }
.tab-close { opacity: 0.4; font-size: 12px; font-weight: 700; }
.tab-close:hover { opacity: 1; }
.tab-new { padding: 5px 10px; cursor: pointer; color: var(--text-muted); font-size: 16px; font-weight: 700; }
.tab-new:hover { color: var(--accent); }

.browser-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-glass);
  background: var(--bg-glass);
}

.tool-btn {
  background: transparent;
  border: 1px solid var(--border-glass);
  color: var(--text-secondary);
  padding: 3px 7px;
  font-size: 11px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all 0.1s;
  font-family: inherit;
}

.tool-btn:hover { border-color: var(--border-glass-hover); color: var(--text-primary); background: var(--overlay-light); }

.nav-buttons { display: flex; gap: 2px; }

.url-bar {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 4px;
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-md);
  padding: 2px 6px;
}

.url-bar:focus-within { border-color: var(--accent); box-shadow: var(--glow); }

.search-mode-selector { display: flex; gap: 2px; }

.mode-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  padding: 2px 4px;
  font-size: 12px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all 0.1s;
}

.mode-btn.active { color: var(--accent); background: var(--purple-dim); }
.mode-btn:hover { color: var(--text-secondary); }

.url-input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-family: inherit;
  font-size: 11px;
  padding: 4px 6px;
  outline: none;
}

.url-input::placeholder { color: var(--text-muted); }

.view-controls { display: flex; gap: 2px; }

.browser-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

.side-panel {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 260px;
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border-left: 1px solid var(--border-glass);
  z-index: 10;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.side-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  font-size: 10px;
  font-weight: 700;
  color: var(--accent);
  border-bottom: 1px solid var(--border-glass);
}

.side-header-actions {
  display: flex;
  gap: 2px;
}

.side-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  font-size: 10px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-glass);
}

.side-item:hover { background: var(--purple-dim); }
.side-item-content { flex: 1; min-width: 0; }
.side-item-title { display: block; color: var(--text-secondary); }
.side-item-url { display: block; color: var(--text-muted); font-size: 8px; }
.side-remove { opacity: 0.3; font-size: 10px; padding: 1px 4px; }
.side-remove:hover { opacity: 1; }
.side-empty { padding: 20px; text-align: center; color: var(--text-muted); font-size: 10px; }

.start-page {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 40px 20px;
  text-align: center;
}

.start-logo { font-size: 48px; filter: drop-shadow(0 0 20px var(--accent)); }
.start-title { font-size: 28px; font-weight: 700; letter-spacing: 3px; }
.start-title .accent { color: var(--accent); text-shadow: 0 0 20px var(--accent); }
.start-subtitle { font-size: 10px; color: var(--text-muted); letter-spacing: 2px; }

.start-search-box {
  display: flex;
  gap: 8px;
  width: 100%;
  max-width: 500px;
}

.start-input {
  flex: 1;
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  color: var(--text-primary);
  font-family: inherit;
  font-size: 14px;
  outline: none;
  transition: all 0.2s;
}

.start-input:focus { border-color: var(--accent); box-shadow: var(--glow); }
.start-input::placeholder { color: var(--text-muted); }

.start-go-btn {
  background: var(--accent);
  border: none;
  color: var(--text-inverse);
  font-family: inherit;
  font-weight: 700;
  font-size: 12px;
  padding: 12px 24px;
  border-radius: var(--radius-md);
  cursor: pointer;
  letter-spacing: 2px;
  transition: all 0.2s;
}

.start-go-btn:hover { box-shadow: var(--glow); transform: scale(1.02); }

.start-shortcuts { margin-top: 20px; width: 100%; max-width: 500px; }

.shortcut-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }

.shortcut-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 10px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s;
}

.shortcut-card:hover { border-color: var(--accent); background: var(--purple-dim); transform: translateY(-2px); }
.shortcut-icon { font-size: 20px; }
.shortcut-label { font-size: 9px; color: var(--text-muted); width: 100%; text-align: center; }

.search-page {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.search-stats {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  font-size: 10px;
}

.search-mode-badge {
  padding: 2px 8px;
  background: var(--purple-dim);
  border: 1px solid var(--border-glass-hover);
  border-radius: var(--radius-sm);
  color: var(--accent);
  font-weight: 700;
  font-size: 9px;
}

.search-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px;
}

.search-results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.result-block {
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  -webkit-backdrop-filter: blur(var(--glass-blur-light));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-md);
  padding: 14px;
  cursor: pointer;
  transition: all 0.2s;
  animation: blockIn 0.3s ease-out both;
  animation-delay: calc(var(--i, 0) * 0.05s);
}

.result-block:hover {
  border-color: var(--accent);
  background: var(--purple-dim);
  transform: translateY(-2px);
  box-shadow: var(--glow);
}

.result-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--accent);
  margin-bottom: 4px;
}

.result-url {
  font-size: 9px;
  margin-bottom: 6px;
  word-break: break-all;
}

.result-snippet {
  font-size: 10px;
  color: var(--text-muted);
  line-height: 1.4;
}

.search-empty {
  grid-column: 1 / -1;
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
  font-size: 11px;
}

.web-fallback-btn {
  margin-top: 12px;
  background: transparent;
  border: 1px solid var(--accent);
  color: var(--accent);
  padding: 8px 16px;
  font-family: inherit;
  font-size: 10px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all 0.15s;
}

.web-fallback-btn:hover { background: var(--purple-dim); box-shadow: var(--glow); }

.page-view {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.page-iframe {
  width: 100%;
  height: 100%;
  border: none;
  background: white;
}

.page-text {
  padding: 20px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
  overflow-y: auto;
  width: 100%;
}

.page-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 40px;
}

@keyframes blockIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.text-muted { color: var(--text-muted) !important; }
.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

::-webkit-scrollbar { width: 4px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: var(--radius-sm); }
::-webkit-scrollbar-thumb:hover { background: var(--scrollbar-thumb-hover); }
</style>
