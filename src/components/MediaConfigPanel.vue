<script setup lang="ts">
import { ref, onMounted } from 'vue'

const emit = defineEmits<{
  close: []
}>()

const thumbnailSize = ref(200)
const thumbnailFormat = ref('webp')
const thumbnailQuality = ref(85)
const defaultResolution = ref('r3')
const autoPlayVideos = ref(true)
const showExifByDefault = ref(false)
const slideshowInterval = ref(3000)
const enableHardwareAccel = ref(true)

const THUMBNAIL_SIZES = [
  { value: 100, label: '100px' },
  { value: 150, label: '150px' },
  { value: 200, label: '200px' },
  { value: 300, label: '300px' },
]

const THUMBNAIL_FORMATS = [
  { value: 'webp', label: 'WebP' },
  { value: 'jpeg', label: 'JPEG' },
  { value: 'png', label: 'PNG' },
]

const RESOLUTIONS = [
  { value: 'r0', label: 'R0 — Thumbnail (200x150)' },
  { value: 'r1', label: 'R1 — Preview (640x480)' },
  { value: 'r2', label: 'R2 — Medium (1920x1080)' },
  { value: 'r3', label: 'R3 — Original' },
]

function save() {
  const config = {
    thumbnailSize: thumbnailSize.value,
    thumbnailFormat: thumbnailFormat.value,
    thumbnailQuality: thumbnailQuality.value,
    defaultResolution: defaultResolution.value,
    autoPlayVideos: autoPlayVideos.value,
    showExifByDefault: showExifByDefault.value,
    slideshowInterval: slideshowInterval.value,
    enableHardwareAccel: enableHardwareAccel.value,
  }
  localStorage.setItem('cybermanju_media_config', JSON.stringify(config))
}

function load() {
  try {
    const saved = localStorage.getItem('cybermanju_media_config')
    if (saved) {
      const config = JSON.parse(saved)
      thumbnailSize.value = config.thumbnailSize ?? 200
      thumbnailFormat.value = config.thumbnailFormat ?? 'webp'
      thumbnailQuality.value = config.thumbnailQuality ?? 85
      defaultResolution.value = config.defaultResolution ?? 'r3'
      autoPlayVideos.value = config.autoPlayVideos ?? true
      showExifByDefault.value = config.showExifByDefault ?? false
      slideshowInterval.value = config.slideshowInterval ?? 3000
      enableHardwareAccel.value = config.enableHardwareAccel ?? true
    }
  } catch {}
}

onMounted(() => load())
</script>

<template>
  <div class="config-panel">
    <div class="config-header">
      <Icon icon="mdi:cog-outline" width="16" height="16" />
      <span>MEDIA SETTINGS</span>
      <button class="close-btn" @click="emit('close')">
        <Icon icon="mdi:close" width="16" height="16" />
      </button>
    </div>

    <div class="config-section">
      <div class="section-title">THUMBNAILS</div>
      <div class="config-row">
        <label>Size</label>
        <select v-model="thumbnailSize" @change="save" class="config-select">
          <option v-for="s in THUMBNAIL_SIZES" :key="s.value" :value="s.value">{{ s.label }}</option>
        </select>
      </div>
      <div class="config-row">
        <label>Format</label>
        <select v-model="thumbnailFormat" @change="save" class="config-select">
          <option v-for="f in THUMBNAIL_FORMATS" :key="f.value" :value="f.value">{{ f.label }}</option>
        </select>
      </div>
      <div class="config-row">
        <label>Quality</label>
        <div class="range-row">
          <input type="range" v-model.number="thumbnailQuality" min="50" max="100" step="5" @change="save" class="config-range" />
          <span class="range-value">{{ thumbnailQuality }}%</span>
        </div>
      </div>
    </div>

    <div class="config-section">
      <div class="section-title">RESOLUTION</div>
      <div class="config-row">
        <label>Default</label>
        <select v-model="defaultResolution" @change="save" class="config-select">
          <option v-for="r in RESOLUTIONS" :key="r.value" :value="r.value">{{ r.label }}</option>
        </select>
      </div>
    </div>

    <div class="config-section">
      <div class="section-title">PLAYBACK</div>
      <div class="config-row">
        <label>Auto-play videos</label>
        <button class="toggle-btn" :class="{ active: autoPlayVideos }" @click="autoPlayVideos = !autoPlayVideos; save()">
          <div class="toggle-track">
            <div class="toggle-thumb"></div>
          </div>
        </button>
      </div>
      <div class="config-row">
        <label>Show EXIF by default</label>
        <button class="toggle-btn" :class="{ active: showExifByDefault }" @click="showExifByDefault = !showExifByDefault; save()">
          <div class="toggle-track">
            <div class="toggle-thumb"></div>
          </div>
        </button>
      </div>
      <div class="config-row">
        <label>Hardware acceleration</label>
        <button class="toggle-btn" :class="{ active: enableHardwareAccel }" @click="enableHardwareAccel = !enableHardwareAccel; save()">
          <div class="toggle-track">
            <div class="toggle-thumb"></div>
          </div>
        </button>
      </div>
      <div class="config-row">
        <label>Slideshow interval</label>
        <div class="range-row">
          <input type="range" v-model.number="slideshowInterval" min="1000" max="10000" step="500" @change="save" class="config-range" />
          <span class="range-value">{{ slideshowInterval / 1000 }}s</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-panel {
  background: #0e0e12;
  border: 1px solid #22222a;
  border-radius: 12px;
  padding: 16px;
  width: 300px;
  font-family: var(--font-mono, monospace);
}

.config-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  font-weight: 800;
  color: #ececf0;
  letter-spacing: 1.5px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #222;
}

.close-btn {
  margin-left: auto;
  background: none;
  border: none;
  color: #50505e;
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
}

.close-btn:hover {
  color: #ff453a;
  background: rgba(255, 69, 58, 0.1);
}

.config-section {
  margin-bottom: 16px;
}

.section-title {
  font-size: 9px;
  font-weight: 700;
  color: #50505e;
  letter-spacing: 2px;
  margin-bottom: 8px;
}

.config-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
}

.config-row label {
  font-size: 10px;
  color: #a0a0b0;
}

.config-select {
  background: #16161c;
  border: 1px solid #333;
  color: #ececf0;
  padding: 4px 8px;
  border-radius: 4px;
  font-family: inherit;
  font-size: 10px;
  outline: none;
}

.config-select:focus {
  border-color: #00ff41;
}

.range-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.config-range {
  width: 100px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: #222;
  border-radius: 2px;
  outline: none;
}

.config-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #00ff41;
  cursor: pointer;
}

.range-value {
  font-size: 10px;
  color: #00ff41;
  min-width: 30px;
  text-align: right;
  font-weight: 600;
}

.toggle-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
}

.toggle-track {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: #333;
  position: relative;
  transition: background 0.2s;
}

.toggle-btn.active .toggle-track {
  background: #00ff41;
}

.toggle-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.2s;
}

.toggle-btn.active .toggle-thumb {
  transform: translateX(16px);
}
</style>
