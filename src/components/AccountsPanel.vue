<template>
  <div class="accounts-panel">
    <div class="panel-header">
      <Icon icon="mdi:account-multiple-outline" width="16" height="16" class="header-icon" />
      <h2 class="panel-title">ACCOUNTS & PROVIDERS</h2>
    </div>

    <div class="section">
      <h3 class="section-title">[CLOUD] CONNECTED PROVIDERS ({{ cloudAccounts.length }})</h3>
      <div class="accounts-list">
        <div
          v-for="acc in cloudAccounts"
          :key="acc.id"
          class="account-card"
          :class="{ expanded: expandedId === acc.id }"
        >
          <div class="ac-header" @click="toggleExpand(acc.id)">
            <Icon :icon="providerIcon(acc)" width="20" height="20" class="ac-icon" :style="{ color: providerColor(acc) }" />
            <div class="ac-info">
              <span class="ac-name">{{ acc.name }}</span>
              <span class="ac-email">{{ accountEmail(acc) || providerLabel(acc) }}</span>
            </div>
            <span class="ac-badge" :style="{ background: providerColor(acc) + '22', color: providerColor(acc), borderColor: providerColor(acc) }">{{ providerLabel(acc) }}</span>
            <span class="ac-status" :class="acc.isActive ? 'online' : 'offline'">{{ acc.isActive ? 'ACTIVE' : 'IDLE' }}</span>
            <Icon :icon="expandedId === acc.id ? 'mdi:chevron-up' : 'mdi:chevron-down'" width="16" height="16" class="ac-chevron" />
          </div>

          <div v-if="expandedId === acc.id" class="ac-body">
            <div class="ac-details">
              <div class="detail-row">
                <span class="detail-key">TYPE</span>
                <span class="detail-val">{{ providerLabel(acc) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-key">PATH</span>
                <span class="detail-val">{{ acc.path || '/' }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-key">CREATED</span>
                <span class="detail-val">{{ formatDate(acc.createdAt) }}</span>
              </div>
            </div>

            <div class="albums-section" v-if="hasAlbums(acc)">
              <h4 class="albums-title">[ALBUMS] {{ accountAlbums(acc).length }} COLLECTIONS</h4>
              <div class="albums-list">
                <div v-for="album in accountAlbums(acc)" :key="album.id" class="album-item" @click="browseAlbum(acc, album)">
                  <Icon icon="mdi:folder-outline" width="14" height="14" class="album-icon" />
                  <span class="album-name">{{ album.name }}</span>
                  <span class="album-count">{{ album.itemIds?.length || 0 }} ITEMS</span>
                </div>
                <div v-if="accountAlbums(acc).length === 0" class="albums-empty">No albums for this provider</div>
              </div>
            </div>

            <div class="ac-actions">
              <button class="bw-btn-sm" @click="scanAccount(acc)">
                <Icon v-if="scanningId === acc.id" icon="svg-spinners:blocks-wave" width="10" height="10" />
                {{ scanningId === acc.id ? 'SCANNING...' : '[ SCAN FILES ]' }}
              </button>
              <button class="bw-btn-sm" @click="browseAccount(acc)">
                <Icon icon="mdi:file-find-outline" width="12" height="12" />
                [ BROWSE ]
              </button>
              <button class="bw-btn-sm bw-btn-danger" @click="disconnectAccount(acc)">[ DISCONNECT ]</button>
            </div>

            <div v-if="accScanResults[acc.id]" class="scan-results">
              <div class="sr-row"><span class="sr-key">FILES</span><span class="sr-val">{{ accScanResults[acc.id] }}</span></div>
            </div>
          </div>
        </div>

        <div v-if="cloudAccounts.length === 0" class="empty-state">
          <Icon icon="mdi:cloud-outline" width="24" height="24" />
          <span>No cloud providers connected. Use Setup Wizard or Settings to connect.</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[LOCAL] LOCAL ACCOUNTS ({{ localAccounts.length }})</h3>
      <div class="accounts-list">
        <div v-for="acc in localAccounts" :key="acc.id" class="account-card local-card">
          <div class="ac-header">
            <Icon icon="mdi:harddisk" width="20" height="20" class="ac-icon" style="color:var(--text-accent)" />
            <div class="ac-info">
              <span class="ac-name">{{ acc.name }}</span>
              <span class="ac-email">{{ acc.path || '/' }}</span>
            </div>
            <span class="ac-badge" style="background:var(--accent-dim);color:var(--text-accent);border-color:var(--border-accent)">LOCAL</span>
            <span class="ac-status online">ACTIVE</span>
          </div>
        </div>
        <div v-if="localAccounts.length === 0" class="empty-state">
          <Icon icon="mdi:harddisk" width="24" height="24" />
          <span>No local accounts configured.</span>
        </div>
      </div>
    </div>

    <div class="section" v-if="store.collections.length > 0">
      <h3 class="section-title">[COL] COLLECTIONS ({{ store.collections.length }})</h3>
      <div class="collections-grid">
        <div v-for="col in store.collections" :key="col.id" class="collection-chip" @click="openCollection(col)">
          <Icon icon="mdi:bookmark-outline" width="12" height="12" />
          <span class="col-name">{{ col.name }}</span>
          <span class="col-count">{{ col.itemIds?.length || 0 }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'
import { loadTokenFromStorage } from '@/wasm/oauth'
import type { Account, Collection } from '@/types'

const store = useAppStore()
const wm = useWindowManager()

const expandedId = ref<string | null>(null)
const scanningId = ref<string | null>(null)
const accScanResults = ref<Record<string, number>>({})
const accountEmails = ref<Record<string, string>>({})

async function loadEmails() {
  const map: Record<string, string> = {}
  for (const acc of store.accounts) {
    const bt = acc.backendType || acc.oauthProvider
    if (bt === 'mega') {
      try {
        const token = await loadTokenFromStorage('mega' as any)
        if (token?.accessToken) {
          const sep = token.accessToken.indexOf('|')
          if (sep !== -1) map[acc.id] = token.accessToken.slice(0, sep)
        }
      } catch {}
    }
  }
  accountEmails.value = map
}

onMounted(loadEmails)
watch(() => store.accounts.length, loadEmails)

const PROVIDER_META: Record<string, { icon: string; color: string; label: string }> = {
  mega: { icon: 'logos:mega', color: '#ff5f57', label: 'MEGA' },
  googleDrive: { icon: 'mdi:google-drive', color: '#ffd700', label: 'GOOGLE DRIVE' },
  googlePhotos: { icon: 'mdi:google-photos', color: '#ff6b9d', label: 'GOOGLE PHOTOS' },
  github: { icon: 'mdi:github', color: '#f0f0f0', label: 'GITHUB' },
  gitlab: { icon: 'mdi:gitlab', color: '#ff6b9d', label: 'GITLAB' },
  telegram: { icon: 'mdi:send', color: '#5af0ff', label: 'TELEGRAM' },
  google: { icon: 'mdi:google', color: '#ffd700', label: 'GOOGLE' },
}

const cloudAccounts = computed(() =>
  store.accounts.filter(a => a.accountType === 'cloud')
)

const localAccounts = computed(() =>
  store.accounts.filter(a => a.accountType !== 'cloud')
)

function providerIcon(acc: Account): string {
  const bt = acc.backendType || acc.oauthProvider || ''
  return PROVIDER_META[bt]?.icon || 'mdi:cloud-outline'
}

function providerColor(acc: Account): string {
  const bt = acc.backendType || acc.oauthProvider || ''
  return PROVIDER_META[bt]?.color || '#888'
}

function providerLabel(acc: Account): string {
  const bt = acc.backendType || acc.oauthProvider || ''
  return PROVIDER_META[bt]?.label || bt.toUpperCase()
}

function hasAlbums(acc: Account): boolean {
  const bt = acc.backendType || acc.oauthProvider || ''
  return ['googlePhotos', 'googleDrive', 'mega'].includes(bt)
}

function accountAlbums(acc: Account): Collection[] {
  return store.collections.filter(c => c.collectionType === (acc.backendType || acc.oauthProvider))
}

function accountEmail(acc: Account): string {
  return accountEmails.value[acc.id] || ''
}

function formatDate(iso: string): string {
  if (!iso) return '--'
  try {
    return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  } catch { return iso }
}

function toggleExpand(id: string) {
  expandedId.value = expandedId.value === id ? null : id
}

async function scanAccount(acc: Account) {
  scanningId.value = acc.id
  try {
    const { listRemoteFiles } = await import('@/wasm/sync')
    const config = {
      id: acc.id,
      name: acc.name,
      backendType: acc.backendType || acc.oauthProvider || '',
      enabled: true,
      basePath: acc.path || '/',
      autoSync: false,
      compressBeforeSync: false,
      maxConcurrentOps: 1,
      createdAt: '',
      updatedAt: '',
    }
    const files = await listRemoteFiles(config as any, '/')
    accScanResults.value = { ...accScanResults.value, [acc.id]: files.length }
    store.notifySuccess(`Scan complete: ${files.length} files found for ${acc.name}`)
  } catch (e) {
    store.notifyError(`Scan failed for ${acc.name}`, e instanceof Error ? e.message : String(e))
  } finally {
    scanningId.value = null
  }
}

function browseAccount(acc: Account) {
  wm.open('import', { accountId: acc.id, backendType: acc.backendType || acc.oauthProvider })
}

function browseAlbum(acc: Account, album: Collection) {
  store.notifySuccess(`Browsing album: ${album.name} (${album.itemIds?.length || 0} items)`)
}

function openCollection(col: Collection) {
  if (col.itemIds && col.itemIds.length > 0) {
    store.selectedFileId = col.itemIds[0]
    store.currentPanel = 'files'
  } else {
    store.notifySuccess(`Collection: ${col.name} (empty)`)
  }
}

async function disconnectAccount(acc: Account) {
  try {
    const data = await import('@/wasm/data')
    await data.deleteAccount(acc.id)
    await store.fetchAccounts()
    store.notifySuccess(`Disconnected: ${acc.name}`)
  } catch (e) {
    store.notifyError(`Failed to disconnect ${acc.name}`, e instanceof Error ? e.message : String(e))
  }
}
</script>

<style scoped>
.accounts-panel {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
  font-family: var(--font-mono);
  background: transparent;
  color: var(--text-primary);
}

.accounts-panel::-webkit-scrollbar { width: 4px; }
.accounts-panel::-webkit-scrollbar-track { background: transparent; }
.accounts-panel::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 2px; }

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 18px;
  padding-bottom: 12px;
  border-bottom: 1px solid #1a1a1a;
}

.header-icon { color: var(--text-primary); flex-shrink: 0; }

.panel-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 2px;
  margin: 0;
}

.section { margin-bottom: 20px; }

.section-title {
  font-size: 9px;
  font-weight: 700;
  color: #555;
  letter-spacing: 1.5px;
  margin: 0 0 10px 0;
}

.accounts-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.account-card {
  background: var(--bg-glass-light);
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: border-color 0.15s;
  backdrop-filter: blur(var(--glass-blur-light));
}

.account-card.expanded {
  border-color: var(--border-accent);
}

.local-card { opacity: 0.7; }

.ac-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  cursor: pointer;
  user-select: none;
  transition: background 0.1s;
}

