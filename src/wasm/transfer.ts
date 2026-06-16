import { invoke, isTauri } from '@/composables/useTauri'
import type { SyncConfig } from '@/wasm/sync'
import { loadTokenFromStorage, getValidToken } from '@/wasm/oauth'

export interface TransferRequest {
  sourceConfig: SyncConfig
  destConfig: SyncConfig
  filePaths: string[]
  deleteSourceAfter: boolean
  saveToLocal: boolean
}

export interface TransferProgress {
  totalFiles: number
  processedFiles: number
  currentFile: string | null
  status: string
  bytesTransferred: number
  errors: string[]
  startedAt: string | null
}

export interface TransferResult {
  filesTransferred: number
  bytesTransferred: number
  errors: string[]
  durationMs: number
}

async function wasmTransferFile(
  source: SyncConfig,
  dest: SyncConfig,
  remotePath: string,
): Promise<number> {
  const { listRemoteFiles, startSync } = await import('@/wasm/sync')
  const sourceFiles = await listRemoteFiles(source as any, '')
  const targetFile = sourceFiles.find(f => f.path === remotePath || f.name === remotePath)
  if (!targetFile) throw new Error(`File not found on source: ${remotePath}`)

  const drive = await import('@/wasm/drive')
  await drive.importFromUrl(targetFile.url, null)

  const tempConfig = { ...dest, basePath: '/' }
  await startSync(tempConfig as any, [targetFile.path])

  return targetFile.sizeBytes
}

export async function transferFiles(
  request: TransferRequest
): Promise<TransferResult> {
  if (isTauri()) {
    return invoke<TransferResult>('transfer_files', { request })
  }

  const start = Date.now()
  let filesTransferred = 0
  let bytesTransferred = 0
  const errors: string[] = []

  for (const fp of request.filePaths) {
    try {
      const b = await wasmTransferFile(request.sourceConfig, request.destConfig, fp)
      bytesTransferred += b
      filesTransferred++
    } catch (e) {
      errors.push(`${fp}: ${e instanceof Error ? e.message : String(e)}`)
    }

    if (request.saveToLocal) {
      // In WASM mode, save to local is implicit — the file stays in the VirtualDrive
    }

    if (request.deleteSourceAfter && errors.length === 0) {
      const token = await loadTokenFromStorage(request.sourceConfig.backendType as any)
      if (token) {
        const valid = await getValidToken(token)
        if (request.sourceConfig.backendType === 'googleDrive') {
          await fetch(
            `https://www.googleapis.com/drive/v3/files/${fp}`,
            { method: 'DELETE', headers: { Authorization: `Bearer ${valid.accessToken}` } }
          )
        }
      }
    }
  }

  return {
    filesTransferred,
    bytesTransferred,
    errors,
    durationMs: Date.now() - start,
  }
}

export async function getTransferProgress(): Promise<TransferProgress> {
  if (isTauri()) {
    return invoke<TransferProgress>('get_transfer_progress')
  }
  return {
    totalFiles: 0,
    processedFiles: 0,
    currentFile: null,
    status: 'idle',
    bytesTransferred: 0,
    errors: [],
    startedAt: null,
  }
}

export async function cancelTransfer(): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('cancel_transfer')
  }
  return false
}
