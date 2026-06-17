import { ref, watch, onUnmounted, type Ref } from 'vue'

const FOCUSABLE =
  'a[href], button, input, textarea, select, [tabindex]:not([tabindex="-1"])'

export interface FocusTrapOptions {
  active?: Ref<boolean>
  initialFocus?: boolean
  onEscape?: () => void
}

export function useFocusTrap(
  containerRef: Ref<HTMLElement | null>,
  options?: FocusTrapOptions,
) {
  const previouslyFocused = ref<HTMLElement | null>(null)

  function getFocusableElements(): HTMLElement[] {
    if (!containerRef.value) return []
    return Array.from(
      containerRef.value.querySelectorAll(FOCUSABLE),
    ) as HTMLElement[]
  }

  function focusFirst() {
    const elements = getFocusableElements()
    if (elements.length > 0) {
      elements[0].focus()
    }
  }

  function activate() {
    previouslyFocused.value = document.activeElement as HTMLElement
    if (options?.initialFocus !== false) {
      setTimeout(() => focusFirst(), 50)
    }
    document.addEventListener('keydown', onKeydown)
    if (options?.active) {
      options.active.value = true
    }
  }

  function deactivate() {
    document.removeEventListener('keydown', onKeydown)
    if (previouslyFocused.value) {
      previouslyFocused.value.focus()
      previouslyFocused.value = null
    }
    if (options?.active) {
      options.active.value = false
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      deactivate()
      options?.onEscape?.()
      return
    }
    if (e.key !== 'Tab') return
    const elements = getFocusableElements()
    if (elements.length === 0) return
    const first = elements[0]
    const last = elements[elements.length - 1]
    if (e.shiftKey) {
      if (document.activeElement === first) {
        e.preventDefault()
        last.focus()
      }
    } else {
      if (document.activeElement === last) {
        e.preventDefault()
        first.focus()
      }
    }
  }

  if (options?.active) {
    watch(
      options.active,
      (active) => {
        if (active) activate()
        else deactivate()
      },
      { immediate: true },
    )
  }

  onUnmounted(() => {
    document.removeEventListener('keydown', onKeydown)
  })

  return { activate, deactivate, onKeydown }
}
