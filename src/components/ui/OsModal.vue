<script setup lang="ts">
import { computed, watch, onUnmounted, ref, nextTick } from 'vue'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'
import { useFocusTrap } from '@/composables/useFocusTrap'

const anim = useGsapAnimation()

const props = withDefaults(defineProps<{
  visible: boolean
  title?: string
  icon?: string
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
  closable?: boolean
  closeOnOverlay?: boolean
  noPadding?: boolean
}>(), {
  visible: false,
  variant: 'default',
  size: 'md',
  closable: true,
  closeOnOverlay: true,
  noPadding: false,
})

const emit = defineEmits<{
  'update:visible': [value: boolean]
  close: []
}>()

const uid = `os-modal-${Math.random().toString(36).slice(2, 9)}`
const overlayRef = ref<HTMLElement | null>(null)
const panelRef = ref<HTMLElement | null>(null)
const { activate: focusActivate, deactivate: focusDeactivate } = useFocusTrap(panelRef)

const titleId = computed(() => props.title ? `${uid}-title` : undefined)

const cls = computed(() => [
  'os-modal',
  `os-modal--${props.variant}`,
  `os-modal--${props.size}`,
  {
    'os-modal--open': props.visible,
    'os-modal--no-pad': props.noPadding,
  },
])

function onOverlayClick() {
  if (props.closeOnOverlay) close()
}

function close() {
  emit('update:visible', false)
  emit('close')
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.closable) close()
}

function onEnter(_el: Element, done: () => void) {
  const overlay = overlayRef.value
  const panel = panelRef.value
  if (overlay && panel) {
    anim.modalEnter(overlay, panel).then(done)
  } else {
    done()
  }
  focusActivate()
}

function onLeave(_el: Element, done: () => void) {
  const overlay = overlayRef.value
  const panel = panelRef.value
  if (overlay && panel) {
    anim.modalLeave(overlay, panel).then(done)
  } else {
    done()
  }
  focusDeactivate()
  document.removeEventListener('keydown', onKeydown)
}

watch(() => props.visible, (v) => {
  if (v) {
    document.addEventListener('keydown', onKeydown)
  } else {
    document.removeEventListener('keydown', onKeydown)
  }
}, { immediate: true })

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
  focusDeactivate()
})
</script>

<template>
  <Teleport to="body">
    <Transition :css="false" @enter="onEnter" @leave="onLeave">
      <div v-if="visible" class="os-modal__overlay gpu" ref="overlayRef" @click.self="onOverlayClick">
        <div
          :class="[...cls, 'gpu']"
          ref="panelRef"
          role="dialog"
          :aria-modal="true"
          :aria-labelledby="titleId"
        >
          <div v-if="title || icon || closable" class="os-modal__header">
            <OsIcon v-if="icon" :icon="icon" :size="16" />
            <span v-if="title" :id="titleId" class="os-modal__title">{{ title }}</span>
            <div class="os-modal__spacer" />
            <button v-if="closable" class="os-modal__close" @click="close" aria-label="Close modal">
              <OsIcon icon="mdi:close" :size="14" />
            </button>
          </div>
          <div :class="['os-modal__body', { 'os-modal__body--no-pad': noPadding }]">
            <slot />
          </div>
          <div v-if="$slots.footer" class="os-modal__footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.os-modal__overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  will-change: opacity;
}

.os-modal {
  display: flex;
  flex-direction: column;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-dropdown);
  color: var(--text-primary);
  max-height: 85vh;
  overflow: hidden;
  will-change: transform, opacity;
}

.os-modal--sm { width: 320px; }
.os-modal--md { width: 480px; }
.os-modal--lg { width: 640px; }
.os-modal--xl { width: 800px; }
.os-modal--full { width: 90vw; height: 90vh; }

.os-modal--glass {
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border: 1px solid var(--border-glass);
  box-shadow: var(--shadow-glass), var(--panel-inset);
}

.os-modal--neon {
  background: var(--bg-surface);
  border: 1px solid rgba(var(--accent-rgb), 0.3);
  box-shadow: 0 0 24px var(--accent-dim), var(--glow-accent);
}

.os-modal--gothic {
  background: linear-gradient(180deg, #1a0a0a 0%, #0f0505 100%);
  border: 1px solid #3a1a1a;
  box-shadow: 0 8px 32px rgba(60, 10, 20, 0.4);
}

.os-modal--cute {
  background: linear-gradient(135deg, rgba(255, 107, 157, 0.08), rgba(179, 136, 255, 0.08));
  border: 1px solid rgba(255, 107, 157, 0.2);
  border-radius: var(--radius-2xl);
}

.os-modal__header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  background: linear-gradient(90deg, var(--border-subtle), var(--accent-dim), var(--border-subtle));
  background-size: 200% 100%;
  animation: shimmer 3s ease-in-out infinite;
}

.os-modal__title {
  font-family: var(--font-mono);
  font-size: var(--font-size-md);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.os-modal__spacer { flex: 1; }

.os-modal__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--text-muted);
  transition: all var(--transition-fast);
}
.os-modal__close:hover {
  background: var(--accent-dim);
  color: var(--text-accent);
  box-shadow: 0 0 8px var(--accent-glow);
}

.os-modal__body {
  padding: 16px;
  overflow-y: auto;
  flex: 1;
}

.os-modal__body--no-pad {
  padding: 0;
}

.os-modal__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}
</style>
