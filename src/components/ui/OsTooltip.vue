<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

const props = withDefaults(defineProps<{
  text: string
  position?: 'top' | 'bottom' | 'left' | 'right'
  delay?: number
  disabled?: boolean
}>(), {
  position: 'top',
  delay: 300,
  disabled: false,
})

const uid = `os-tooltip-${Math.random().toString(36).slice(2, 9)}`
const show = ref(false)
const tooltipRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)
let timer: ReturnType<typeof setTimeout> | null = null

function onEnter() {
  if (props.disabled) return
  timer = setTimeout(() => {
    show.value = true
    nextTick(() => {
      gsapCtx.value?.add(() => {
        if (tooltipRef.value) anim.fadeIn(tooltipRef.value, { from: { y: 4 } })
      })
    })
  }, props.delay)
}

function onLeave() {
  if (timer) clearTimeout(timer)
  if (show.value && tooltipRef.value) {
    gsapCtx.value?.add(() => {
      anim.fadeOut(tooltipRef.value!).then(() => {
        show.value = false
      })
    })
  } else {
    show.value = false
  }
}

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div
    class="os-tooltip-wrapper"
    :aria-describedby="show ? uid : undefined"
    @mouseenter="onEnter"
    @mouseleave="onLeave"
    @focus="onEnter"
    @blur="onLeave"
  >
    <slot />
    <div
      v-if="show"
      :id="uid"
      ref="tooltipRef"
      :class="['os-tooltip', `os-tooltip--${position}`]"
      role="tooltip"
    >
      {{ text }}
    </div>
  </div>
</template>

<style scoped>
.os-tooltip-wrapper {
  position: relative;
  display: inline-flex;
}

.os-tooltip {
  position: absolute;
  z-index: 10000;
  padding: 4px 8px;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-primary);
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-sm);
  white-space: nowrap;
  pointer-events: none;
  box-shadow: var(--shadow-card);
  will-change: transform, opacity;
}

.os-tooltip--top {
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 4px;
}

.os-tooltip--bottom {
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 4px;
}

.os-tooltip--left {
  right: 100%;
  top: 50%;
  transform: translateY(-50%);
  margin-right: 4px;
}

.os-tooltip--right {
  left: 100%;
  top: 50%;
  transform: translateY(-50%);
  margin-left: 4px;
}
</style>
