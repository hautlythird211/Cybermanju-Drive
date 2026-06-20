<template>
  <div class="map-view">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-map">[@]</span>
        <h2 class="panel-title">GEOGRAPHY VIEW</h2>
      </div>
      <div class="header-actions">
        <button class="refresh-btn" @click="toggleFullscreen" title="FULLSCREEN (F)">[F]</button>
        <button class="refresh-btn" :class="{ active: mapStyle === 'satellite' }" @click="toggleMapStyle" title="TOGGLE MAP STYLE">[{{ mapStyle === 'satellite' ? 'SAT' : 'OSM' }}]</button>
        <button class="refresh-btn" @click="handleRefresh" title="REFRESH">[R]</button>
      </div>
    </div>

    <div class="search-location-bar">
      <input
        v-model="locationQuery"
        class="loc-search-input"
        placeholder="SEARCH LOCATION (E.G. TOKYO, 35.68,139.76)..."
        @keyup.enter="handleLocationSearch"
        aria-label="SEARCH LOCATION"
      />
      <button class="refresh-btn" @click="handleLocationSearch" title="SEARCH">[G]</button>
    </div>

    <div class="map-container" v-if="geoMarkers.length > 0">
      <div ref="mapContainer" class="maplibre-map"></div>
      <div class="map-stats-overlay">
        <span>{{ geoMarkers.length }} LOCATIONS</span>
      </div>
    </div>

    <div class="empty-state" v-if="geoMarkers.length === 0 && !isLoading">
      <Icon icon="svg-spinners:3-dots-rotate" width="20" height="20" class="mv-empty-spinner" />
      <p>NO GEOTAGGED FILES FOUND. PHOTOS WITH GPS EXIF DATA WILL APPEAR HERE.</p>
    </div>

    <div class="empty-state" v-if="isLoading">
      <Icon icon="svg-spinners:tadpole" width="32" height="32" class="mv-spinner" />
      <p>LOADING GEO DATA..</p>
    </div>

    <div class="section" v-if="geoMarkers.length > 0">
      <h3 class="section-title">[LIST] GEOTAGGED FILES ({{ geoMarkers.length }})</h3>
      <div class="geo-list">
        <div v-for="marker in geoMarkers" :key="'list-' + marker.fileId" class="geo-list-item" @click="flyToMarker(marker)">
          <span class="geo-list-pin">[@]</span>
          <div class="geo-list-info">
            <span class="geo-list-name">{{ marker.fileName }}</span>
            <span class="geo-list-address text-muted" v-if="marker.address">{{ marker.address }}</span>
          </div>
          <span class="geo-list-coords mono">{{ marker.lat.toFixed(3) }}, {{ marker.lng.toFixed(3) }}</span>
        </div>
      </div>
    </div>

    <div class="status-footer">
      <span>GPS EXTRACTION VIA KAMADAK-EXIF (RUST) | MAPLIBRE GL</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import type { GeoMarker } from '@/types'

const store = useAppStore()
const emit = defineEmits<{ close: [] }>()

const geoMarkers = computed(() => store.geoMarkers)
const isLoading = computed(() => store.isLoading)
const mapContainer = ref<HTMLDivElement | null>(null)
const locationQuery = ref('')
const mapStyle = ref<'osm' | 'satellite'>('osm')

let map: any = null
let maplibreglModule: any = null
let markers: any[] = []

onMounted(async () => {
  await store.fetchGeoFiles()
  await nextTick()
  if (geoMarkers.value.length > 0) initMap()
})

onUnmounted(() => destroyMap())

watch(geoMarkers, async (newMarkers) => {
  if (newMarkers.length > 0 && !map) {
    await nextTick()
    initMap()
  } else if (map) {
    updateMarkers()
  }
})

async function initMap() {
  if (!mapContainer.value || map) return
  try {
    maplibreglModule = await import('maplibre-gl')
    const center = getMapCenter()
    map = new maplibreglModule.Map({
      container: mapContainer.value,
      style: getMapStyle(),
      center: [center.lng, center.lat],
      zoom: center.zoom,
      attributionControl: false,
    })
    map.on('load', () => addMarkers())
  } catch (e) { console.warn('MapLibre GL failed:', e) }
}

function getMapStyle() {
  if (mapStyle.value === 'satellite') {
    return {
      version: 8,
      sources: {
        satellite: { type: 'raster', tiles: ['https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'], tileSize: 256, attribution: '&copy; Esri' },
        labels: { type: 'raster', tiles: ['https://server.arcgisonline.com/ArcGIS/rest/services/Reference/World_Boundaries_and_Places/MapServer/tile/{z}/{y}/{x}'], tileSize: 256, attribution: '&copy; Esri' },
      },
      layers: [
        { id: 'satellite', type: 'raster', source: 'satellite' },
        { id: 'labels', type: 'raster', source: 'labels' },
      ],
    }
  }
  return {
    version: 8,
    sources: {
      osm: { type: 'raster', tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'], tileSize: 256, attribution: '&copy; OpenStreetMap contributors' },
    },
    layers: [{ id: 'osm', type: 'raster', source: 'osm' }],
  }
}

function toggleMapStyle() {
  mapStyle.value = mapStyle.value === 'osm' ? 'satellite' : 'osm'
  if (map) {
    destroyMap()
    setTimeout(() => initMap(), 100)
  }
}

function toggleFullscreen() {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen()
  } else {
    document.exitFullscreen()
  }
}

