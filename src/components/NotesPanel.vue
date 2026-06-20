<template>
  <div class="notes-panel">
    <div class="notes-sidebar">
      <div class="notes-sidebar-header">
        <span class="notes-brand">NOTES</span>
        <button class="notes-add-btn" @click="createNote" title="New Note">+</button>
      </div>
      <div class="notes-search">
        <input v-model="searchQuery" class="notes-search-input" placeholder="Search notes..." />
      </div>
      <div class="notes-list">
        <div
          v-for="note in filteredNotes"
          :key="note.id"
          class="note-item"
          :class="{ active: selectedNoteId === note.id }"
          @click="selectNote(note.id)"
        >
          <div class="note-item-title">{{ note.title || 'Untitled' }}</div>
          <div class="note-item-preview truncate">{{ note.content.slice(0, 60) || 'Empty note' }}</div>
          <div class="note-item-date">{{ formatDate(note.updatedAt) }}</div>
        </div>
        <div v-if="filteredNotes.length === 0" class="empty-notes">
          <p>{{ searchQuery ? 'No matches' : 'No notes yet' }}</p>
        </div>
      </div>
    </div>

    <div class="notes-editor">
      <div v-if="selectedNote" class="editor-inner">
        <div class="editor-header">
          <input
            v-model="selectedNote.title"
            class="note-title-input"
            placeholder="Note title..."
            @blur="saveNotes"
          />
          <div class="editor-actions">
            <button class="note-action-btn" @click="pasteFromClipboard" title="Paste from clipboard">Paste</button>
            <button class="note-action-btn" @click="copyNote" title="Copy note">Copy</button>
            <button class="note-action-btn note-action-delete" @click="deleteNote" title="Delete">Delete</button>
          </div>
        </div>
        <div class="note-tags-row">
          <span
            v-for="tag in selectedNote.tags"
            :key="tag"
            class="note-tag"
          >
            {{ tag }}
            <button class="tag-remove" @click="removeTag(tag)">x</button>
          </span>
          <input
            v-model="newTag"
            class="tag-input"
            placeholder="+ tag"
            @keydown.enter="addTag"
          />
        </div>
        <textarea
          v-model="selectedNote.content"
          class="note-content"
          placeholder="Start writing..."
          @input="onContentChange"
        ></textarea>
        <div class="note-footer">
          <span class="note-stat">{{ wordCount(selectedNote.content) }} words</span>
          <span class="note-stat">{{ formatDate(selectedNote.updatedAt) }}</span>
        </div>
      </div>
      <div v-else class="no-note-selected">
        <p class="empty-msg">Select or create a note</p>
        <button class="note-create-btn" @click="createNote">+ New Note</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

interface Note {
  id: string
  title: string
  content: string
  tags: string[]
  createdAt: string
  updatedAt: string
}

const notes = ref<Note[]>([])
const selectedNoteId = ref<string | null>(null)
const searchQuery = ref('')
const newTag = ref('')
const saveDebounce = ref<ReturnType<typeof setTimeout> | null>(null)

const selectedNote = computed(() =>
  notes.value.find(n => n.id === selectedNoteId.value) || null
)

const filteredNotes = computed(() => {
  const q = searchQuery.value.toLowerCase()
  const list = [...notes.value].sort((a, b) =>
    new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
  )
  if (!q) return list
  return list.filter(n =>
    n.title.toLowerCase().includes(q) ||
    n.content.toLowerCase().includes(q) ||
    n.tags.some(t => t.toLowerCase().includes(q))
  )
})

function wordCount(text: string): number {
  if (!text || !text.trim()) return 0
  return text.trim().split(/\s+/).length
}

function formatDate(iso: string): string {
  const d = new Date(iso)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffMin = Math.floor(diffMs / 60000)
  if (diffMin < 1) return 'just now'
  if (diffMin < 60) return `${diffMin}m ago`
  const diffHr = Math.floor(diffMin / 60)
  if (diffHr < 24) return `${diffHr}h ago`
  const diffDay = Math.floor(diffHr / 24)
  if (diffDay < 7) return `${diffDay}d ago`
  return d.toLocaleDateString()
}

