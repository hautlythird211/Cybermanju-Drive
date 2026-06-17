// Cybermanju Drive — Main Entry
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './assets/main.css'
import { isWasmSupported } from '@/wasm'
import { clickOutside } from '@/directives/clickOutside'

const app = createApp(App)
app.directive('click-outside', clickOutside)
app.use(createPinia())
app.mount('#app')

// Bootstrap WASM module in background (non-blocking)
if (isWasmSupported()) {
  import('@/wasm').then(async ({ initWasm }) => {
    try {
      await initWasm()
      console.log('[WASM] Cybermanju Drive WASM module initialized')
    } catch (err) {
      console.warn('[WASM] Module not available, falling back to pure JS:', err)
    }
  })
} else {
  console.log('[WASM] WebAssembly not supported, running in pure JS mode')
}