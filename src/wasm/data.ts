import { initWasm, getWasm } from './bridge'
import { kvSet, kvGet } from './storage'
import type { OAuthProvider, OAuthToken } from './oauth'
import { saveTokenToStorage, loadTokenFromStorage } from './oauth'

// ── Types ─────────────────────────────────────────────────

export interface WasmAccount {
  id: string
  name: string
  accountType: 'local' | 'cloud' | 'network'
  backendType?: string
  path?: string
  color: string
  isActive: boolean
  oauthProvider?: OAuthProvider
  oauthEmail?: string
  createdAt: string
  updatedAt: string
}

export interface WasmCollection {
  id: string
  name: string
  collectionType: string
  color: string
  description?: string
  itemIds: string[]
  createdAt: string
  updatedAt: string
}

export interface WasmFaceGroup {
  id: string
  name: string
  color?: string
  icon?: string
  fileIds: string[]
  embeddingCount: number
  algorithm?: string
  createdAt: string
}

export interface WasmLooseGroup {
  id: string
  name: string
  color: string
  icon?: string
  fileIds: string[]
  createdAt: string
}

export interface WasmEncryptionKey {
  id: string
  algorithm: string
  label?: string
  publicKey: string
  hasPrivateKey: boolean
  createdAt: string
}

// ── Storage keys ──────────────────────────────────────────

const KEYS = {
  accounts: 'wasm:accounts',
  collections: 'wasm:collections',
  collectionItems: 'wasm:collectionItems',
  faceGroups: 'wasm:faceGroups',
  looseGroups: 'wasm:looseGroups',
  encryptionKeys: 'wasm:encryptionKeys',
  encryptionStatus: 'wasm:encryptionStatus',
  users: 'wasm:users',
  locations: 'wasm:locations',
} as const

// ── Generic helpers ───────────────────────────────────────

async function init() {
  await initWasm()
}

function now(): string {
  return getWasm().now_utc()
}

function uuid(): string {
  return getWasm().generate_uuid()
}

// ── Accounts ──────────────────────────────────────────────
export async function listAccounts(): Promise<WasmAccount[]> {
  await init()
  return (await kvGet<WasmAccount[]>(KEYS.accounts)) || []
}

export async function getAccount(id: string): Promise<WasmAccount | null> {
  const all = await listAccounts()
  return all.find(a => a.id === id) || null
}

export async function createAccount(data: {
  name: string
  accountType: 'local' | 'cloud' | 'network'
  backendType?: string
  path?: string
  color?: string
  oauthProvider?: OAuthProvider
}): Promise<WasmAccount> {
  await init()
  const all = await listAccounts()
  const account: WasmAccount = {
    id: uuid(),
    name: data.name,
    accountType: data.accountType,
    backendType: data.backendType,
    path: data.path,
    color: data.color || '#FFFFFF',
    isActive: all.length === 0,
    oauthProvider: data.oauthProvider,
    createdAt: now(),
    updatedAt: now(),
  }
  all.push(account)
  await kvSet(KEYS.accounts, all)
  return account
}

export async function updateAccount(id: string, updates: Partial<WasmAccount>): Promise<WasmAccount | null> {
  const all = await listAccounts()
  const idx = all.findIndex(a => a.id === id)
  if (idx === -1) return null
  all[idx] = { ...all[idx], ...updates, updatedAt: now() }
  await kvSet(KEYS.accounts, all)
  return all[idx]
}

export async function deleteAccount(id: string): Promise<void> {
  const all = await listAccounts()
  await kvSet(KEYS.accounts, all.filter(a => a.id !== id))
}

export async function setActiveAccount(id: string): Promise<void> {
  const all = await listAccounts()
  for (const a of all) a.isActive = a.id === id
  await kvSet(KEYS.accounts, all)
}

