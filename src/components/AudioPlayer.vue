<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useMedia } from '@/composables/useMedia'
import type { FileMediaData } from '@/types'

const props = defineProps<{
  fileData: FileMediaData
  fileBytes: Uint8Array
}>()

const emit = defineEmits<{
  close: []
}>()

const { playbackState, playbackPosition, volume, isMuted, playbackSpeed, play, pause, stop, seek, setVolume, toggleMute, setSpeed } = useMedia()

const audioRef = ref<HTMLAudioElement | null>(null)
const progressRef = ref<HTMLDivElement | null>(null)
const visualizerCanvas = ref<HTMLCanvasElement | null>(null)
const audioContext = ref<AudioContext | null>(null)
const analyser = ref<AnalyserNode | null>(null)
const animationId = ref<number>(0)

const SPEEDS = [0.5, 0.75, 1, 1.25, 1.5, 2]
const showSpeedMenu = ref(false)

const audioUrl = computed(() => {
  if (props.fileBytes) {
    const copy = new Uint8Array(props.fileBytes)
    const blob = new Blob([copy], { type: props.fileData.mimeType })
    return URL.createObjectURL(blob)
  }
  return ''
})

const progress = computed(() => {
  if (playbackPosition.value.totalSecs === 0) return 0
  return (playbackPosition.value.currentSecs / playbackPosition.value.totalSecs) * 100
})

function formatTime(secs: number): string {
  const m = Math.floor(secs / 60)
  const s = Math.floor(secs % 60)
  return `${m}:${String(s).padStart(2, '0')}`
}

function handleTimeUpdate() {
  if (audioRef.value) {
    playbackPosition.value = {
      ...playbackPosition.value,
      currentSecs: audioRef.value.currentTime,
      totalSecs: audioRef.value.duration || 0,
    }
  }
}

function togglePlayPause() {
  if (!audioRef.value) return
  if (audioRef.value.paused) {
    audioRef.value.play()
  } else {
    audioRef.value.pause()
  }
}

function handleProgressClick(e: MouseEvent) {
  if (!progressRef.value || !audioRef.value) return
  const rect = progressRef.value.getBoundingClientRect()
  const ratio = (e.clientX - rect.left) / rect.width
  audioRef.value.currentTime = ratio * (audioRef.value.duration || 0)
}

function handleVolumeChange(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value)
  setVolume(val)
  if (audioRef.value) audioRef.value.volume = val
}

function initVisualizer() {
  if (!audioRef.value) return
  audioContext.value = new AudioContext()
  analyser.value = audioContext.value.createAnalyser()
  analyser.value.fftSize = 256
  const source = audioContext.value.createMediaElementSource(audioRef.value)
  source.connect(analyser.value)
  analyser.value.connect(audioContext.value.destination)
  drawVisualizer()
}

function drawVisualizer() {
  if (!visualizerCanvas.value || !analyser.value) return
  const canvas = visualizerCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const bufferLength = analyser.value.frequencyBinCount
  const dataArray = new Uint8Array(bufferLength)
  const context = ctx

  function draw() {
    animationId.value = requestAnimationFrame(draw)
    analyser.value!.getByteFrequencyData(dataArray)

    context.fillStyle = 'rgba(8, 8, 10, 0.3)'
    context.fillRect(0, 0, canvas.width, canvas.height)

    const barWidth = (canvas.width / bufferLength) * 2.5
    let x = 0

    for (let i = 0; i < bufferLength; i++) {
      const barHeight = (dataArray[i] / 255) * canvas.height
      const hue = (i / bufferLength) * 120 + 100
      context.fillStyle = `hsla(${hue}, 100%, 50%, 0.8)`
      context.fillRect(x, canvas.height - barHeight, barWidth, barHeight)
      x += barWidth + 1
    }
  }

  draw()
}

function handleKeydown(e: KeyboardEvent) {
  if (!audioRef.value) return
  switch (e.key) {
    case 'Escape':
      emit('close')
      break
    case ' ':
      e.preventDefault()
      togglePlayPause()
      break
    case 'ArrowLeft':
      audioRef.value.currentTime = Math.max(0, audioRef.value.currentTime - 10)
      break
    case 'ArrowRight':
      audioRef.value.currentTime = Math.min(audioRef.value.duration, audioRef.value.currentTime + 10)
      break
    case 'm':
      toggleMute()
      if (audioRef.value) audioRef.value.muted = !audioRef.value.muted
      break
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  setTimeout(() => initVisualizer(), 100)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (animationId.value) cancelAnimationFrame(animationId.value)
  if (audioContext.value) audioContext.value.close()
  if (audioUrl.value) URL.revokeObjectURL(audioUrl.value)
})
</script>

