<template>
  <div class="book-writer">
    <!-- Sidebar: Chapter List -->
    <div class="book-sidebar">
      <div class="book-sidebar-header">
        <span class="book-logo">{{ bookEmoji }}</span>
        <span class="book-title">{{ bookTitle || 'UNTITLED BOOK' }}</span>
        <button class="book-action-btn" @click="addChapter" title="New Chapter">+</button>
      </div>
      <div class="chapter-list">
        <div
          v-for="(ch, i) in chapters"
          :key="i"
          class="chapter-item"
          :class="{ active: currentChapter === i }"
          @click="selectChapter(i)"
        >
          <span class="ch-num">{{ i + 1 }}</span>
          <span class="ch-title truncate">{{ ch.title || 'Untitled Chapter' }}</span>
          <span class="ch-words">{{ wordCount(ch.content) }}w</span>
        </div>
        <div v-if="chapters.length === 0" class="empty-chapters">
          <p>No chapters yet</p>
          <button class="book-btn" @click="addChapter">+ Add Chapter</button>
        </div>
      </div>
      <div class="book-stats">
        <div class="stat"><span class="stat-label">CHAPTERS</span><span class="stat-value">{{ chapters.length }}</span></div>
        <div class="stat"><span class="stat-label">TOTAL WORDS</span><span class="stat-value">{{ totalWords }}</span></div>
      </div>
    </div>

    <!-- Main: Editor -->
    <div class="book-editor">
      <div class="editor-toolbar">
        <input
          v-model="bookTitle"
          class="book-title-input"
          placeholder="Book Title..."
          @blur="saveBook"
        />
        <div class="toolbar-actions">
          <button class="tool-btn" @click="copyBookText" title="Copy all text">📋</button>
          <button class="tool-btn" @click="exportBook" title="Export as text">💾</button>
          <button class="tool-btn" @click="importFromClipboard" title="Paste from clipboard">📥</button>
        </div>
      </div>

      <div v-if="chapters.length > 0 && currentChapter < chapters.length" class="chapter-editor">
        <input
          v-model="chapters[currentChapter].title"
          class="chapter-title-input"
          placeholder="Chapter Title..."
          @blur="saveBook"
        />
        <div class="chapter-meta">
          <span class="ch-meta-info">{{ wordCount(chapters[currentChapter].content) }} words</span>
          <span class="ch-meta-info">{{ sentenceCount(chapters[currentChapter].content) }} sentences</span>
        </div>
        <textarea
          v-model="chapters[currentChapter].content"
          class="chapter-content"
          placeholder="Write your chapter here..."
          @input="onContentChange"
          @keydown.tab.prevent="insertTab"
        ></textarea>
      </div>
      <div v-else class="no-chapter-selected">
        <p class="empty-msg">{{ chapters.length === 0 ? 'Create your first chapter to begin writing' : 'Select a chapter from the sidebar' }}</p>
        <button v-if="chapters.length === 0" class="book-btn" @click="addChapter">+ New Chapter</button>
      </div>
    </div>

    <!-- Panel: Chapter Navigation (bottom) -->
    <div class="chapter-nav" v-if="chapters.length > 1">
      <button class="nav-btn" @click="prevChapter" :disabled="currentChapter === 0">← Previous</button>
      <span class="nav-info">{{ currentChapter + 1 }} / {{ chapters.length }}</span>
      <button class="nav-btn" @click="nextChapter" :disabled="currentChapter === chapters.length - 1">Next →</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@/composables/useTauri'

const bookEmoji = '📖'

interface Chapter {
  title: string
  content: string
}

const bookTitle = ref('')
const chapters = ref<Chapter[]>([])
const currentChapter = ref(0)
const saveDebounce = ref<ReturnType<typeof setTimeout> | null>(null)

const totalWords = computed(() =>
  chapters.value.reduce((sum, ch) => sum + wordCount(ch.content), 0)
)

