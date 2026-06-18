// Cybermanju Drive — Tauri IPC Composable
// Supports both Tauri desktop (IPC) and Server/Web (REST) modes
//
// In Tauri mode: delegates to @tauri-apps/api/core invoke
// In Web mode: calls the Web Dashboard REST API (port 3456 by default)

import type { FileNode } from '@/types'

// ── Module-level connection state ─────────────────────────────

let _serverUrl = ''
let _authToken = ''

/** Configure the Web Dashboard REST API base URL. */
export function setServerUrl(url: string): void {
  _serverUrl = url.replace(/\/+$/, '')
}

/** Configure the Bearer token for ZimaOS JWT auth. */
export function setAuthToken(token: string): void {
  _authToken = token
}

/** Read the current server URL (for diagnostics). */
export function getServerUrl(): string {
  return _serverUrl
}

// ── Environment Detection ────────────────────────────────────

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

export function isWebMode(): boolean {
  return !isTauri()
}

// ── REST helpers ─────────────────────────────────────────────

/** Resolve the base URL for REST calls. */
function getBaseUrl(): string {
  if (_serverUrl) return _serverUrl
  // If the app is served directly from the web dashboard, use same origin
  if (typeof window !== 'undefined' && window.location?.origin) {
    const port = window.location.port
    // Port 3456 is the web dashboard — use same origin
    if (port === '3456') return window.location.origin
  }
  return 'http://localhost:3456'
}

/** Build headers including optional auth. */
function buildHeaders(): Record<string, string> {
  const h: Record<string, string> = {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  }
  if (_authToken) {
    h['Authorization'] = `Bearer ${_authToken}`
  }
  return h
}

/** Generic REST fetch with proper error handling. */
async function restFetch<T>(method: string, path: string, body?: unknown): Promise<T> {
  const url = `${getBaseUrl()}${path}`
  const init: RequestInit = {
    method,
    headers: buildHeaders(),
  }
  if (body !== undefined) {
    init.body = JSON.stringify(body)
  }

  let res: Response
  try {
    res = await fetch(url, init)
  } catch (err) {
    throw new Error(
      `Network error calling ${method} ${path}: ${err instanceof Error ? err.message : String(err)}`
    )
  }

  if (!res.ok) {
    let message = `HTTP ${res.status} ${res.statusText}`
    try {
      const errBody = await res.json()
      if (errBody?.message) message = errBody.message
      else if (errBody?.error) message = `${res.status}: ${errBody.error}`
    } catch {
      // ignore parse failure
    }
    throw new Error(message)
  }

  // 204 No Content
  if (res.status === 204) return undefined as T

  return res.json() as Promise<T>
}

// ── Response key transformation ──────────────────────────────
// The REST API returns raw redb JSON. Field names may be snake_case
// (Rust default). The Tauri IPC layer uses serde camelCase renaming.
// We convert snake_case → camelCase so responses match TypeScript types.
// The `_key` and `_raw` fields added by list_all_json are stripped.

function toCamelCase(s: string): string {
  return s.replace(/_([a-z])/g, (_, ch: string) => ch.toUpperCase())
}

function transformResponseKeys(value: unknown): unknown {
  if (value === null || value === undefined || typeof value !== 'object') return value
  if (Array.isArray(value)) return value.map(transformResponseKeys)

  const src = value as Record<string, unknown>
  const out: Record<string, unknown> = {}
  for (const [key, val] of Object.entries(src)) {
    // Strip _raw field (unparseable fallback from list_all_json)
    if (key === '_raw') continue
    // Promote _key → id when the object lacks its own id
    if (key === '_key') {
      if (!('id' in src)) {
        out['id'] = val
      }
      continue
    }
    out[toCamelCase(key)] = transformResponseKeys(val)
  }
  return out
}

// ── REST command mapping ────────────────────────────────────
// Maps Tauri IPC command names → Web Dashboard REST endpoints.

interface RestMapping {
  method: string
  buildPath: (args: Record<string, unknown>) => string
  transformRequest?: (args: Record<string, unknown>) => unknown
  transformResponse?: (raw: unknown, args: Record<string, unknown>) => unknown
}