<template>
  <div class="audio-player">
    <div class="audio-visual">
      <canvas ref="visualizerCanvas" class="visualizer" width="600" height="200" />
      <div class="album-art">
        <Icon icon="mdi:music-note" width="48" height="48" class="music-icon" />
      </div>
    </div>

    <div class="audio-info">
      <div class="audio-title">{{ fileData.filename }}</div>
      <div class="audio-meta">{{ fileData.mimeType }}</div>
    </div>

    <div class="audio-controls">
      <div ref="progressRef" class="progress-bar" @click="handleProgressClick">
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
        </div>
      </div>

      <div class="time-row">
        <span>{{ formatTime(playbackPosition.currentSecs) }}</span>
        <span>{{ formatTime(playbackPosition.totalSecs) }}</span>
      </div>

      <div class="control-buttons">
        <button class="ctrl-btn" @click="audioRef && (audioRef.currentTime = Math.max(0, audioRef.currentTime - 10))">
          <Icon icon="mdi:rewind-10" width="20" height="20" />
        </button>
        <button class="play-btn" @click="togglePlayPause">
          <Icon :icon="playbackState === 'playing' ? 'mdi:pause' : 'mdi:play'" width="28" height="28" />
        </button>
        <button class="ctrl-btn" @click="audioRef && (audioRef.currentTime = Math.min(audioRef.duration, audioRef.currentTime + 10))">
          <Icon icon="mdi:fast-forward-10" width="20" height="20" />
        </button>
      </div>

      <div class="bottom-controls">
        <button class="ctrl-btn" @click="toggleMute(); audioRef && (audioRef.muted = !audioRef.muted)">
          <Icon :icon="isMuted ? 'mdi:volume-off' : 'mdi:volume-high'" width="18" height="18" />
        </button>
        <input
          type="range"
          class="volume-slider"
          min="0"
          max="2"
          step="0.05"
          :value="volume"
          @input="handleVolumeChange"
        />
        <div class="speed-selector">
          <button class="ctrl-btn speed-btn" @click="showSpeedMenu = !showSpeedMenu">
            {{ playbackSpeed }}x
          </button>
          <div v-if="showSpeedMenu" class="dropdown-menu">
            <button
              v-for="s in SPEEDS"
              :key="s"
              class="dropdown-item"
              :class="{ active: playbackSpeed === s }"
              @click="setSpeed(s); showSpeedMenu = false"
            >
              {{ s }}x
            </button>
          </div>
        </div>
        <button class="ctrl-btn close-btn" @click="emit('close')">
          <Icon icon="mdi:close" width="18" height="18" />
        </button>
      </div>
    </div>

    <audio ref="audioRef" :src="audioUrl" @timeupdate="handleTimeUpdate" preload="auto" />
  </div>
</template>

<style scoped>
.audio-player {
  position: fixed;
  inset: 0;
  background: #08080a;
  z-index: 10001;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono, monospace);
  gap: 24px;
}

.audio-visual {
  position: relative;
  width: 400px;
  height: 200px;
}

.visualizer {
  width: 100%;
  height: 100%;
  border-radius: 12px;
}

.album-art {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.music-icon {
  color: rgba(0, 255, 65, 0.3);
}

.audio-info {
  text-align: center;
}

.audio-title {
  font-size: 14px;
  font-weight: 700;
  color: #ececf0;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.audio-meta {
  font-size: 10px;
  color: #50505e;
}

.audio-controls {
  width: 400px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-bar {
  cursor: pointer;
}

.progress-track {
  height: 4px;
  background: #222;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #00ff41;
  border-radius: 2px;
}

.progress-bar:hover .progress-track {
  height: 6px;
}

.time-row {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  color: #50505e;
}

.control-buttons {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
}

.ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: #a0a0b0;
  padding: 8px;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.15s;
}

.ctrl-btn:hover {
  color: #ececf0;
  background: #1e1e26;
}

.play-btn {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: #00ff41;
  color: #08080a;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}

.play-btn:hover {
  transform: scale(1.05);
  box-shadow: 0 0 20px rgba(0, 255, 65, 0.3);
}

.bottom-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.volume-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: #222;
  border-radius: 2px;
  outline: none;
}

.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #00ff41;
  cursor: pointer;
}

.speed-selector {
  position: relative;
}

.speed-btn {
  font-size: 10px;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 4px;
}

.dropdown-menu {
  position: absolute;
  bottom: 100%;
  right: 0;
  background: #16161c;
  border: 1px solid #333;
  border-radius: 8px;
  padding: 4px;
  margin-bottom: 8px;
  z-index: 10;
  min-width: 80px;
}

.dropdown-item {
  display: block;
  width: 100%;
  background: none;
  border: none;
  color: #a0a0b0;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-family: inherit;
  font-size: 10px;
  text-align: left;
}

.dropdown-item:hover {
  background: #1e1e26;
  color: #ececf0;
}

.dropdown-item.active {
  color: #00ff41;
  background: rgba(0, 255, 65, 0.1);
}

.close-btn:hover {
  color: #ff453a;
  background: rgba(255, 69, 58, 0.1);
}
</style>
