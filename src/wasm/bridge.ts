/// <reference types="vite/client" />

type WasmModule = typeof import('cybermanju-drive-wasm')

let wasm: WasmModule | null = null
let initPromise: Promise<WasmModule> | null = null

export async function initWasm(): Promise<WasmModule> {
  if (wasm) return wasm
  if (initPromise) return initPromise
  initPromise = (async () => {
    try {
      const mod = await import('cybermanju-drive-wasm')
      mod.init()
      wasm = mod
      return mod
    } catch (err) {
      initPromise = null
      throw new Error(`WASM init failed: ${err}`)
    }
  })()
  return initPromise
}

export function getWasm(): WasmModule {
  if (!wasm) throw new Error('WASM not initialized. Call initWasm() first.')
  return wasm
}

export async function withWasm<T>(fn: (mod: WasmModule) => T): Promise<T> {
  const mod = await initWasm()
  return fn(mod)
}

export function isWasmSupported(): boolean {
  return typeof WebAssembly === 'object' && WebAssembly !== null
}

export function isWasmReady(): boolean {
  return wasm !== null
}
