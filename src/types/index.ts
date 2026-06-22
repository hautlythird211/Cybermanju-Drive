export type ViewMode = 'grid' | 'list' | 'masonry'
export type PanelType = 'landing' | 'files' | 'preview' | 'encryption' | 'compression' | 'collections' | 'faces' | 'map' | 'code' | 'search' | 'style' | 'accounts' | 'loose-groups' | 'sync' | 'webdash' | 'users' | 'dashboard' | 'settings' | 'trash' | 'activity' | 'favorites' | 'recent' | 'storage' | 'history' | 'import' | 'transfer' | 'permissions' | 'system-monitor' | 'task-manager' | 'terminal' | 'duplicates' | 'browser' | 'book' | 'notes' | 'plugins' | 'art-maker' | 'media-library'
export type SidebarSection = 'tree' | 'locations' | 'collections' | 'people' | 'styles' | 'loose' | 'users' | 'sync' | 'dashboard' | 'landing' | 'tools' | 'media'

export interface ModuleInfo {
  id: PanelType
  label: string
  icon: string
  color: string
  gradient: string
  description: string
  requiresAuth: boolean
}
export type EncryptionAlgo = 'kyber1024' | 'dilithium5' | 'frodokem1344' | 'hybrid' | 'aes256'
export type CompressionType = 'none' | 'lz4' | 'zstd' | 'triple'
export type AccountType = 'local' | 'cloud' | 'network'
export type CollectionType = 'highlights' | 'best_moments' | 'custom'

export interface FileNode {
  id: string
  name: string
  fileType: string
  parentId?: string
  sizeBytes: number
  mimeType?: string
  hashBlake3?: string
  encrypted: boolean
  encryptionAlgorithm?: string
  compressionLayers: string[]
  thumbnailPath?: string
  contextData?: Record<string, unknown>
  tags?: string[]
  collectionIds?: string[]
  faceGroupIds?: string[]
  looseGroupIds?: string[]
  gpsLat?: number
  gpsLon?: number
  createdAt: string
  modifiedAt: string
  isStarred?: boolean
  isHidden?: boolean
  path?: string
  children?: FileNode[]
  accountId?: string
  locationId?: string
  contentText?: string
  treeSitterAst?: string
  permissions?: FilePermission[]
}

export interface FilePermission {
  userId: string
  username: string
  access: 'read' | 'write' | 'admin'
}

export interface Account {
  id: string
  name: string
  accountType: string
  backendType?: string
  path?: string
  color: string
  isActive: boolean
  oauthProvider?: string
  createdAt: string
  updatedAt: string
}

