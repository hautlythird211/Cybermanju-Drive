<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

const props = withDefaults(defineProps<{
  position?: 'bottom' | 'top' | 'left' | 'right'
  width?: number
  offset?: number
  disabled?: boolean
  closeOnClick?: boolean
}>(), {
  position: 'bottom',
  offset: 8,
  disabled: false,
  closeOnClick: true,
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const open = ref(false)
const popoverRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const cls = computed(() => [
  'os-popover',
  `os-popover--${props.position}`,
  { 'os-popover--open': open.value },
])

const style = computed(() => ({
  width: props.width ? `${props.width}px` : undefined,
}))

function toggle() {
  if (!props.disabled) {
    open.value = !open.value
    if (open.value) {
      nextTick(() => {
        gsapCtx.value?.add(() => {
          if (popoverRef.value) anim.dropdownEnter(popoverRef.value)
        })
      })
    }
  }
}

function close() {
  if (popoverRef.value) {
    gsapCtx.value?.add(() => {
      anim.dropdownLeave(popoverRef.value!).then(() => {
        open.value = false
      })
    })
  } else {
    open.value = false
  }
}

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div class="os-popover-wrapper" v-click-outside="close">
    <div @click="toggle" :aria-expanded="open" aria-haspopup="dialog" role="button" tabindex="0" @keydown.enter="toggle" @keydown.space.prevent="toggle">
      <slot name="trigger" />
    </div>
    <div v-if="open" :class="cls" :style="style" ref="popoverRef" role="dialog" @click="closeOnClick && close()">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.os-popover-wrapper {
  position: relative;
  display: inline-flex;
}

.os-popover {
  position: absolute;
  z-index: 5000;
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-lg);
  padding: 8px;
  box-shadow: var(--shadow-dropdown);
  min-width: 120px;
  will-change: transform, opacity;
}

.os-popover--bottom {
  top: calc(100% + v-bind('props.offset + "px"'));
  left: 50%;
  transform: translateX(-50%);
}

.os-popover--top {
  bottom: calc(100% + v-bind('props.offset + "px"'));
  left: 50%;
  transform: translateX(-50%);
}

.os-popover--left {
  right: calc(100% + v-bind('props.offset + "px"'));
  top: 50%;
  transform: translateY(-50%);
}

.os-popover--right {
  left: calc(100% + v-bind('props.offset + "px"'));
  top: 50%;
  transform: translateY(-50%);
}
</style>