const REST_ROUTES: Record<string, RestMapping> = {
  // ── Files ─────────────────────────────────────────────────
  list_files: {
    method: 'GET',
    buildPath: () => '/api/files',
    transformResponse: (raw, args) => {
      let files = transformResponseKeys(raw) as FileNode[]
      // The REST API returns ALL files — filter by parentPath if provided
      const parentPath = args.parentPath as string | undefined
      if (parentPath) {
        files = files.filter(f => {
          if (f.parentId) return f.parentId === parentPath
          if (f.path) {
            // Match files whose path starts with parentPath and have no deeper separator
            const prefix = parentPath === '/' ? '/' : `${parentPath}/`
            return f.path.startsWith(prefix) && !f.path.slice(prefix.length).includes('/')
          }
          return false
        })
      }
      return files
    },
  },

  get_file: {
    method: 'GET',
    buildPath: (args) => `/api/files/${args.fileId}`,
  },

  delete_file: {
    method: 'DELETE',
    buildPath: (args) => `/api/files/${args.fileId}`,
  },

  search_files: {
    method: 'GET',
    buildPath: (args) => `/api/search?q=${encodeURIComponent(String(args.query ?? ''))}`,
    transformResponse: (raw) => {
      // REST returns raw file objects; map to SearchResult shape
      const items = transformResponseKeys(raw) as Array<Record<string, unknown>>
      return items.map(item => ({
        fileId: item.id ?? '',
        fileName: item.name ?? '',
        score: 1.0,
        snippet: '',
      }))
    },
  },

  get_geo_files: {
    method: 'GET',
    buildPath: () => '/api/geo-files',
    transformResponse: (raw) => {
      const items = transformResponseKeys(raw) as Array<Record<string, unknown>>
      return items
        .filter(f => f.gpsLat != null && f.gpsLon != null)
        .map(f => ({
          id: f.id,
          name: f.name,
          gpsLat: f.gpsLat as number,
          gpsLon: f.gpsLon as number,
        }))
    },
  },

  // ── Accounts ──────────────────────────────────────────────
  list_accounts: {
    method: 'GET',
    buildPath: () => '/api/accounts',
  },

  // ── Collections ───────────────────────────────────────────
  list_collections: {
    method: 'GET',
    buildPath: () => '/api/collections',
  },

  get_collection_items: {
    method: 'GET',
    buildPath: () => '/api/collection-items',
  },

  // ── Face groups ───────────────────────────────────────────
  list_face_groups: {
    method: 'GET',
    buildPath: () => '/api/face-groups',
  },

  // ── Loose groups ──────────────────────────────────────────
  list_loose_groups: {
    method: 'GET',
    buildPath: () => '/api/loose-groups',
  },

  // ── Encryption ────────────────────────────────────────────
  get_encryption_status: {
    method: 'GET',
    buildPath: () => '/api/encryption/status',
    transformResponse: (raw) => {
      const data = transformResponseKeys(raw) as Record<string, unknown>
      return {
        isEncrypted: false,
        algorithm: undefined,
        nistLevel: undefined,
        keyId: undefined,
        encryptedAt: undefined,
        // Include extra info from the REST response
        available: data.available ?? false,
        supportedAlgorithms: data.supportedAlgorithms ?? [],
        engine: data.engine ?? '',
      }
    },
  },

  list_keys: {
    method: 'GET',
    buildPath: () => '/api/encryption/keys',
  },

  // ── User management ───────────────────────────────────────
  list_users: {
    method: 'GET',
    buildPath: () => '/api/users',
  },

  authenticate_user: {
    method: 'POST',
    buildPath: () => '/api/users/login',
    transformRequest: (args) => ({
      username: args.username,
      password: args.password,
    }),
    transformResponse: (raw) => transformResponseKeys(raw),
  },

  register_user: {
    method: 'POST',
    buildPath: () => '/api/users/register',
    transformRequest: (args) => ({
      username: args.username,
      password: args.password,
      displayName: args.displayName,
      role: args.role,
    }),
    transformResponse: (raw) => transformResponseKeys(raw),
  },

  // ── Permissions ───────────────────────────────────────────
  get_file_permissions: {
    method: 'GET',
    buildPath: (args) => `/api/permissions/${args.fileId}`,
  },

  grant_file_permission: {
    method: 'POST',
    buildPath: () => '/api/permissions',
    transformRequest: (args) => ({
      userId: args.userId,
      fileId: args.fileId,
      access: args.access,
    }),
  },

  verify_file_access: {
    method: 'POST',
    buildPath: () => '/api/permissions/verify',
    transformRequest: (args) => ({
      userId: args.userId,
      fileId: args.fileId,
      requiredAccess: args.requiredAccess,
    }),
  },

  // ── Locations ─────────────────────────────────────────────
  list_locations: {
    method: 'GET',
    buildPath: () => '/api/locations',
  },

  // ── Dashboard ─────────────────────────────────────────────
  dashboard_status: {
    method: 'GET',
    buildPath: () => '/api/health',
    transformResponse: (raw) => {
      const data = transformResponseKeys(raw) as Record<string, unknown>
      return {
        running: data.status === 'ok',
        port: 3456,
        url: getBaseUrl(),
        activeConnections: 0,
        service: data.service,
        timestamp: data.timestamp,
      }
    },
  },
}

