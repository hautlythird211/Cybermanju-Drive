const DB_NAME = 'CybermanjuDrive'
const DB_VERSION = 1

let db: IDBDatabase | null = null
let initPromise: Promise<IDBDatabase> | null = null

export interface StoredFile {
  id: string
  name: string
  data: ArrayBuffer
  mimeType: string
  size: number
  createdAt: string
  modifiedAt: string
  parentId: string | null
  tags: string[]
  encrypted: boolean
  hash: string | null
}

export interface StoredSyncEntry {
  id: string
  fileId: string
  backendType: string
  status: string
  localChanges: number
  lastSyncedAt: string | null
  errorMessage: string | null
}

function openDb(): Promise<IDBDatabase> {
  if (db) return Promise.resolve(db)
  if (initPromise) return initPromise

  initPromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onupgradeneeded = (event) => {
      const database = (event.target as IDBOpenDBRequest).result

      if (!database.objectStoreNames.contains('files')) {
        const fileStore = database.createObjectStore('files', { keyPath: 'id' })
        fileStore.createIndex('parentId', 'parentId', { unique: false })
        fileStore.createIndex('name', 'name', { unique: false })
        fileStore.createIndex('mimeType', 'mimeType', { unique: false })
        fileStore.createIndex('modifiedAt', 'modifiedAt', { unique: false })
        fileStore.createIndex('tags', 'tags', { multiEntry: true })
        fileStore.createIndex('encrypted', 'encrypted', { unique: false })
      }

      if (!database.objectStoreNames.contains('sync_entries')) {
        const syncStore = database.createObjectStore('sync_entries', { keyPath: 'id' })
        syncStore.createIndex('fileId', 'fileId', { unique: false })
        syncStore.createIndex('backendType', 'backendType', { unique: false })
        syncStore.createIndex('status', 'status', { unique: false })
      }

      if (!database.objectStoreNames.contains('sync_configs')) {
        database.createObjectStore('sync_configs', { keyPath: 'id' })
      }

      if (!database.objectStoreNames.contains('oauth_tokens')) {
        database.createObjectStore('oauth_tokens', { keyPath: 'provider' })
      }

      if (!database.objectStoreNames.contains('drive_meta')) {
        database.createObjectStore('drive_meta', { keyPath: 'key' })
      }

      if (!database.objectStoreNames.contains('kv_store')) {
        database.createObjectStore('kv_store', { keyPath: 'key' })
      }
    }

    request.onsuccess = (event) => {
      db = (event.target as IDBOpenDBRequest).result
      resolve(db!)
    }

    request.onerror = (event) => {
      initPromise = null
      reject((event.target as IDBOpenDBRequest).error)
    }
  })

  return initPromise
}

async function withStore<T>(
  storeName: string,
  mode: IDBTransactionMode,
  fn: (store: IDBObjectStore) => IDBRequest<T>
): Promise<T> {
  const database = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(storeName, mode)
    const store = transaction.objectStore(storeName)
    const request = fn(store)

    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)

    transaction.oncomplete = () => {
      if (request.readyState === 'done') resolve(request.result)
    }
  })
}

// ── File Operations ──────────────────────────────────────────

export async function storeFile(file: StoredFile): Promise<void> {
  await withStore('files', 'readwrite', (store) => store.put(file))
}

export async function getFile(id: string): Promise<StoredFile | undefined> {
  return withStore('files', 'readonly', (store) => store.get(id))
}

export async function deleteFile(id: string): Promise<void> {
  await withStore('files', 'readwrite', (store) => store.delete(id))
}

export async function listFiles(parentId: string | null): Promise<StoredFile[]> {
  const database = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = database.transaction('files', 'readonly')
    const store = transaction.objectStore('files')
    const index = store.index('parentId')
    const request = index.getAll(parentId)

    request.onsuccess = () => resolve(request.result || [])
    request.onerror = () => reject(request.error)
  })
}

