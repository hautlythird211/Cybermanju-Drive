<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from 'vue'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

const props = withDefaults(defineProps<{
  title?: string
  icon?: string
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute' | 'apple'
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
  maximized?: boolean
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
  maximized: false,
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
  if (!props.draggable || props.maximized) return
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
    :class="[`os-window--${variant}`, `os-window--${size}`, { focused, 'os-window--maximized': props.maximized, exiting: isExiting }]"
    :style="[windowStyle, { willChange: draggable ? 'left, top, transform, opacity, box-shadow' : 'transform, opacity, box-shadow' }]"
    role="window"
    :aria-label="title"
    @mousedown.prevent="emit('focus')"
  >
    <div class="os-window__titlebar" @mousedown.prevent="startDrag" @dblclick.prevent="maximizable && emit('maximize')">
      <div class="os-window__dots">
        <span class="os-window__dot os-window__dot--close" @click.stop="emit('close')" title="Close" role="button" tabindex="0" aria-label="Close window" @keydown.enter="emit('close')" />
        <span class="os-window__dot os-window__dot--minimize" v-if="minimizable" @click.stop="emit('minimize')" title="Minimize" role="button" tabindex="0" aria-label="Minimize window" @keydown.enter="emit('minimize')" />
        <span class="os-window__dot os-window__dot--maximize" v-if="maximizable" @click.stop="emit('maximize')" title="Toggle Maximize" role="button" tabindex="0" aria-label="Toggle Maximize" @keydown.enter="emit('maximize')" />
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
  border-radius: 12px;
  transition: box-shadow 0.2s, border-color 0.2s, border-radius 0.2s;
  will-change: transform, opacity, box-shadow;
  contain: layout style;
  background: rgba(14, 14, 18, 0.94);
  border: 1px solid rgba(255, 255, 255, 0.06);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.45);
}
.os-window.focused {
  border-color: rgba(255, 255, 255, 0.1);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(var(--accent-rgb), 0.15);
}
.os-window--maximized {
  border-radius: 0;
  border: none;
}

.os-window--default { background: rgba(14, 14, 18, 0.94); }
.os-window--glass {
  background: rgba(10, 10, 14, 0.88);
  backdrop-filter: blur(24px) saturate(1.6);
  -webkit-backdrop-filter: blur(24px) saturate(1.6);
}
.os-window--glass.focused {
  border-color: rgba(var(--accent-rgb), 0.1);
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(var(--accent-rgb), 0.06);
}
.os-window--neon {
  border-color: rgba(var(--accent-rgb), 0.12);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 0 20px rgba(0, 255, 65, 0.03);
}
.os-window--neon.focused {
  border-color: rgba(0, 255, 65, 0.25);
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.5), 0 0 30px rgba(var(--accent-rgb), 0.06);
}
.os-window--gothic { background: rgba(10, 5, 5, 0.94); border-color: rgba(60, 15, 15, 0.5); }
.os-window--gothic.focused { border-color: rgba(80, 20, 20, 0.7); }
.os-window--cute { background: rgba(16, 10, 14, 0.94); border-color: rgba(255, 107, 157, 0.08); }
.os-window--cute.focused { border-color: rgba(255, 107, 157, 0.2); }

.os-window--apple {
  background: rgba(14, 14, 18, 0.82);
  backdrop-filter: blur(40px) saturate(1.8);
  -webkit-backdrop-filter: blur(40px) saturate(1.8);
  border: 1px solid rgba(255, 255, 255, 0.07);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.04);
}
.os-window--apple.focused {
  border-color: rgba(var(--accent-rgb), 0.08);
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.04), 0 0 30px rgba(0, 255, 65, 0.02);
}

.os-window--sm { width: 380px; min-height: 220px; }
.os-window--md { width: 540px; min-height: 320px; }
.os-window--lg { width: 720px; min-height: 420px; }
.os-window--xl { width: 880px; min-height: 520px; }
.os-window--full { width: 100%; height: 100%; }

.os-window__titlebar {
  display: flex;
  align-items: center;
  height: 32px;
  padding: 0 12px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  cursor: default;
  user-select: none;
  flex-shrink: 0;
  gap: 8px;
}

.os-window--glass .os-window__titlebar { background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid rgba(255, 255, 255, 0.03); }
.os-window--neon .os-window__titlebar { border-bottom-color: rgba(var(--accent-rgb), 0.06); }
.os-window--gothic .os-window__titlebar { border-bottom-color: rgba(60, 15, 15, 0.3); }
.os-window--cute .os-window__titlebar { border-bottom-color: rgba(255, 107, 157, 0.06); }
.os-window--apple .os-window__titlebar { background: rgba(0, 0, 0, 0.15); border-bottom: 1px solid rgba(255, 255, 255, 0.03); }

.os-window__dots {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.os-window__dot {
  width: 11px;
  height: 11px;
  border-radius: 50%;
  cursor: pointer;
  transition: opacity 0.15s, transform 0.15s;
  opacity: 0.8;
}
.os-window__dot:hover {
  opacity: 1;
  transform: scale(1.15);
}
.os-window__dot:active {
  transform: scale(0.9);
}

.os-window__dot--close { background: #ff453a; }
.os-window__dot--minimize { background: #ffd60a; }
.os-window__dot--maximize { background: #30d158; }

.os-window__title-icon {
  flex-shrink: 0;
  opacity: 0.5;
}

.os-window__title-label {
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif;
  font-size: 11px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 0.2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.os-window__spacer { flex: 1; }

.os-window__content {
  flex: 1;
  overflow: auto;
  position: relative;
  padding: 10px;
}

.os-window__content--no-pad {
  padding: 0;
}

.os-window__content > :deep(*) {
  height: 100%;
}
</style>