function wordCount(text: string): number {
  if (!text || !text.trim()) return 0
  return text.trim().split(/\s+/).length
}

function sentenceCount(text: string): number {
  if (!text || !text.trim()) return 0
  return text.split(/[.!?]+/).filter(s => s.trim()).length
}

function selectChapter(index: number) {
  if (index >= 0 && index < chapters.value.length) {
    currentChapter.value = index
  }
}

function addChapter() {
  chapters.value.push({ title: '', content: '' })
  currentChapter.value = chapters.value.length - 1
  saveBook()
}

function nextChapter() {
  if (currentChapter.value < chapters.value.length - 1) {
    currentChapter.value++
  }
}

function prevChapter() {
  if (currentChapter.value > 0) {
    currentChapter.value--
  }
}

function insertTab(e: KeyboardEvent) {
  const textarea = e.target as HTMLTextAreaElement
  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const content = chapters.value[currentChapter.value].content
  chapters.value[currentChapter.value].content =
    content.substring(0, start) + '  ' + content.substring(end)
  setTimeout(() => {
    textarea.selectionStart = textarea.selectionEnd = start + 2
  }, 0)
}

async function copyBookText() {
  const fullText = chapters.value
    .map((ch, i) => `## ${i + 1}. ${ch.title || 'Untitled'}\n\n${ch.content}`)
    .join('\n\n---\n\n')
  try {
    await navigator.clipboard.writeText(fullText)
  } catch {
    /* fallback */
  }
}

