<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useMedia } from '@/composables/useMedia'
import type { FileMediaData, ResolutionLevel, ResolutionData } from '@/types'

const props = defineProps<{
  fileData: FileMediaData
  fileBytes: Uint8Array
  allFiles?: FileMediaData[]
}>()

const emit = defineEmits<{
  close: []
  next: []
  prev: []
  transform: [action: string]
}>()

const {
  currentResolution,
  getResolutionData,
  transformImage,
  setResolution,
} = useMedia()

const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)
const rotation = ref(0)
const flipH = ref(false)
const flipV = ref(false)
const loading = ref(false)
const showExif = ref(false)
const showResolutionPicker = ref(false)
const fitToScreen = ref(true)
const imgRef = ref<HTMLImageElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const resolutionData = ref<ResolutionData | null>(null)
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })

const RESOLUTIONS: { level: ResolutionLevel; label: string; desc: string }[] = [
  { level: 'r0', label: 'R0', desc: '200x150 Thumbnail' },
  { level: 'r1', label: 'R1', desc: '640x480 Preview' },
  { level: 'r2', label: 'R2', desc: '1920x1080 Medium' },
  { level: 'r3', label: 'R3', desc: 'Original' },
]

const imageUrl = computed(() => {
  if (resolutionData.value) {
    const bytes = atob(resolutionData.value.dataBase64)
    const arr = new Uint8Array(bytes.length)
    for (let i = 0; i < bytes.length; i++) arr[i] = bytes.charCodeAt(i)
    const blob = new Blob([arr], { type: `image/${resolutionData.value.format}` })
    return URL.createObjectURL(blob)
  }
  if (props.fileBytes) {
    const copy = new Uint8Array(props.fileBytes)
    const blob = new Blob([copy])
    return URL.createObjectURL(blob)
  }
  return ''
})

