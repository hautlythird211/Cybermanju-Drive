<script setup lang="ts">
import { computed } from 'vue'
import OsIcon from './OsIcon.vue'

export interface StatusItem {
  id: string
  icon?: string
  label?: string
  value?: string
  color?: string
  pulse?: boolean
  onClick?: () => void
}

const props = withDefaults(defineProps<{
  items?: StatusItem[]
  variant?: 'default' | 'glass' | 'neon' | 'gothic'
  height?: number
}>(), {
  items: () => [],
  variant: 'default',
  height: 24,
})

const cls = computed(() => [
  'os-statusbar',
  `os-statusbar--${props.variant}`,
])
</script>

<template>
  <div :class="[...cls, 'gpu-layer']" :style="{ height: height + 'px' }" role="status" aria-live="polite">
    <div class="os-statusbar__left">
      <slot name="left" />
      <div
        v-for="item in items.filter(i => !i.onClick)"
        :key="item.id"
        class="os-statusbar__item"
        :style="{ color: item.color }"
      >
        <OsIcon v-if="item.icon" :icon="item.icon" :size="10" :class="{ 'os-statusbar__pulse': item.pulse }" />
        <span v-if="item.label" class="os-statusbar__label">{{ item.label }}</span>
        <span v-if="item.value" class="os-statusbar__value">{{ item.value }}</span>
      </div>
    </div>
    <div class="os-statusbar__right">
      <slot name="right" />
      <div
        v-for="item in items.filter(i => i.onClick)"
        :key="item.id"
        class="os-statusbar__item os-statusbar__item--clickable"
        :style="{ color: item.color }"
        role="button"
        :aria-label="item.label"
        tabindex="0"
        @click="item.onClick"
        @keydown.enter="item.onClick"
        @keydown.space.prevent="item.onClick"
      >
        <OsIcon v-if="item.icon" :icon="item.icon" :size="10" :class="{ 'os-statusbar__pulse': item.pulse }" />
        <span v-if="item.label" class="os-statusbar__label">{{ item.label }}</span>
        <span v-if="item.value" class="os-statusbar__value">{{ item.value }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.os-statusbar {
  display: flex;
  align-items: center;
  padding: 0 10px;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  user-select: none;
  flex-shrink: 0;
  gap: 12px;
}

.os-statusbar--default {
  background: var(--bg-elevated);
  border-top: 1px solid var(--border-subtle);
  color: var(--text-muted);
}

.os-statusbar--glass {
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border-top: 1px solid var(--border-glass);
  color: var(--text-muted);
  position: relative;
}

.os-statusbar--glass::before {
  content: '';
  position: absolute;
  top: -1px;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--border-subtle), var(--accent-dim), var(--border-subtle));
  background-size: 200% 100%;
  animation: shimmer 3s ease-in-out infinite;
}

.os-statusbar--neon {
  background: var(--bg-surface);
  border-top: 1px solid rgba(0, 255, 65, 0.1);
  color: var(--text-muted);
}

.os-statusbar--gothic {
  background: #150808;
  border-top: 1px solid #2a1010;
  color: #886060;
}

.os-statusbar__left,
.os-statusbar__right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.os-statusbar__left { flex: 1; }
.os-statusbar__right { flex-shrink: 0; }

.os-statusbar__item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.os-statusbar__item--clickable {
  cursor: pointer;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}
.os-statusbar__item--clickable:hover {
  background: var(--bg-overlay);
  color: var(--text-primary);
}

.os-statusbar__item--clickable:focus-visible {
  box-shadow: var(--focus-ring);
  outline: none;
}

.os-statusbar__label {
  text-transform: uppercase;
}

.os-statusbar__value {
  font-weight: 600;
}

.os-statusbar__pulse {
  animation: bw-pulse 1.5s ease-in-out infinite;
}
</style>