// ── WASM Bridge fallback ───────────────────────────────────
// For commands that can be handled locally via the WASM bridge + IndexedDB

async function tryWasmInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  let wasm: { initWasm: Function; drive: any; sync: any; crypto: any }
  try {
    wasm = await import('@/wasm')
    await wasm.initWasm()
  } catch (err) {
    console.warn(`[Web Mode] WASM bridge unavailable for "${cmd}": ${err}. Falling through to REST.`)
    return null
  }

  const { drive, sync, crypto } = wasm

  try {
    switch (cmd) {
      // ── File operations via WASM drive ────────────────────
      case 'create_folder': {
        const name = args?.name as string
        const parentId = args?.parentId as string | undefined
        const folder = await drive.createFolder(name, parentId ?? null)
        return folder as unknown as T
      }
      case 'rename_file': {
        const fileId = args?.fileId as string
        const newName = args?.newName as string
        await drive.renameDriveFile(fileId, newName)
        return undefined as T
      }
      case 'delete_file': {
        const fileId = args?.fileId as string
        await drive.deleteDriveFile(fileId)
        return undefined as T
      }
      case 'list_files': {
        const parentPath = args?.parentPath as string | undefined
        const allFiles = await drive.getAllDriveFiles()
        const fileNodes = await drive.toFileNodes(
          parentPath
            ? allFiles.filter((f: any) => f.parentId === parentPath || (parentPath === '/' && !f.parentId))
            : allFiles
        )
        return fileNodes as unknown as T
      }
      case 'get_file': {
        const fileId = args?.fileId as string
        const file = await drive.getDriveFile(fileId)
        if (!file) throw new Error('File not found')
        const nodes = await drive.toFileNodes([file])
        return nodes[0] as unknown as T
      }
      case 'search_files':
      case 'search_files_paginated': {
        const query = args?.query as string
        const results = await drive.searchDriveFiles(query || '')
        return results as unknown as T
      }
      case 'get_geo_files': {
        const geoFiles = await drive.getGeoFiles()
        return geoFiles as unknown as T
      }

      // ── Accounts (persisted in IndexedDB via data module) ─
      case 'list_accounts': {
        const data = await import('@/wasm/data')
        return (await data.listAccounts()) as unknown as T
      }
      case 'create_account': {
        const data = await import('@/wasm/data')
        const { name, accountType: act, path, color } = args ?? {}
        return (await data.createAccount({
          name: name as string,
          accountType: (act as any) || 'local',
          path: path as string | undefined,
          color: color as string | undefined,
        })) as unknown as T
      }
      case 'switch_account': {
        const data = await import('@/wasm/data')
        const accountId = args?.accountId as string
        await data.setActiveAccount(accountId)
        return undefined as T
      }

      // ── Collections (persisted in IndexedDB) ────────────
      case 'list_collections': {
        const data = await import('@/wasm/data')
        return (await data.listCollections()) as unknown as T
      }
      case 'get_collection_items': {
        const data = await import('@/wasm/data')
        return (await data.listCollections()) as unknown as T
      }
      case 'create_collection': {
        const data = await import('@/wasm/data')
        const { name, collectionType: colType, color, description } = args ?? {}
        return (await data.createCollection({
          name: name as string,
          collectionType: (colType as string) || 'custom',
          color: color as string | undefined,
          description: description as string | undefined,
        })) as unknown as T
      }
      case 'add_to_collection': {
        const data = await import('@/wasm/data')
        const { collectionId, fileId } = args ?? {}
        await data.addToCollection(collectionId as string, fileId as string)
        return undefined as T
      }
      case 'remove_from_collection': {
        const data = await import('@/wasm/data')
        const { collectionId, fileId } = args ?? {}
        await data.removeFromCollection(collectionId as string, fileId as string)
        return undefined as T
      }

      // ── Face groups (persisted in IndexedDB) ────────────
      case 'list_face_groups': {
        const data = await import('@/wasm/data')
        return (await data.listFaceGroups()) as unknown as T
      }
      case 'list_loose_groups': {
        const data = await import('@/wasm/data')
        return (await data.listLooseGroups()) as unknown as T
      }
      case 'rename_face_group': {
        const data = await import('@/wasm/data')
        const groupId = args?.group_id as string
        const newName = args?.new_name as string
        const groups = await data.listFaceGroups()
        const group = groups.find(g => g.id === groupId)
        if (!group) throw new Error('Face group not found')
        const updated = await data.updateFaceGroup(groupId, { name: newName })
        return updated as unknown as T
      }

      // ── Encryption (persisted in IndexedDB) ─────────────
      case 'get_encryption_status': {
        const data = await import('@/wasm/data')
        return (await data.getEncryptionStatus()) as unknown as T
      }
      case 'list_keys': {
        const data = await import('@/wasm/data')
        return (await data.listEncryptionKeys()) as unknown as T
      }

      // ── Users (persisted in IndexedDB) ──────────────────
      case 'list_users': {
        const data = await import('@/wasm/data')
        return (await data.listUsers()) as unknown as T
      }
      case 'register_user': {
        const data = await import('@/wasm/data')
        const { username, password, displayName, role } = args ?? {}
        return (await data.createUser({
          username: username as string,
          password: password as string,
          displayName: displayName as string | undefined,
          role: role as string | undefined,
        })) as unknown as T
      }
      case 'authenticate_user': {
        const data = await import('@/wasm/data')
        const { username, password } = args ?? {}
        return (await data.authenticateUser(username as string, password as string)) as unknown as T
      }

      // ── Dashboard status ────────────────────────────────
      case 'dashboard_status': {
        return { running: false, port: 0, url: window.location.origin, activeConnections: 0, service: 'wasm-bridge', timestamp: new Date().toISOString() } as unknown as T
      }

      // ── Locations ───────────────────────────────────────
      case 'list_locations': {
        const data = await import('@/wasm/data')
        return (await data.listLocations()) as unknown as T
      }

      // ── Sync operations via WASM sync ────────────────────
      case 'list_sync_configs': {
        const configs = await sync.getSyncConfigs()
        return configs as unknown as T
      }
      case 'create_sync_config': {
        const config = args?.config as any
        const created = await sync.saveSyncConfig(config)
        return created as unknown as T
      }
      case 'delete_sync_config': {
        const configId = args?.configId as string
        await sync.deleteSyncConfig(configId)
        return undefined as T
      }
      case 'start_sync': {
        const configId = args?.configId as string
        const fileIds = args?.fileIds as string[]
        const configs = await sync.getSyncConfigs()
        const config = configs.find((c: any) => c.id === configId)
        if (!config) throw new Error('Sync config not found')
        sync.startSync(config, fileIds).catch(console.error)
        return undefined as T
      }
      case 'cancel_sync': {
        sync.cancelSync()
        return undefined as T
      }
      case 'get_sync_progress': {
        return sync.getProgress() as unknown as T
      }
      case 'list_remote_files': {
        const config = args?.config as any
        const prefix = args?.prefix as string
        const files = await sync.listRemoteFiles(config, prefix || '')
        return files as unknown as T
      }
      case 'test_sync_connection': {
        const config = args?.config as any
        if (config?.backendType === 'mega') {
          const token = (config?.token || '') as string
          const parts = token.split('|')
          const secondFactorCode = config?.secondFactorCode as string | undefined
          if (parts.length === 2) {
            return await sync.testMegaConnection(parts[0], parts[1], secondFactorCode) as unknown as T
          }
        }
        return true as unknown as T
      }

      // ── Encryption via WASM crypto ───────────────────────
      case 'chacha20_encrypt': {
        const key = new Uint8Array(args?.key as number[])
        const nonce = new Uint8Array(args?.nonce as number[])
        const plaintext = new Uint8Array(args?.plaintext as number[])
        return (await crypto.encryptData(key, nonce, plaintext)) as unknown as T
      }
      case 'chacha20_decrypt': {
        const key = new Uint8Array(args?.key as number[])
        const nonce = new Uint8Array(args?.nonce as number[])
        const ciphertext = new Uint8Array(args?.ciphertext as number[])
        return (await crypto.decryptData(key, nonce, ciphertext)) as unknown as T
      }

      // ── KV Store (persisted in IndexedDB) ─────────────────
      case 'kv_set': {
        const storage = await import('@/wasm/storage')
        await storage.kvSet(args?.key as string, args?.value as string)
        return undefined as T
      }
      case 'kv_get': {
        const storage = await import('@/wasm/storage')
        const val = await storage.kvGet<string>(args?.key as string)
        return val as unknown as T
      }

      // ── Auth (requires REST, stubs for WASM) ──────────────
      case 'authenticate_user': {
        return null // fall through to REST
      }
      case 'register_user': {
        return null // fall through to REST
      }
    }
  } catch (err) {
    if (cmd in REST_ROUTES) {
      console.warn(`[Web Mode] WASM bridge failed for "${cmd}": ${err}. Falling through to REST.`)
      return null
    }
    throw err
  }
  return null
}

