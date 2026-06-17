<template>
  <OsPanel variant="neon" padding="md">
    <OsSection title="STORAGE DASHBOARD" icon="mdi:harddisk" variant="neon" spaced>
      <OsSection title="FILE COUNTS" icon="mdi:counter" collapsible>
        <div class="stats-grid">
          <OsCard :ref="(el) => setCardRef(0, el)" variant="neon" padding="md" class="stat-card">
            <span :ref="(el) => setStatRef(0, el)" class="stat-value" :data-count="store.files.length">{{ store.files.length }}</span>
            <span class="stat-label">TOTAL FILES</span>
          </OsCard>
          <OsCard :ref="(el) => setCardRef(1, el)" variant="neon" padding="md" class="stat-card">
            <span :ref="(el) => setStatRef(1, el)" class="stat-value" :data-count="store.folders.length">{{ store.folders.length }}</span>
            <span class="stat-label">FOLDERS</span>
          </OsCard>
          <OsCard :ref="(el) => setCardRef(2, el)" variant="neon" padding="md" class="stat-card">
            <span :ref="(el) => setStatRef(2, el)" class="stat-value" :data-count="store.encryptedFiles.length">{{ store.encryptedFiles.length }}</span>
            <span class="stat-label">ENCRYPTED</span>
          </OsCard>
          <OsCard :ref="(el) => setCardRef(3, el)" variant="neon" padding="md" class="stat-card">
            <span :ref="(el) => setStatRef(3, el)" class="stat-value" :data-count="store.compressedFiles.length">{{ store.compressedFiles.length }}</span>
            <span class="stat-label">COMPRESSED</span>
          </OsCard>
          <OsCard :ref="(el) => setCardRef(4, el)" variant="neon" padding="md" class="stat-card">
            <span :ref="(el) => setStatRef(4, el)" class="stat-value" :data-count="store.starredFiles.length">{{ store.starredFiles.length }}</span>
            <span class="stat-label">STARRED</span>
          </OsCard>
          <OsCard :ref="(el) => setCardRef(5, el)" variant="neon" padding="md" class="stat-card">
            <span :ref="(el) => setStatRef(5, el)" class="stat-value" :data-count="trashCount">{{ trashCount }}</span>
            <span class="stat-label">IN TRASH</span>
          </OsCard>
        </div>
      </OsSection>

      <OsDivider />

      <OsSection title="SIZE BY TYPE" icon="mdi:chart-bar" collapsible>
        <div class="type-breakdown">
          <div v-for="entry in byType" :key="entry.label" class="type-row">
            <span class="type-label">{{ entry.label }}</span>
            <div class="type-bar">
              <div
                :ref="(el) => { if (el) setBarRef(entry.label, el as HTMLElement) }"
                class="type-bar-fill"
                :style="{ width: entry.percent + '%', background: entry.color }"
                :data-percent="entry.percent"
              />
            </div>
            <span class="type-size">{{ formatSize(entry.totalBytes) }}</span>
          </div>
        </div>
      </OsSection>

      <OsDivider />

      <OsSection title="STORAGE FOOTPRINT" icon="mdi:chart-pie" collapsible>
        <OsCard variant="glass" padding="md">
          <div class="info-row"><span class="info-key">TOTAL SIZE</span><span class="info-value">{{ totalSizeFormatted }}</span></div>
          <div class="info-row"><span class="info-key">LARGEST FILE</span><span class="info-value">{{ largestFile }}</span></div>
          <div class="info-row"><span class="info-key">AVG FILE SIZE</span><span class="info-value">{{ avgSizeFormatted }}</span></div>
          <div class="info-row"><span class="info-key">FILES WITH GPS</span><span class="info-value">{{ gpsCount }}</span></div>
          <div class="info-row"><span class="info-key">FILES WITH FACES</span><span class="info-value">{{ faceCount }}</span></div>
        </OsCard>
      </OsSection>
    </OsSection>
  </OsPanel>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useAppStore } from '@/stores/app'
import { useGsapAnimation } from '@/composables/useGsapAnimation'
import { OsPanel, OsSection, OsCard, OsBadge, OsDivider } from '@/components/ui'

const anim = useGsapAnimation()
const store = useAppStore()
const wasmQuota = ref<{ usedBytes: number; fileCount: number; folderCount: number } | null>(null)
const wasmFiles = ref<any[]>([])

const cardRefs = ref<(HTMLElement | null)[]>([])
const statsRefs = ref<(HTMLElement | null)[]>([])
const barRefs = ref<Map<string, HTMLElement>>(new Map())
const gsapCtx = ref<gsap.Context | null>(null)

function setCardRef(idx: number, el: any) {
  cardRefs.value[idx] = el as HTMLElement
}

function setStatRef(idx: number, el: any) {
  statsRefs.value[idx] = el as HTMLElement
}

function setBarRef(key: string, el: HTMLElement) {
  barRefs.value.set(key, el)
}