function exportBook() {
  const fullText = chapters.value
    .map((ch, i) => `## Chapter ${i + 1}: ${ch.title || 'Untitled'}\n\n${ch.content}`)
    .join('\n\n---\n\n')
  const blob = new Blob([fullText], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${bookTitle.value || 'book'}.txt`
  a.click()
  URL.revokeObjectURL(url)
}

async function importFromClipboard() {
  try {
    const text = await navigator.clipboard.readText()
    if (text && chapters.value.length > 0) {
      chapters.value[currentChapter.value].content += text
      saveBook()
    }
  } catch {
    /* clipboard access denied */
  }
}

function saveBook() {
  if (saveDebounce.value) clearTimeout(saveDebounce.value)
  saveDebounce.value = setTimeout(() => {
    const data = { title: bookTitle.value, chapters: chapters.value }
    try { localStorage.setItem('cybermanju_book', JSON.stringify(data)) } catch {}
  }, 500)
}

function loadBook() {
  try {
    const raw = localStorage.getItem('cybermanju_book')
    if (raw) {
      const data = JSON.parse(raw)
      bookTitle.value = data.title || ''
      chapters.value = data.chapters || []
    }
  } catch {}
}

function onContentChange() {
  saveBook()
}

onMounted(loadBook)
</script>

<style scoped>
.book-writer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0a0a0a;
  color: #e0e0e0;
  font-family: 'Georgia', 'Times New Roman', serif;
}

.book-sidebar {
  width: 220px;
  background: #111;
  border-right: 1px solid #2a2a2a;
  display: flex;
  flex-direction: column;
  position: absolute;
  left: 0;
  top: 0;
  bottom: 32px;
  z-index: 2;
}

.book-sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border-bottom: 1px solid #2a2a2a;
  background: #1a1a1a;
}

.book-logo { font-size: 16px; }

.book-title {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #ffd700;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.book-action-btn {
  background: transparent;
  border: 1px solid #333;
  color: #ffd700;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
}

.book-action-btn:hover {
  border-color: #ffd700;
  background: rgba(255, 215, 0, 0.1);
}

.chapter-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.chapter-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.1s;
  border: 1px solid transparent;
}

.chapter-item:hover {
  background: #1a1a1a;
  border-color: #2a2a2a;
}

.chapter-item.active {
  background: rgba(255, 215, 0, 0.05);
  border-color: rgba(255, 215, 0, 0.2);
}

.ch-num {
  font-size: 8px;
  font-weight: 700;
  color: #555;
  width: 16px;
  text-align: center;
  font-family: 'Courier New', monospace;
}

.ch-title {
  flex: 1;
  font-size: 11px;
  color: #ccc;
}

.ch-words {
  font-size: 8px;
  color: #555;
  font-family: 'Courier New', monospace;
}

.empty-chapters {
  text-align: center;
  padding: 20px 12px;
  color: #555;
  font-size: 10px;
}

.book-btn {
  background: transparent;
  border: 1px solid #333;
  color: #ffd700;
  padding: 6px 12px;
  font-size: 10px;
  font-family: 'Courier New', monospace;
  font-weight: 700;
  cursor: pointer;
  border-radius: 4px;
  margin-top: 8px;
  transition: all 0.1s;
}

.book-btn:hover {
  border-color: #ffd700;
  background: rgba(255, 215, 0, 0.1);
}

.book-stats {
  padding: 10px 12px;
  border-top: 1px solid #2a2a2a;
  background: #111;
}

.stat {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2px 0;
}

.stat-label {
  font-size: 8px;
  font-weight: 700;
  color: #555;
  letter-spacing: 0.5px;
  font-family: 'Courier New', monospace;
}

.stat-value {
  font-size: 10px;
  font-weight: 700;
  color: #ffd700;
  font-family: 'Courier New', monospace;
}

.book-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  margin-left: 220px;
  position: relative;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid #2a2a2a;
  background: #111;
}

.book-title-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #ffd700;
  font-size: 14px;
  font-weight: 700;
  font-family: 'Georgia', serif;
  outline: none;
  padding: 4px 0;
  letter-spacing: 0.5px;
}

.book-title-input::placeholder { color: #555; }

.toolbar-actions {
  display: flex;
  gap: 4px;
}

.tool-btn {
  background: transparent;
  border: 1px solid #333;
  color: #aaa;
  width: 28px;
  height: 28px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
}

.tool-btn:hover {
  border-color: #555;
  color: #e0e0e0;
  background: #1a1a1a;
}

.chapter-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px 24px;
}

.chapter-title-input {
  background: transparent;
  border: none;
  color: #e0e0e0;
  font-size: 20px;
  font-weight: 700;
  font-family: 'Georgia', serif;
  outline: none;
  padding: 4px 0;
  margin-bottom: 4px;
  border-bottom: 1px solid #2a2a2a;
}

.chapter-title-input::placeholder { color: #444; }

.chapter-meta {
  display: flex;
  gap: 16px;
  padding: 6px 0;
  margin-bottom: 8px;
}

.ch-meta-info {
  font-size: 9px;
  color: #555;
  font-family: 'Courier New', monospace;
}

.chapter-content {
  flex: 1;
  background: transparent;
  border: none;
  color: #ccc;
  font-size: 14px;
  font-family: 'Georgia', serif;
  line-height: 1.8;
  resize: none;
  outline: none;
  padding: 8px 0;
}

.chapter-content::placeholder { color: #333; }

.no-chapter-selected {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: #555;
}

.empty-msg {
  font-size: 12px;
  font-family: 'Courier New', monospace;
}

.chapter-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  border-top: 1px solid #2a2a2a;
  background: #111;
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 1;
}

.nav-btn {
  background: transparent;
  border: 1px solid #333;
  color: #aaa;
  padding: 4px 12px;
  font-size: 9px;
  font-family: 'Courier New', monospace;
  font-weight: 700;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.1s;
}

.nav-btn:hover:not(:disabled) {
  border-color: #555;
  color: #e0e0e0;
  background: #1a1a1a;
}

.nav-btn:disabled { opacity: 0.3; cursor: not-allowed; }

.nav-info {
  font-size: 9px;
  color: #555;
  font-family: 'Courier New', monospace;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
