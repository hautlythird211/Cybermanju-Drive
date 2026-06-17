<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  variant?: 'default' | 'accent' | 'danger' | 'success' | 'warning' | 'info' | 'pink' | 'purple' | 'gold'
  size?: 'xs' | 'sm' | 'md'
  dot?: boolean
  pulse?: boolean
  glow?: boolean
  ariaLabel?: string
}>(), {
  variant: 'default',
  size: 'xs',
  dot: false,
  pulse: false,
  glow: false,
  ariaLabel: '',
})

const cls = computed(() => [
  'os-badge',
  `os-badge--${props.variant}`,
  `os-badge--${props.size}`,
  {
    'os-badge--dot': props.dot,
    'os-badge--pulse': props.pulse,
    'os-badge--glow': props.glow,
  },
])
</script>

<template>
  <span :class="[...cls, 'gpu']" :aria-label="ariaLabel || undefined">
    <span v-if="dot" class="os-badge__dot" />
    <slot />
  </span>
</template>

<style scoped>
.os-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: var(--font-mono);
  font-weight: 700;
  text-transform: uppercase;
  border-radius: var(--radius-sm);
  white-space: nowrap;
  line-height: 1;
  will-change: opacity;
}

.os-badge--xs { padding: 1px 6px; font-size: var(--font-size-xs); }
.os-badge--sm { padding: 2px 8px; font-size: var(--font-size-xs); }
.os-badge--md { padding: 3px 10px; font-size: var(--font-size-sm); }

.os-badge--default {
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  color: var(--text-secondary);
}

.os-badge--accent {
  background: var(--gradient-accent);
  background-size: 200% 100%;
  animation: shimmer 3s ease-in-out infinite;
  border: 1px solid rgba(0, 255, 65, 0.2);
  color: var(--text-accent);
}

.os-badge--danger {
  background: var(--danger-dim);
  border: 1px solid rgba(255, 95, 87, 0.2);
  color: var(--text-danger);
}
.os-badge--danger.os-badge--pulse {
  box-shadow: 0 0 8px var(--danger);
  animation: badge-pulse 1.5s ease-in-out infinite;
}

.os-badge--success {
  background: var(--success-dim);
  border: 1px solid rgba(40, 200, 64, 0.2);
  color: var(--text-success);
}

.os-badge--warning {
  background: var(--warning-dim);
  border: 1px solid rgba(254, 188, 46, 0.2);
  color: var(--text-warning);
}

.os-badge--info {
  background: var(--info-dim);
  border: 1px solid rgba(90, 240, 255, 0.2);
  color: var(--text-info);
}

.os-badge--pink {
  background: var(--pink-dim);
  border: 1px solid rgba(255, 107, 157, 0.2);
  color: var(--text-pink);
}

.os-badge--purple {
  background: var(--purple-dim);
  border: 1px solid rgba(179, 136, 255, 0.2);
  color: var(--text-purple);
}

.os-badge--gold {
  background: var(--gold-dim);
  border: 1px solid rgba(255, 215, 0, 0.2);
  color: var(--text-gold);
}

.os-badge__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  flex-shrink: 0;
}

.os-badge--pulse .os-badge__dot {
  animation: bw-pulse 1.5s ease-in-out infinite;
}

.os-badge--glow {
  box-shadow: 0 0 8px currentColor;
}

.os-badge:hover {
  filter: brightness(1.2);
  transition: filter var(--duration-fast) var(--ease-spring);
}
</style>