onMounted(async () => {
  try {
    const { drive, isWasmReady, initWasm } = await import('@/wasm')
    await initWasm()
    if (isWasmReady()) {
      wasmQuota.value = await drive.getDriveQuota()
      wasmFiles.value = await drive.getAllDriveFiles()
    }
  } catch { /* WASM not available */ }

  await nextTick()

  gsapCtx.value = gsap.context(() => {
    // Stagger in cards
    const cards = cardRefs.value.filter(Boolean) as HTMLElement[]
    if (cards.length > 0) {
      anim.staggerIn(cards, { stagger: 0.06, from: 'start', duration: 0.3 })
    }

    // Count up stat values
    const stats = statsRefs.value.filter(Boolean) as HTMLElement[]
    stats.forEach((el) => {
      const target = parseInt(el.dataset.count || '0', 10)
      anim.countUp(el, target, { duration: 0.8 })
    })

    // Animate progress bars
    barRefs.value.forEach((el) => {
      const percent = parseFloat(el.dataset.percent || '0')
      anim.animateProgress(el, 0, percent, { duration: 0.6 })
    })
  })
})

const files = computed(() => wasmFiles.value.length > 0 ? wasmFiles.value : store.files)
const trashCount = computed(() => store.trashItems.length)
const totalSize = computed(() => files.value.reduce((s: number, f: any) => s + (f.sizeBytes || f.size || 0), 0))
const totalSizeFormatted = computed(() => formatSize(totalSize.value))

const largestFile = computed(() => {
  if (files.value.length === 0) return '--'
  const biggest = [...files.value].sort((a: any, b: any) => (b.sizeBytes || b.size || 0) - (a.sizeBytes || a.size || 0))[0]
  return `${biggest.name} (${formatSize(biggest.sizeBytes || biggest.size || 0)})`
})

const avgSizeFormatted = computed(() => {
  if (files.value.length === 0) return '--'
  return formatSize(totalSize.value / files.value.length)
})

const gpsCount = computed(() => files.value.filter((f: any) => f.gpsLat || f.gpsLon).length)
const faceCount = computed(() => files.value.filter((f: any) => f.faceGroupIds?.length).length)

const byType = computed(() => {
  const groups: Record<string, { totalBytes: number; count: number; color: string }> = {
    image: { totalBytes: 0, count: 0, color: '#00ff41' },
    text: { totalBytes: 0, count: 0, color: '#5af0ff' },
    video: { totalBytes: 0, count: 0, color: '#ff6b9d' },
    audio: { totalBytes: 0, count: 0, color: '#b388ff' },
    archive: { totalBytes: 0, count: 0, color: '#ffd700' },
    other: { totalBytes: 0, count: 0, color: '#555' },
  }
  for (const f of files.value) {
    const mt = f.mimeType || ''
    let type = 'other'
    if (mt.startsWith('image/')) type = 'image'
    else if (mt.startsWith('text/') || mt.includes('json')) type = 'text'
    else if (mt.startsWith('video/')) type = 'video'
    else if (mt.startsWith('audio/')) type = 'audio'
    else if (mt.includes('zip') || mt.includes('gzip') || mt.includes('tar')) type = 'archive'
    groups[type].totalBytes += f.sizeBytes || f.size || 0
    groups[type].count++
  }
  const total = totalSize.value || 1
  const labels: Record<string, string> = {
    image: 'IMAGES', text: 'TEXT', video: 'VIDEO', audio: 'AUDIO', archive: 'ARCHIVE', other: 'OTHER',
  }
  return Object.entries(groups).map(([key, val]) => ({
    label: labels[key] || key.toUpperCase(),
    totalBytes: val.totalBytes,
    count: val.count,
    percent: (val.totalBytes / total) * 100,
    color: val.color,
  })).filter(e => e.count > 0)
})

function formatSize(bytes?: number): string {
  if (!bytes) return '-'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0; let s = bytes
  while (s >= 1024 && i < units.length - 1) { s /= 1024; i++ }
  return `${s.toFixed(1)} ${units[i]}`
}

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<style scoped>
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 8px;
}

.stat-card {
  text-align: center;
  will-change: transform, opacity;
}

.stat-value {
  display: block;
  font-family: var(--font-mono);
  font-size: var(--font-size-2xl);
  font-weight: 800;
  color: var(--text-accent);
  text-shadow: 0 0 8px var(--accent-glow);
  line-height: 1.2;
  will-change: transform, opacity;
}

.stat-label {
  display: block;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-top: 4px;
}

.type-breakdown {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.type-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.type-label {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  font-weight: 600;
  min-width: 64px;
  text-transform: uppercase;
}

.type-bar {
  flex: 1;
  height: 10px;
  background: var(--bg-overlay);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.type-bar-fill {
  height: 100%;
  border-radius: var(--radius-full);
  min-width: 2px;
  will-change: transform, width, opacity;
}

.type-size {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  min-width: 64px;
  text-align: right;
}

.info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 0;
  border-bottom: 1px solid var(--border-glass);
}

.info-row:last-child { border-bottom: none; }

.info-key {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-transform: uppercase;
}

.info-value {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  color: var(--text-accent);
  font-weight: 600;
}
</style>
