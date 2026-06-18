import { initWasm, getWasm, withWasm } from './bridge'
import * as storage from './storage'
import {
  saveTokenToStorage,
  loadTokenFromStorage,
  getValidToken,
  type OAuthToken,
} from './oauth'

export type SyncStatus = 'idle' | 'scanning' | 'compressing' | 'syncing' | 'error' | 'done' | 'cancelled'

export interface SyncProgress {
  totalFiles: number
  processedFiles: number
  currentFile: string | null
  status: SyncStatus
  bytesProcessed: number
  errors: string[]
  startedAt: string | null
}

export interface SyncConfig {
  id: string
  name: string
  backendType: string
  enabled: boolean
  basePath: string | null
  autoSync: boolean
  compressBeforeSync: boolean
  maxConcurrentOps: number
  createdAt: string
  updatedAt: string
}

export interface SyncFileInfo {
  id: string
  originalPath: string
  sizeBytes: number
  hash: string | null
  status: SyncStatus
  localChanges: number
  lastSyncedAt: string | null
}

type ProgressCallback = (progress: SyncProgress) => void

let progressListeners: ProgressCallback[] = []
let currentProgress: SyncProgress = {
  totalFiles: 0,
  processedFiles: 0,
  currentFile: null,
  status: 'idle',
  bytesProcessed: 0,
  errors: [],
  startedAt: null,
}

function notifyProgress() {
  for (const listener of progressListeners) {
    listener({ ...currentProgress })
  }
}

export function onProgress(callback: ProgressCallback): () => void {
  progressListeners.push(callback)
  return () => {
    progressListeners = progressListeners.filter((l) => l !== callback)
  }
}

export function getProgress(): SyncProgress {
  return { ...currentProgress }
}

function resetProgress(totalFiles: number) {
  currentProgress = {
    totalFiles,
    processedFiles: 0,
    currentFile: null,
    status: 'scanning',
    bytesProcessed: 0,
    errors: [],
    startedAt: new Date().toISOString(),
  }
  notifyProgress()
}

// ── Config Management ────────────────────────────────────────

export async function getSyncConfigs(): Promise<SyncConfig[]> {
  await initWasm()
  const raw = await storage.kvGet<SyncConfig[]>('sync_configs')
  return raw || []
}

export async function saveSyncConfig(
  config: Omit<SyncConfig, 'id' | 'createdAt' | 'updatedAt'>
): Promise<SyncConfig> {
  await initWasm()
  const wasm = getWasm()
  const now = wasm.now_utc()
  const id = wasm.generate_uuid()
  const newConfig: SyncConfig = {
    ...config,
    id,
    createdAt: now,
    updatedAt: now,
  }
  const configs = await getSyncConfigs()
  configs.push(newConfig)
  await storage.kvSet('sync_configs', configs)
  return newConfig
}

export async function updateSyncConfig(config: SyncConfig): Promise<void> {
  const configs = await getSyncConfigs()
  const idx = configs.findIndex((c) => c.id === config.id)
  if (idx !== -1) {
    config.updatedAt = new Date().toISOString()
    configs[idx] = config
    await storage.kvSet('sync_configs', configs)
  }
}

export async function deleteSyncConfig(id: string): Promise<void> {
  const configs = await getSyncConfigs()
  await storage.kvSet(
    'sync_configs',
    configs.filter((c) => c.id !== id)
  )
}

// ── Sync Engine ──────────────────────────────────────────────

export async function scanLocalFiles(
  _config: SyncConfig
): Promise<SyncFileInfo[]> {
  await initWasm()
  const storedFiles = await storage.getAllFiles()
  const wasm = getWasm()

  const wasmEngine = new wasm.SyncEngine()
  const entries: SyncFileInfo[] = []

  for (const f of storedFiles) {
    try {
      wasmEngine.add_file(f.id, f.size, _config.backendType)
      entries.push({
        id: f.id,
        originalPath: f.name,
        sizeBytes: f.size,
        hash: f.hash,
        status: 'idle' as SyncStatus,
        localChanges: 0,
        lastSyncedAt: null,
      })
    } catch {
      // Skip files that can't be added
    }
  }

  await storage.kvSet('wasm_sync_state', wasmEngine.to_json())
  return entries
}

