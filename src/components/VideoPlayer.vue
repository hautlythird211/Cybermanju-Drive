<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useMedia } from '@/composables/useMedia'
import type { FileMediaData, ResolutionLevel } from '@/types'

const props = defineProps<{
  fileData: FileMediaData
  fileBytes: Uint8Array
}>()

const emit = defineEmits<{
  close: []
}>()

const {
  playbackState,
  playbackPosition,
  volume,
  isMuted,
  playbackSpeed,
  play,
  pause,
  stop,
  seek,
  setVolume,
  toggleMute,
  setSpeed,
  currentResolution,
  setResolution,
} = useMedia()

const videoRef = ref<HTMLVideoElement | null>(null)
const progressRef = ref<HTMLDivElement | null>(null)
const showControls = ref(true)
const controlsTimeout = ref<ReturnType<typeof setTimeout> | null>(null)
const showSpeedMenu = ref(false)
const showResolutionMenu = ref(false)
const isBuffering = ref(false)

const RESOLUTIONS: { level: ResolutionLevel; label: string; desc: string }[] = [
  { level: 'r0', label: 'R0', desc: 'Thumbnail' },
  { level: 'r1', label: 'R1', desc: '480p' },
  { level: 'r2', label: 'R2', desc: '1080p' },
  { level: 'r3', label: 'R3', desc: 'Original' },
]

const SPEEDS = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 2, 4]

const videoUrl = computed(() => {
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
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = Math.floor(secs % 60)
  if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  return `${m}:${String(s).padStart(2, '0')}`
}

function handleTimeUpdate() {
  if (videoRef.value) {
    playbackPosition.value = {
      ...playbackPosition.value,
      currentSecs: videoRef.value.currentTime,
      totalSecs: videoRef.value.duration || 0,
    }
  }
}

function handlePlay() {
  playbackState.value = 'playing'
}

function handlePause() {
  playbackState.value = 'paused'
}

function handleWaiting() {
  isBuffering.value = true
}

function handleCanPlay() {
  isBuffering.value = false
}

function togglePlayPause() {
  if (!videoRef.value) return
  if (videoRef.value.paused) {
    videoRef.value.play()
  } else {
    videoRef.value.pause()
  }
}

function handleProgressClick(e: MouseEvent) {
  if (!progressRef.value || !videoRef.value) return
  const rect = progressRef.value.getBoundingClientRect()
  const ratio = (e.clientX - rect.left) / rect.width
  const newTime = ratio * (videoRef.value.duration || 0)
  videoRef.value.currentTime = newTime
}

function handleVolumeChange(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value)
  setVolume(val)
  if (videoRef.value) {
    videoRef.value.volume = val
  }
}

function handleSpeedChange(s: number) {
  setSpeed(s)
  if (videoRef.value) {
    videoRef.value.playbackRate = s
  }
  showSpeedMenu.value = false
}

function handleFullscreen() {
  if (videoRef.value) {
    if (document.fullscreenElement) {
      document.exitFullscreen()
    } else {
      videoRef.value.requestFullscreen()
    }
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!videoRef.value) return
  switch (e.key) {
    case 'Escape':
      emit('close')
      break
    case ' ':
      e.preventDefault()
      togglePlayPause()
      break
    case 'ArrowLeft':
      videoRef.value.currentTime = Math.max(0, videoRef.value.currentTime - 10)
      break
    case 'ArrowRight':
      videoRef.value.currentTime = Math.min(videoRef.value.duration, videoRef.value.currentTime + 10)
      break
    case 'ArrowUp':
      e.preventDefault()
      setVolume(Math.min(2, volume.value + 0.1))
      videoRef.value.volume = volume.value
      break
    case 'ArrowDown':
      e.preventDefault()
      setVolume(Math.max(0, volume.value - 0.1))
      videoRef.value.volume = volume.value
      break
    case 'm':
      toggleMute()
      if (videoRef.value) videoRef.value.muted = !videoRef.value.muted
      break
    case 'f':
      handleFullscreen()
      break
  }
}

function showControlsTemporarily() {
  showControls.value = true
  if (controlsTimeout.value) clearTimeout(controlsTimeout.value)
  controlsTimeout.value = setTimeout(() => {
    if (playbackState.value === 'playing') {
      showControls.value = false
    }
  }, 3000)
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  window.addEventListener('mousemove', showControlsTemporarily)
  showControlsTemporarily()
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('mousemove', showControlsTemporarily)
  if (controlsTimeout.value) clearTimeout(controlsTimeout.value)
  if (videoUrl.value) URL.revokeObjectURL(videoUrl.value)
})
</script>

