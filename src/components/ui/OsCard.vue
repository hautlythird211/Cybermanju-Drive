<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const cardRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)
const hoverTween = ref<gsap.core.Tween | null>(null)

const props = withDefaults(defineProps<{
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute' | 'liquid'
  padding?: 'none' | 'sm' | 'md' | 'lg'
  hover?: boolean
  glow?: boolean
  vintage?: boolean
  as?: string
  title?: string
  hoverable?: boolean
}>(), {
  variant: 'default',
  padding: 'md',
  hover: false,
  glow: false,
  vintage: false,
  as: 'div',
  title: '',
  hoverable: false,
})

const cls = computed(() => [
  'os-card',
  `os-card--${props.variant}`,
  `os-card--pad-${props.padding}`,
  {
    'os-card--hover': props.hover,
    'os-card--glow': props.glow,
    'os-card--vintage': props.vintage,
    'os-card--hoverable': props.hoverable,
  },
])

function onMouseEnter() {
  if (!props.hoverable || !cardRef.value) return
  hoverTween.value?.play()
}

function onMouseLeave() {
  if (!props.hoverable || !cardRef.value) return
  hoverTween.value?.reverse()
}

onMounted(() => {
  const el = cardRef.value
  if (!el) return
  const dataIndex = el.dataset.index
  const delay = dataIndex ? Number(dataIndex) * 0.1 : 0
  gsapCtx.value = gsap.context(() => {
    hoverTween.value = gsap.to(el, {
      scale: 1.02,
      boxShadow: 'var(--shadow-dropdown)',
      duration: 0.2,
      ease: 'cubic-bezier(0.22, 1, 0.36, 1)',
      paused: true,
      force3D: true,
      overwrite: 'auto',
    })
    anim.fadeIn(el, { from: { y: 8 }, delay })
  }, el)
})

onUnmounted(() => {
  gsapCtx.value?.revert()
  hoverTween.value?.kill()
})
</script>

<template>
  <component
    :is="as"
    :class="cls"
    ref="cardRef"
    :aria-label="title || undefined"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <div v-if="$slots.header" class="os-card__header">
      <slot name="header" />
    </div>
    <div v-if="$slots.default" class="os-card__body">
      <slot />
    </div>
    <div v-if="$slots.footer" class="os-card__footer">
      <slot name="footer" />
    </div>
  </component>
</template>

<style scoped>
.os-card {
  border-radius: var(--radius-lg);
  transition: all var(--transition-normal);
  will-change: transform, box-shadow;
}

/* padding */
.os-card--pad-none { padding: 0; }
.os-card--pad-sm { padding: 8px; }
.os-card--pad-md { padding: 12px; }
.os-card--pad-lg { padding: 16px; }

/* variant: default */
.os-card--default {
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  box-shadow: var(--shadow-card);
}

/* variant: glass */
.os-card--glass {
  position: relative;
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-glass);
  box-shadow: var(--shadow-glass);
}
.os-card--glass::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: linear-gradient(135deg, rgba(255,255,255,0.06), transparent 50%);
  pointer-events: none;
  z-index: 0;
}

/* variant: neon */
.os-card--neon {
  background: var(--bg-surface);
  border: 1px solid rgba(0, 255, 65, 0.2);
  color: var(--text-primary);
  box-shadow: 0 0 12px var(--accent-dim);
}
.os-card--neon:hover {
  border-color: rgba(0, 255, 65, 0.4);
}

/* variant: gothic */
.os-card--gothic {
  background: linear-gradient(180deg, #1a0a0a 0%, #0f0505 100%);
  border: 1px solid #3a1a1a;
  color: #d4a0b0;
  box-shadow: 0 4px 16px rgba(60, 10, 20, 0.3);
}

/* variant: cute */
.os-card--cute {
  background: linear-gradient(135deg, rgba(255, 107, 157, 0.08), rgba(179, 136, 255, 0.08));
  border: 1px solid rgba(255, 107, 157, 0.15);
  color: var(--text-secondary);
  border-radius: var(--radius-xl);
}

/* variant: liquid */
.os-card--liquid {
  position: relative;
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-glass);
  box-shadow: var(--shadow-glass);
  overflow: hidden;
}
.os-card--liquid::before {
  content: '';
  position: absolute;
  inset: -50%;
  z-index: 0;
  pointer-events: none;
  background: linear-gradient(
    135deg,
    var(--liquid-start) 0%,
    var(--liquid-mid) 25%,
    transparent 50%,
    var(--liquid-mid) 75%,
    var(--liquid-end) 100%
  );
  background-size: 400% 400%;
  animation: liquid-shift 8s ease-in-out infinite;
  mix-blend-mode: screen;
}

.os-card--hover:hover {
  border-color: var(--border-medium);
  box-shadow: var(--shadow-dropdown);
}

.os-card--glow {
  box-shadow: 0 0 20px var(--accent-glow);
}

.os-card--vintage::after {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0, 0, 0, 0.04) 2px,
    rgba(0, 0, 0, 0.04) 4px
  );
  border-radius: inherit;
}

.os-card__header,
.os-card__footer {
  display: flex;
  align-items: center;
  gap: 8px;
}

.os-card__header {
  margin-bottom: 8px;
  font-weight: 700;
  font-size: var(--font-size-base);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-primary);
}

.os-card__footer {
  margin-top: 8px;
}

.os-card__body {
  position: relative;
  z-index: 1;
}
</style>
