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
  resizable?: boolean
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
  resizable: false,
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
  resize: [w: number, h: number]
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
    class="os-window"
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
  border-radius: var(--radius-xl);
  overflow: hidden;
  transition: box-shadow var(--transition-normal), border-color var(--transition-normal);
  will-change: transform, opacity, box-shadow;
}

.os-window--default {
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  box-shadow: var(--shadow-window), inset 0 1px 0 rgba(255, 255, 255, 0.05);
}
.os-window--default.focused {
  border-color: var(--border-medium);
  box-shadow: 0 12px 48px rgba(0, 255, 65, 0.08), var(--shadow-window), inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

.os-window--glass {
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur-heavy));
  -webkit-backdrop-filter: blur(var(--glass-blur-heavy));
  border: 1px solid var(--border-glass);
  box-shadow: var(--shadow-glass);
}
.os-window--glass.focused {
  border-color: var(--border-glass-hover);
  box-shadow: 0 0 24px var(--accent-dim), var(--shadow-glass);
}

.os-window--neon {
  background: var(--bg-surface);
  border: 1px solid rgba(0, 255, 65, 0.2);
  box-shadow: 0 0 16px var(--accent-dim), var(--shadow-window);
}
.os-window--neon.focused {
  border-color: rgba(0, 255, 65, 0.4);
  box-shadow: 0 0 32px var(--accent-glow), var(--shadow-window);
}

.os-window--gothic {
  background: linear-gradient(180deg, #150808 0%, #0a0303 100%);
  border: 1px solid #2a1010;
  box-shadow: 0 8px 32px rgba(60, 10, 20, 0.3);
}
.os-window--gothic.focused {
  border-color: #3a1a1a;
  box-shadow: 0 8px 32px rgba(60, 10, 20, 0.5), 0 0 16px rgba(255, 107, 157, 0.08);
}

.os-window--cute {
  background: linear-gradient(135deg, rgba(255, 107, 157, 0.06), rgba(179, 136, 255, 0.06));
  border: 1px solid rgba(255, 107, 157, 0.15);
  border-radius: var(--radius-2xl);
}
.os-window--cute.focused {
  border-color: rgba(255, 107, 157, 0.3);
  box-shadow: 0 0 16px rgba(255, 107, 157, 0.08);
}

.os-window--sm { width: 360px; min-height: 240px; }
.os-window--md { width: 560px; min-height: 360px; }
.os-window--lg { width: 720px; min-height: 480px; }
.os-window--xl { width: 900px; min-height: 600px; }
.os-window--full { width: 100%; height: 100%; }

.os-window__titlebar {
  display: flex;
  align-items: center;
  height: 34px;
  padding: 0 12px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border-subtle);
  cursor: default;
  user-select: none;
  flex-shrink: 0;
  gap: 8px;
}

.os-window--glass .os-window__titlebar {
  background: rgba(0, 0, 0, 0.3);
}
.os-window--neon .os-window__titlebar {
  background: var(--bg-elevated);
  border-bottom: 1px solid rgba(0, 255, 65, 0.15);
}
.os-window--gothic .os-window__titlebar {
  background: #1a0808;
  border-bottom: 1px solid #2a1010;
}
.os-window--cute .os-window__titlebar {
  background: rgba(255, 107, 157, 0.05);
  border-bottom: 1px solid rgba(255, 107, 157, 0.1);
}

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
  transition: filter var(--transition-fast);
}
.os-window__dot:hover { filter: brightness(1.3); }

.os-window__dot--close { background: #ff5f57; }
.os-window__dot--minimize { background: #febc2e; }
.os-window__dot--maximize { background: #28c840; }

.os-window__title-icon { flex-shrink: 0; opacity: 0.7; }

.os-window__title-label {
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
  font-weight: 600;
  color: var(--text-secondary);
  letter-spacing: 0.3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.os-window__spacer { flex: 1; }

.os-window__content {
  flex: 1;
  overflow: auto;
  position: relative;
  padding: 12px;
}

.os-window__content--no-pad {
  padding: 0;
}

.os-window__content > :deep(*) {
  height: 100%;
}
</style>
