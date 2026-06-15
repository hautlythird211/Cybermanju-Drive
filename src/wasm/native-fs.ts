declare global {
  interface FileSystemHandle {
    queryPermission(descriptor?: FileSystemPermissionDescriptor): Promise<PermissionState>
    requestPermission(descriptor?: FileSystemPermissionDescriptor): Promise<PermissionState>
  }

  interface FileSystemPermissionDescriptor {
    handle: FileSystemHandle
    mode?: 'read' | 'readwrite'
  }

  interface FileSystemDirectoryHandle {
    [Symbol.asyncIterator](): AsyncIterableIterator<FileSystemDirectoryHandle | FileSystemFileHandle>
    values(): AsyncIterableIterator<FileSystemDirectoryHandle | FileSystemFileHandle>
    entries(): AsyncIterableIterator<[string, FileSystemDirectoryHandle | FileSystemFileHandle]>
    keys(): AsyncIterableIterator<string>
  }

  interface FilePickerAcceptType {
    description?: string
    accept: Record<string, string[]>
  }

  interface FilePickerOptions {
    types?: FilePickerAcceptType[]
    excludeAcceptAllOption?: boolean
    id?: string
    startIn?: FileSystemHandle | 'desktop' | 'documents' | 'downloads' | 'music' | 'pictures' | 'videos'
  }

  interface OpenFilePickerOptions extends FilePickerOptions {
    multiple?: boolean
  }

  interface SaveFilePickerOptions extends FilePickerOptions {
    suggestedName?: string
  }

  interface DirectoryPickerOptions {
    id?: string
    startIn?: FileSystemHandle | 'desktop' | 'documents' | 'downloads' | 'music' | 'pictures' | 'videos'
    mode?: 'read' | 'readwrite'
  }

  interface Window {
    showDirectoryPicker(options?: DirectoryPickerOptions): Promise<FileSystemDirectoryHandle>
    showOpenFilePicker(options?: OpenFilePickerOptions): Promise<FileSystemFileHandle[]>
    showSaveFilePicker(options?: SaveFilePickerOptions): Promise<FileSystemFileHandle>
  }

  interface FileSystemChangeRecord {
    readonly changedHandle: FileSystemHandle | null
    readonly root: FileSystemHandle
    readonly relativePathComponents: string[]
    readonly relativePathMovedFrom?: string[]
    readonly type: 'appeared' | 'disappeared' | 'errored' | 'modified' | 'moved' | 'unknown'
  }

  interface FileSystemObserverCallback {
    (records: FileSystemChangeRecord[], observer: FileSystemObserver): void
  }

  interface FileSystemObserver {
    observe(handle: FileSystemHandle, options?: { recursive?: boolean }): Promise<void>
    disconnect(): void
  }
  var FileSystemObserver: {
    new(callback: FileSystemObserverCallback): FileSystemObserver
    prototype: FileSystemObserver
  }
}

import * as storage from './storage'

const HANDLE_KEY = 'native-fs-root-handle'
const MODE_KEY = 'native-fs-root-mode'

export interface NativeFileEntry {
  name: string
  path: string
  kind: 'file' | 'directory'
  size: number
  mimeType: string
  modifiedAt: string
  handle: FileSystemFileHandle | FileSystemDirectoryHandle
  parentPath: string
}

export interface FallbackFileEntry {
  name: string
  path: string
  size: number
  mimeType: string
  modifiedAt: string
  data: ArrayBuffer
  kind: 'file'
  parentPath: string
}

export function isSupported(): boolean {
  return 'showDirectoryPicker' in window
}

export function isObserverSupported(): boolean {
  return typeof FileSystemObserver !== 'undefined'
}

// ── Picker ─────────────────────────────────────────────────

export async function openDirectory(
  mode: 'read' | 'readwrite' = 'readwrite'
): Promise<FileSystemDirectoryHandle | null> {
  if (!isSupported()) return null
  try {
    const handle = await window.showDirectoryPicker({ mode })
    await persistHandle(handle, mode)
    return handle
  } catch (err) {
    if ((err as DOMException).name === 'AbortError') return null
    throw err
  }
}

