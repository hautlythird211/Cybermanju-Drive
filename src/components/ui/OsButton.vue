<script setup lang="ts">
import { computed, useSlots, ref, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const slots = useSlots()
const btnRef = ref<HTMLElement | null>(null)
const ctx = ref<gsap.Context | null>(null)
const hoverTween = ref<gsap.core.Tween | null>(null)

const props = withDefaults(defineProps<{
  variant?: 'default' | 'primary' | 'danger' | 'ghost' | 'glass' | 'neon' | 'gothic' | 'cute'
  size?: 'xs' | 'sm' | 'md' | 'lg'
  icon?: string
  iconRight?: boolean
  loading?: boolean
  disabled?: boolean
  block?: boolean
  glow?: boolean
  vintage?: boolean
  pill?: boolean
  ariaLabel?: string
}>(), {
  variant: 'default',
  size: 'md',
  iconRight: false,
  loading: false,
  disabled: false,
  block: false,
  glow: false,
  vintage: false,
  pill: false,
})

const emit = defineEmits<{ click: [e: MouseEvent] }>()

const computedLabel = computed(() => props.ariaLabel || props.icon || undefined)

const cls = computed(() => [
  'os-btn',
  `os-btn--${props.variant}`,
  `os-btn--${props.size}`,
  {
    'os-btn--block': props.block,
    'os-btn--loading': props.loading,
    'os-btn--glow': props.glow,
    'os-btn--vintage': props.vintage,
    'os-btn--pill': props.pill,
    'os-btn--icon-only': props.icon && !slots.default,
    'os-btn--disabled': props.disabled,
  },
])

function onClick(e: MouseEvent) {
  if (!props.disabled && !props.loading) emit('click', e)
}

function onEnter() {
  if (props.disabled || props.loading) return
  hoverTween.value?.play()
}

function onLeave() {
  hoverTween.value?.reverse()
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault()
    onClick(e as unknown as MouseEvent)
  }
}

function onMousedown(e: MouseEvent) {
  const btn = btnRef.value
  if (!btn || props.disabled || props.loading) return
  const rect = btn.getBoundingClientRect()
  const ripple = document.createElement('span')
  const size = Math.max(rect.width, rect.height)
  const x = e.clientX - rect.left - size / 2
  const y = e.clientY - rect.top - size / 2
  ripple.style.cssText = `width:${size}px;height:${size}px;left:${x}px;top:${y}px`
  ripple.className = 'os-btn__ripple'
  btn.appendChild(ripple)
  ripple.addEventListener('animationend', () => ripple.remove())
}

onMounted(() => {
  const el = btnRef.value
  if (!el) return
  ctx.value = gsap.context(() => {
    hoverTween.value = gsap.to(el, {
      scale: 1.02, duration: 0.2, ease: 'cubic-bezier(0.22, 1, 0.36, 1)',
      paused: true, force3D: true,
    })
    anim.fadeIn(el, { from: { y: 8, opacity: 0 } })
  }, el)
})

onUnmounted(() => {
  ctx.value?.revert()
  hoverTween.value?.kill()
})
</script>

<template>
  <button
    ref="btnRef"
    :class="[...cls, 'gpu']"
    :disabled="disabled || loading"
    :role="variant === 'cute' ? 'button' : undefined"
    :aria-disabled="disabled || loading || undefined"
    :aria-label="computedLabel"
    :tabindex="disabled ? -1 : 0"
    @click="onClick"
    @mouseenter="onEnter"
    @mouseleave="onLeave"
    @keydown="onKeydown"
    @mousedown="onMousedown"
  >
    <Icon v-if="icon && !iconRight" :icon="icon" class="os-btn__icon" />
    <span v-if="$slots.default" class="os-btn__text">
      <slot />
    </span>
    <Icon v-if="icon && iconRight" :icon="icon" class="os-btn__icon os-btn__icon--right" />
    <span v-if="loading" class="os-btn__spinner" />
  </button>
</template>

<style scoped>
.os-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: var(--radius-md);
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif;
  font-weight: 600;
  letter-spacing: 0.2px;
  cursor: pointer;
  user-select: none;
  transition: all var(--duration-fast) var(--ease-spring);
  white-space: nowrap;
  position: relative;
  overflow: hidden;
  will-change: transform;
}

.os-btn:active {
  transform: translateY(1px);
}

.os-btn:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.os-btn--disabled,
.os-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
}

