<script setup lang="ts">
import { computed, ref, watch, onMounted, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const fillRef = ref<HTMLElement | null>(null)
const sliderRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const props = withDefaults(defineProps<{
  modelValue: number
  min?: number
  max?: number
  step?: number
  variant?: 'default' | 'neon' | 'gothic' | 'cute'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  showValue?: boolean
}>(), {
  min: 0,
  max: 100,
  step: 1,
  variant: 'default',
  size: 'md',
  disabled: false,
  showValue: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const percent = computed(() => ((props.modelValue - props.min) / (props.max - props.min)) * 100)

const cls = computed(() => [
  'os-slider',
  `os-slider--${props.variant}`,
  `os-slider--${props.size}`,
  { 'os-slider--disabled': props.disabled },
])

function onInput(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value)
  if (!isNaN(val)) emit('update:modelValue', val)
}

function onKeydown(e: KeyboardEvent) {
  if (props.disabled) return
  let val = props.modelValue
  if (e.key === 'ArrowRight' || e.key === 'ArrowUp') {
    e.preventDefault()
    val = Math.min(props.max, val + props.step)
  } else if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') {
    e.preventDefault()
    val = Math.max(props.min, val - props.step)
  } else {
    return
  }
  emit('update:modelValue', val)
}

watch(() => props.modelValue, async (newVal) => {
  if (fillRef.value) {
    const pct = ((newVal - props.min) / (props.max - props.min)) * 100
    await nextTick()
    gsapCtx.value?.add(() => {
      anim.animateProgress(fillRef.value!, 0, pct, { duration: 0.3 })
    })
  }
})

onMounted(() => {
  gsapCtx.value = gsap.context(() => {
    if (sliderRef.value) anim.fadeIn(sliderRef.value, { from: { y: 6, opacity: 0 } })
  })
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div
    ref="sliderRef"
    :class="cls"
    role="slider"
    :aria-valuenow="modelValue"
    :aria-valuemin="min"
    :aria-valuemax="max"
    :aria-disabled="disabled || undefined"
    :tabindex="disabled ? -1 : 0"
    @keydown="onKeydown"
  >
    <input
      type="range"
      :min="min"
      :max="max"
      :step="step"
      :value="modelValue"
      :disabled="disabled"
      class="os-slider__input"
      @input="onInput"
    />
    <div class="os-slider__track gpu">
      <div ref="fillRef" class="os-slider__fill" :style="{ width: percent + '%' }" />
    </div>
    <div class="os-slider__thumb" :style="{ left: percent + '%' }" />
    <span v-if="showValue" class="os-slider__value">{{ modelValue }}</span>
  </div>
</template>

<style scoped>
.os-slider {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 20px;
}

.os-slider__input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
  z-index: 2;
}

.os-slider--disabled { opacity: 0.4; }
.os-slider--disabled .os-slider__input { cursor: not-allowed; }

.os-slider__track {
  flex: 1;
  background: var(--bg-overlay);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.os-slider--sm .os-slider__track { height: 4px; }
.os-slider--md .os-slider__track { height: 6px; }
.os-slider--lg .os-slider__track { height: 8px; }

.os-slider__fill {
  height: 100%;
  border-radius: var(--radius-full);
  will-change: width;
  transition: width var(--duration-normal) var(--ease-spring);
}

.os-slider--default .os-slider__fill { background: var(--accent); }
.os-slider--neon .os-slider__fill { background: var(--accent); box-shadow: 0 0 6px var(--accent-glow); }
.os-slider--gothic .os-slider__fill { background: var(--pink); }
.os-slider--cute .os-slider__fill { background: var(--pink); }

.os-slider__thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  border-radius: 50%;
  background: #fff;
  pointer-events: none;
  z-index: 1;
  will-change: transform;
  transition: box-shadow var(--duration-fast) var(--ease-spring);
}

.os-slider:active .os-slider__thumb,
.os-slider:focus-within .os-slider__thumb {
  box-shadow: 0 0 8px var(--accent-glow);
}

.os-slider--sm .os-slider__thumb { width: 10px; height: 10px; margin-left: 0; }
.os-slider--md .os-slider__thumb { width: 14px; height: 14px; }
.os-slider--lg .os-slider__thumb { width: 18px; height: 18px; }

.os-slider--neon .os-slider__thumb { background: var(--accent); box-shadow: 0 0 4px var(--accent-glow); }
.os-slider--gothic .os-slider__thumb { background: var(--pink); }
.os-slider--cute .os-slider__thumb { background: var(--pink); }

.os-slider__value {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  min-width: 28px;
  text-align: right;
  flex-shrink: 0;
}
</style>