export async function getAllFiles(): Promise<StoredFile[]> {
  const database = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = database.transaction('files', 'readonly')
    const store = transaction.objectStore('files')
    const request = store.getAll()

    request.onsuccess = () => resolve(request.result || [])
    request.onerror = () => reject(request.error)
  })
}

export async function searchFiles(query: string): Promise<StoredFile[]> {
  const all = await getAllFiles()
  const q = query.toLowerCase()
  return all.filter(
    (f) =>
      f.name.toLowerCase().includes(q) ||
      f.tags.some((t) => t.toLowerCase().includes(q)) ||
      f.mimeType.toLowerCase().includes(q)
  )
}

export async function renameFile(id: string, newName: string): Promise<void> {
  const file = await getFile(id)
  if (!file) throw new Error('File not found')
  file.name = newName
  file.modifiedAt = new Date().toISOString()
  await storeFile(file)
}

export async function moveFile(id: string, newParentId: string | null): Promise<void> {
  const file = await getFile(id)
  if (!file) throw new Error('File not found')
  file.parentId = newParentId
  file.modifiedAt = new Date().toISOString()
  await storeFile(file)
}

// ── Sync Entry Operations ────────────────────────────────────

export async function storeSyncEntry(entry: StoredSyncEntry): Promise<void> {
  await withStore('sync_entries', 'readwrite', (store) => store.put(entry))
}

export async function getSyncEntry(id: string): Promise<StoredSyncEntry | undefined> {
  return withStore('sync_entries', 'readonly', (store) => store.get(id))
}

export async function deleteSyncEntry(id: string): Promise<void> {
  await withStore('sync_entries', 'readwrite', (store) => store.delete(id))
}

export async function getAllSyncEntries(): Promise<StoredSyncEntry[]> {
  const database = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = database.transaction('sync_entries', 'readonly')
    const store = transaction.objectStore('sync_entries')
    const request = store.getAll()
    request.onsuccess = () => resolve(request.result || [])
    request.onerror = () => reject(request.error)
  })
}

// ── Key-Value Store ──────────────────────────────────────────

export async function kvSet(key: string, value: unknown): Promise<void> {
  await withStore('kv_store', 'readwrite', (store) => store.put({ key, value }))
}

export async function kvGet<T>(key: string): Promise<T | undefined> {
  const result = await withStore<{ key: string; value: T } | undefined>('kv_store', 'readonly', (store) => store.get(key))
  return result?.value
}

export async function kvDelete(key: string): Promise<void> {
  await withStore('kv_store', 'readwrite', (store) => store.delete(key))
}

// ── Drive Metadata ───────────────────────────────────────────

export async function setDriveMeta(key: string, value: unknown): Promise<void> {
  await withStore('drive_meta', 'readwrite', (store) => store.put({ key, value }))
}

export async function getDriveMeta<T>(key: string): Promise<T | undefined> {
  const result = await withStore<{ key: string; value: T } | undefined>('drive_meta', 'readonly', (store) => store.get(key))
  return result?.value
}

// ── Utility ──────────────────────────────────────────────────

export async function getStorageStats(): Promise<{
  fileCount: number
  totalSize: number
  syncEntryCount: number
}> {
  const files = await getAllFiles()
  const syncEntries = await getAllSyncEntries()
  return {
    fileCount: files.length,
    totalSize: files.reduce((sum, f) => sum + f.size, 0),
    syncEntryCount: syncEntries.length,
  }
}

export async function clearAllData(): Promise<void> {
  const database = await openDb()
  const storeNames = Array.from(database.objectStoreNames)
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(storeNames, 'readwrite')
    transaction.oncomplete = () => resolve()
    transaction.onerror = () => reject(transaction.error)
    for (const name of storeNames) {
      transaction.objectStore(name).clear()
    }
  })
}

export async function closeDb(): Promise<void> {
  if (db) {
    db.close()
    db = null
    initPromise = null
  }
}

export async function deleteDb(): Promise<void> {
  await closeDb()
  return new Promise((resolve, reject) => {
    const request = indexedDB.deleteDatabase(DB_NAME)
    request.onsuccess = () => resolve()
    request.onerror = () => reject(request.error)
  })
}