async function handleLocationSearch() {
  const q = locationQuery.value.trim()
  if (!q) return
  const coordsMatch = q.match(/^(-?\d+\.?\d*)\s*[,;]\s*(-?\d+\.?\d*)$/)
  if (coordsMatch) {
    const lat = parseFloat(coordsMatch[1])
    const lng = parseFloat(coordsMatch[2])
    if (map) map.flyTo({ center: [lng, lat], zoom: 12, essential: true })
    return
  }
  try {
    const res = await fetch(`https://nominatim.openstreetmap.org/search?format=json&q=${encodeURIComponent(q)}&limit=1`)
    const data = await res.json()
    if (data && data.length > 0) {
      const { lat, lon } = data[0]
      if (map) map.flyTo({ center: [parseFloat(lon), parseFloat(lat)], zoom: 12, essential: true })
    }
  } catch {}
}

function destroyMap() {
  if (map) {
    markers.forEach((m: any) => m.remove())
    markers = []
    map.remove()
    map = null
  }
}

function getMapCenter() {
  if (geoMarkers.value.length === 0) return { lat: 20, lng: 0, zoom: 2 }
  const lats = geoMarkers.value.map(m => m.lat)
  const lngs = geoMarkers.value.map(m => m.lng)
  const avgLat = lats.reduce((a, b) => a + b, 0) / lats.length
  const avgLng = lngs.reduce((a, b) => a + b, 0) / lngs.length
  const latSpan = Math.max(...lats) - Math.min(...lats)
  const lngSpan = Math.max(...lngs) - Math.min(...lngs)
  const span = Math.max(latSpan, lngSpan)
  const zoom = span > 100 ? 2 : span > 50 ? 3 : span > 20 ? 4 : span > 5 ? 6 : 8
  return { lat: avgLat, lng: avgLng, zoom }
}

function addMarkers() {
  if (!map) return
  markers.forEach((m: any) => m.remove())
  markers = []
  geoMarkers.value.forEach((marker) => {
    const el = document.createElement('div')
    el.className = 'bw-marker'
    el.style.cssText = 'width:16px;height:16px;border:2px solid #000;background:#fff;cursor:pointer;'
    el.addEventListener('mouseenter', () => { el.style.transform = 'scale(1.4)' })
    el.addEventListener('mouseleave', () => { el.style.transform = 'scale(1)' })
    const m = new maplibreglModule!.Marker({ element: el }).setLngLat([marker.lng, marker.lat]).addTo(map)
    markers.push(m)
  })
}

function updateMarkers() { addMarkers() }

function flyToMarker(marker: GeoMarker) {
  if (map) map.flyTo({ center: [marker.lng, marker.lat], zoom: 12, essential: true })
}

async function handleRefresh() { await store.fetchGeoFiles() }
</script>

<style scoped>
.map-view {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}

.map-view::-webkit-scrollbar { width: 4px; }
.map-view::-webkit-scrollbar-track { background: transparent; }
.map-view::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 2px; }

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-map { font-size: 14px; color: var(--text-accent); }

.panel-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--text-primary);
  margin: 0;
}

.header-actions { display: flex; gap: 6px; }

.search-location-bar {
  display: flex;
  gap: 6px;
  align-items: center;
}

.loc-search-input {
  flex: 1;
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 6px 10px;
  border-radius: var(--radius-sm);
}

.loc-search-input::placeholder {
  color: var(--text-muted);
  opacity: 0.5;
}

.loc-search-input:focus {
  border-color: var(--border-accent);
  outline: none;
}

.refresh-btn {
  background: transparent;
  border: 1px solid var(--border-medium);
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px 8px;
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.refresh-btn:hover { background: var(--accent-dim); border-color: var(--border-accent); color: var(--text-accent); }
.refresh-btn.active { background: var(--accent-dim); border-color: var(--border-accent); color: var(--text-accent); }

.map-container {
  width: 100%;
  position: relative;
  min-height: 300px;
  border: 1px solid var(--border-glass);
  overflow: hidden;
  border-radius: var(--radius-md);
}

.maplibre-map { width: 100%; height: 300px; }

.map-stats-overlay {
  position: absolute;
  bottom: 6px;
  right: 6px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border: 1px solid var(--border-glass);
  padding: 4px 8px;
  font-size: 9px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  z-index: 5;
  border-radius: var(--radius-sm);
}

.section { display: flex; flex-direction: column; gap: 8px; }

.section-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-glass);
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: var(--font-mono);
}

.geo-list { display: flex; flex-direction: column; gap: 8px; }

.geo-list-item {
  border: 1px solid var(--border-glass);
  padding: 8px 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.geo-list-item:hover { background: var(--accent-dim); border-color: var(--border-accent); }

.geo-list-pin { flex-shrink: 0; font-size: 11px; }

.geo-list-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.geo-list-name { font-size: 11px; font-weight: 600; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.geo-list-address { font-size: 9px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.geo-list-coords { font-size: 9px; color: var(--text-muted); opacity: 0.7; flex-shrink: 0; }

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 40px;
  text-align: center;
}

.empty-state p { font-size: 11px; color: var(--text-muted); margin: 0; }

.mv-spinner {
  opacity: 0.8;
}

.mv-empty-spinner {
  opacity: 0.4;
}

.status-footer {
  margin-top: auto;
  padding-top: 10px;
  border-top: 1px solid var(--border-glass);
  font-size: 9px;
  color: var(--text-muted);
  opacity: 0.5;
  text-align: center;
}

.mono { font-family: var(--font-mono); }
.text-muted { color: var(--text-muted) !important; }
</style>

<style>
.maplibregl-popup-content {
  background: var(--bg-glass-heavy) !important;
  border: 1px solid var(--border-glass) !important;
  backdrop-filter: blur(var(--glass-blur-light));
  padding: 8px 12px !important;
  color: var(--text-primary) !important;
  font-family: var(--font-mono) !important;
  font-size: 11px !important;
  border-radius: var(--radius-sm) !important;
}
.maplibregl-popup-tip { border-top-color: transparent !important; }
</style>