.ac-header:hover { background: var(--bg-surface); }

.ac-icon { flex-shrink: 0; }

.ac-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.ac-name {
  font-size: 11px;
  font-weight: 700;
  color: #ccc;
}

.ac-email {
  font-size: 9px;
  color: #555;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ac-badge {
  font-size: 7px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid;
  white-space: nowrap;
}

.ac-status {
  font-size: 8px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 2px 8px;
  border-radius: 4px;
}

.ac-status.online { color: var(--text-accent); background: var(--bg-surface); }
.ac-status.offline { color: #555; background: rgba(255, 255, 255, 0.05); }

.ac-chevron { color: #444; flex-shrink: 0; }

.ac-body {
  border-top: 1px solid #1a1a1a;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ac-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-row {
  display: flex;
  gap: 8px;
  font-size: 9px;
}

.detail-key {
  color: #555;
  min-width: 60px;
}

.detail-val { color: #aaa; }

.albums-section { }

.albums-title {
  font-size: 9px;
  font-weight: 700;
  color: #555;
  letter-spacing: 1px;
  margin: 0 0 8px 0;
}

.albums-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.album-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid #1a1a1a;
  border-radius: 4px;
  cursor: pointer;
  transition: border-color 0.15s;
  font-size: 10px;
}

.album-item:hover { border-color: var(--border-accent); }

.album-icon { color: var(--text-accent); flex-shrink: 0; }

.album-name { flex: 1; color: #ccc; }

.album-count { font-size: 8px; color: #555; }

.albums-empty {
  font-size: 9px;
  color: #444;
  padding: 8px;
}

.ac-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.scan-results {
  padding: 8px;
  background: var(--bg-surface);
  border: 1px solid #1a1a1a;
  border-radius: 4px;
}

.sr-row {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
}

.sr-key { color: #555; }
.sr-val { color: var(--text-accent); font-weight: 700; }

.collections-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.collection-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.15s;
  font-size: 10px;
}

.collection-chip:hover { border-color: var(--border-accent); }

.col-name { color: #ccc; }
.col-count { color: #555; font-size: 8px; margin-left: auto; }

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px;
  color: #444;
  font-size: 10px;
  text-align: center;
}
</style>
