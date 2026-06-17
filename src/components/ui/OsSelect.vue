<script setup lang="ts">
import { ref, computed, shallowRef, onMounted, watch, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const dropdownRef = ref<HTMLElement | null>(null)
const containerRef = ref<HTMLElement | null>(null)
const activeIndex = ref(-1)
const gsapCtx = ref<gsap.Context | null>(null)

export interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

const props = withDefaults(defineProps<{
  modelValue: string
  options: SelectOption[]
  placeholder?: string
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  clearable?: boolean
}>(), {
  placeholder: 'Select...',
  variant: 'default',
  size: 'md',
  disabled: false,
  clearable: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const open = ref(false)
const optionsRef = shallowRef(props.options)

const selected = computed(() => props.options.find(o => o.value === props.modelValue))

const cls = computed(() => [
  'os-select',
  `os-select--${props.variant}`,
  `os-select--${props.size}`,
  {
    'os-select--open': open.value,
    'os-select--disabled': props.disabled,
    'os-select--has-value': !!props.modelValue,
  },
])

watch(() => props.options, (val) => {
  optionsRef.value = val
}, { deep: true })

watch(open, async (val) => {
  if (val) {
    await nextTick()
    if (dropdownRef.value) {
      gsapCtx.value?.add(() => {
        anim.dropdownEnter(dropdownRef.value!)
      })
    }
    activeIndex.value = Math.max(0, props.options.findIndex(o => o.value === props.modelValue))
  } else {
    if (dropdownRef.value) {
      gsapCtx.value?.add(() => {
        anim.dropdownLeave(dropdownRef.value!)
      })
    }
    activeIndex.value = -1
  }
})

function toggle() {
  if (!props.disabled) open.value = !open.value
}

function select(val: string) {
  emit('update:modelValue', val)
  open.value = false
}

function clear(e: MouseEvent) {
  e.stopPropagation()
  emit('update:modelValue', '')
}

function onKeydown(e: KeyboardEvent) {
  if (props.disabled) return
  if (!open.value) {
    if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
      e.preventDefault()
      open.value = true
    }
    return
  }
  const opts = props.options.filter(o => !o.disabled)
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      activeIndex.value = opts.findIndex((_, i) => i > activeIndex.value)
      if (activeIndex.value === -1) activeIndex.value = 0
      break
    case 'ArrowUp':
      e.preventDefault()
      {
        const idx = activeIndex.value - 1
        activeIndex.value = idx < 0 ? opts.length - 1 : idx
      }
      break
    case 'Enter':
      e.preventDefault()
      if (activeIndex.value >= 0 && opts[activeIndex.value]) {
        select(opts[activeIndex.value].value)
      }
      break
    case 'Escape':
      e.preventDefault()
      open.value = false
      break
  }
}

onMounted(() => {
  gsapCtx.value = gsap.context(() => {
    if (containerRef.value) anim.fadeIn(containerRef.value, { from: { y: 6, opacity: 0 } })
  })
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div
    ref="containerRef"
    :class="cls"
    role="combobox"
    :aria-expanded="open"
    aria-haspopup="listbox"
    :aria-label="placeholder"
    tabindex="0"
    @click="toggle"
    @keydown="onKeydown"
    v-click-outside="() => open = false"
  >
    <span class="os-select__value">{{ selected?.label || placeholder }}</span>
    <button v-if="clearable && modelValue" class="os-select__clear" @click="clear" tabindex="-1">
      <OsIcon icon="mdi:close-circle" :size="12" />
    </button>
    <OsIcon icon="mdi:chevron-down" :size="14" class="os-select__arrow" />
    <div v-if="open" ref="dropdownRef" class="os-select__dropdown" role="listbox">
      <div
        v-for="(opt, i) in options"
        :key="opt.value"
        :id="`os-select-opt-${opt.value}`"
        class="os-select__option"
        :class="{
          'os-select__option--selected': opt.value === modelValue,
          'os-select__option--disabled': opt.disabled,
          'os-select__option--focused': i === activeIndex,
        }"
        role="option"
        :aria-selected="opt.value === modelValue"
        :aria-disabled="opt.disabled || undefined"
        @click.stop="!opt.disabled && select(opt.value)"
      >
        {{ opt.label }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.os-select {
  position: relative;
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  user-select: none;
  font-family: var(--font-mono);
  transition: all var(--transition-fast);
  min-width: 100px;
}

.os-select:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.os-select--sm { padding: 3px 8px; font-size: var(--font-size-xs); height: 22px; border-radius: var(--radius-sm); }
.os-select--md { padding: 5px 10px; font-size: var(--font-size-base); height: 28px; border-radius: var(--radius-md); }
.os-select--lg { padding: 7px 12px; font-size: var(--font-size-md); height: 34px; border-radius: var(--radius-md); }

.os-select--default {
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  color: var(--text-primary);
}
.os-select--default:hover,
.os-select--default.os-select--open { border-color: var(--accent); }

.os-select--glass {
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur-light));
  border: 1px solid var(--border-glass);
  color: var(--text-primary);
}
.os-select--glass:hover,
.os-select--glass.os-select--open { background: rgba(255,255,255,0.08); border-color: var(--border-glass-hover); }

.os-select--neon {
  background: transparent;
  border: 1px solid rgba(0, 255, 65, 0.3);
  color: var(--text-accent);
}
.os-select--neon:hover,
.os-select--neon.os-select--open { border-color: var(--accent); box-shadow: 0 0 8px var(--accent-dim); }

.os-select--gothic {
  background: #1a0808;
  border: 1px solid #3a1a1a;
  color: #ff8db3;
}
.os-select--gothic:hover,
.os-select--gothic.os-select--open { border-color: #5a2a2a; }

.os-select--cute {
  background: rgba(255,107,157,0.05);
  border: 1px solid rgba(255,107,157,0.2);
  color: var(--text-pink);
  border-radius: var(--radius-xl);
}
.os-select--cute:hover,
.os-select--cute.os-select--open { border-color: rgba(255,107,157,0.4); }

.os-select--disabled { opacity: 0.4; cursor: not-allowed; }

.os-select__value {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.os-select--has-value .os-select__value { color: inherit; }

.os-select__clear {
  display: flex;
  align-items: center;
  color: var(--text-muted);
  cursor: pointer;
  background: none;
  border: none;
  padding: 0;
}
.os-select__clear:hover { color: var(--text-primary); }

.os-select__arrow {
  flex-shrink: 0;
  color: var(--text-muted);
  transition: transform var(--transition-fast);
}
.os-select--open .os-select__arrow { transform: rotate(180deg); }

.os-select__dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-lg);
  padding: 4px;
  box-shadow: var(--shadow-dropdown);
  z-index: 1000;
  max-height: 200px;
  overflow-y: auto;
  will-change: transform, opacity;
}

.os-select__option {
  padding: 6px 10px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  font-size: inherit;
}
.os-select__option:hover {
  background: var(--bg-overlay);
  color: var(--text-primary);
}
.os-select__option--selected {
  color: var(--text-accent);
  background: var(--accent-dim);
}
.os-select__option--focused {
  outline: 1px solid var(--accent);
  outline-offset: -1px;
}
.os-select__option--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