const imageStyle = computed(() => ({
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${zoom.value}) rotate(${rotation.value}deg) scaleX(${flipH.value ? -1 : 1}) scaleY(${flipV.value ? -1 : 1})`,
  transition: isDragging.value ? 'none' : 'transform 0.2s ease',
  cursor: isDragging.value ? 'grabbing' : 'grab',
}))

const exifData = computed(() => props.fileData.imageInfo?.exif)

async function loadResolution(level: ResolutionLevel) {
  loading.value = true
  try {
    const data = await getResolutionData(
      props.fileData.fileId,
      level,
      props.fileBytes,
      level === 'r0' ? 200 : level === 'r1' ? 640 : level === 'r2' ? 1920 : undefined,
      level === 'r0' ? 150 : level === 'r1' ? 480 : level === 'r2' ? 1080 : undefined
    )
    resolutionData.value = data
    setResolution(level)
  } catch (e) {
    console.error('Failed to load resolution:', e)
  } finally {
    loading.value = false
  }
}

function handleWheel(e: WheelEvent) {
  e.preventDefault()
  const delta = e.deltaY > 0 ? -0.1 : 0.1
  zoom.value = Math.max(0.1, Math.min(10, zoom.value + delta))
}

function handleMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  isDragging.value = true
  dragStart.value = { x: e.clientX - panX.value, y: e.clientY - panY.value }
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  panX.value = e.clientX - dragStart.value.x
  panY.value = e.clientY - dragStart.value.y
}

function handleMouseUp() {
  isDragging.value = false
}

function handleDoubleClick() {
  if (zoom.value > 1) {
    resetView()
  } else {
    zoom.value = 2
  }
}

function resetView() {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
  rotation.value = 0
  flipH.value = false
  flipV.value = false
  fitToScreen.value = true
}

async function rotateCW() {
  rotation.value = (rotation.value + 90) % 360
  emit('transform', 'rotate_cw')
}

async function rotateCCW() {
  rotation.value = (rotation.value - 90 + 360) % 360
  emit('transform', 'rotate_ccw')
}

function handleKeydown(e: KeyboardEvent) {
  switch (e.key) {
    case 'Escape':
      emit('close')
      break
    case 'ArrowRight':
      emit('next')
      break
    case 'ArrowLeft':
      emit('prev')
      break
    case '+':
    case '=':
      zoom.value = Math.min(10, zoom.value + 0.25)
      break
    case '-':
      zoom.value = Math.max(0.1, zoom.value - 0.25)
      break
    case '0':
      resetView()
      break
    case 'r':
      rotateCW()
      break
    case 'R':
      rotateCCW()
      break
    case 'f':
      flipH.value = !flipH.value
      break
    case 'F':
      flipV.value = !flipV.value
      break
    case 'i':
      showExif.value = !showExif.value
      break
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  await loadResolution('r3')
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (imageUrl.value) URL.revokeObjectURL(imageUrl.value)
})
</script>

<template>
  <div class="image-viewer" @wheel.prevent="handleWheel">
    <div
      ref="containerRef"
      class="viewer-canvas"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseUp"
      @dblclick="handleDoubleClick"
    >
      <div v-if="loading" class="loading-spinner">
        <div class="spinner-ring"></div>
        <span>LOADING...</span>
      </div>
      <img
        v-else-if="imageUrl"
        ref="imgRef"
        :src="imageUrl"
        :style="imageStyle"
        class="viewer-image"
        draggable="false"
        @load="fitToScreen = false"
      />
    </div>

    <div class="viewer-toolbar">
      <div class="toolbar-left">
        <button class="tool-btn" @click="emit('prev')" title="Previous (Left Arrow)">
          <Icon icon="mdi:chevron-left" width="20" height="20" />
        </button>
        <span class="file-name">{{ fileData.filename }}</span>
        <span class="resolution-badge">{{ currentResolution.toUpperCase() }}</span>
        <button class="tool-btn" @click="emit('next')" title="Next (Right Arrow)">
          <Icon icon="mdi:chevron-right" width="20" height="20" />
        </button>
      </div>

      <div class="toolbar-center">
        <button class="tool-btn" @click="rotateCCW" title="Rotate CCW (Shift+R)">
          <Icon icon="mdi:rotate-left" width="18" height="18" />
        </button>
        <button class="tool-btn" @click="rotateCW" title="Rotate CW (R)">
          <Icon icon="mdi:rotate-right" width="18" height="18" />
        </button>
        <button class="tool-btn" @click="flipH = !flipH" title="Flip Horizontal (F)">
          <Icon icon="mdi:flip-horizontal" width="18" height="18" />
        </button>
        <button class="tool-btn" @click="flipV = !flipV" title="Flip Vertical (Shift+F)">
          <Icon icon="mdi:flip-vertical" width="18" height="18" />
        </button>
        <div class="toolbar-divider"></div>
        <button class="tool-btn" @click="zoom = Math.max(0.1, zoom - 0.25)" title="Zoom Out (-)">
          <Icon icon="mdi:magnify-minus-outline" width="18" height="18" />
        </button>
        <span class="zoom-label">{{ Math.round(zoom * 100) }}%</span>
        <button class="tool-btn" @click="zoom = Math.min(10, zoom + 0.25)" title="Zoom In (+)">
          <Icon icon="mdi:magnify-plus-outline" width="18" height="18" />
        </button>
        <button class="tool-btn" @click="resetView" title="Reset View (0)">
          <Icon icon="mdi:fit-to-screen" width="18" height="18" />
        </button>
      </div>

      <div class="toolbar-right">
        <div class="resolution-selector">
          <button
            class="tool-btn"
            @click="showResolutionPicker = !showResolutionPicker"
            title="Resolution"
          >
            <Icon icon="mdi:quality-high" width="18" height="18" />
            <span class="res-label">{{ currentResolution.toUpperCase() }}</span>
          </button>
          <div v-if="showResolutionPicker" class="resolution-dropdown">
            <button
              v-for="res in RESOLUTIONS"
              :key="res.level"
              class="res-option"
              :class="{ active: currentResolution === res.level }"
              @click="loadResolution(res.level); showResolutionPicker = false"
            >
              <span class="res-level">{{ res.label }}</span>
              <span class="res-desc">{{ res.desc }}</span>
            </button>
          </div>
        </div>
        <button class="tool-btn" @click="showExif = !showExif" title="EXIF Info (I)">
          <Icon icon="mdi:information-outline" width="18" height="18" />
        </button>
        <button class="tool-btn close-btn" @click="emit('close')" title="Close (Esc)">
          <Icon icon="mdi:close" width="20" height="20" />
        </button>
      </div>
    </div>

    <Transition name="slide-right">
      <div v-if="showExif && exifData" class="exif-panel">
        <div class="exif-header">
          <span>EXIF DATA</span>
          <button class="tool-btn" @click="showExif = false">
            <Icon icon="mdi:close" width="16" height="16" />
          </button>
        </div>
        <div class="exif-grid">
          <div v-if="exifData.cameraMake" class="exif-item">
            <span class="exif-label">CAMERA</span>
            <span class="exif-value">{{ exifData.cameraMake }} {{ exifData.cameraModel }}</span>
          </div>
          <div v-if="exifData.dateTaken" class="exif-item">
            <span class="exif-label">DATE</span>
            <span class="exif-value">{{ exifData.dateTaken }}</span>
          </div>
          <div v-if="exifData.iso" class="exif-item">
            <span class="exif-label">ISO</span>
            <span class="exif-value">{{ exifData.iso }}</span>
          </div>
          <div v-if="exifData.fNumber" class="exif-item">
            <span class="exif-label">APERTURE</span>
            <span class="exif-value">f/{{ exifData.fNumber }}</span>
          </div>
          <div v-if="exifData.exposureTime" class="exif-item">
            <span class="exif-label">EXPOSURE</span>
            <span class="exif-value">{{ exifData.exposureTime }}</span>
          </div>
          <div v-if="exifData.focalLength" class="exif-item">
            <span class="exif-label">FOCAL</span>
            <span class="exif-value">{{ exifData.focalLength }}mm</span>
          </div>
          <div v-if="exifData.gpsLat" class="exif-item">
            <span class="exif-label">GPS</span>
            <span class="exif-value">{{ exifData.gpsLat.toFixed(6) }}, {{ exifData.gpsLon?.toFixed(6) }}</span>
          </div>
          <div class="exif-item">
            <span class="exif-label">SIZE</span>
            <span class="exif-value">{{ fileData.imageInfo?.width }}x{{ fileData.imageInfo?.height }}</span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.image-viewer {
  position: fixed;
  inset: 0;
  background: #08080a;
  z-index: 10001;
  display: flex;
  flex-direction: column;
  font-family: var(--font-mono, monospace);
}

.viewer-canvas {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  position: relative;
}

.viewer-image {
  max-width: 90vw;
  max-height: 85vh;
  object-fit: contain;
  user-select: none;
  -webkit-user-drag: none;
}

.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: #a0a0b0;
  font-size: 11px;
  letter-spacing: 2px;
}

.spinner-ring {
  width: 32px;
  height: 32px;
  border: 2px solid #222;
  border-top-color: #00ff41;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.viewer-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: #0e0e12;
  border-top: 1px solid #22222a;
  gap: 16px;
}

.toolbar-left,
.toolbar-center,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-left {
  flex: 1;
}

.toolbar-right {
  flex: 1;
  justify-content: flex-end;
}

.file-name {
  font-size: 11px;
  font-weight: 700;
  color: #ececf0;
  letter-spacing: 1px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resolution-badge {
  font-size: 9px;
  font-weight: 800;
  color: #00ff41;
  background: rgba(0, 255, 65, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  letter-spacing: 1px;
}

.tool-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: 1px solid transparent;
  color: #a0a0b0;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 10px;
  font-family: inherit;
  transition: all 0.15s;
}

.tool-btn:hover {
  background: #1e1e26;
  color: #ececf0;
  border-color: #333;
}

.close-btn:hover {
  background: rgba(255, 69, 58, 0.15);
  color: #ff453a;
  border-color: rgba(255, 69, 58, 0.3);
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: #333;
}

.zoom-label {
  font-size: 10px;
  color: #a0a0b0;
  min-width: 40px;
  text-align: center;
  font-weight: 600;
}

.res-label {
  font-size: 9px;
  font-weight: 800;
  letter-spacing: 1px;
}

.resolution-selector {
  position: relative;
}

.resolution-dropdown {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  background: #16161c;
  border: 1px solid #333;
  border-radius: 8px;
  padding: 4px;
  margin-bottom: 8px;
  z-index: 10;
  min-width: 180px;
}

.res-option {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  background: none;
  border: none;
  color: #a0a0b0;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  text-align: left;
}

.res-option:hover {
  background: #1e1e26;
  color: #ececf0;
}

.res-option.active {
  background: rgba(0, 255, 65, 0.1);
  color: #00ff41;
}

.res-level {
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 1px;
  min-width: 24px;
}

.res-desc {
  font-size: 10px;
  color: #50505e;
}

.res-option.active .res-desc {
  color: #00ff41;
}

.exif-panel {
  position: fixed;
  top: 48px;
  right: 0;
  width: 280px;
  bottom: 48px;
  background: #0e0e12;
  border-left: 1px solid #22222a;
  padding: 16px;
  overflow-y: auto;
  z-index: 10002;
}

.exif-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 800;
  color: #ececf0;
  letter-spacing: 1.5px;
  margin-bottom: 16px;
}

.exif-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.exif-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.exif-label {
  font-size: 9px;
  font-weight: 700;
  color: #50505e;
  letter-spacing: 1.5px;
}

.exif-value {
  font-size: 11px;
  color: #ececf0;
}

.slide-right-enter-active,
.slide-right-leave-active {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
