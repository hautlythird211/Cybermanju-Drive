<template>
  <OsWindow
    :title="win.title"
    :icon="win.icon"
    variant="glass"
    :x="win.x"
    :y="win.y"
    :width="win.width || undefined"
    :height="win.height || undefined"
    :z-index="win.zIndex"
    :focused="focused"
    :anim-state="win.animState"
    no-padding
    :style="tileStyle"
    @close="onClose"
    @minimize="onMinimize"
    @focus="$emit('focus', win.id)"
    @move="(x, y) => $emit('move', win.id, x, y)"
    @resize="(w, h) => $emit('resize', win.id, w, h)"
  >
    <div ref="windowRef" class="app-window-content">
      <component :is="win.component" v-bind="win.props" @close="onClose" />
    </div>
  </OsWindow>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted } from 'vue'
import gsap from 'gsap'
import { OsWindow } from '@/components/ui'
import { useGsapAnimation } from '@/composables/useGsapAnimation'
import type { WindowState } from '@/composables/useWindowManager'

const anim = useGsapAnimation()
const windowRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const props = defineProps<{
  win: WindowState
  focused: boolean
  tileStyle?: Record<string, string | number>
}>()

const emit = defineEmits<{
  close: [id: string]
  minimize: [id: string]
  focus: [id: string]
  move: [id: string, x: number, y: number]
  resize: [id: string, w: number, h: number]
}>()

onMounted(() => {
  gsapCtx.value = gsap.context(() => {
    if (windowRef.value) {
      anim.fadeIn(windowRef.value, { from: { y: 8, opacity: 0 } })
    }
  })
})

watch(() => props.win.animState, (state) => {
  if (state === 'exiting' && windowRef.value) {
    gsapCtx.value?.add(() => {
      anim.fadeOut(windowRef.value!, { duration: 0.2 })
    })
  }
})

function onClose() {
  emit('close', props.win.id)
}

function onMinimize() {
  emit('minimize', props.win.id)
}

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<style scoped>
.app-window-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  will-change: transform, opacity;
}

.app-window-content > :deep(*) {
  flex: 1;
}
</style>
