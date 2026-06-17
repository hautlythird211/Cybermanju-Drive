<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from 'vue'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

const props = withDefaults(defineProps<{
  title?: string
  icon?: string
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
  closable?: boolean
  minimizable?: boolean
  maximizable?: boolean
  draggable?: boolean
  noPadding?: boolean
  x?: number
  y?: number
  width?: number | string
  height?: number | string
  zIndex?: number
  focused?: boolean
  animState?: 'idle' | 'entering' | 'exiting'
}>(), {
  variant: 'default',
  size: 'md',
  closable: true,
  minimizable: true,
  maximizable: false,
  draggable: true,
  noPadding: false,
  x: 0,
  y: 0,
  zIndex: 10,
  focused: false,
  animState: 'entering',
})

const emit = defineEmits<{
  close: []
  minimize: []
  maximize: []
  focus: []
  move: [x: number, y: number]
  'close-complete': []
}>()

const windowRef = ref<HTMLElement | null>(null)

const isExiting = computed(() => props.animState === 'exiting')

const windowStyle = computed(() => ({
  left: `${props.x}px`,
  top: `${props.y}px`,
  width: typeof props.width === 'number' ? `${props.width}px` : props.width,
  height: typeof props.height === 'number' ? `${props.height}px` : props.height,
  zIndex: props.zIndex,
}))

let dragging = false
let dragStartX = 0
let dragStartY = 0
let dragOrigX = 0
let dragOrigY = 0
let rafId: number | null = null
let latestX = 0
let latestY = 0

function startDrag(e: MouseEvent) {
  if (!props.draggable) return
  dragging = true
  dragStartX = e.clientX
  dragStartY = e.clientY
  dragOrigX = props.x
  dragOrigY = props.y
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

function onDrag(e: MouseEvent) {
  if (!dragging || !props.draggable) return
  latestX = e.clientX
  latestY = e.clientY
  if (rafId) return
  rafId = requestAnimationFrame(() => {
    const dx = latestX - dragStartX
    const dy = latestY - dragStartY
    emit('move', Math.max(0, dragOrigX + dx), Math.max(0, dragOrigY + dy))
    rafId = null
  })
}

function stopDrag() {
  dragging = false
  if (rafId) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

onMounted(async () => {
  await nextTick()
  if (windowRef.value && props.animState === 'entering') {
    anim.fadeIn(windowRef.value)
  }
})

watch(() => props.animState, (state) => {
  if (state === 'exiting' && windowRef.value) {
    anim.fadeOut(windowRef.value).then(() => {
      emit('close-complete')
    })
  }
})

onUnmounted(() => { stopDrag() })
</script>

<template>
  <div
    ref="windowRef"
    class="os-window gpu"
    :class="[`os-window--${variant}`, `os-window--${size}`, { focused, exiting: isExiting }]"
    :style="[windowStyle, { willChange: draggable ? 'left, top, transform, opacity, box-shadow' : 'transform, opacity, box-shadow' }]"
    role="window"
    :aria-label="title"
    @mousedown.prevent="emit('focus')"
  >
    <div class="os-window__titlebar" @mousedown.prevent="startDrag">
      <div class="os-window__dots">
        <span class="os-window__dot os-window__dot--close" @click.stop="emit('close')" title="Close" role="button" tabindex="0" aria-label="Close window" @keydown.enter="emit('close')" />
        <span class="os-window__dot os-window__dot--minimize" v-if="minimizable" @click.stop="emit('minimize')" title="Minimize" role="button" tabindex="0" aria-label="Minimize window" @keydown.enter="emit('minimize')" />
        <span class="os-window__dot os-window__dot--maximize" v-if="maximizable" @click.stop="emit('maximize')" title="Maximize" role="button" tabindex="0" aria-label="Maximize window" @keydown.enter="emit('maximize')" />
      </div>
      <OsIcon v-if="icon" :icon="icon" :size="12" class="os-window__title-icon" />
      <div class="os-window__title-label">{{ title }}</div>
      <div class="os-window__spacer" />
    </div>
    <div class="os-window__content" :class="{ 'os-window__content--no-pad': noPadding }">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.os-window {
  position: absolute;
  display: flex;
  flex-direction: column;
  border-radius: 10px;
  overflow: hidden;
  transition: box-shadow 0.2s, border-color 0.2s;
  will-change: transform, opacity, box-shadow;
  contain: layout style;
  background: rgba(14, 14, 14, 0.96);
  border: 1px solid rgba(255, 255, 255, 0.06);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}
.os-window.focused {
  border-color: rgba(255, 255, 255, 0.1);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(0, 255, 65, 0.15);
}

.os-window--default { background: rgba(14, 14, 14, 0.96); }
.os-window--glass { background: rgba(10, 10, 10, 0.92); backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px); }
.os-window--neon { border-color: rgba(0, 255, 65, 0.15); }
.os-window--neon.focused { border-color: rgba(0, 255, 65, 0.3); }
.os-window--gothic { background: rgba(10, 5, 5, 0.96); border-color: rgba(60, 15, 15, 0.6); }
.os-window--gothic.focused { border-color: rgba(80, 20, 20, 0.8); }
.os-window--cute { background: rgba(16, 10, 14, 0.96); border-color: rgba(255, 107, 157, 0.1); }
.os-window--cute.focused { border-color: rgba(255, 107, 157, 0.25); }

.os-window--sm { width: 360px; min-height: 200px; }
.os-window--md { width: 520px; min-height: 300px; }
.os-window--lg { width: 680px; min-height: 400px; }
.os-window--xl { width: 840px; min-height: 500px; }
.os-window--full { width: 100%; height: 100%; }

.os-window__titlebar {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 10px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  cursor: default;
  user-select: none;
  flex-shrink: 0;
  gap: 6px;
}

.os-window--glass .os-window__titlebar { background: rgba(0, 0, 0, 0.25); }
.os-window--neon .os-window__titlebar { border-bottom-color: rgba(0, 255, 65, 0.08); }
.os-window--gothic .os-window__titlebar { border-bottom-color: rgba(60, 15, 15, 0.4); }
.os-window--cute .os-window__titlebar { border-bottom-color: rgba(255, 107, 157, 0.08); }

.os-window__dots {
  display: flex;
  gap: 5px;
  flex-shrink: 0;
}

.os-window__dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  cursor: pointer;
  transition: opacity 0.15s;
  opacity: 0.7;
}
.os-window__dot:hover { opacity: 1; }

.os-window__dot--close { background: #ff5f57; }
.os-window__dot--minimize { background: #febc2e; }
.os-window__dot--maximize { background: #28c840; }

.os-window__title-icon { flex-shrink: 0; opacity: 0.5; }

.os-window__title-label {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 0.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.os-window__spacer { flex: 1; }

.os-window__content {
  flex: 1;
  overflow: auto;
  position: relative;
  padding: 8px;
}

.os-window__content--no-pad {
  padding: 0;
}

.os-window__content > :deep(*) {
  height: 100%;
}
</style>