export async function openFiles(
  multiple = false,
  types?: FilePickerAcceptType[]
): Promise<FileSystemFileHandle[]> {
  if (!isSupported()) return []
  try {
    return await window.showOpenFilePicker({ multiple, types })
  } catch (err) {
    if ((err as DOMException).name === 'AbortError') return []
    throw err
  }
}

export async function saveFile(
  suggestedName?: string,
  types?: FilePickerAcceptType[]
): Promise<FileSystemFileHandle | null> {
  if (!isSupported()) return null
  try {
    return await window.showSaveFilePicker({ suggestedName, types })
  } catch (err) {
    if ((err as DOMException).name === 'AbortError') return null
    throw err
  }
}

// ── Persistence ────────────────────────────────────────────

export async function persistHandle(
  handle: FileSystemDirectoryHandle,
  mode: 'read' | 'readwrite'
): Promise<void> {
  await storage.kvSet(HANDLE_KEY, handle)
  await storage.kvSet(MODE_KEY, mode)
}

export async function getPersistedHandle(): Promise<FileSystemDirectoryHandle | null> {
  return (await storage.kvGet<FileSystemDirectoryHandle>(HANDLE_KEY)) ?? null
}

export async function getPersistedMode(): Promise<'read' | 'readwrite'> {
  return (await storage.kvGet<'read' | 'readwrite'>(MODE_KEY)) ?? 'read'
}

export async function clearPersistedHandle(): Promise<void> {
  await storage.kvDelete(HANDLE_KEY)
  await storage.kvDelete(MODE_KEY)
}

// ── Permission ─────────────────────────────────────────────

export async function verifyPermission(
  handle: FileSystemDirectoryHandle | FileSystemFileHandle,
  mode: 'read' | 'readwrite' = 'read'
): Promise<boolean> {
  try {
    const opts: FileSystemPermissionDescriptor = { handle, mode }
    if ((await handle.queryPermission(opts)) === 'granted') return true
    return (await handle.requestPermission(opts)) === 'granted'
  } catch {
    return false
  }
}

// ── Directory Traversal ────────────────────────────────────

export async function listDirectory(
  dirHandle: FileSystemDirectoryHandle,
  recursive = false,
  path = ''
): Promise<NativeFileEntry[]> {
  const entries: NativeFileEntry[] = []
  for await (const entry of dirHandle.values()) {
    const entryPath = path ? `${path}/${entry.name}` : entry.name
    if (entry.kind === 'file') {
      const file = await entry.getFile()
      entries.push({
        name: entry.name,
        path: entryPath,
        kind: 'file',
        size: file.size,
        mimeType: file.type || 'application/octet-stream',
        modifiedAt: file.lastModified
          ? new Date(file.lastModified).toISOString()
          : new Date().toISOString(),
        handle: entry,
        parentPath: path,
      })
    } else if (entry.kind === 'directory') {
      entries.push({
        name: entry.name,
        path: entryPath,
        kind: 'directory',
        size: 0,
        mimeType: 'inode/directory',
        modifiedAt: new Date().toISOString(),
        handle: entry,
        parentPath: path,
      })
      if (recursive) {
        const sub = await listDirectory(entry, true, entryPath)
        entries.push(...sub)
      }
    }
  }
  return entries
}

// ── File Read ──────────────────────────────────────────────

export async function readFile(handle: FileSystemFileHandle): Promise<ArrayBuffer> {
  const file = await handle.getFile()
  return file.arrayBuffer()
}

export async function readFileText(handle: FileSystemFileHandle): Promise<string> {
  const buf = await readFile(handle)
  return new TextDecoder().decode(buf)
}

// ── File Write (Synced Passthrough) ────────────────────────