/* sizes */
.os-btn--xs { padding: 3px 9px; font-size: var(--font-size-xs); height: 22px; }
.os-btn--sm { padding: 4px 12px; font-size: var(--font-size-sm); height: 26px; }
.os-btn--md { padding: 6px 16px; font-size: var(--font-size-base); height: 30px; }
.os-btn--lg { padding: 8px 22px; font-size: var(--font-size-md); height: 36px; }

.os-btn--block { width: 100%; }
.os-btn--pill { border-radius: var(--radius-full); }

/* variant: default */
.os-btn--default {
  background: var(--bg-elevated);
  border: 1px solid var(--border-medium);
  color: var(--text-secondary);
}
.os-btn--default:hover {
  background: var(--bg-overlay);
  border-color: var(--border-strong);
  color: var(--text-primary);
}

/* variant: primary */
.os-btn--primary {
  background: var(--accent);
  border: 1px solid var(--accent);
  color: var(--text-inverse);
}
.os-btn--primary:hover {
  background: #00cc35;
  border-color: #00cc35;
}

/* variant: danger */
.os-btn--danger {
  background: var(--danger-dim);
  border: 1px solid rgba(255, 69, 58, 0.25);
  color: var(--text-danger);
}
.os-btn--danger:hover {
  background: var(--danger-dim);
  border-color: var(--text-danger);
}

/* variant: ghost */
.os-btn--ghost {
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-secondary);
}
.os-btn--ghost:hover {
  background: rgba(255, 255, 255, 0.04);
  color: var(--text-primary);
}

/* variant: glass */
.os-btn--glass {
  background: rgba(14, 14, 18, 0.6);
  backdrop-filter: blur(30px) saturate(1.6);
  -webkit-backdrop-filter: blur(30px) saturate(1.6);
  border: 1px solid rgba(255, 255, 255, 0.06);
  color: var(--text-secondary);
  box-shadow: var(--panel-inset);
}
.os-btn--glass:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

/* variant: neon */
.os-btn--neon {
  background: transparent;
  border: 1px solid var(--accent);
  color: var(--text-accent);
  text-shadow: 0 0 4px var(--accent-glow);
  box-shadow: 0 0 8px var(--accent-dim), inset 0 0 8px rgba(0, 255, 65, 0.04);
}
.os-btn--neon:hover {
  background: var(--accent-dim);
  box-shadow: var(--glow-accent), 0 0 16px var(--accent-glow), inset 0 0 12px rgba(0, 255, 65, 0.08);
}

/* variant: gothic */
.os-btn--gothic {
  background: #1a0a0a;
  border: 1px solid #3a1a1a;
  color: #ff6b9d;
}
.os-btn--gothic:hover {
  background: #2a0a0a;
  border-color: #5a2a2a;
  color: #ff8db3;
  box-shadow: 0 0 12px rgba(255, 107, 157, 0.12);
}

/* variant: cute */
.os-btn--cute {
  background: linear-gradient(135deg, rgba(255, 107, 157, 0.12), rgba(179, 136, 255, 0.12));
  border: 1px solid rgba(255, 107, 157, 0.2);
  color: var(--text-pink);
  border-radius: var(--radius-xl);
}
.os-btn--cute:hover {
  background: linear-gradient(135deg, rgba(255, 107, 157, 0.2), rgba(179, 136, 255, 0.2));
  border-color: rgba(255, 107, 157, 0.4);
}

/* glow effect */
.os-btn--glow:not(:disabled) {
  animation: neon-pulse 2s ease-in-out infinite;
}

/* vintage effect */
.os-btn--vintage::after {
  content: '';
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0, 0, 0, 0.04) 2px,
    rgba(0, 0, 0, 0.04) 4px
  );
  pointer-events: none;
}

.os-btn__icon { font-size: 15px; flex-shrink: 0; }
.os-btn__icon--right { order: 1; }
.os-btn__text { position: relative; z-index: 1; }

.os-btn--xs .os-btn__icon { font-size: 11px; }
.os-btn--sm .os-btn__icon { font-size: 13px; }
.os-btn--lg .os-btn__icon { font-size: 17px; }

.os-btn--icon-only { padding: 6px; }
.os-btn--xs.os-btn--icon-only { padding: 2px; }
.os-btn--lg.os-btn--icon-only { padding: 8px; }

.os-btn__spinner {
  width: 12px;
  height: 12px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: bw-spin 0.6s linear infinite;
}

.os-btn__ripple {
  position: absolute;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.3);
  pointer-events: none;
  animation: os-btn-ripple 0.5s ease-out forwards;
}

@keyframes os-btn-ripple {
  from { transform: scale(0); opacity: 1; }
  to { transform: scale(2.5); opacity: 0; }
}
</style>
