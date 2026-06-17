<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  variant?: 'default' | 'neon' | 'gothic' | 'thin'
  maxHeight?: string
}>(), {
  variant: 'default',
})

const cls = computed(() => [
  'os-scrollarea',
  `os-scrollarea--${props.variant}`,
])

const style = computed(() => {
  if (props.maxHeight) return { maxHeight: props.maxHeight }
  return {}
})
</script>

<template>
  <div :class="cls" :style="style" role="region">
    <slot />
  </div>
</template>

<style scoped>
.os-scrollarea {
  overflow-y: auto;
  overflow-x: hidden;
  overscroll-behavior: contain;
  scrollbar-gutter: stable;
  contain: paint;
  content-visibility: auto;
  contain-intrinsic-size: 0 500px;
}

.os-scrollarea--default {
  scrollbar-width: thin;
}

.os-scrollarea--thin {
  scrollbar-width: thin;
}

.os-scrollarea--thin::-webkit-scrollbar { width: 3px; }
.os-scrollarea--thin::-webkit-scrollbar-track { background: transparent; }
.os-scrollarea--thin::-webkit-scrollbar-thumb { background: var(--border-subtle); border-radius: 2px; }

.os-scrollarea--neon::-webkit-scrollbar-thumb { background: var(--accent-dim); }
.os-scrollarea--neon::-webkit-scrollbar-thumb:hover { background: var(--accent-glow); }

.os-scrollarea--gothic::-webkit-scrollbar-thumb { background: rgba(255, 107, 157, 0.3); }
.os-scrollarea--gothic::-webkit-scrollbar-thumb:hover { background: rgba(255, 107, 157, 0.5); }

.os-scrollarea::-webkit-scrollbar { width: 6px; }
.os-scrollarea::-webkit-scrollbar-track { background: transparent; }
.os-scrollarea::-webkit-scrollbar-thumb { background: var(--border-subtle); border-radius: 3px; }
.os-scrollarea::-webkit-scrollbar-thumb:hover { background: var(--accent); }
</style>