function isMegaTestConnection(cmd: string, args?: Record<string, unknown>): boolean {
  return cmd === 'test_sync_connection' && (args?.config as any)?.backendType === 'mega'
}

/** The core invoke — works in Tauri, Web REST, and WASM bridge modes. */
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    const core = await import('@tauri-apps/api/core')
    try {
      return await core.invoke<T>(cmd, args)
    } catch (rustError) {
      // For mega login, fall back to WASM bridge if the Rust backend fails
      // (e.g. 2FA not supported by megalib, or API incompatibility)
      if (isMegaTestConnection(cmd, args)) {
        console.warn(`[Tauri] Rust backend failed for mega login: ${rustError}. Falling back to WASM bridge.`)
        try {
          const wasmResult = await tryWasmInvoke<T>(cmd, args)
          if (wasmResult !== null) return wasmResult
        } catch {
          // WASM also failed — throw the original Rust error
        }
      }
      throw rustError
    }
  }

  // ── Try WASM bridge first (fast, local, no network needed) ──
  if (isWebMode()) {
    const wasmResult = await tryWasmInvoke<T>(cmd, args)
    if (wasmResult !== null) return wasmResult
  }

  // ── Web / REST path ────────────────────────────────────
  const mapping = REST_ROUTES[cmd]
  if (mapping) {
    try {
      const path = mapping.buildPath(args ?? {})
      let body: unknown = undefined
      if (mapping.transformRequest && args) {
        body = mapping.transformRequest(args)
      } else if (mapping.method !== 'GET' && mapping.method !== 'HEAD' && args) {
        body = args
      }
      const raw = await restFetch<unknown>(mapping.method, path, body)
      if (mapping.transformResponse) {
        return (mapping.transformResponse(raw, args ?? {})) as T
      }
      return transformResponseKeys(raw) as T
    } catch (err) {
      // If REST fails and this is a GET/read-only command, return empty defaults
      const isReadCmd = cmd.startsWith('list_') || cmd.startsWith('get_') || cmd.startsWith('search_')
      if (isReadCmd) {
        console.warn(`[Web Mode] REST fallback failed for "${cmd}": ${err}. Returning empty default.`)
        return [] as unknown as T
      }
      throw err
    }
  }

  // Unsupported command in web mode
  console.warn(`[Web Mode] Command "${cmd}" is not supported via REST or WASM bridge.`)
  // Return empty defaults for read/list commands instead of throwing
  if (cmd.startsWith('list_') || cmd.startsWith('get_') || cmd.startsWith('search_')) {
    return [] as unknown as T
  }
  throw new Error(
    `[Web Mode] Command "${cmd}" is not supported. The WASM bridge and Web Dashboard REST API do not provide this endpoint.`
  )
}