function createNote() {
  const note: Note = {
    id: `note-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
    title: '',
    content: '',
    tags: [],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  notes.value.unshift(note)
  selectedNoteId.value = note.id
  saveNotes()
}

function selectNote(id: string) {
  selectedNoteId.value = id
}

function deleteNote() {
  if (!selectedNoteId.value) return
  notes.value = notes.value.filter(n => n.id !== selectedNoteId.value)
  selectedNoteId.value = notes.value.length > 0 ? notes.value[0].id : null
  saveNotes()
}

function addTag() {
  const tag = newTag.value.trim()
  if (tag && selectedNote.value && !selectedNote.value.tags.includes(tag)) {
    selectedNote.value.tags.push(tag)
    newTag.value = ''
    saveNotes()
  }
}

function removeTag(tag: string) {
  if (selectedNote.value) {
    selectedNote.value.tags = selectedNote.value.tags.filter(t => t !== tag)
    saveNotes()
  }
}

async function pasteFromClipboard() {
  try {
    const text = await navigator.clipboard.readText()
    if (text && selectedNote.value) {
      selectedNote.value.content += text
      onContentChange()
    }
  } catch { /* denied */ }
}

async function copyNote() {
  if (!selectedNote.value) return
  const text = `${selectedNote.value.title}\n\n${selectedNote.value.content}`
  try { await navigator.clipboard.writeText(text) } catch {}
}

function onContentChange() {
  if (selectedNote.value) {
    selectedNote.value.updatedAt = new Date().toISOString()
  }
  saveNotes()
}

function saveNotes() {
  if (saveDebounce.value) clearTimeout(saveDebounce.value)
  saveDebounce.value = setTimeout(() => {
    try { localStorage.setItem('cybermanju_notes', JSON.stringify(notes.value)) } catch {}
  }, 500)
}

function loadNotes() {
  try {
    const raw = localStorage.getItem('cybermanju_notes')
    if (raw) {
      notes.value = JSON.parse(raw)
      if (notes.value.length > 0) selectedNoteId.value = notes.value[0].id
    }
  } catch {}
}

onMounted(loadNotes)
</script>

<style scoped>
.notes-panel {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}
.notes-sidebar {
  width: 200px;
  background: #111;
  border-right: 1px solid #2a2a2a;
  display: flex;
  flex-direction: column;
}
.notes-sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid #2a2a2a;
  background: #1a1a1a;
}
.notes-brand {
  font-size: 10px;
  font-weight: 700;
  color: #5af0ff;
  letter-spacing: 1px;
  flex: 1;
}
.notes-add-btn {
  background: transparent;
  border: 1px solid #333;
  color: #5af0ff;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
}
.notes-add-btn:hover { border-color: #5af0ff; background: rgba(90, 240, 255, 0.1); }
.notes-search { padding: 6px 8px; border-bottom: 1px solid #2a2a2a; }
.notes-search-input {
  width: 100%;
  background: #1a1a1a;
  border: 1px solid #333;
  color: #ccc;
  font-size: 9px;
  padding: 4px 8px;
  border-radius: 4px;
  font-family: var(--font-mono);
  outline: none;
  box-sizing: border-box;
}
.notes-search-input:focus { border-color: #555; }
.notes-list { flex: 1; overflow-y: auto; padding: 4px; }
.note-item {
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid transparent;
  margin-bottom: 2px;
  transition: background 0.1s;
}
.note-item:hover { background: #1a1a1a; border-color: #2a2a2a; }
.note-item.active { background: rgba(90, 240, 255, 0.05); border-color: rgba(90, 240, 255, 0.2); }
.note-item-title { font-size: 10px; font-weight: 700; color: #e0e0e0; margin-bottom: 2px; }
.note-item-preview { font-size: 8px; color: #666; margin-bottom: 2px; }
.note-item-date { font-size: 7px; color: #444; }
.empty-notes { text-align: center; padding: 20px; color: #555; font-size: 9px; }
.notes-editor { flex: 1; display: flex; flex-direction: column; }
.editor-inner { flex: 1; display: flex; flex-direction: column; }
.editor-header { display: flex; align-items: center; gap: 8px; padding: 8px 12px; border-bottom: 1px solid #2a2a2a; background: #111; }
.note-title-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #5af0ff;
  font-size: 13px;
  font-weight: 700;
  font-family: var(--font-mono);
  outline: none;
  padding: 4px 0;
}
.note-title-input::placeholder { color: #444; }
.editor-actions { display: flex; gap: 4px; }
.note-action-btn {
  background: transparent;
  border: 1px solid #333;
  color: #aaa;
  padding: 2px 8px;
  font-size: 8px;
  font-family: var(--font-mono);
  font-weight: 700;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.1s;
}
.note-action-btn:hover { border-color: #555; color: #e0e0e0; background: #1a1a1a; }
.note-action-delete:hover { border-color: #ff5f57; color: #ff5f57; }
.note-tags-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 6px 12px;
  border-bottom: 1px solid #2a2a2a;
}
.note-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  font-size: 8px;
  border: 1px solid #333;
  border-radius: 4px;
  color: #5af0ff;
}
.tag-remove {
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
  font-size: 10px;
  padding: 0;
  line-height: 1;
}
.tag-remove:hover { color: #ff5f57; }
.tag-input {
  background: transparent;
  border: 1px solid transparent;
  color: #aaa;
  font-size: 8px;
  padding: 2px 6px;
  outline: none;
  font-family: var(--font-mono);
  width: 60px;
}
.tag-input:focus { border-color: #333; }
.note-content {
  flex: 1;
  background: transparent;
  border: none;
  color: #ccc;
  font-size: 12px;
  font-family: var(--font-mono);
  line-height: 1.8;
  resize: none;
  outline: none;
  padding: 12px;
}
.note-content::placeholder { color: #333; }
.note-footer {
  display: flex;
  justify-content: space-between;
  padding: 4px 12px;
  border-top: 1px solid #2a2a2a;
  background: #111;
}
.note-stat { font-size: 8px; color: #555; }
.no-note-selected { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; }
.empty-msg { font-size: 10px; color: #555; }
.note-create-btn {
  background: transparent;
  border: 1px solid #333;
  color: #5af0ff;
  padding: 6px 12px;
  font-size: 9px;
  font-family: var(--font-mono);
  font-weight: 700;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.1s;
}
.note-create-btn:hover { border-color: #5af0ff; background: rgba(90, 240, 255, 0.1); }
.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