<template>
  <div class="video-player" @mousemove="showControlsTemporarily">
    <div class="video-container">
      <video
        ref="videoRef"
        :src="videoUrl"
        class="video-element"
        @timeupdate="handleTimeUpdate"
        @play="handlePlay"
        @pause="handlePause"
        @waiting="handleWaiting"
        @canplay="handleCanPlay"
        @dblclick="handleFullscreen"
        preload="auto"
      />

      <div v-if="isBuffering" class="buffering-overlay">
        <div class="buffering-spinner"></div>
      </div>

      <div class="play-overlay" v-if="playbackState === 'stopped'" @click="togglePlayPause">
        <div class="play-button-large">
          <Icon icon="mdi:play" width="48" height="48" />
        </div>
      </div>

      <Transition name="fade">
        <div v-show="showControls" class="controls-overlay">
          <div class="top-bar">
            <div class="top-left">
              <span class="video-title">{{ fileData.filename }}</span>
              <span class="resolution-tag">{{ currentResolution.toUpperCase() }}</span>
            </div>
            <div class="top-right">
              <div class="resolution-picker">
                <button class="ctrl-btn" @click="showResolutionMenu = !showResolutionMenu">
                  <Icon icon="mdi:quality-high" width="16" height="16" />
                </button>
                <div v-if="showResolutionMenu" class="dropdown-menu">
                  <button
                    v-for="res in RESOLUTIONS"
                    :key="res.level"
                    class="dropdown-item"
                    :class="{ active: currentResolution === res.level }"
                    @click="setResolution(res.level); showResolutionMenu = false"
                  >
                    {{ res.label }} — {{ res.desc }}
                  </button>
                </div>
              </div>
              <button class="ctrl-btn" @click="handleFullscreen">
                <Icon icon="mdi:fullscreen" width="18" height="18" />
              </button>
              <button class="ctrl-btn" @click="emit('close')">
                <Icon icon="mdi:close" width="18" height="18" />
              </button>
            </div>
          </div>

          <div class="bottom-bar">
            <div ref="progressRef" class="progress-bar" @click="handleProgressClick">
              <div class="progress-track">
                <div class="progress-fill" :style="{ width: progress + '%' }"></div>
              </div>
            </div>

            <div class="controls-row">
              <div class="controls-left">
                <button class="ctrl-btn" @click="togglePlayPause">
                  <Icon :icon="playbackState === 'playing' ? 'mdi:pause' : 'mdi:play'" width="22" height="22" />
                </button>
                <button class="ctrl-btn" @click="stop(); videoRef && (videoRef.currentTime = 0)">
                  <Icon icon="mdi:stop" width="18" height="18" />
                </button>
                <span class="time-display">
                  {{ formatTime(playbackPosition.currentSecs) }} / {{ formatTime(playbackPosition.totalSecs) }}
                </span>
              </div>

              <div class="controls-center">
                <button class="ctrl-btn" @click="videoRef && (videoRef.currentTime = Math.max(0, videoRef.currentTime - 10))">
                  <Icon icon="mdi:rewind-10" width="18" height="18" />
                </button>
                <button class="ctrl-btn" @click="videoRef && (videoRef.currentTime = Math.min(videoRef.duration, videoRef.currentTime + 10))">
                  <Icon icon="mdi:fast-forward-10" width="18" height="18" />
                </button>
              </div>

              <div class="controls-right">
                <button class="ctrl-btn" @click="toggleMute(); videoRef && (videoRef.muted = !videoRef.muted)">
                  <Icon :icon="isMuted ? 'mdi:volume-off' : volume > 1 ? 'mdi:volume-high' : 'mdi:volume-medium'" width="18" height="18" />
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
                  <div v-if="showSpeedMenu" class="dropdown-menu speed-menu">
                    <button
                      v-for="s in SPEEDS"
                      :key="s"
                      class="dropdown-item"
                      :class="{ active: playbackSpeed === s }"
                      @click="handleSpeedChange(s)"
                    >
                      {{ s }}x
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.video-player {
  position: fixed;
  inset: 0;
  background: #000;
  z-index: 10001;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono, monospace);
}

.video-container {
  position: relative;
  width: 100%;
  height: 100%;
}

.video-element {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.buffering-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
}

.buffering-spinner {
  width: 48px;
  height: 48px;
  border: 3px solid rgba(255, 255, 255, 0.2);
  border-top-color: #00ff41;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.play-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.play-button-large {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: rgba(0, 255, 65, 0.2);
  border: 2px solid #00ff41;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #00ff41;
  transition: all 0.2s;
}

.play-button-large:hover {
  background: rgba(0, 255, 65, 0.3);
  transform: scale(1.1);
}

.controls-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  pointer-events: none;
}

.controls-overlay > * {
  pointer-events: auto;
}

.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: linear-gradient(to bottom, rgba(0,0,0,0.8), transparent);
}

.top-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.video-title {
  font-size: 12px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.5px;
}

.resolution-tag {
  font-size: 9px;
  font-weight: 800;
  color: #00ff41;
  background: rgba(0, 255, 65, 0.15);
  padding: 2px 6px;
  border-radius: 4px;
  letter-spacing: 1px;
}

.top-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.bottom-bar {
  padding: 0 16px 12px;
  background: linear-gradient(to top, rgba(0,0,0,0.8), transparent);
}

.progress-bar {
  padding: 8px 0;
  cursor: pointer;
}

.progress-track {
  height: 4px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #00ff41;
  border-radius: 2px;
  transition: width 0.1s linear;
}

.progress-bar:hover .progress-track {
  height: 6px;
}

.controls-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.controls-left,
.controls-center,
.controls-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: #fff;
  padding: 6px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
}

.ctrl-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #00ff41;
}

.time-display {
  font-size: 11px;
  color: #ccc;
  font-weight: 600;
  letter-spacing: 0.5px;
  margin-left: 8px;
}

.volume-slider {
  width: 80px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: rgba(255, 255, 255, 0.2);
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

.speed-btn {
  font-size: 10px;
  font-weight: 700;
  padding: 4px 8px;
}

.speed-selector,
.resolution-picker {
  position: relative;
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
  min-width: 120px;
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

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