// ── KV Store convenience ────────────────────────────────────

/** Persist a key-value pair (routes to Rust redb or WASM IndexedDB). */
export async function kvSet(key: string, value: string): Promise<void> {
  await invoke('kv_set', { key, value })
}

/** Retrieve a value by key (routes to Rust redb or WASM IndexedDB). */
export async function kvGet(key: string): Promise<string | undefined> {
  const val = await invoke<string | null>('kv_get', { key })
  return val ?? undefined
}

// ── Composable ──────────────────────────────────────────────

// ── Native FS helper ─────────────────────────────────────

async function getNativeFs() {
  return import('@/wasm/native-fs')
}

async function tryNativeFsReadDirectory(rootPath: string): Promise<FileNode[] | null> {
  try {
    const nativeFs = await getNativeFs()
    const rootHandle = await nativeFs.getPersistedHandle()
    if (!rootHandle) return null

    const dirHandle = rootPath === '/' || !rootPath
      ? rootHandle
      : await nativeFs.resolveHandle(rootHandle, rootPath)

    if (!dirHandle || dirHandle.kind !== 'directory') return null

    const entries = await nativeFs.listDirectory(dirHandle as FileSystemDirectoryHandle, false)
    return entries.map((e, i) => ({
      id: `native-${e.path}`,
      name: e.name,
      fileType: e.kind,
      parentId: e.parentPath || undefined,
      path: e.path,
      sizeBytes: e.size,
      mimeType: e.mimeType,
      encrypted: false,
      compressionLayers: [],
      createdAt: e.modifiedAt,
      modifiedAt: e.modifiedAt,
    }))
  } catch {
    return null
  }
}

