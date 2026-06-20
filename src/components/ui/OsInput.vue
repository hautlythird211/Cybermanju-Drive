<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()
const inputRef = ref<HTMLElement | null>(null)
const wrapperRef = ref<HTMLElement | null>(null)

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  type?: string
  size?: 'sm' | 'md' | 'lg'
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  prefix?: string
  suffix?: string
  disabled?: boolean
  readonly?: boolean
  rows?: number
  monospace?: boolean
  glow?: boolean
  vintage?: boolean
  ariaLabel?: string
  ariaInvalid?: boolean
  autocomplete?: string
  spellcheck?: boolean
}>(), {
  modelValue: '',
  placeholder: '',
  type: 'text',
  size: 'md',
  variant: 'default',
  disabled: false,
  readonly: false,
  monospace: true,
  glow: false,
  vintage: false,
  ariaInvalid: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  focus: [e: FocusEvent]
  blur: [e: FocusEvent]
  keydown: [e: KeyboardEvent]
}>()

const isTextarea = computed(() => props.type === 'textarea')

const cls = computed(() => [
  'os-input-wrapper',
  `os-input--${props.variant}`,
  `os-input--${props.size}`,
  {
    'os-input--disabled': props.disabled,
    'os-input--glow': props.glow,
    'os-input--vintage': props.vintage,
    'os-input--textarea': isTextarea.value,
  },
])

function onInput(e: Event) {
  const target = e.target as HTMLInputElement | HTMLTextAreaElement
  emit('update:modelValue', target.value)
}

function onFocus(e: FocusEvent) {
  if (wrapperRef.value) {
    anim.killTweens(wrapperRef.value)
    anim.fadeIn(wrapperRef.value, { from: { opacity: 1 } })
  }
  emit('focus', e)
}

function onBlur(e: FocusEvent) {
  emit('blur', e)
}

function onKeydown(e: KeyboardEvent) {
  emit('keydown', e)
}

onMounted(() => {
  if (inputRef.value) anim.fadeIn(inputRef.value, { from: { y: 6, opacity: 0 } })
})
</script>

<template>
  <div ref="wrapperRef" :class="[...cls, 'gpu']">
    <span v-if="prefix" class="os-input__prefix">{{ prefix }}</span>
    <textarea
      v-if="isTextarea"
      ref="inputRef"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :rows="rows || 3"
      class="os-input__field"
      :class="{ 'os-input__mono': monospace }"
      :aria-label="ariaLabel"
      :aria-invalid="ariaInvalid"
      :autocomplete="autocomplete"
      :spellcheck="spellcheck"
      @input="onInput"
      @focus="onFocus"
      @blur="onBlur"
      @keydown="onKeydown"
    />
    <input
      v-else
      ref="inputRef"
      :value="modelValue"
      :type="type"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      class="os-input__field"
      :class="{ 'os-input__mono': monospace }"
      :aria-label="ariaLabel"
      :aria-invalid="ariaInvalid || undefined"
      :autocomplete="autocomplete"
      :spellcheck="spellcheck"
      @input="onInput"
      @focus="onFocus"
      @blur="onBlur"
      @keydown="onKeydown"
    />
    <span v-if="suffix" class="os-input__suffix">{{ suffix }}</span>
  </div>
</template>

<style scoped>
.os-input-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
  border-radius: var(--radius-md);
  transition: all var(--duration-fast) var(--ease-spring);
  overflow: hidden;
  will-change: box-shadow;
  contain: layout style;
}

.os-input-wrapper:focus-within {
  box-shadow: var(--focus-ring);
}

.os-input-wrapper:focus-within {
  transition: all var(--ease-spring, cubic-bezier(0.22, 1, 0.36, 1));
}

.os-input__field {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: inherit;
  font-family: inherit;
  font-size: inherit;
  padding: 0;
  min-width: 0;
}

.os-input__field::placeholder {
  color: var(--text-muted);
}

.os-input__mono {
  font-family: var(--font-mono);
}

.os-input__prefix,
.os-input__suffix {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  white-space: nowrap;
  flex-shrink: 0;
}

/* sizes */
.os-input--sm { padding: 3px 8px; font-size: var(--font-size-sm); }
.os-input--md { padding: 6px 10px; font-size: var(--font-size-base); }
.os-input--lg { padding: 8px 12px; font-size: var(--font-size-md); }

/* variant: default */
.os-input--default {
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  color: var(--text-primary);
}
.os-input--default:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-dim);
}

/* variant: glass */
.os-input--glass {
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border: 1px solid var(--border-glass);
  color: var(--text-primary);
  box-shadow: var(--panel-inset);
}
.os-input--glass:focus-within {
  border-color: var(--border-glass-hover);
  background: rgba(255, 255, 255, 0.08);
}

/* variant: neon */
.os-input--neon {
  background: transparent;
  border: 1px solid rgba(var(--accent-rgb), 0.3);
  color: var(--text-accent);
  text-shadow: 0 0 2px var(--accent-glow);
}
.os-input--neon:focus-within {
  border-color: var(--accent);
  box-shadow: var(--glow-accent), 0 0 12px var(--accent-dim);
}

/* variant: gothic */
.os-input--gothic {
  background: #1a0a0a;
  border: 1px solid #3a1a1a;
  color: #ff8db3;
}
.os-input--gothic:focus-within {
  border-color: #5a2a2a;
  box-shadow: 0 0 8px rgba(255, 107, 157, 0.1);
}

/* variant: cute */
.os-input--cute {
  background: rgba(255, 107, 157, 0.05);
  border: 1px solid rgba(255, 107, 157, 0.2);
  color: var(--text-pink);
  border-radius: var(--radius-xl);
}
.os-input--cute:focus-within {
  border-color: rgba(255, 107, 157, 0.4);
}

.os-input--disabled { opacity: 0.4; pointer-events: none; }

.os-input--glow:focus-within {
  box-shadow: 0 0 16px var(--accent-glow);
}

.os-input--textarea {
  align-items: flex-start;
}

.os-input--textarea .os-input__field {
  resize: vertical;
  min-height: 48px;
  line-height: 1.5;
}

.os-input--vintage {
  position: relative;
}
.os-input--vintage::after {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0, 0, 0, 0.03) 2px,
    rgba(0, 0, 0, 0.03) 4px
  );
}
</style>