export interface CloudAccount {
  id: string
  name: string
  backendType: SyncBackendType
  token?: string
  config: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

export interface Collection {
  id: string
  name: string
  collectionType: CollectionType
  color: string
  description?: string
  itemIds: string[]
  createdAt: string
  updatedAt: string
}

export interface FaceGroup {
  id: string
  name: string
  color?: string
  icon?: string
  fileIds: string[]
  centroidEmbedding?: number[]
  binaryHash?: number
  cohesion?: number
  embeddingCount: number
  algorithm?: string
  createdAt: string
}

export interface EncryptionKeyInfo {
  id: string
  algorithm: string
  algorithmDisplay: string
  nistLevel: number
  color: string
  publicKeyPreview: string
  hasPrivateKey: boolean
  createdAt: string
}

export interface EncryptionStatus {
  isEncrypted: boolean
  algorithm?: string
  nistLevel?: number
  keyId?: string
  encryptedAt?: string
}

export interface LooseGroup {
  id: string
  name: string
  color: string
  icon?: string
  fileIds: string[]
  createdAt: string
}

export interface SearchResult {
  fileId: string
  fileName: string
  score: number
  snippet?: string
  matchType?: string
}

export interface GeoMarker {
  fileId: string
  fileName: string
  lat: number
  lng: number
  address?: string
  thumbnail?: string
}

export interface CompressionStats {
  originalSize: number
  compressedSize: number
  ratio: number
  layer: string
  layerDetails: LayerDetail[]
  blake3Hash: string
  durationMs: number
}

export interface LayerDetail {
  name: string
  algorithm: string
  inputSize: number
  outputSize: number
  ratio: number
  color: string
}

export interface CodeSymbol {
  name: string
  kind: string
  startLine: number
  endLine: number
  detail?: string
  children: CodeSymbol[]
}

export interface ParseResult {
  filePath: string
  language: string
  symbols: CodeSymbol[]
  totalLines: number
  parseTimeMs: number
}

export interface User {
  id: string
  username: string
  passwordHash?: string
  displayName?: string
  role: 'admin' | 'user' | 'viewer'
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface UserFilePermission {
  id: string
  userId: string
  fileId: string
  access: 'read' | 'write' | 'admin'
  grantedBy: string
  grantedAt: string
}

export interface AuthResult {
  userId: string
  username: string
  role: string
  displayName?: string
  token: string
}

export interface DashboardStatus {
  running: boolean
  port: number
  url: string
  activeConnections: number
}

export interface ApiEndpoint {
  method: string
  path: string
  description: string
}

export interface TrashItem {
  id: string
  originalFile: FileNode
  deletedAt: string
  deletedBy?: string
  restorePath?: string
}

export interface AuditEntry {
  id: string
  action: string
  entityType: string
  entityId: string
  userId?: string
  details?: Record<string, unknown>
  timestamp: string
}

export interface FileVersion {
  id: string
  fileId: string
  versionNumber: number
  hashBlake3?: string
  sizeBytes: number
  snapshotData?: string
  createdAt: string
}

export interface ShareLink {
  id: string
  fileId: string
  token: string
  expiresAt: string
  url: string
}

export const CYBER = {
  bgDeep: '#08080a',
  bgPanel: '#0e0e12',
  bgCard: '#16161c',
  bgHover: '#1e1e26',
  borderHeavy: '#22222a',
  borderNeon: '#00ff41',
  borderGold: '#ffd700',
  saffronGold: '#ff9f0a',
  lotusPink: '#ff6b9d',
  templeOrange: '#ff5500',
  prayerBlue: '#5af0ff',
  prayerWhite: '#f0f0f0',
  prayerRed: '#ff453a',
  prayerGreen: '#30d158',
  prayerYellow: '#ffd700',
  matrixGreen: '#00ff41',
  matrixDarkGreen: '#006b1a',
  cyberBlue: '#007aff',
  cyberPurple: '#b388ff',
  neonPink: '#ff6b9d',
  neonYellow: '#ffd700',
  textPrimary: '#ececf0',
  textSecondary: '#a0a0b0',
  textMuted: '#50505e',
  textNeon: '#00ff41',
} as const

export const PRAYER_FLAGS = ['#5af0ff', '#f0f0f0', '#ff3333', '#28c840', '#ffd700'] as const

export const ENCRYPTION_INFO: Record<EncryptionAlgo, { name: string; nistLevel: number; description: string; color: string }> = {
  kyber1024: {
    name: 'ML-KEM (Kyber-1024)',
    nistLevel: 5,
    description: 'NIST FIPS 203 - Lattice-based key encapsulation. Resistant to Shor\'s algorithm and all known quantum attacks.',
    color: '#00ff41',
  },
  dilithium5: {
    name: 'ML-DSA (Dilithium-5)',
    nistLevel: 5,
    description: 'NIST FIPS 204 - Lattice-based digital signature. Maximum security level, quantum-resistant signing.',
    color: '#5af0ff',
  },
  frodokem1344: {
    name: 'FrodoKEM-1344',
    nistLevel: 3,
    description: 'Learning-with-errors based. Conservative security estimates with classical ring structure.',
    color: '#b388ff',
  },
  hybrid: {
    name: 'Hybrid PQ+Classical',
    nistLevel: 5,
    description: 'Combines ML-KEM with X25519 for defense-in-depth transitional security.',
    color: '#ffd700',
  },
  aes256: {
    name: 'AES-256-GCM',
    nistLevel: 0,
    description: 'Classical symmetric encryption. Not quantum-resistant - recommended only in hybrid mode.',
    color: '#ff5f57',
  },
}

export const COMPRESSION_INFO: Record<CompressionType, { name: string; description: string; color: string; speed: string }> = {
  none: { name: 'None', description: 'Uncompressed raw data', color: '#555', speed: 'Instant' },
  lz4: { name: 'LZ4 (lz4_flex)', description: 'Ultra-fast pure Rust compression (~400 MB/s). Real-time previews and streaming.', color: '#5af0ff', speed: 'Ultra-Fast' },
  zstd: { name: 'Zstandard (zstd)', description: 'Facebook\'s algorithm. Excellent ratio/speed balance, configurable levels 1-22.', color: '#b388ff', speed: 'Fast' },
  triple: { name: 'Triple-Layer', description: 'LZ4 -> ZSTD-15 -> Brotli-11 cascading. Maximum compression for archival.', color: '#ffd700', speed: 'Slow' },
}

export type SyncBackendType = 'local' | 'github' | 'gitlab' | 'codeberg' | 'gitea' | 'googleDrive' | 'googlePhotos' | 'telegram' | 'mega'
export type SyncStatusType = 'idle' | 'scanning' | 'compressing' | 'uploading' | 'linking' | 'cleaning' | 'error' | 'done'

export interface SyncConfig {
  id: string
  backendType: SyncBackendType
  enabled: boolean
  accountId?: string
  name?: string
  basePath?: string
  repoName?: string
  branch?: string
  token?: string
  folderId?: string
  albumId?: string
  chatId?: string
  autoSync: boolean
  compressBeforeUpload: boolean
  createPreviews: boolean
  deleteRawAfterSync: boolean
  maxConcurrentUploads: number
  useGitLfs?: boolean
  lfsRepo?: string
  repoLayout?: 'flat' | 'sharded' | 'split'
  createdAt?: string
  updatedAt?: string
}

export interface SyncFile {
  id: string
  originalPath: string
  compressedPath?: string
  previewPath?: string
  remoteUrl?: string
  sizeBytes: number
  compressedSizeBytes?: number
  hashBlake3?: string
  backendType: SyncBackendType
  syncedAt?: string
  status: SyncStatusType
  errorMessage?: string
}

export interface SyncProgress {
  totalFiles: number
  processedFiles: number
  currentFile?: string
  status: SyncStatusType
  bytesUploaded: number
  errors: string[]
  startedAt?: string
  estimatedRemainingSeconds?: number
}

export interface SyncResult {
  filesSynced: number
  bytesUploaded: number
  bytesSavedByCompression: number
  errors: string[]
  durationMs: number
}

export interface RemoteFile {
  name: string
  path: string
  sizeBytes: number
  modifiedAt: string
  url: string
}

export const MODULE_METADATA: Record<PanelType, ModuleInfo> = {
  landing: { id: 'landing', label: 'HOME', icon: 'mdi:home-outline', color: '#00ff41', gradient: 'linear-gradient(135deg, #08080a 0%, #0e0e12 50%, #08080a 100%)', description: 'Quantum-resistant encrypted file manager', requiresAuth: false },
  files: { id: 'files', label: 'FILES', icon: 'mdi:folder-outline', color: '#5af0ff', gradient: 'linear-gradient(135deg, #08080a 0%, #0a0a18 50%, #08080a 100%)', description: 'Browse and manage your encrypted files', requiresAuth: true },
  search: { id: 'search', label: 'SEARCH', icon: 'mdi:magnify', color: '#b388ff', gradient: 'linear-gradient(135deg, #08080a 0%, #0e0a1a 50%, #08080a 100%)', description: 'Tantivy BM25 full-text search', requiresAuth: true },
  collections: { id: 'collections', label: 'COLLECTIONS', icon: 'mdi:bookmark-multiple-outline', color: '#ffd700', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0800 50%, #08080a 100%)', description: 'Curate and organize file collections', requiresAuth: true },
  faces: { id: 'faces', label: 'PEOPLE', icon: 'mdi:face-man-outline', color: '#ff6b9d', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0008 50%, #08080a 100%)', description: 'AI face detection and clustering', requiresAuth: true },
  map: { id: 'map', label: 'MAP', icon: 'mdi:map-outline', color: '#30d158', gradient: 'linear-gradient(135deg, #08080a 0%, #000a0c 50%, #08080a 100%)', description: 'GPS-tagged files on MapLibre GL', requiresAuth: true },
  code: { id: 'code', label: 'CODE', icon: 'mdi:code-tags', color: '#5af0ff', gradient: 'linear-gradient(135deg, #08080a 0%, #000c08 50%, #08080a 100%)', description: 'Tree-sitter code intelligence', requiresAuth: true },
  sync: { id: 'sync', label: 'SYNC', icon: 'mdi:sync', color: '#007aff', gradient: 'linear-gradient(135deg, #08080a 0%, #00080c 50%, #08080a 100%)', description: 'Multi-backend cloud sync', requiresAuth: true },
  accounts: { id: 'accounts', label: 'ACCOUNTS', icon: 'mdi:account-outline', color: '#ff9f0a', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0800 50%, #08080a 100%)', description: 'Manage local and cloud accounts', requiresAuth: true },
  'loose-groups': { id: 'loose-groups', label: 'LOOSE', icon: 'mdi:file-group-outline', color: '#ffd700', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0c00 50%, #08080a 100%)', description: 'Ad-hoc file grouping', requiresAuth: true },
  style: { id: 'style', label: 'TAGS', icon: 'mdi:tag-outline', color: '#b388ff', gradient: 'linear-gradient(135deg, #08080a 0%, #0a000c 50%, #08080a 100%)', description: 'CLIP-based visual style tags', requiresAuth: true },
  users: { id: 'users', label: 'USERS', icon: 'mdi:account-group-outline', color: '#ff453a', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0000 50%, #08080a 100%)', description: 'Multi-user access control', requiresAuth: true },
  dashboard: { id: 'dashboard', label: 'REMOTE', icon: 'mdi:monitor-dashboard', color: '#5af0ff', gradient: 'linear-gradient(135deg, #08080a 0%, #00080c 50%, #08080a 100%)', description: 'Web dashboard and API status', requiresAuth: true },
  webdash: { id: 'webdash', label: 'OVERLAY', icon: 'mdi:web', color: '#007aff', gradient: 'linear-gradient(135deg, #08080a 0%, #04040a 50%, #08080a 100%)', description: 'Remote access dashboard', requiresAuth: true },
  preview: { id: 'preview', label: 'PREVIEW', icon: 'mdi:eye-outline', color: '#00ff41', gradient: 'linear-gradient(135deg, #08080a 0%, #08080a 50%, #08080a 100%)', description: 'File preview panel', requiresAuth: true },
  encryption: { id: 'encryption', label: 'ENCRYPT', icon: 'mdi:lock-outline', color: '#ff453a', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0000 50%, #08080a 100%)', description: 'Post-quantum encryption management', requiresAuth: true },
  compression: { id: 'compression', label: 'COMPRESS', icon: 'mdi:package-variant-closed', color: '#30d158', gradient: 'linear-gradient(135deg, #08080a 0%, #000c00 50%, #08080a 100%)', description: 'Triple-layer compression pipeline', requiresAuth: true },
  settings: { id: 'settings', label: 'SETTINGS', icon: 'mdi:cog-outline', color: '#a0a0b0', gradient: 'linear-gradient(135deg, #08080a 0%, #08080a 50%, #08080a 100%)', description: 'Application settings and preferences', requiresAuth: true },
  trash: { id: 'trash', label: 'TRASH', icon: 'mdi:delete-outline', color: '#ff453a', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0000 50%, #08080a 100%)', description: 'Deleted files', requiresAuth: true },
  activity: { id: 'activity', label: 'ACTIVITY', icon: 'mdi:history', color: '#5af0ff', gradient: 'linear-gradient(135deg, #08080a 0%, #00080c 50%, #08080a 100%)', description: 'File activity log', requiresAuth: true },
  favorites: { id: 'favorites', label: 'FAVORITES', icon: 'mdi:star-outline', color: '#ffd700', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0800 50%, #08080a 100%)', description: 'Starred files', requiresAuth: true },
  recent: { id: 'recent', label: 'RECENT', icon: 'mdi:clock-outline', color: '#b388ff', gradient: 'linear-gradient(135deg, #08080a 0%, #080808 50%, #08080a 100%)', description: 'Recently modified files', requiresAuth: true },
  storage: { id: 'storage', label: 'STORAGE', icon: 'mdi:harddisk', color: '#30d158', gradient: 'linear-gradient(135deg, #08080a 0%, #000a00 50%, #08080a 100%)', description: 'Storage usage dashboard', requiresAuth: true },
  history: { id: 'history', label: 'HISTORY', icon: 'mdi:history', color: '#5af0ff', gradient: 'linear-gradient(135deg, #08080a 0%, #00080c 50%, #08080a 100%)', description: 'Atomic undo/redo history', requiresAuth: true },
  import: { id: 'import', label: 'IMPORT', icon: 'mdi:file-import-outline', color: '#30d158', gradient: 'linear-gradient(135deg, #08080a 0%, #000c08 50%, #08080a 100%)', description: 'Import files from cloud sources with per-source personalization', requiresAuth: true },
  transfer: { id: 'transfer', label: 'TRANSFER', icon: 'mdi:transfer', color: '#5af0ff', gradient: 'linear-gradient(135deg, #08080a 0%, #00080c 50%, #08080a 100%)', description: 'Transfer files between any two backends', requiresAuth: true },
  permissions: { id: 'permissions', label: 'PERMISSIONS', icon: 'mdi:shield-account-outline', color: '#ff9f0a', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0800 50%, #08080a 100%)', description: 'Fine-grained file permissions', requiresAuth: true },
  'system-monitor': { id: 'system-monitor', label: 'SYSTEM MONITOR', icon: 'mdi:chart-line-variant', color: '#5af0ff', gradient: 'linear-gradient(135deg, #08080a 0%, #00080c 50%, #08080a 100%)', description: 'Real-time system performance monitoring', requiresAuth: true },
  'task-manager': { id: 'task-manager', label: 'TASK MANAGER', icon: 'mdi:memory', color: '#ff6b9d', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0008 50%, #08080a 100%)', description: 'Manage running processes and tasks', requiresAuth: true },
  terminal: { id: 'terminal', label: 'TERMINAL', icon: 'mdi:console', color: '#30d158', gradient: 'linear-gradient(135deg, #08080a 0%, #000c00 50%, #08080a 100%)', description: 'Full-featured terminal emulator', requiresAuth: true },
  duplicates: { id: 'duplicates', label: 'DUPLICATES', icon: 'mdi:content-copy', color: '#ff453a', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0000 50%, #08080a 100%)', description: 'Find and manage duplicate files by BLAKE3 hash', requiresAuth: true },
  browser: { id: 'browser', label: 'WEB', icon: 'mdi:web', color: '#b388ff', gradient: 'linear-gradient(135deg, #08080a 0%, #0e0420 30%, #0c0015 60%, #08080a 100%)', description: 'Futuristic web browser with DuckDuckGo search engine', requiresAuth: true },
  book: { id: 'book', label: 'BOOK', icon: 'mdi:book-open-outline', color: '#ffd700', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0800 50%, #08080a 100%)', description: 'Book writing app with chapters and direct creation', requiresAuth: true },
  notes: { id: 'notes', label: 'NOTES', icon: 'mdi:note-text-outline', color: '#5af0ff', gradient: 'linear-gradient(135deg, #08080a 0%, #000c0c 50%, #08080a 100%)', description: 'Quick notes with clipboard integration', requiresAuth: true },
  plugins: { id: 'plugins', label: 'PLUGINS', icon: 'mdi:puzzle-outline', color: '#30d158', gradient: 'linear-gradient(135deg, #08080a 0%, #000c00 50%, #08080a 100%)', description: 'Visual plugin builder for Cybermanju OS workspace', requiresAuth: true },
  'art-maker': { id: 'art-maker', label: 'ART MAKER', icon: 'mdi:palette-outline', color: '#b388ff', gradient: 'linear-gradient(135deg, #08080a 0%, #0a0015 50%, #08080a 100%)', description: 'Real-time visual art canvas with interactive controls', requiresAuth: false },
  'media-library': { id: 'media-library', label: 'MEDIA', icon: 'mdi:image-multiple-outline', color: '#ff6b9d', gradient: 'linear-gradient(135deg, #08080a 0%, #0c0008 50%, #08080a 100%)', description: 'Browse all images, videos, and audio files', requiresAuth: true },
}

export const SYNC_BACKEND_INFO: Record<SyncBackendType, { name: string; description: string; color: string; icon: string }> = {
  local: {
    name: 'Local Storage',
    description: 'Sync files to a local directory on this machine. Fast, no network required.',
    color: '#00ff41',
    icon: 'HardDrive',
  },
  github: {
    name: 'GitHub',
    description: 'Sync files to a GitHub repository using the Contents API + Git LFS for large files.',
    color: '#f0f0f0',
    icon: 'Github',
  },
  gitlab: {
    name: 'GitLab',
    description: 'Sync files to a GitLab project repository. Full CRUD via GitLab API v4 with LFS support.',
    color: '#ff6b9d',
    icon: 'GitBranch',
  },
  codeberg: {
    name: 'Codeberg',
    description: 'Sync files to Codeberg (Forgejo) repositories. Free, privacy-focused, open-source git hosting.',
    color: '#30d158',
    icon: 'Code',
  },
  gitea: {
    name: 'Gitea/Forgejo',
    description: 'Sync files to any Gitea or Forgejo instance. Self-hosted git with API v1.',
    color: '#5af0ff',
    icon: 'Server',
  },
  googleDrive: {
    name: 'Google Drive',
    description: 'Sync files to Google Drive folders. Full CRUD via Drive API v3.',
    color: '#ffd700',
    icon: 'FolderSync',
  },
  googlePhotos: {
    name: 'Google Photos',
    description: 'Upload photos and videos to Google Photos. Optimized for media files.',
    color: '#ff6b9d',
    icon: 'Camera',
  },
  telegram: {
    name: 'Telegram',
    description: 'Send files to a Telegram chat, channel, or group via Bot API. Files up to 50 MB per upload.',
    color: '#5af0ff',
    icon: 'MessageCircle',
  },
  mega: {
    name: 'Mega',
    description: 'Mega.nz encrypted cloud storage. Login with email & password. 20 GB free.',
    color: '#ff5f57',
    icon: 'Cloud',
  },
}

export type HistoryActionType =
  | 'file:delete' | 'file:restore' | 'file:rename' | 'file:create' | 'file:move'
  | 'trash:delete' | 'trash:empty'
  | 'encryption:encrypt' | 'encryption:decrypt'
  | 'compression:compress' | 'compression:decompress'
  | 'collection:create' | 'collection:delete' | 'collection:add' | 'collection:remove'
  | 'face:rename' | 'face:merge' | 'face:delete'
  | 'account:create' | 'account:switch'
  | 'sync:create' | 'sync:delete'
  | 'user:create' | 'user:delete' | 'user:role'
  | 'share:create' | 'share:delete'
  | 'version:create' | 'version:revert'

export interface HistoryAction {
  source: 'invoke' | 'store'
  cmd: string
  args: Record<string, unknown>
}

export interface HistoryEntry {
  id: string
  type: HistoryActionType
  description: string
  timestamp: number
  affectedFileIds: string[]
  undo: HistoryAction
  redo: HistoryAction
}

export type ResolutionLevel = 'r0' | 'r1' | 'r2' | 'r3'

export interface ResolutionInfo {
  level: ResolutionLevel
  width?: number
  height?: number
  format?: string
  sizeBytes?: number
  keyTier: string
  encrypted: boolean
}

export interface ImageInfo {
  width: number
  height: number
  format: string
  colorType: string
  hasAlpha: boolean
  bitsPerChannel: number
  exif?: ExifData
}

export interface ExifData {
  cameraMake?: string
  cameraModel?: string
  dateTaken?: string
  gpsLat?: number
  gpsLon?: number
  exposureTime?: string
  fNumber?: number
  iso?: number
  focalLength?: number
  orientation?: number
  imageWidth?: number
  imageHeight?: number
}

export interface VideoInfo {
  durationSecs: number
  width: number
  height: number
  codec: string
  fps: number
  bitrate: number
  audioCodec?: string
  audioSampleRate?: number
  audioChannels?: number
  container: string
  totalFrames?: number
}

export type PlaybackState = 'stopped' | 'playing' | 'paused' | 'buffering' | 'error'

export interface PlaybackPosition {
  currentSecs: number
  totalSecs: number
  speed: number
}

export interface TrackInfo {
  index: number
  trackType: string
  codec: string
  language?: string
  title?: string
  isDefault: boolean
}

export interface VideoPlayerState {
  state: PlaybackState
  position: PlaybackPosition
  volume: number
  isMuted: boolean
  isFullscreen: boolean
  currentTrack?: TrackInfo
  availableTracks: TrackInfo[]
}

export interface FileMediaData {
  fileId: string
  filename: string
  mimeType: string
  isImage: boolean
  isVideo: boolean
  isAudio: boolean
  imageInfo?: ImageInfo
  videoInfo?: VideoInfo
  availableResolutions: ResolutionInfo[]
  selectedResolution: ResolutionLevel
}

export interface ResolutionData {
  fileId: string
  level: string
  dataBase64: string
  width: number
  height: number
  format: string
}

export interface MediaInfo {
  mimeType: string
  extension: string
  isImage: boolean
  isVideo: boolean
  isAudio: boolean
  supportedResolutions: string[]
  estimatedFileType: string
}

export interface ThumbnailResult {
  data: number[]
  width: number
  height: number
  format: string
  sizeBytes: number
}