async function tryNativeFsReadText(relPath: string): Promise<string | null> {
  try {
    const nativeFs = await getNativeFs()
    const rootHandle = await nativeFs.getPersistedHandle()
    if (!rootHandle) return null
    const handle = await nativeFs.resolveHandle(rootHandle, relPath)
    if (!handle || handle.kind !== 'file') return null
    return nativeFs.readFileText(handle as FileSystemFileHandle)
  } catch {
    return null
  }
}

async function tryNativeFsWriteText(relPath: string, contents: string): Promise<boolean> {
  try {
    const nativeFs = await getNativeFs()
    const rootHandle = await nativeFs.getPersistedHandle()
    if (!rootHandle) return false
    const handle = await nativeFs.resolveHandle(rootHandle, relPath)
    if (!handle || handle.kind !== 'file') return false
    await nativeFs.writeFileText(handle as FileSystemFileHandle, contents)
    return true
  } catch {
    return false
  }
}

async function tryNativeFsCreateDir(relPath: string): Promise<boolean> {
  try {
    const nativeFs = await getNativeFs()
    const rootHandle = await nativeFs.getPersistedHandle()
    if (!rootHandle) return false
    const parts = relPath.split('/').filter(Boolean)
    if (parts.length === 0) return false
    const name = parts.pop()!
    const parentPath = parts.join('/')
    const parent = parentPath
      ? await nativeFs.resolveHandle(rootHandle, parentPath)
      : rootHandle
    if (!parent || parent.kind !== 'directory') return false
    await nativeFs.createDirectory(parent as FileSystemDirectoryHandle, name)
    return true
  } catch {
    return false
  }
}

async function tryNativeFsDelete(relPath: string): Promise<boolean> {
  try {
    const nativeFs = await getNativeFs()
    const rootHandle = await nativeFs.getPersistedHandle()
    if (!rootHandle) return false
    const parts = relPath.split('/').filter(Boolean)
    if (parts.length === 0) return false
    const name = parts.pop()!
    const parentPath = parts.join('/')
    const parent = parentPath
      ? await nativeFs.resolveHandle(rootHandle, parentPath)
      : rootHandle
    if (!parent || parent.kind !== 'directory') return false
    await nativeFs.removeEntry(parent as FileSystemDirectoryHandle, name, true)
    return true
  } catch {
    return false
  }
}

