<script setup lang="ts">
import { computed, ref, watch, onMounted, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const barRef = ref<HTMLElement | null>(null)
const progressRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const props = withDefaults(defineProps<{
  value?: number
  max?: number
  variant?: 'default' | 'accent' | 'danger' | 'success' | 'warning' | 'info' | 'pink' | 'purple' | 'gold'
  size?: 'sm' | 'md' | 'lg'
  indeterminate?: boolean
  showLabel?: boolean
  label?: string
}>(), {
  value: 0,
  max: 100,
  variant: 'accent',
  size: 'md',
  indeterminate: false,
  showLabel: false,
})

const percent = computed(() => Math.min(100, Math.max(0, (props.value / props.max) * 100)))

const cls = computed(() => [
  'os-progress',
  `os-progress--${props.variant}`,
  `os-progress--${props.size}`,
  { 'os-progress--indeterminate': props.indeterminate },
])

watch(() => props.value, async (newVal) => {
  if (props.indeterminate || !barRef.value) return
  const pct = Math.min(100, Math.max(0, (newVal / props.max) * 100))
  await nextTick()
  gsapCtx.value?.add(() => {
    anim.animateProgress(barRef.value!, 0, pct, { duration: 0.5 })
  })
})

onMounted(async () => {
  gsapCtx.value = gsap.context(() => {
    if (props.indeterminate || !barRef.value) return
    const pct = percent.value
    anim.animateProgress(barRef.value, 0, pct, { duration: 0.6 })
    if (progressRef.value) anim.fadeIn(progressRef.value, { from: { opacity: 0 } })
  })
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div
    ref="progressRef"
    :class="cls"
    role="progressbar"
    :aria-valuenow="indeterminate ? undefined : value"
    :aria-valuemin="0"
    :aria-valuemax="max"
    :aria-label="label || undefined"
  >
    <div class="os-progress__track">
      <div
        ref="barRef"
        class="os-progress__bar"
        :style="{ width: indeterminate ? '40%' : percent + '%' }"
      />
    </div>
    <span v-if="showLabel || label" class="os-progress__label">
      {{ label || `${Math.round(percent)}%` }}
    </span>
  </div>
</template>

<style scoped>
.os-progress {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.os-progress__track {
  flex: 1;
  background: var(--bg-overlay);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.os-progress--sm .os-progress__track { height: 4px; }
.os-progress--md .os-progress__track { height: 6px; }
.os-progress--lg .os-progress__track { height: 10px; }

.os-progress__bar {
  height: 100%;
  border-radius: var(--radius-full);
  will-change: width;
}

.os-progress--accent .os-progress__bar { background: var(--accent); box-shadow: 0 0 6px var(--accent-glow); }
.os-progress--default .os-progress__bar { background: var(--text-secondary); }
.os-progress--danger .os-progress__bar { background: var(--danger); }
.os-progress--success .os-progress__bar { background: var(--success); }
.os-progress--warning .os-progress__bar { background: var(--warning); }
.os-progress--info .os-progress__bar { background: var(--info); }
.os-progress--pink .os-progress__bar { background: var(--pink); }
.os-progress--purple .os-progress__bar { background: var(--purple); }
.os-progress--gold .os-progress__bar { background: var(--gold); }

.os-progress--indeterminate .os-progress__bar {
  animation: os-progress-indeterminate 1.5s ease-in-out infinite;
  width: 40% !important;
}

@keyframes os-progress-indeterminate {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(350%); }
}

.os-progress__label {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  white-space: nowrap;
  flex-shrink: 0;
}
</style>
