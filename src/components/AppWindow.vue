<template>
  <OsWindow class="app-window"
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
    :maximizable="true"
    :maximized="win.maximized"
    no-padding
    @close="onClose"
    @minimize="onMinimize"
    @maximize="$emit('maximize', win.id)"
    @focus="$emit('focus', win.id)"
    @move="(x, y) => $emit('move', win.id, x, y)"
  >
    <div class="app-window-content">
      <component :is="win.component" v-bind="win.props" @close="onClose" />
    </div>
  </OsWindow>
</template>

<script setup lang="ts">
import { OsWindow } from '@/components/ui'
import type { WindowState } from '@/composables/useWindowManager'

const props = defineProps<{
  win: WindowState
  focused: boolean
}>()

const emit = defineEmits<{
  close: [id: string]
  minimize: [id: string]
  maximize: [id: string]
  focus: [id: string]
  move: [id: string, x: number, y: number]
}>()

function onClose() {
  emit('close', props.win.id)
}

function onMinimize() {
  emit('minimize', props.win.id)
}
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

/* Enhanced glass for app windows */
.app-window {
  contain: layout style;
  isolation: isolate;
}

.app-window.variant-glass {
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  box-shadow: var(--shadow-glass), var(--panel-inset), var(--glow-accent);
}
</style>
