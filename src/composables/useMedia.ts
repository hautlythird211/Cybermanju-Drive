import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app'
import type {
  FileMediaData,
  ResolutionData,
  ResolutionLevel,
  MediaInfo,
  ThumbnailResult,
  PlaybackState,
  PlaybackPosition,
} from '@/types'

const currentMediaData = ref<FileMediaData | null>(null)
const currentResolution = ref<ResolutionLevel>('r3')
const resolutionCache = ref<Map<string, ResolutionData>>(new Map())
const mediaOverlayVisible = ref(false)
const mediaOverlayType = ref<'image' | 'video' | 'audio'>('image')
const mediaFileData = ref<FileMediaData | null>(null)
const mediaFileBytes = ref<Uint8Array | null>(null)

const playbackState = ref<PlaybackState>('stopped')
const playbackPosition = ref<PlaybackPosition>({ currentSecs: 0, totalSecs: 0, speed: 1 })
const volume = ref(1.0)
const isMuted = ref(false)
const playbackSpeed = ref(1.0)

const slideshowActive = ref(false)
const slideshowInterval = ref(3000)
const slideshowIndex = ref(0)

export function useMedia() {
  async function getMediaInfo(
    fileId: string,
    filename: string,
    data: Uint8Array
  ): Promise<FileMediaData> {
    const result = await invoke<FileMediaData>('get_media_info', {
      fileId,
      filename,
      data: Array.from(data),
    })
    currentMediaData.value = result
    return result
  }

  async function getResolutionData(
    fileId: string,
    level: ResolutionLevel,
    data: Uint8Array,
    width?: number,
    height?: number
  ): Promise<ResolutionData> {
    const cacheKey = `${fileId}:${level}:${width || 0}:${height || 0}`
    if (resolutionCache.value.has(cacheKey)) {
      return resolutionCache.value.get(cacheKey)!
    }

    const result = await invoke<ResolutionData>('get_resolution_data', {
      fileId,
      level,
      data: Array.from(data),
      width,
      height,
    })

    resolutionCache.value.set(cacheKey, result)
    return result
  }

  async function transformImage(
    data: Uint8Array,
    transform: string
  ): Promise<Uint8Array> {
    const result = await invoke<number[]>('transform_image_file', {
      data: Array.from(data),
      transform,
    })
    return new Uint8Array(result)
  }

  async function generateThumbnail(
    data: Uint8Array,
    maxSize: number = 200,
    format: string = 'webp',
    quality: number = 85
  ): Promise<ThumbnailResult> {
    return await invoke<ThumbnailResult>('generate_thumbnail_cmd', {
      data: Array.from(data),
      maxSize,
      format,
      quality,
    })
  }

  async function detectMediaType(
    data: Uint8Array,
    filename: string
  ): Promise<MediaInfo> {
    return await invoke<MediaInfo>('detect_media_type_cmd', {
      data: Array.from(data),
      filename,
    })
  }

  async function getFileBytesForPreview(fileId: string): Promise<Uint8Array> {
    const result = await invoke<number[]>('get_file_bytes_for_preview', {
      fileId,
    })
    return new Uint8Array(result)
  }

  async function getFileRawBytes(fileId: string): Promise<Uint8Array> {
    const result = await invoke<number[]>('get_file_raw_bytes', {
      fileId,
    })
    return new Uint8Array(result)
  }

  async function getTextPreview(fileId: string, maxChars?: number): Promise<string> {
    return await invoke<string>('get_text_preview', {
      fileId,
      maxChars,
    })
  }

  async function getMediaInfoWithPreview(
    fileId: string,
    filename: string
  ): Promise<{ mediaData: FileMediaData; fileBytes: Uint8Array }> {
    const [mediaData, fileBytesArr] = await invoke<[FileMediaData, number[]]>(
      'get_media_info_with_preview',
      { fileId, filename }
    )
    return { mediaData, fileBytes: new Uint8Array(fileBytesArr) }
  }

  async function batchGenerateThumbnails(
    items: [string, Uint8Array][],
    maxSize: number = 200,
    format: string = 'webp',
    quality: number = 85
  ): Promise<Map<string, ThumbnailResult | null>> {
    const result = await invoke<[string, ThumbnailResult | null][]>(
      'batch_generate_thumbnails_cmd',
      {
        items: items.map(([id, data]) => [id, Array.from(data)]),
        maxSize,
        format,
        quality,
      }
    )
    const map = new Map<string, ThumbnailResult | null>()
    for (const [id, thumb] of result) {
      map.set(id, thumb)
    }
    return map
  }

  function openMediaOverlay(
    type: 'image' | 'video' | 'audio',
    fileData: FileMediaData,
    fileBytes: Uint8Array
  ) {
    const store = useAppStore()
    mediaOverlayType.value = type
    mediaFileData.value = fileData
    mediaFileBytes.value = fileBytes
    mediaOverlayVisible.value = true
    store.mediaOverlayVisible = true
    store.mediaOverlayType = type
    store.mediaFileData = fileData
    store.mediaFileBytes = fileBytes
    if (type === 'image') {
      currentResolution.value = 'r3'
    } else if (type === 'video') {
      playbackState.value = 'playing'
    }
  }

  function closeMediaOverlay() {
    const store = useAppStore()
    mediaOverlayVisible.value = false
    mediaFileData.value = null
    mediaFileBytes.value = null
    store.mediaOverlayVisible = false
    store.mediaFileData = null
    store.mediaFileBytes = null
    playbackState.value = 'stopped'
    playbackPosition.value = { currentSecs: 0, totalSecs: 0, speed: 1 }
    slideshowActive.value = false
  }

  function setResolution(level: ResolutionLevel) {
    currentResolution.value = level
  }

  function play() {
    playbackState.value = 'playing'
  }

  function pause() {
    playbackState.value = 'paused'
  }

  function stop() {
    playbackState.value = 'stopped'
    playbackPosition.value = { currentSecs: 0, totalSecs: 0, speed: 1 }
  }

  function seek(secs: number) {
    playbackPosition.value = {
      ...playbackPosition.value,
      currentSecs: Math.max(0, Math.min(secs, playbackPosition.value.totalSecs)),
    }
  }

  function setVolume(v: number) {
    volume.value = Math.max(0, Math.min(2, v))
  }

  function toggleMute() {
    isMuted.value = !isMuted.value
  }

  function setSpeed(s: number) {
    playbackSpeed.value = Math.max(0.25, Math.min(4, s))
    playbackPosition.value = {
      ...playbackPosition.value,
      speed: playbackSpeed.value,
    }
  }

  function toggleSlideshow() {
    slideshowActive.value = !slideshowActive.value
  }

  function setSlideshowInterval(ms: number) {
    slideshowInterval.value = ms
  }

  const currentResolutionInfo = computed(() => {
    if (!currentMediaData.value) return null
    return currentMediaData.value.availableResolutions.find(
      (r) => r.level === currentResolution.value
    )
  })

  const isMediaFile = computed(() => {
    if (!mediaFileData.value) return false
    return mediaFileData.value.isImage || mediaFileData.value.isVideo || mediaFileData.value.isAudio
  })

  return {
    currentMediaData,
    currentResolution,
    resolutionCache,
    mediaOverlayVisible,
    mediaOverlayType,
    mediaFileData,
    mediaFileBytes,
    playbackState,
    playbackPosition,
    volume,
    isMuted,
    playbackSpeed,
    slideshowActive,
    slideshowInterval,
    slideshowIndex,
    currentResolutionInfo,
    isMediaFile,
    getMediaInfo,
    getResolutionData,
    transformImage,
    generateThumbnail,
    detectMediaType,
    batchGenerateThumbnails,
    getFileBytesForPreview,
    getFileRawBytes,
    getTextPreview,
    getMediaInfoWithPreview,
    openMediaOverlay,
    closeMediaOverlay,
    setResolution,
    play,
    pause,
    stop,
    seek,
    setVolume,
    toggleMute,
    setSpeed,
    toggleSlideshow,
    setSlideshowInterval,
  }
}