/** Create or update an account from an OAuth2 token. */
export async function upsertOAuthAccount(
  provider: OAuthProvider,
  token: OAuthToken
): Promise<WasmAccount> {
  const all = await listAccounts()
  const existing = all.find(a => a.oauthProvider === provider)
  if (existing) {
    existing.updatedAt = now()
    existing.isActive = true
    // Deactivate others
    for (const a of all) if (a.id !== existing.id) a.isActive = false
    await kvSet(KEYS.accounts, all)
    return existing
  }
  const account: WasmAccount = {
    id: uuid(),
    name: provider.charAt(0).toUpperCase() + provider.slice(1).replace(/([A-Z])/g, ' $1'),
    accountType: 'cloud',
    backendType: provider,
    color: '#FFFFFF',
    isActive: true,
    oauthProvider: provider,
    createdAt: now(),
    updatedAt: now(),
  }
  for (const a of all) a.isActive = false
  all.push(account)
  await kvSet(KEYS.accounts, all)
  return account
}

// ── Collections ───────────────────────────────────────────
export async function listCollections(): Promise<WasmCollection[]> {
  await init()
  return (await kvGet<WasmCollection[]>(KEYS.collections)) || []
}

export async function createCollection(data: {
  name: string
  collectionType: string
  color?: string
  description?: string
}): Promise<WasmCollection> {
  await init()
  const all = await listCollections()
  const collection: WasmCollection = {
    id: uuid(),
    name: data.name,
    collectionType: data.collectionType,
    color: data.color || '#FFFFFF',
    description: data.description,
    itemIds: [],
    createdAt: now(),
    updatedAt: now(),
  }
  all.push(collection)
  await kvSet(KEYS.collections, all)
  return collection
}

export async function deleteCollection(id: string): Promise<void> {
  const all = await listCollections()
  await kvSet(KEYS.collections, all.filter(c => c.id !== id))
}

export async function addToCollection(collectionId: string, fileId: string): Promise<void> {
  const all = await listCollections()
  const col = all.find(c => c.id === collectionId)
  if (col && !col.itemIds.includes(fileId)) {
    col.itemIds.push(fileId)
    col.updatedAt = now()
    await kvSet(KEYS.collections, all)
  }
}

export async function removeFromCollection(collectionId: string, fileId: string): Promise<void> {
  const all = await listCollections()
  const col = all.find(c => c.id === collectionId)
  if (col) {
    col.itemIds = col.itemIds.filter(id => id !== fileId)
    col.updatedAt = now()
    await kvSet(KEYS.collections, all)
  }
}

// ── Face Groups ───────────────────────────────────────────
export async function listFaceGroups(): Promise<WasmFaceGroup[]> {
  await init()
  return (await kvGet<WasmFaceGroup[]>(KEYS.faceGroups)) || []
}

export async function createFaceGroup(name: string, fileIds?: string[]): Promise<WasmFaceGroup> {
  await init()
  const all = await listFaceGroups()
  const group: WasmFaceGroup = {
    id: uuid(),
    name,
    fileIds: fileIds || [],
    embeddingCount: 0,
    createdAt: now(),
  }
  all.push(group)
  await kvSet(KEYS.faceGroups, all)
  return group
}

export async function deleteFaceGroup(id: string): Promise<void> {
  const all = await listFaceGroups()
  await kvSet(KEYS.faceGroups, all.filter(g => g.id !== id))
}

export async function updateFaceGroup(id: string, updates: Partial<WasmFaceGroup>): Promise<WasmFaceGroup | null> {
  const all = await listFaceGroups()
  const idx = all.findIndex(g => g.id === id)
  if (idx === -1) return null
  all[idx] = { ...all[idx], ...updates }
  await kvSet(KEYS.faceGroups, all)
  return all[idx]
}

export async function addFileToFaceGroup(groupId: string, fileId: string): Promise<void> {
  const all = await listFaceGroups()
  const g = all.find(g => g.id === groupId)
  if (g && !g.fileIds.includes(fileId)) {
    g.fileIds.push(fileId)
    await kvSet(KEYS.faceGroups, all)
  }
}

// ── Loose Groups ──────────────────────────────────────────
export async function listLooseGroups(): Promise<WasmLooseGroup[]> {
  await init()
  return (await kvGet<WasmLooseGroup[]>(KEYS.looseGroups)) || []
}

