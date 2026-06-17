<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const panelRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const props = withDefaults(defineProps<{
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  padding?: 'none' | 'sm' | 'md' | 'lg'
  scrollable?: boolean
  fullHeight?: boolean
}>(), {
  variant: 'default',
  padding: 'md',
  scrollable: true,
  fullHeight: true,
})

const cls = computed(() => [
  'os-panel',
  `os-panel--${props.variant}`,
  `os-panel--pad-${props.padding}`,
  {
    'os-panel--scrollable': props.scrollable,
    'os-panel--full-height': props.fullHeight,
  },
])

onMounted(() => {
  gsapCtx.value = gsap.context(() => {
    if (panelRef.value) anim.fadeIn(panelRef.value, { from: { y: 4 } })
  })
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div :class="[...cls, 'gpu']" ref="panelRef">
    <slot />
  </div>
</template>

<style scoped>
.os-panel {
  display: flex;
  flex-direction: column;
  will-change: transform, opacity;
  contain: layout style;
}

.os-panel--full-height {
  height: 100%;
}

.os-panel--scrollable {
  overflow-y: auto;
  overflow-x: hidden;
}

.os-panel--pad-none { padding: 0; }
.os-panel--pad-sm { padding: 8px; }
.os-panel--pad-md { padding: 12px; }
.os-panel--pad-lg { padding: 16px; }

.os-panel--default {
  color: var(--text-secondary);
}

.os-panel--glass {
  position: relative;
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-glass), var(--panel-inset);
}
.os-panel--glass::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: linear-gradient(135deg, rgba(255,255,255,0.06), transparent 50%);
  pointer-events: none;
  z-index: 0;
}

.os-panel--neon {
  border: 1px solid rgba(0, 255, 65, 0.15);
  border-radius: var(--radius-lg);
  box-shadow: var(--glow-accent);
}

.os-panel--gothic {
  border: 1px solid #2a1010;
  border-radius: var(--radius-lg);
  background: rgba(20, 5, 5, 0.2);
}

.os-panel--cute {
  border: 1px solid rgba(255, 107, 157, 0.1);
  border-radius: var(--radius-xl);
}
</style>
