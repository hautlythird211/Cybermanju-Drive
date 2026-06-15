import { initWasm, getWasm } from './bridge'
import * as storage from './storage'
import type { FileNode } from '@/types'

export interface DriveFile {
  id: string
  name: string
  fileType: 'file' | 'folder'
  parentId: string | null
  sizeBytes: number
  mimeType: string | null
  hash: string | null
  encrypted: boolean
  tags: string[]
  createdAt: string
  modifiedAt: string
  isStarred: boolean
}

export interface DriveQuota {
  usedBytes: number
  totalBytes: number
  fileCount: number
  folderCount: number
}

export async function initDrive(): Promise<void> {
  await initWasm()
  const wasm = getWasm()
  new wasm.VirtualDrive()
}

export async function listDriveFiles(parentId?: string | null): Promise<DriveFile[]> {
  const storedFiles = await storage.listFiles(parentId ?? null)
  return storedFiles.map(toDriveFile)
}

export async function getAllDriveFiles(): Promise<DriveFile[]> {
  const storedFiles = await storage.getAllFiles()
  return storedFiles.map(toDriveFile)
}

export async function createFolder(
  name: string,
  parentId: string | null
): Promise<DriveFile> {
  await initWasm()
  const wasm = getWasm()
  const now = wasm.now_utc()
  const id = wasm.generate_uuid()
  const hash = wasm.hash_file_meta(name, 0, now)

  const driveFile: DriveFile = {
    id,
    name,
    fileType: 'folder',
    parentId,
    sizeBytes: 0,
    mimeType: 'inode/directory',
    hash,
    encrypted: false,
    tags: [],
    createdAt: now,
    modifiedAt: now,
    isStarred: false,
  }
  await storage.storeFile(toStoredFile(driveFile, new ArrayBuffer(0)))
  return driveFile
}

export async function addFile(
  name: string,
  data: ArrayBuffer,
  parentId: string | null,
  mimeType = 'application/octet-stream',
  tags: string[] = []
): Promise<DriveFile> {
  await initWasm()
  const wasm = getWasm()
  const now = wasm.now_utc()
  const id = wasm.generate_uuid()
  const hash = wasm.hash_file_meta(name, data.byteLength, now)

  const driveFile: DriveFile = {
    id,
    name,
    fileType: 'file',
    parentId,
    sizeBytes: data.byteLength,
    mimeType,
    hash,
    encrypted: false,
    tags,
    createdAt: now,
    modifiedAt: now,
    isStarred: false,
  }
  await storage.storeFile(toStoredFile(driveFile, data))
  return driveFile
}

export async function getDriveFile(id: string): Promise<DriveFile | null> {
  const stored = await storage.getFile(id)
  return stored ? toDriveFile(stored) : null
}

export async function readFileData(id: string): Promise<ArrayBuffer | null> {
  const stored = await storage.getFile(id)
  return stored?.data || null
}

export async function readFileText(id: string): Promise<string | null> {
  const data = await readFileData(id)
  if (!data) return null
  return new TextDecoder().decode(data)
}

export async function updateFileData(
  id: string,
  data: ArrayBuffer
): Promise<void> {
  const stored = await storage.getFile(id)
  if (!stored) throw new Error('File not found')
  stored.data = data
  stored.size = data.byteLength
  await storage.storeFile(stored)
}

export async function deleteDriveFile(id: string): Promise<void> {
  await storage.deleteFile(id)
}

export async function renameDriveFile(id: string, newName: string): Promise<void> {
  await storage.renameFile(id, newName)
}

export async function moveDriveFile(
  id: string,
  newParentId: string | null
): Promise<void> {
  await storage.moveFile(id, newParentId)
}

export async function toggleStar(id: string): Promise<boolean> {
  const stored = await storage.getFile(id)
  if (!stored) throw new Error('File not found')
  stored.tags = stored.tags.includes('starred')
    ? stored.tags.filter((t) => t !== 'starred')
    : [...stored.tags, 'starred']
  await storage.storeFile(stored)
  return stored.tags.includes('starred')
}

export async function getStarredFiles(): Promise<DriveFile[]> {
  const all = await storage.getAllFiles()
  return all.filter((f) => f.tags.includes('starred')).map(toDriveFile)
}

export async function searchDriveFiles(query: string): Promise<DriveFile[]> {
  const results = await storage.searchFiles(query)
  return results.map(toDriveFile)
}

export async function getDriveQuota(): Promise<DriveQuota> {
  const all = await storage.getAllFiles()
  const fileCount = all.filter((f) => f.mimeType !== 'inode/directory').length
  const folderCount = all.filter((f) => f.mimeType === 'inode/directory').length
  const usedBytes = all.reduce((sum, f) => sum + f.size, 0)
  return { usedBytes, totalBytes: 0, fileCount, folderCount }
}

export async function getGeoFiles(): Promise<DriveFile[]> {
  const all = await storage.getAllFiles()
  return all.filter((f) => {
    const geoData = f.tags.filter((t) => t.startsWith('geo:'))
    return geoData.length > 0
  }).map(toDriveFile)
}

export async function importFromUrl(url: string, parentId: string | null): Promise<DriveFile> {
  const response = await fetch(url)
  if (!response.ok) throw new Error(`Failed to fetch URL: ${response.status}`)
  const data = await response.arrayBuffer()
  const urlPath = new URL(url).pathname
  const name = urlPath.split('/').pop() || 'downloaded-file'
  const mimeType = response.headers.get('content-type') || 'application/octet-stream'
  return addFile(name, data, parentId, mimeType)
}

export async function toFileNodes(driveFiles: DriveFile[]): Promise<FileNode[]> {
  return driveFiles.map((f) => ({
    id: f.id,
    name: f.name,
    fileType: f.fileType,
    parentId: f.parentId || undefined,
    sizeBytes: f.sizeBytes,
    mimeType: f.mimeType || undefined,
    path: `/${f.name}`,
    hashBlake3: f.hash || undefined,
    encrypted: f.encrypted,
    compressionLayers: [],
    tags: f.tags,
    isStarred: f.isStarred,
    createdAt: f.createdAt,
    modifiedAt: f.modifiedAt,
  }))
}

function toDriveFile(stored: storage.StoredFile): DriveFile {
  return {
    id: stored.id,
    name: stored.name,
    fileType: stored.mimeType === 'inode/directory' ? 'folder' : 'file',
    parentId: stored.parentId,
    sizeBytes: stored.size,
    mimeType: stored.mimeType,
    hash: stored.hash,
    encrypted: stored.encrypted,
    tags: stored.tags || [],
    createdAt: stored.createdAt,
    modifiedAt: stored.modifiedAt,
    isStarred: (stored.tags || []).includes('starred'),
  }
}

function toStoredFile(
  drive: DriveFile,
  data: ArrayBuffer
): storage.StoredFile {
  return {
    id: drive.id,
    name: drive.name,
    data,
    mimeType: drive.mimeType || 'application/octet-stream',
    size: drive.sizeBytes,
    createdAt: drive.createdAt,
    modifiedAt: drive.modifiedAt,
    parentId: drive.parentId,
    tags: drive.tags,
    encrypted: drive.encrypted,
    hash: drive.hash,
  }
}
