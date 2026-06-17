<script setup lang="ts">
import { computed, ref, onMounted, watch, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const trackRef = ref<HTMLElement | null>(null)
const thumbRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const props = withDefaults(defineProps<{
  modelValue: boolean
  variant?: 'default' | 'neon' | 'gothic' | 'cute'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  label?: string
}>(), {
  variant: 'default',
  size: 'md',
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const cls = computed(() => [
  'os-toggle',
  `os-toggle--${props.variant}`,
  `os-toggle--${props.size}`,
  {
    'os-toggle--on': props.modelValue,
    'os-toggle--off': !props.modelValue,
    'os-toggle--disabled': props.disabled,
  },
])

function toggle() {
  if (!props.disabled) emit('update:modelValue', !props.modelValue)
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault()
    toggle()
  }
}

onMounted(() => {
  gsapCtx.value = gsap.context(() => {
    if (trackRef.value) anim.fadeIn(trackRef.value, { from: { scale: 0.9, opacity: 0 } })
  })
})

watch(() => props.modelValue, (val) => {
  const el = thumbRef.value
  if (el) {
    gsapCtx.value?.add(() => {
      if (val) {
        anim.toggleOn(el)
      } else {
        anim.toggleOff(el)
      }
    })
  }
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <label
    :class="cls"
    role="switch"
    :aria-checked="modelValue"
    :aria-label="label"
    :tabindex="disabled ? -1 : 0"
    @click="toggle"
    @keydown="onKeydown"
  >
    <div ref="trackRef" class="os-toggle__track gpu-layer">
      <div ref="thumbRef" class="os-toggle__thumb" />
    </div>
    <span v-if="label" class="os-toggle__label">{{ label }}</span>
  </label>
</template>

<style scoped>
.os-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
}

.os-toggle--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.os-toggle__track {
  position: relative;
  border-radius: var(--radius-full);
  transition: all var(--duration-normal) var(--ease-spring);
  flex-shrink: 0;
  will-change: transform, opacity;
}

.os-toggle--sm .os-toggle__track { width: 28px; height: 16px; }
.os-toggle--md .os-toggle__track { width: 36px; height: 20px; }
.os-toggle--lg .os-toggle__track { width: 44px; height: 24px; }

/* default variant */
.os-toggle--default .os-toggle__track { background: var(--bg-overlay); border: 1px solid var(--border-medium); }
.os-toggle--default.os-toggle--on .os-toggle__track { background: var(--accent); border-color: var(--accent); }

/* neon variant */
.os-toggle--neon .os-toggle__track { background: var(--bg-surface); border: 1px solid rgba(0, 255, 65, 0.2); }
.os-toggle--neon.os-toggle--on .os-toggle__track { background: var(--accent-dim); border-color: var(--accent); box-shadow: 0 0 8px var(--accent-glow); }

/* gothic variant */
.os-toggle--gothic .os-toggle__track { background: #1a0808; border: 1px solid #3a1a1a; }
.os-toggle--gothic.os-toggle--on .os-toggle__track { background: rgba(255, 107, 157, 0.2); border-color: var(--pink); }

/* cute variant */
.os-toggle--cute .os-toggle__track { background: rgba(255, 107, 157, 0.1); border: 1px solid rgba(255, 107, 157, 0.2); }
.os-toggle--cute.os-toggle--on .os-toggle__track { background: var(--pink-dim); border-color: var(--pink); }

.os-toggle__thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  background: var(--text-muted);
  border-radius: 50%;
  transition: all var(--duration-normal) var(--ease-spring);
  will-change: transform;
}

.os-toggle--on .os-toggle__thumb {
  box-shadow: 0 0 6px var(--accent-glow);
}

.os-toggle--sm .os-toggle__thumb { width: 10px; height: 10px; }
.os-toggle--md .os-toggle__thumb { width: 14px; height: 14px; }
.os-toggle--lg .os-toggle__thumb { width: 18px; height: 18px; }

.os-toggle--on .os-toggle__thumb {
  left: auto;
  background: #fff;
}

.os-toggle--sm.os-toggle--on .os-toggle__thumb { left: calc(100% - 12px); }
.os-toggle--md.os-toggle--on .os-toggle__thumb { left: calc(100% - 16px); }
.os-toggle--lg.os-toggle--on .os-toggle__thumb { left: calc(100% - 20px); }

.os-toggle--neon.os-toggle--on .os-toggle__thumb { background: var(--accent); box-shadow: 0 0 4px var(--accent-glow); }
.os-toggle--gothic.os-toggle--on .os-toggle__thumb { background: var(--pink); }
.os-toggle--cute.os-toggle--on .os-toggle__thumb { background: var(--pink); }

.os-toggle__label {
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
  color: var(--text-secondary);
}
</style>
