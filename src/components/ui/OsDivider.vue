<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  variant?: 'solid' | 'dashed' | 'dotted' | 'glow' | 'gothic'
  spacing?: 'sm' | 'md' | 'lg'
  label?: string
  gradient?: boolean
}>(), {
  variant: 'solid',
  spacing: 'md',
  gradient: false,
})

const cls = computed(() => [
  'os-divider',
  `os-divider--${props.variant}`,
  `os-divider--${props.spacing}`,
  {
    'os-divider--labeled': !!props.label,
    'os-divider--gradient': props.gradient,
  },
])
</script>

<template>
  <div :class="cls" role="separator">
    <span v-if="label" class="os-divider__label">{{ label }}</span>
  </div>
</template>

<style scoped>
.os-divider {
  display: flex;
  align-items: center;
  width: 100%;
  height: 1px;
  background: var(--border-subtle);
  border: none;
}

.os-divider--sm { margin: 4px 0; }
.os-divider--md { margin: 8px 0; }
.os-divider--lg { margin: 16px 0; }

.os-divider--dashed { background: repeating-linear-gradient(90deg, var(--border-subtle) 0, var(--border-subtle) 6px, transparent 6px, transparent 8px); }
.os-divider--dotted { background: repeating-linear-gradient(90deg, var(--border-subtle) 0, var(--border-subtle) 2px, transparent 2px, transparent 6px); }
.os-divider--glow { background: var(--accent); box-shadow: 0 0 6px var(--accent-glow); }
.os-divider--gothic { background: linear-gradient(90deg, transparent, #3a1a1a, transparent); }

.os-divider--gradient {
  position: relative;
  background: var(--gradient-accent);
  background-size: 200% 100%;
  animation: shimmer 3s ease-in-out infinite;
}
.os-divider--gradient:not(.os-divider--labeled)::before {
  content: '';
  position: absolute;
  inset: 0;
  filter: blur(6px);
  background: var(--gradient-accent);
  opacity: 0.4;
  z-index: -1;
  background-size: 200% 100%;
  animation: shimmer 3s ease-in-out infinite;
}

.os-divider--gradient::after {
  content: '';
  position: absolute;
  inset: -2px;
  filter: blur(4px);
  background: var(--gradient-accent);
  opacity: 0.15;
  z-index: -2;
  background-size: 200% 100%;
  animation: shimmer 4s ease-in-out infinite reverse;
}

.os-divider--labeled {
  background: none;
  gap: 12px;
}
.os-divider--labeled::before,
.os-divider--labeled::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--border-subtle);
}

.os-divider--labeled.os-divider--glow::before,
.os-divider--labeled.os-divider--glow::after {
  background: var(--accent);
  box-shadow: 0 0 4px var(--accent-glow);
}

.os-divider--labeled.os-divider--gradient::before,
.os-divider--labeled.os-divider--gradient::after {
  background: var(--gradient-accent);
}

.os-divider__label {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 1px;
  flex-shrink: 0;
}
</style>
