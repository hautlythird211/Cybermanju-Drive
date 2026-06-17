<script setup lang="ts">
import { computed, ref, onMounted, watch, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'
import OsIcon from './OsIcon.vue'

const anim = useGsapAnimation()
const bodyRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const props = withDefaults(defineProps<{
  title?: string
  icon?: string
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  collapsible?: boolean
  defaultOpen?: boolean
  spaced?: boolean
}>(), {
  variant: 'default',
  collapsible: false,
  defaultOpen: true,
  spaced: false,
})

const open = defineModel<boolean>('open', { default: true })

const bodyVisible = ref(!props.collapsible || props.defaultOpen)

const cls = computed(() => [
  'os-section',
  `os-section--${props.variant}`,
  {
    'os-section--collapsible': props.collapsible,
    'os-section--spaced': props.spaced,
    'os-section--collapsed': props.collapsible && !open.value,
  },
])

function toggle() {
  if (props.collapsible) open.value = !open.value
}

watch(open, async (val) => {
  const el = bodyRef.value
  if (!props.collapsible || !el) return
  if (val) {
    bodyVisible.value = true
    await nextTick()
    gsapCtx.value?.add(() => {
      anim.slideIn(el, 'top', 20)
    })
  } else {
    gsapCtx.value?.add(() => {
      anim.slideOut(el, 'top', 20)
    })
    bodyVisible.value = false
  }
})

onMounted(() => {
  gsapCtx.value = gsap.context(() => {
    if (props.collapsible && open.value && bodyRef.value) {
      anim.slideIn(bodyRef.value, 'top', 20)
    }
  })
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <section :class="cls">
    <div
      v-if="title || icon || $slots.header"
      class="os-section__header"
      @click="toggle"
      :aria-expanded="collapsible ? open : undefined"
      :role="collapsible ? 'button' : undefined"
      :tabindex="collapsible ? 0 : undefined"
    >
      <OsIcon v-if="icon" :icon="icon" :size="12" />
      <span v-if="title" class="os-section__title">{{ title }}</span>
      <slot name="header" />
      <span v-if="collapsible" class="os-section__chevron">{{ open ? '▾' : '▸' }}</span>
    </div>
    <div v-if="!collapsible || bodyVisible" ref="bodyRef" class="os-section__body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.os-section {
  display: flex;
  flex-direction: column;
}

.os-section--spaced {
  margin-bottom: 12px;
}

.os-section__header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 0;
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-primary);
  user-select: none;
}

.os-section--collapsible .os-section__header {
  cursor: pointer;
}

.os-section--collapsible .os-section__header:hover {
  color: var(--text-accent);
}

.os-section__title {
  flex: 1;
}

.os-section__chevron {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  transition: transform var(--transition-fast);
}

.os-section--collapsed .os-section__chevron {
  transform: rotate(-90deg);
}

.os-section__body {
  padding-left: 0;
}

.os-section--neon .os-section__header { color: var(--text-accent); text-shadow: 0 0 4px var(--accent-glow); }
.os-section--gothic .os-section__header { color: var(--text-pink); }
.os-section--cute .os-section__header { color: var(--text-pink); }
.os-section--glass .os-section__header { color: var(--text-primary); }
</style>
