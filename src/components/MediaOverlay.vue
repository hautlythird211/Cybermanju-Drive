<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useMedia } from '@/composables/useMedia'
import type { FileMediaData, FileNode } from '@/types'
import ImageViewer from './ImageViewer.vue'
import VideoPlayer from './VideoPlayer.vue'
import AudioPlayer from './AudioPlayer.vue'

const props = defineProps<{
  visible: boolean
  file?: FileNode | null
}>()

const emit = defineEmits<{
  close: []
}>()

const {
  mediaOverlayType,
  mediaFileData,
  mediaFileBytes,
  getMediaInfo,
  closeMediaOverlay,
} = useMedia()

const loading = ref(false)
const error = ref<string | null>(null)

const fileType = computed(() => {
  if (!mediaFileData.value) return null
  if (mediaFileData.value.isImage) return 'image'
  if (mediaFileData.value.isVideo) return 'video'
  if (mediaFileData.value.isAudio) return 'audio'
  return null
})

watch(
  () => props.file,
  async (file) => {
    if (!file) return
    loading.value = true
    error.value = null
    try {
      const data = new Uint8Array(0)
      await getMediaInfo(file.id, file.name, data)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  },
  { immediate: true }
)

function handleClose() {
  closeMediaOverlay()
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <Transition name="media-overlay">
      <div v-if="visible && mediaFileData" class="media-overlay">
        <ImageViewer
          v-if="fileType === 'image'"
          :file-data="mediaFileData"
          :file-bytes="mediaFileBytes!"
          @close="handleClose"
        />
        <VideoPlayer
          v-else-if="fileType === 'video'"
          :file-data="mediaFileData"
          :file-bytes="mediaFileBytes!"
          @close="handleClose"
        />
        <AudioPlayer
          v-else-if="fileType === 'audio'"
          :file-data="mediaFileData"
          :file-bytes="mediaFileBytes!"
          @close="handleClose"
        />
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.media-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
}

.media-overlay-enter-active,
.media-overlay-leave-active {
  transition: opacity 0.2s ease;
}

.media-overlay-enter-from,
.media-overlay-leave-to {
  opacity: 0;
}
</style>