export async function createLooseGroup(name: string, color?: string): Promise<WasmLooseGroup> {
  await init()
  const all = await listLooseGroups()
  const group: WasmLooseGroup = {
    id: uuid(),
    name,
    color: color || '#FFFFFF',
    fileIds: [],
    createdAt: now(),
  }
  all.push(group)
  await kvSet(KEYS.looseGroups, all)
  return group
}

export async function deleteLooseGroup(id: string): Promise<void> {
  const all = await listLooseGroups()
  await kvSet(KEYS.looseGroups, all.filter(g => g.id !== id))
}

// ── Encryption Keys ───────────────────────────────────────
export async function listEncryptionKeys(): Promise<WasmEncryptionKey[]> {
  await init()
  return (await kvGet<WasmEncryptionKey[]>(KEYS.encryptionKeys)) || []
}

export async function createEncryptionKey(data: {
  algorithm: string
  label?: string
  publicKey: string
  hasPrivateKey?: boolean
}): Promise<WasmEncryptionKey> {
  await init()
  const all = await listEncryptionKeys()
  const key: WasmEncryptionKey = {
    id: uuid(),
    algorithm: data.algorithm,
    label: data.label,
    publicKey: data.publicKey,
    hasPrivateKey: data.hasPrivateKey ?? false,
    createdAt: now(),
  }
  all.push(key)
  await kvSet(KEYS.encryptionKeys, all)
  return key
}

export async function getEncryptionStatus(): Promise<{
  isEncrypted: boolean
  available: boolean
  supportedAlgorithms: string[]
  engine: string
}> {
  await init()
  const keys = await listEncryptionKeys()
  return {
    isEncrypted: keys.length > 0,
    available: true,
    supportedAlgorithms: ['chacha20-poly1305', 'blake3', 'x25519', 'ml-dsa-65'],
    engine: 'wasm',
  }
}

// ── Locations ───────────────────────────────────────────
export async function listLocations(): Promise<Array<{ id: string; name: string; path: string; isDefault: boolean; createdAt: string }>> {
  await init()
  return (await kvGet(KEYS.locations)) || []
}

export async function createLocation(data: { name: string; path: string; isDefault?: boolean }): Promise<{ id: string; name: string; path: string; isDefault: boolean; createdAt: string }> {
  await init()
  const all = await listLocations()
  const location = {
    id: uuid(),
    name: data.name,
    path: data.path,
    isDefault: data.isDefault || false,
    createdAt: now(),
  }
  all.push(location)
  await kvSet(KEYS.locations, all)
  return location
}

// ── Users (for auth) ──────────────────────────────────────
export async function listUsers(): Promise<Array<{
  id: string; username: string; displayName?: string; role: string; isActive: boolean; createdAt: string
}>> {
  await init()
  return (await kvGet(KEYS.users)) || []
}

export async function createUser(data: {
  username: string
  password: string
  displayName?: string
  role?: string
}): Promise<{ id: string; username: string; displayName?: string; role: string }> {
  await init()
  const all = await listUsers()
  const user = {
    id: uuid(),
    username: data.username,
    displayName: data.displayName || data.username,
    role: data.role || 'user',
    isActive: true,
    createdAt: now(),
  }
  all.push(user)
  await kvSet(KEYS.users, all)
  return { ...user, isActive: undefined, createdAt: undefined } as any
}

export async function authenticateUser(username: string, _password: string): Promise<{
  userId: string; username: string; role: string; displayName?: string; token: string
} | null> {
  await init()
  // Check OAuth2 accounts first
  const accounts = await listAccounts()
  const oauthAccount = accounts.find(a => a.isActive && a.oauthProvider)
  if (oauthAccount) {
    const token = await loadTokenFromStorage(oauthAccount.oauthProvider!)
    if (token) {
      return {
        userId: oauthAccount.id,
        username: oauthAccount.name,
        role: 'user',
        displayName: oauthAccount.name,
        token: token.accessToken,
      }
    }
  }
  // Fall back to local user auth
  const users = await listUsers()
  const user = users.find(u => u.username === username)
  if (!user) return null
  return {
    userId: user.id,
    username: user.username,
    role: user.role,
    displayName: user.displayName,
    token: uuid(),
  }
}