export async function syncFile(
  fileId: string,
  _config: SyncConfig
): Promise<void> {
  await initWasm()
  currentProgress.currentFile = fileId
  currentProgress.status = 'syncing'
  notifyProgress()

  try {
    const oauthToken = await loadTokenFromStorage(_config.backendType as any)
    let token: OAuthToken | null = null
    if (oauthToken) {
      token = await getValidToken(oauthToken)
      await saveTokenToStorage(token!)
    }

    const entry: storage.StoredSyncEntry = {
      id: crypto.randomUUID(),
      fileId,
      backendType: _config.backendType,
      status: 'done',
      localChanges: 0,
      lastSyncedAt: new Date().toISOString(),
      errorMessage: null,
    }
    await storage.storeSyncEntry(entry)

    currentProgress.processedFiles++
    currentProgress.bytesProcessed += (await storage.getFile(fileId))?.size || 0
    notifyProgress()
  } catch (err) {
    const entry: storage.StoredSyncEntry = {
      id: crypto.randomUUID(),
      fileId,
      backendType: _config.backendType,
      status: 'error',
      localChanges: 1,
      lastSyncedAt: null,
      errorMessage: String(err),
    }
    await storage.storeSyncEntry(entry)
    currentProgress.errors.push(String(err))
    notifyProgress()
  }
}

export async function startSync(
  config: SyncConfig,
  fileIds: string[]
): Promise<void> {
  await initWasm()
  resetProgress(fileIds.length)

  for (let i = 0; i < fileIds.length; i++) {
    if (currentProgress.status === 'cancelled') break
    currentProgress.currentFile = fileIds[i]
    currentProgress.status = 'syncing'
    currentProgress.processedFiles = i
    notifyProgress()
    await syncFile(fileIds[i], config)
  }

  currentProgress.status = currentProgress.errors.length > 0 ? 'error' : 'done'
  currentProgress.currentFile = null
  notifyProgress()
}

export function cancelSync(): void {
  currentProgress.status = 'cancelled'
  notifyProgress()
}

// ── Local Changes Tracking ───────────────────────────────────

export async function getChangedFiles(): Promise<storage.StoredFile[]> {
  const allFiles = await storage.getAllFiles()
  const syncEntries = await storage.getAllSyncEntries()
  const syncedIds = new Set(
    syncEntries.filter((e) => e.status === 'done').map((e) => e.fileId)
  )
  return allFiles.filter((f) => !syncedIds.has(f.id))
}

export async function getSyncedFiles(): Promise<storage.StoredFile[]> {
  const allFiles = await storage.getAllFiles()
  const syncEntries = await storage.getAllSyncEntries()
  const syncedIds = new Set(
    syncEntries.filter((e) => e.status === 'done').map((e) => e.fileId)
  )
  return allFiles.filter((f) => syncedIds.has(f.id))
}

export async function getSyncSummary(): Promise<{
  totalFiles: number
  syncedFiles: number
  changedFiles: number
  errorFiles: number
  totalBytes: number
}> {
  const allFiles = await storage.getAllFiles()
  const syncEntries = await storage.getAllSyncEntries()

  const syncedFiles = syncEntries.filter((e) => e.status === 'done').length
  const errorFiles = syncEntries.filter((e) => e.status === 'error').length
  const syncedIds = new Set(
    syncEntries.filter((e) => e.status === 'done').map((e) => e.fileId)
  )
  const changedFiles = allFiles.filter((f) => !syncedIds.has(f.id)).length

  return {
    totalFiles: allFiles.length,
    syncedFiles,
    changedFiles,
    errorFiles,
    totalBytes: allFiles.reduce((sum, f) => sum + f.size, 0),
  }
}

// ── Remote Backend Interaction via OAuth ──────────────────────

export interface RemoteFileInfo {
  name: string
  path: string
  sizeBytes: number
  modifiedAt: string
  url: string
}

export async function listMegaFiles(
  email: string,
  password: string,
  _prefix: string
): Promise<RemoteFileInfo[]> {
  try {
    const { Storage } = await import('megajs')
    const storage = new Storage({ email, password, autoload: true, autologin: true, keepalive: false })
    await storage.ready

    if (!storage.root) {
      try { storage.close() } catch {}
      return []
    }

    const files: RemoteFileInfo[] = []

    function walk(file: any, dirPath: string) {
      if (!file) return
      if (file.directory && Array.isArray(file.children)) {
        for (const child of file.children) {
          if (!child || typeof child !== 'object') continue
          walk(child, dirPath + '/' + (child.name || ''))
        }
      } else if (!file.directory) {
        files.push({
          name: file.name || 'unknown',
          path: (dirPath + '/' + (file.name || 'unknown')).replace(/^\/+/, '/'),
          sizeBytes: typeof file.size === 'number' ? file.size : 0,
          modifiedAt: file.timestamp ? new Date(file.timestamp * 1000).toISOString() : '',
          url: '',
        })
      }
    }

    walk(storage.root, '')
    try { storage.close() } catch {}
    return files
  } catch (e) {
    throw new Error(`Failed to list Mega files: ${e instanceof Error ? e.message : String(e)}`)
  }
}

