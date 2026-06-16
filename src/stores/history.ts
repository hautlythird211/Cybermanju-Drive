import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@/composables/useTauri'
import { useNotifications } from '@/composables/useNotifications'
import { useAppStore } from './app'
import { kvGet, kvSet } from '@/wasm/storage'
import type { HistoryEntry, HistoryActionType, HistoryAction } from '@/types'

const MAX_HISTORY = 500
const STORAGE_KEY = 'cybermanju_history'

let entryCounter = 0

function makeId(): string {
  return `hist-${++entryCounter}-${Date.now().toString(36)}`
}

function now(): number {
  return Date.now()
}

const REFRESH_MAP: Record<string, () => Promise<void>> = {}
let storeReady = false

function ensureStore() {
  if (storeReady) return
  storeReady = true
  const store = useAppStore()
  REFRESH_MAP['file'] = async () => { await Promise.all([store.fetchFiles(), store.fetchTrashItems()]) }
  REFRESH_MAP['trash'] = async () => { await store.fetchTrashItems() }
  REFRESH_MAP['encryption'] = async () => { await Promise.all([store.fetchFiles(), store.fetchEncryptionStatus()]) }
  REFRESH_MAP['compression'] = async () => { await store.fetchFiles() }
  REFRESH_MAP['collection'] = async () => { await store.fetchCollections() }
  REFRESH_MAP['face'] = async () => { await store.fetchFaceGroups() }
  REFRESH_MAP['account'] = async () => { await store.fetchAccounts() }
  REFRESH_MAP['sync'] = async () => { await store.fetchSyncConfigs() }
  REFRESH_MAP['user'] = async () => { await store.fetchUsers() }
  REFRESH_MAP['share'] = async () => { await store.fetchShareLinks() }
}

function getRefreshForType(type: HistoryActionType): (() => Promise<void>) | null {
  ensureStore()
  const key = type.split(':')[0]
  return REFRESH_MAP[key] || null
}

export const useHistoryStore = defineStore('history', () => {
  const undoStack = ref<HistoryEntry[]>([])
  const redoStack = ref<HistoryEntry[]>([])
  const isRestoring = ref(false)
  const lastSave = ref(0)
  let pendingSave = false

  const { notify } = useNotifications()

  const canUndo = computed(() => undoStack.value.length > 0 && !isRestoring.value)
  const canRedo = computed(() => redoStack.value.length > 0 && !isRestoring.value)

  const allEntries = computed(() => undoStack.value)

  // ── Persistence ──────────────────────────────────────────

  async function load() {
    try {
      const saved = await kvGet<{ undo: HistoryEntry[]; redo: HistoryEntry[] }>(STORAGE_KEY)
      if (saved) {
        undoStack.value = saved.undo || []
        redoStack.value = saved.redo || []
        if (undoStack.value.length > 0) {
          entryCounter = Math.max(
            ...undoStack.value.map(e => parseInt((e.id.match(/hist-(\d+)/) || ['0', '0'])[1], 10)),
            0,
          )
        }
      }
    } catch {
      // storage not available
    }
  }

  async function save(force = false) {
    const nowTs = now()
    if (!force && nowTs - lastSave.value < 2000) {
      pendingSave = true
      return
    }
    lastSave.value = nowTs
    pendingSave = false
    try {
      await kvSet(STORAGE_KEY, {
        undo: undoStack.value.slice(0, MAX_HISTORY),
        redo: redoStack.value.slice(0, MAX_HISTORY),
      })
    } catch {
      // storage failure — non-critical
    }
  }

  // Flush any pending save immediately
  async function flush() {
    if (pendingSave) {
      lastSave.value = 0
      await save(true)
    }
  }

  // ── Core ─────────────────────────────────────────────────

  function push(
    type: HistoryActionType,
    description: string,
    affectedFileIds: string[],
    undo: HistoryAction,
    redo: HistoryAction,
  ) {
    if (isRestoring.value) return

    const entry: HistoryEntry = {
      id: makeId(),
      type,
      description,
      timestamp: now(),
      affectedFileIds,
      undo,
      redo,
    }
    undoStack.value.unshift(entry)
    redoStack.value = []

    if (undoStack.value.length > MAX_HISTORY) {
      undoStack.value.pop()
    }

    save()
  }

  async function applyAction(action: HistoryAction): Promise<void> {
    if (action.source === 'store') {
      const store = useAppStore()
      const fn = (store as any)[action.cmd]
      if (typeof fn === 'function') {
        const args = Object.values(action.args) as unknown[]
        await fn.call(store, ...args)
        return
      }
    }
    await invoke(action.cmd, action.args)
  }

  async function undo() {
    if (!canUndo.value) return
    const entry = undoStack.value[0]
    if (!entry) return

    isRestoring.value = true
    try {
      await applyAction(entry.undo)
      undoStack.value.shift()
      redoStack.value.unshift(entry)
      notify('info', `UNDO: ${entry.description}`, 3000)
      await save()
      const refresh = getRefreshForType(entry.type)
      if (refresh) await refresh()
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      notify('error', `UNDO FAILED: ${entry.description} — ${msg}`, 5000)
    } finally {
      isRestoring.value = false
    }
  }

  async function redo() {
    if (!canRedo.value) return
    const entry = redoStack.value[0]
    if (!entry) return

    isRestoring.value = true
    try {
      await applyAction(entry.redo)
      redoStack.value.shift()
      undoStack.value.unshift(entry)
      notify('info', `REDO: ${entry.description}`, 3000)
      await save()
      const refresh = getRefreshForType(entry.type)
      if (refresh) await refresh()
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      notify('error', `REDO FAILED: ${entry.description} — ${msg}`, 5000)
    } finally {
      isRestoring.value = false
    }
  }

  function clear() {
    undoStack.value = []
    redoStack.value = []
    save()
  }

  function removeByFileId(fileId: string) {
    undoStack.value = undoStack.value.filter(e => !e.affectedFileIds.includes(fileId))
    redoStack.value = redoStack.value.filter(e => !e.affectedFileIds.includes(fileId))
    save()
  }

  // Auto-flush on page unload
  if (typeof window !== 'undefined') {
    window.addEventListener('beforeunload', () => flush())
    // Also flush periodically as backup
    setInterval(() => { if (pendingSave) flush() }, 5000)
  }

  return {
    undoStack,
    redoStack,
    isRestoring,
    canUndo,
    canRedo,
    allEntries,
    load,
    save,
    push,
    undo,
    redo,
    clear,
    removeByFileId,
  }
})
