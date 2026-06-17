<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'

const props = withDefaults(defineProps<{
  icon: string
  size?: number | string
  variant?: 'default' | 'neon' | 'gothic' | 'cute' | 'glitch'
  color?: string
  spin?: boolean
  pulse?: boolean
  glow?: boolean
  ariaLabel?: string
}>(), {
  size: 14,
  variant: 'default',
  spin: false,
  pulse: false,
  glow: false,
  ariaLabel: '',
})

const cls = computed(() => [
  'os-icon',
  `os-icon--${props.variant}`,
  {
    'os-icon--spin': props.spin,
    'os-icon--pulse': props.pulse,
    'os-icon--glow': props.glow,
  },
])

const style = computed(() => {
  const s: Record<string, string | number> = {
    fontSize: typeof props.size === 'number' ? `${props.size}px` : props.size,
  }
  if (props.color) s.color = props.color
  return s
})
</script>

<template>
  <Icon
    :icon="icon"
    :class="[...cls, 'gpu']"
    :style="style"
    :aria-hidden="ariaLabel ? undefined : 'true'"
    :aria-label="ariaLabel || undefined"
    v-bind="$attrs"
  />
</template>

<style scoped>
.os-icon {
  flex-shrink: 0;
  transition: all var(--transition-fast);
}

.os-icon--neon {
  filter: drop-shadow(0 0 3px var(--accent-glow));
}

.os-icon--gothic {
  filter: drop-shadow(0 0 3px rgba(255, 107, 157, 0.3));
}

.os-icon--cute {
  filter: drop-shadow(0 0 2px rgba(255, 107, 157, 0.2));
}

.os-icon--glitch {
  animation: bw-glitch 0.3s ease infinite;
}

.os-icon--spin {
  animation: bw-spin var(--duration-slow) linear infinite;
}

.os-icon--pulse {
  animation: bw-pulse 1.5s ease-in-out infinite;
}

.os-icon--glow {
  filter: drop-shadow(0 0 6px currentColor);
}
</style>