export async function listRemoteFiles(
  _config: SyncConfig,
  prefix: string
): Promise<RemoteFileInfo[]> {
  if (!_config || !_config.backendType) {
    throw new Error('Invalid sync config: missing backendType')
  }

  try {
    if (_config.backendType === 'mega') {
      const token = await loadTokenFromStorage('mega' as any)
      if (!token || !token.accessToken) return []
      const sep = token.accessToken.indexOf('|')
      if (sep === -1) return []
      const email = token.accessToken.slice(0, sep)
      const password = token.accessToken.slice(sep + 1)
      if (!email || !password) return []
      return listMegaFiles(email, password, prefix)
    }

    const oauthToken = await loadTokenFromStorage(_config.backendType as any)
    if (!oauthToken) return []

    const token = await getValidToken(oauthToken)
    await saveTokenToStorage(token)

    if (_config.backendType === 'googleDrive') {
      return listGoogleDriveFiles(token, prefix)
    } else if (_config.backendType === 'github') {
      return listGitHubFiles(token, prefix)
    }

    return []
  } catch (e) {
    throw new Error(`Failed to list remote files for ${_config?.backendType || 'unknown'}: ${e instanceof Error ? e.message : String(e)}`)
  }
}

async function listGoogleDriveFiles(
  token: OAuthToken,
  _prefix: string
): Promise<RemoteFileInfo[]> {
  const response = await fetch(
    `https://www.googleapis.com/drive/v3/files?q=trashed=false&pageSize=100&fields=files(id,name,size,modifiedTime,webViewLink)`,
    { headers: { Authorization: `Bearer ${token.accessToken}` } }
  )
  if (!response.ok) throw new Error(`Google Drive API error: ${response.status}`)
  const data = await response.json()
  return (data.files || []).map((f: any) => ({
    name: f.name,
    path: f.id,
    sizeBytes: parseInt(f.size || '0', 10),
    modifiedAt: f.modifiedTime,
    url: f.webViewLink || '',
  }))
}

async function listGitHubFiles(
  token: OAuthToken,
  prefix: string
): Promise<RemoteFileInfo[]> {
  const [owner, repo] = prefix.split('/')
  if (!owner || !repo) return []

  const response = await fetch(
    `https://api.github.com/repos/${owner}/${repo}/contents`,
    { headers: { Authorization: `Bearer ${token.accessToken}` } }
  )
  if (!response.ok) throw new Error(`GitHub API error: ${response.status}`)
  const data = await response.json()
  return (Array.isArray(data) ? data : []).map((f: any) => ({
    name: f.name,
    path: f.path,
    sizeBytes: f.size || 0,
    modifiedAt: new Date().toISOString(),
    url: f.html_url || '',
  }))
}

// ── Sync Without Upload (local-only tracking) ────────────────

export async function markAllFilesSynced(): Promise<void> {
  await initWasm()
  const allFiles = await storage.getAllFiles()
  for (const f of allFiles) {
    const entry: storage.StoredSyncEntry = {
      id: crypto.randomUUID(),
      fileId: f.id,
      backendType: 'local',
      status: 'done',
      localChanges: 0,
      lastSyncedAt: new Date().toISOString(),
      errorMessage: null,
    }
    await storage.storeSyncEntry(entry)
  }
}

export async function markFileSynced(fileId: string): Promise<void> {
  const entry: storage.StoredSyncEntry = {
    id: crypto.randomUUID(),
    fileId,
    backendType: 'local',
    status: 'done',
    localChanges: 0,
    lastSyncedAt: new Date().toISOString(),
    errorMessage: null,
  }
  await storage.storeSyncEntry(entry)
}

export async function testMegaConnection(email: string, password: string, secondFactorCode?: string): Promise<boolean> {
  if (!email || !password) throw new Error('Email and password are required')
  try {
    const { Storage } = await import('megajs')
    const storage = new Storage({ email, password, secondFactorCode, autoload: false, autologin: true, keepalive: false })
    await storage.ready
    try { storage.close() } catch {}
    return true
  } catch (e) {
    throw new Error(`Mega connection failed: ${e instanceof Error ? e.message : String(e)}`)
  }
}
