import type { Directive } from 'vue'

export const clickOutside: Directive = {
  mounted(el, binding) {
    function onClick(e: MouseEvent) {
      if (!el.contains(e.target as Node)) {
        binding.value()
      }
    }
    el.__clickOutside = onClick
    document.addEventListener('click', onClick)
  },
  unmounted(el) {
    document.removeEventListener('click', el.__clickOutside)
    delete el.__clickOutside
  },
}
