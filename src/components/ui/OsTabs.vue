<script setup lang="ts">
import { computed, ref, watch, onMounted, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const tablistRef = ref<HTMLElement | null>(null)
const indicatorRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

export interface Tab {
  id: string
  label: string
  icon?: string
  badge?: string | number
  disabled?: boolean
}

const props = withDefaults(defineProps<{
  tabs: Tab[]
  modelValue: string
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute' | 'underline'
  size?: 'sm' | 'md' | 'lg'
}>(), {
  variant: 'default',
  size: 'md',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const cls = computed(() => [
  'os-tabs',
  `os-tabs--${props.variant}`,
  `os-tabs--${props.size}`,
])

function select(id: string) {
  const tab = props.tabs.find(t => t.id === id)
  if (tab && !tab.disabled) emit('update:modelValue', id)
}

function moveIndicator() {
  if (!indicatorRef.value || !tablistRef.value) return
  const activeTab = tablistRef.value.querySelector('.os-tabs__tab--active') as HTMLElement | null
  if (!activeTab) return
  const offsetLeft = activeTab.offsetLeft
  const width = activeTab.offsetWidth
  indicatorRef.value.style.transform = `translateX(${offsetLeft}px)`
  indicatorRef.value.style.width = `${width}px`
}

function onKeydown(e: KeyboardEvent) {
  let idx = props.tabs.findIndex(t => t.id === props.modelValue)
  if (idx === -1) idx = 0
  if (e.key === 'ArrowRight') {
    e.preventDefault()
    let next = idx
    do {
      next = (next + 1) % props.tabs.length
    } while (props.tabs[next].disabled && next !== idx)
    if (!props.tabs[next].disabled) select(props.tabs[next].id)
  } else if (e.key === 'ArrowLeft') {
    e.preventDefault()
    let prev = idx
    do {
      prev = (prev - 1 + props.tabs.length) % props.tabs.length
    } while (props.tabs[prev].disabled && prev !== idx)
    if (!props.tabs[prev].disabled) select(props.tabs[prev].id)
  }
}

watch(() => props.modelValue, async () => {
  await nextTick()
  moveIndicator()
})

onMounted(async () => {
  gsapCtx.value = gsap.context(() => {
    moveIndicator()
    if (tablistRef.value) anim.fadeIn(tablistRef.value, { from: { y: 6, opacity: 0 } })
  })
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div ref="tablistRef" :class="[...cls, 'gpu']" role="tablist" @keydown="onKeydown">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      :id="`os-tab-${tab.id}`"
      class="os-tabs__tab"
      :class="{
        'os-tabs__tab--active': modelValue === tab.id,
        'os-tabs__tab--disabled': tab.disabled,
      }"
      role="tab"
      :aria-selected="modelValue === tab.id"
      :aria-controls="`os-tabpanel-${tab.id}`"
      :tabindex="modelValue === tab.id ? 0 : -1"
      @click="select(tab.id)"
    >
      {{ tab.label }}
      <span v-if="tab.badge" class="os-tabs__badge">{{ tab.badge }}</span>
    </button>
    <div
      v-if="variant === 'default' || variant === 'underline'"
      ref="indicatorRef"
      class="os-tabs__indicator"
    />
    <div
      v-for="tab in tabs"
      :key="`panel-${tab.id}`"
      :id="`os-tabpanel-${tab.id}`"
      role="tabpanel"
      :aria-labelledby="`os-tab-${tab.id}`"
      :hidden="modelValue !== tab.id"
      class="os-tabs__content"
    >
      <slot :name="tab.id" />
    </div>
    <div class="os-tabs__default-content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.os-tabs {
  display: flex;
  flex-wrap: wrap;
  font-family: var(--font-mono);
  position: relative;
}

.os-tabs--sm .os-tabs__tab { padding: 4px 10px; font-size: var(--font-size-xs); }
.os-tabs--md .os-tabs__tab { padding: 6px 14px; font-size: var(--font-size-base); }
.os-tabs--lg .os-tabs__tab { padding: 8px 18px; font-size: var(--font-size-md); }

.os-tabs__tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-family: inherit;
  font-size: inherit;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  transition: all var(--transition-fast);
  white-space: nowrap;
  position: relative;
}

.os-tabs__tab:hover {
  color: var(--text-secondary);
  transform: translateY(-1px);
  transition: transform var(--duration-fast) var(--ease-spring), color var(--duration-fast) var(--ease-spring);
}

.os-tabs__tab--active {
  color: var(--text-primary);
}

.os-tabs__tab--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.os-tabs__badge {
  font-size: var(--font-size-xs);
  background: var(--accent-dim);
  color: var(--text-accent);
  padding: 0 5px;
  border-radius: var(--radius-full);
  line-height: 14px;
}

/* indicator for default/underline variants */
.os-tabs__indicator {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 2px;
  background: var(--accent);
  border-radius: var(--radius-full);
  transition: width var(--duration-normal) var(--ease-spring), transform var(--duration-normal) var(--ease-spring);
  will-change: transform, width;
  pointer-events: none;
  box-shadow: 0 0 6px var(--accent-glow);
}

/* variant: default */
.os-tabs--default { border-bottom: 1px solid var(--border-subtle); gap: 0; }
.os-tabs--default .os-tabs__tab--active { border-bottom: 2px solid var(--accent); margin-bottom: -1px; }

/* variant: underline */
.os-tabs--underline { gap: 4px; }
.os-tabs--underline .os-tabs__tab { border-radius: var(--radius-sm); }
.os-tabs--underline .os-tabs__tab--active { background: var(--accent-dim); color: var(--text-accent); }

/* variant: glass */
.os-tabs--glass { gap: 4px; }
.os-tabs--glass .os-tabs__tab { border-radius: var(--radius-md); }
.os-tabs--glass .os-tabs__tab--active { background: var(--bg-glass); backdrop-filter: blur(var(--glass-blur-light)); border: 1px solid var(--border-glass); }

/* variant: neon */
.os-tabs--neon { gap: 4px; }
.os-tabs--neon .os-tabs__tab { border: 1px solid transparent; border-radius: var(--radius-sm); }
.os-tabs--neon .os-tabs__tab--active { border-color: rgba(0, 255, 65, 0.3); color: var(--text-accent); text-shadow: 0 0 4px var(--accent-glow); }

/* variant: gothic */
.os-tabs--gothic { gap: 2px; }
.os-tabs--gothic .os-tabs__tab { border-radius: var(--radius-sm); }
.os-tabs--gothic .os-tabs__tab--active { background: rgba(255, 107, 157, 0.1); color: var(--text-pink); }

/* variant: cute */
.os-tabs--cute { gap: 4px; }
.os-tabs--cute .os-tabs__tab { border-radius: var(--radius-xl); }
.os-tabs--cute .os-tabs__tab--active { background: var(--pink-dim); color: var(--text-pink); }

.os-tabs__content {
  width: 100%;
}

.os-tabs__default-content {
  width: 100%;
}
</style>
