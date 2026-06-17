<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import gsap from 'gsap'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const indicatorRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const props = withDefaults(defineProps<{
  variant?: 'spinner' | 'dots' | 'pulse' | 'matrix' | 'gothic' | 'cute'
  size?: 'sm' | 'md' | 'lg'
  label?: string
  text?: string
  fullPage?: boolean
  overlay?: boolean
  skeleton?: boolean
}>(), {
  variant: 'spinner',
  size: 'md',
  fullPage: false,
  overlay: false,
  skeleton: false,
})

const cls = computed(() => [
  'os-loading',
  `os-loading--${props.variant}`,
  `os-loading--${props.size}`,
  {
    'os-loading--fullpage': props.fullPage,
    'os-loading--overlay': props.overlay,
    'os-loading--has-label': !!(props.label || props.text),
    'os-loading--skeleton': props.skeleton,
  },
])

onMounted(() => {
  gsapCtx.value = gsap.context(() => {
    if (props.variant === 'dots' && indicatorRef.value) {
      const dots = indicatorRef.value.querySelectorAll('.os-loading__dot')
      dots.forEach((dot, i) => {
        gsap.delayedCall(i * 0.15, () => anim.pulse(dot))
      })
    }
  })
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div :class="cls" role="status" aria-label="Loading" aria-busy="true">
    <template v-if="skeleton">
      <div class="os-loading__skeleton">
        <slot />
      </div>
    </template>
    <template v-else>
      <div ref="indicatorRef" class="os-loading__indicator">
        <template v-if="variant === 'spinner'">
          <div class="os-loading__spinner" />
        </template>
        <template v-else-if="variant === 'dots'">
          <span class="os-loading__dot" />
          <span class="os-loading__dot" />
          <span class="os-loading__dot" />
        </template>
        <template v-else-if="variant === 'pulse'">
          <div class="os-loading__pulse" />
        </template>
        <template v-else-if="variant === 'matrix'">
          <div class="os-loading__matrix">
            <OsIcon icon="mdi:matrix" :size="size === 'sm' ? 18 : size === 'lg' ? 36 : 24" variant="neon" />
          </div>
        </template>
        <template v-else-if="variant === 'gothic'">
          <div class="os-loading__gothic">
            <OsIcon icon="mdi:heart" :size="size === 'sm' ? 16 : size === 'lg' ? 32 : 22" color="#ff6b9d" pulse />
          </div>
        </template>
        <template v-else-if="variant === 'cute'">
          <div class="os-loading__cute">
            <OsIcon icon="mdi:star-four-points" :size="size === 'sm' ? 16 : size === 'lg' ? 32 : 22" color="#ff6b9d" pulse />
          </div>
        </template>
      </div>
      <span v-if="label" class="os-loading__label">{{ label }}</span>
      <span v-if="text" class="os-loading__text">{{ text }}</span>
    </template>
  </div>
</template>

<style scoped>
.os-loading {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-muted);
}

.os-loading--fullpage {
  position: fixed;
  inset: 0;
  z-index: 9999;
}

.os-loading--overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  z-index: 100;
}

.os-loading--skeleton {
  display: block;
  width: 100%;
}

.os-loading__skeleton {
  width: 100%;
  animation: bw-pulse 1.5s ease-in-out infinite;
}

.os-loading__indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.os-loading__spinner {
  border-radius: 50%;
  background: conic-gradient(from 0deg, var(--border-subtle), var(--accent), var(--accent), var(--border-subtle));
  mask: radial-gradient(farthest-side, transparent calc(100% - 2px), #fff calc(100% - 2px));
  -webkit-mask: radial-gradient(farthest-side, transparent calc(100% - 2px), #fff calc(100% - 2px));
  animation: bw-spin 0.6s linear infinite;
}

.os-loading--sm .os-loading__spinner { width: 16px; height: 16px; }
.os-loading--md .os-loading__spinner { width: 24px; height: 24px; }
.os-loading--lg .os-loading__spinner { width: 36px; height: 36px; }

.os-loading__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
  animation: os-loading-dot 1.2s ease-in-out infinite;
}
.os-loading__dot:nth-child(1) { animation-delay: 0s; }
.os-loading__dot:nth-child(2) { animation-delay: 0.2s; }
.os-loading__dot:nth-child(3) { animation-delay: 0.4s; }

@keyframes os-loading-dot {
  0%, 80%, 100% { transform: scale(0.4); opacity: 0.4; }
  40% { transform: scale(1); opacity: 1; }
}

.os-loading__pulse {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--accent);
  animation: os-loading-pulse 1.5s ease-out infinite;
}

@keyframes os-loading-pulse {
  0% { transform: scale(0.1); opacity: 0.6; }
  50% { transform: scale(1); opacity: 0.2; }
  100% { transform: scale(0.1); opacity: 0.6; }
}

.os-loading--neon .os-loading__spinner { background: conic-gradient(from 0deg, var(--border-subtle), var(--accent), var(--accent), var(--border-subtle)); mask: radial-gradient(farthest-side, transparent calc(100% - 2px), #fff calc(100% - 2px)); -webkit-mask: radial-gradient(farthest-side, transparent calc(100% - 2px), #fff calc(100% - 2px)); }
.os-loading--gothic .os-loading__spinner { background: conic-gradient(from 0deg, var(--border-subtle), var(--pink), var(--pink), var(--border-subtle)); mask: radial-gradient(farthest-side, transparent calc(100% - 2px), #fff calc(100% - 2px)); -webkit-mask: radial-gradient(farthest-side, transparent calc(100% - 2px), #fff calc(100% - 2px)); }
.os-loading--cute .os-loading__spinner { background: conic-gradient(from 0deg, var(--border-subtle), var(--pink), var(--pink), var(--border-subtle)); mask: radial-gradient(farthest-side, transparent calc(100% - 2px), #fff calc(100% - 2px)); -webkit-mask: radial-gradient(farthest-side, transparent calc(100% - 2px), #fff calc(100% - 2px)); }

.os-loading__label {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  animation: bw-pulse 1.5s ease-in-out infinite;
}

.os-loading__text {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