async function tryNativeFsExists(relPath: string): Promise<boolean> {
  try {
    const nativeFs = await getNativeFs()
    const rootHandle = await nativeFs.getPersistedHandle()
    if (!rootHandle) return false
    const handle = await nativeFs.resolveHandle(rootHandle, relPath)
    return handle !== null
  } catch {
    return false
  }
}

export function useTauri() {
  async function pickFolder(): Promise<string | null> {
    if (isWebMode()) {
      const nativeFs = await getNativeFs()
      if (nativeFs.isSupported()) {
        const handle = await nativeFs.openDirectory('readwrite')
        return handle ? handle.name : null
      }
      // Fallback: webkitdirectory (returns first folder name)
      return null
    }
    const { open } = await import('@tauri-apps/plugin-dialog')
    const result = await open({ directory: true, multiple: false })
    return result as string | null
  }

  async function pickFiles(multiple = false): Promise<string[] | null> {
    if (isWebMode()) {
      const nativeFs = await getNativeFs()
      if (nativeFs.isSupported()) {
        const handles = await nativeFs.openFiles(multiple)
        return handles.map(h => h.name)
      }
      return null
    }
    const { open } = await import('@tauri-apps/plugin-dialog')
    const result = await open({ directory: false, multiple })
    return result as string[] | null
  }

  async function readDirectory(path: string): Promise<FileNode[]> {
    if (isWebMode()) {
      // 1) Native FS (real filesystem via File System Access API)
      const nativeResult = await tryNativeFsReadDirectory(path)
      if (nativeResult !== null) return nativeResult

      // 2) WASM drive (IndexedDB virtual FS)
      try {
        const { drive, isWasmReady } = await import('@/wasm')
        if (isWasmReady()) {
          const allFiles = await drive.getAllDriveFiles()
          const filtered = allFiles.filter(f => {
            if (!path || path === '/') return !f.parentId
            return f.parentId === path
          })
          return drive.toFileNodes(filtered)
        }
      } catch { /* fall through */ }

      // 3) REST API
      try {
        const allFiles = await invoke<FileNode[]>('list_files', {})
        return allFiles
          .filter(f => {
            if (!f.path) return false
            const dir = f.path.substring(0, f.path.lastIndexOf('/')) || '/'
            return dir === path || (path === '/' && !f.path.includes('/'))
          })
          .map((f, i) => ({
            ...f,
            id: f.id || `web-${i}-${f.name}`,
          }))
      } catch {
        return []
      }
    }
    const { readDir } = await import('@tauri-apps/plugin-fs')
    const entries = await readDir(path)
    return entries.map((entry, i) => ({
      id: `local-${i}-${entry.name}`,
      name: entry.name,
      path: `${path}/${entry.name}`,
      fileType: (entry as unknown as { isDirectory: boolean }).isDirectory ? 'folder' as const : 'file' as const,
      sizeBytes: 0,
      encrypted: false,
      compressionLayers: [],
      createdAt: new Date().toISOString(),
      modifiedAt: new Date().toISOString(),
    }))
  }

  async function readTextFile(pathOrId: string): Promise<string> {
    if (isWebMode()) {
      // 1) Native FS (real file, synced passthrough)
      const nativeResult = await tryNativeFsReadText(pathOrId)
      if (nativeResult !== null) return nativeResult

      // 2) WASM drive (IndexedDB)
      try {
        const { drive, isWasmReady } = await import('@/wasm')
        if (isWasmReady()) {
          const text = await drive.readFileText(pathOrId)
          if (text !== null) return text
        }
      } catch { /* fall through */ }
      throw new Error('[Web Mode] File not found.')
    }
    const { readFile } = await import('@tauri-apps/plugin-fs')
    return await readFile(pathOrId) as unknown as string
  }

  async function writeTextFile(pathOrId: string, contents: string): Promise<void> {
    if (isWebMode()) {
      // 1) Native FS (writes directly to original file — synced passthrough)
      const nativeWritten = await tryNativeFsWriteText(pathOrId, contents)
      if (nativeWritten) return

      // 2) WASM drive (IndexedDB)
      try {
        const { drive, isWasmReady } = await import('@/wasm')
        if (isWasmReady()) {
          const data = new TextEncoder().encode(contents).buffer
          await drive.updateFileData(pathOrId, data)
          return
        }
      } catch { /* fall through */ }
      throw new Error('[Web Mode] File write failed.')
    }
    const { writeFile } = await import('@tauri-apps/plugin-fs')
    await writeFile(pathOrId, new TextEncoder().encode(contents))
  }

  async function createDir(name: string): Promise<void> {
    if (isWebMode()) {
      // 1) Native FS
      const nativeCreated = await tryNativeFsCreateDir(name)
      if (nativeCreated) return

      // 2) WASM drive
      try {
        const { drive, isWasmReady } = await import('@/wasm')
        if (isWasmReady()) {
          await drive.createFolder(name, null)
          return
        }
      } catch { /* fall through */ }
      throw new Error('[Web Mode] Directory creation failed.')
    }
    const { mkdir } = await import('@tauri-apps/plugin-fs')
    await mkdir(name, { recursive: true })
  }

  async function deletePath(path: string): Promise<void> {
    if (isWebMode()) {
      // 1) Native FS
      const nativeDeleted = await tryNativeFsDelete(path)
      if (nativeDeleted) return

      // 2) WASM drive
      try {
        const { drive, isWasmReady } = await import('@/wasm')
        if (isWasmReady()) {
          await drive.deleteDriveFile(path)
          return
        }
      } catch { /* fall through */ }
      throw new Error('[Web Mode] Deletion failed.')
    }
    const { remove } = await import('@tauri-apps/plugin-fs')
    await remove(path)
  }

  async function renamePath(oldPath: string, newPath: string): Promise<void> {
    if (isWebMode()) {
      // Native FS rename not directly supported (move via read+write+delete)
      // Fall through to WASM drive
      try {
        const { drive, isWasmReady } = await import('@/wasm')
        if (isWasmReady()) {
          await drive.renameDriveFile(oldPath, newPath)
          return
        }
      } catch { /* fall through */ }
      throw new Error('[Web Mode] Rename failed.')
    }
    const { rename } = await import('@tauri-apps/plugin-fs')
    await rename(oldPath, newPath)
  }

  async function copyPath(src: string, dest: string): Promise<void> {
    if (isWebMode()) {
      // Native FS: read from source, write to dest
      try {
        const nativeFs = await getNativeFs()
        const rootHandle = await nativeFs.getPersistedHandle()
        if (rootHandle) {
          const srcHandle = await nativeFs.resolveHandle(rootHandle, src)
          if (srcHandle && srcHandle.kind === 'file') {
            const data = await nativeFs.readFile(srcHandle as FileSystemFileHandle)
            const destHandle = await nativeFs.resolveHandle(rootHandle, dest)
            if (destHandle && destHandle.kind === 'file') {
              await nativeFs.writeFile(destHandle as FileSystemFileHandle, data)
              return
            }
          }
        }
      } catch { /* fall through */ }

      // WASM drive fallback
      try {
        const { drive, isWasmReady } = await import('@/wasm')
        if (isWasmReady()) {
          const data = await drive.readFileData(src)
          if (data) {
            const file = await drive.getDriveFile(src)
            if (file) {
              await drive.addFile(file.name, data, null, file.mimeType || undefined)
              return
            }
          }
        }
      } catch { /* fall through */ }
      throw new Error('[Web Mode] Copy failed.')
    }
    const { copyFile } = await import('@tauri-apps/plugin-fs')
    await copyFile(src, dest)
  }

  async function pathExists(id: string): Promise<boolean> {
    if (isWebMode()) {
      // 1) Native FS
      const nativeExists = await tryNativeFsExists(id)
      if (nativeExists) return true

      // 2) WASM drive
      try {
        const { drive, isWasmReady } = await import('@/wasm')
        if (isWasmReady()) {
          return !!(await drive.getDriveFile(id))
        }
      } catch { /* fall through */ }
      // 3) REST API
      try {
        await restFetch<unknown>('GET', `/api/files`)
        return true
      } catch {
        return false
      }
    }
    const { exists } = await import('@tauri-apps/plugin-fs')
    return await exists(id)
  }

  return {
    invoke,
    pickFolder,
    pickFiles,
    readDirectory,
    readTextFile,
    writeTextFile,
    createDir,
    deletePath,
    renamePath,
    copyPath,
    pathExists,
    isTauri,
    isWebMode,
  }
}