export async function writeFile(
  handle: FileSystemFileHandle,
  data: ArrayBuffer,
  keepExistingData = false
): Promise<void> {
  const writable = await handle.createWritable({ keepExistingData })
  await writable.write(data)
  await writable.close()
}

export async function writeFileText(
  handle: FileSystemFileHandle,
  text: string
): Promise<void> {
  const data = new TextEncoder().encode(text).buffer
  await writeFile(handle, data)
}

// ── File/Directory Create & Delete ─────────────────────────

export async function createDirectory(
  dirHandle: FileSystemDirectoryHandle,
  name: string
): Promise<FileSystemDirectoryHandle> {
  return dirHandle.getDirectoryHandle(name, { create: true })
}

export async function createFile(
  dirHandle: FileSystemDirectoryHandle,
  name: string
): Promise<FileSystemFileHandle> {
  return dirHandle.getFileHandle(name, { create: true })
}

export async function removeEntry(
  dirHandle: FileSystemDirectoryHandle,
  name: string,
  recursive = false
): Promise<void> {
  await dirHandle.removeEntry(name, { recursive })
}

// ── Resolve path relative to root ──────────────────────────

export async function resolveHandle(
  rootHandle: FileSystemDirectoryHandle,
  filePath: string
): Promise<FileSystemFileHandle | FileSystemDirectoryHandle | null> {
  const parts = filePath.split('/').filter(Boolean)
  let current: FileSystemDirectoryHandle | FileSystemFileHandle = rootHandle
  for (let i = 0; i < parts.length; i++) {
    if (current.kind !== 'directory') return null
    const dir = current as FileSystemDirectoryHandle
    const isLast = i === parts.length - 1
    try {
      if (isLast) {
        return await dir.getFileHandle(parts[i])
      }
      current = await dir.getDirectoryHandle(parts[i])
    } catch {
      try {
        if (isLast) {
          return await dir.getDirectoryHandle(parts[i])
        }
        current = await dir.getDirectoryHandle(parts[i])
      } catch {
        return null
      }
    }
  }
  return current
}

// ── Observer (Chrome 133+) ─────────────────────────────────

export async function observeDirectory(
  handle: FileSystemDirectoryHandle | FileSystemFileHandle,
  callback: (records: FileSystemChangeRecord[]) => void,
  recursive = false
): Promise<FileSystemObserver | null> {
  if (!isObserverSupported()) return null
  const observer = new FileSystemObserver((records) => callback(records))
  await observer.observe(handle, { recursive })
  return observer
}

// ── Fallback: <input webkitdirectory> ──────────────────────

export function openDirectoryFallback(): Promise<FallbackFileEntry[]> {
  return new Promise((resolve) => {
    const input = document.createElement('input')
    input.type = 'file'
    input.webkitdirectory = true

    input.addEventListener('change', async () => {
      const files = input.files
      if (!files) { resolve([]); return }
      const entries: FallbackFileEntry[] = []
      for (let i = 0; i < files.length; i++) {
        const f = files[i]
        const relPath = (f as any).webkitRelativePath || f.name
        const parentPath = relPath.includes('/')
          ? relPath.slice(0, relPath.lastIndexOf('/'))
          : ''
        entries.push({
          name: f.name,
          path: relPath,
          size: f.size,
          mimeType: f.type || 'application/octet-stream',
          modifiedAt: new Date(f.lastModified).toISOString(),
          data: await f.arrayBuffer(),
          kind: 'file',
          parentPath,
        })
      }
      resolve(entries)
    })

    input.addEventListener('cancel', () => resolve([]))

    input.click()
  })
}

// ── Restore persisted session on load ──────────────────────

export async function restoreSession(): Promise<{
  handle: FileSystemDirectoryHandle | null
  mode: 'read' | 'readwrite'
}> {
  const handle = await getPersistedHandle()
  const mode = await getPersistedMode()
  if (handle) {
    const granted = await verifyPermission(handle, mode)
    if (!granted) {
      await clearPersistedHandle()
      return { handle: null, mode: 'read' }
    }
  }
  return { handle, mode }
}
