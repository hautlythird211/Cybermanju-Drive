import { shallowRef } from 'vue'

export interface MoireOptions {
  width: number
  height: number
  scale?: number
  speed?: number
  layers?: number
  baseSpacing?: number
  baseRotation?: number
  contrast?: number
  hueShift?: number
  saturation?: number
  brightness?: number
  lineWidth?: number
  wrapMode?: 'repeat' | 'mirror' | 'clamp'
}

const DEFAULT_OPTS: MoireOptions = {
  width: 1920,
  height: 1080,
  scale: 0.25,
  speed: 1,
  layers: 4,
  baseSpacing: 18,
  baseRotation: 0.15,
  contrast: 0.6,
  hueShift: 0,
  saturation: 50,
  brightness: 45,
  lineWidth: 0.5,
  wrapMode: 'repeat',
}

export function useMoireTexture(options?: Partial<MoireOptions>) {
  const opts: MoireOptions = { ...DEFAULT_OPTS, ...options }
  const canvas = shallowRef<HTMLCanvasElement | null>(null)
  let offscreen: HTMLCanvasElement | null = null
  let offCtx: CanvasRenderingContext2D | null = null
  let animId = 0
  let t = 0
  let internalW = 0
  let internalH = 0

  function createCanvas(w: number, h: number) {
    internalW = Math.ceil(w * (opts.scale ?? 0.25))
    internalH = Math.ceil(h * (opts.scale ?? 0.25))
    offscreen = document.createElement('canvas')
    offscreen.width = internalW
    offscreen.height = internalH
    offCtx = offscreen.getContext('2d')
    const c = document.createElement('canvas')
    c.width = w
    c.height = h
    c.style.width = w + 'px'
    c.style.height = h + 'px'
    c.style.objectFit = 'cover'
    canvas.value = c
  }

  function resize(w: number, h: number) {
    opts.width = w
    opts.height = h
    if (canvas.value) {
      createCanvas(w, h)
    }
  }

  /** Render a single frame of the moiré pattern onto a given context.
   *  Useful for embedding in CanvasEngine or other renderers. */
  function renderFrame(
    ctx: CanvasRenderingContext2D,
    destW: number,
    destH: number,
    time: number,
    overrides?: Partial<MoireOptions>
  ): void {
    const renderOpts = { ...opts, ...overrides }
    const spacing = (renderOpts.baseSpacing ?? 18) * (renderOpts.scale ?? 0.25)
    const speed = (renderOpts.speed ?? 1) * 0.005
    const layers = renderOpts.layers ?? 4
    const rot = renderOpts.baseRotation ?? 0.15
    const hueShift = renderOpts.hueShift ?? 0
    const sat = renderOpts.saturation ?? 50
    const bright = renderOpts.brightness ?? 45
    const contrast = renderOpts.contrast ?? 0.6
    const lw = renderOpts.lineWidth ?? 0.5

    // Render at internal resolution then scale
    const iw = Math.ceil(destW * (renderOpts.scale ?? 0.25))
    const ih = Math.ceil(destH * (renderOpts.scale ?? 0.25))

    // If we have an offscreen at the right size, reuse it; else draw direct
    if (offscreen && offCtx && offscreen.width === iw && offscreen.height === ih) {
      offCtx.fillStyle = `hsl(${hueShift}, ${sat}%, ${bright}%)`
      offCtx.fillRect(0, 0, iw, ih)

      for (let l = 0; l < layers; l++) {
        const angle = rot + (l / layers) * Math.PI / 3 + time * 0.0002 * (l + 1) * 0.3
        const layerSpacing = spacing * (1 + l * 0.3)
        const cosA = Math.cos(angle)
        const sinA = Math.sin(angle)
        const offset = time * speed * (1 + l * 0.2) * layerSpacing

        for (let line = -50; line < Math.max(iw, ih) * 2 + 50; line++) {
          const proj = line * layerSpacing + offset
          const x = cosA * proj
          const y = sinA * proj

          if (x < -layerSpacing || x > iw + layerSpacing || y < -layerSpacing || y > ih + layerSpacing) continue

          const hue = (hueShift + l * 40 + line * 3 + time * 0.3) % 360
          const alpha = (0.15 + (Math.sin(line * 0.3) * 0.5 + 0.5) * 0.35) * contrast
          offCtx.strokeStyle = `hsla(${hue|0}, ${sat}%, ${bright + 20 + Math.sin(line * 0.2) * 15|0}%, ${alpha})`
          offCtx.lineWidth = lw + (Math.sin(line * 0.4 + l) * 0.5 + 0.5) * 1.2
          offCtx.beginPath()

          const perpX = -sinA * 9999
          const perpY = cosA * 9999
          offCtx.moveTo(x - perpX, y - perpY)
          offCtx.lineTo(x + perpX, y + perpY)
          offCtx.stroke()
        }
      }

      ctx.imageSmoothingEnabled = false
      ctx.drawImage(offscreen, 0, 0, destW, destH)
    } else {
      // Fallback: render directly (lower perf but works anywhere)
      for (let l = 0; l < layers; l++) {
        const angle = rot + (l / layers) * Math.PI / 3 + time * 0.0002 * (l + 1) * 0.3
        const layerSpacing = spacing * (1 + l * 0.3)
        const cosA = Math.cos(angle)
        const sinA = Math.sin(angle)
        const offset = time * speed * (1 + l * 0.2) * layerSpacing
        const hueOff = l * 40

        ctx.beginPath()
        for (let line = -50; line < Math.max(destW, destH) * 2 + 50; line++) {
          const proj = line * layerSpacing + offset
          const lx = cosA * proj
          const ly = sinA * proj
          if (lx < -layerSpacing || lx > destW + layerSpacing || ly < -layerSpacing || ly > destH + layerSpacing) continue

          const perpX = -sinA * 9999
          const perpY = cosA * 9999
          ctx.moveTo(lx - perpX, ly - perpY)
          ctx.lineTo(lx + perpX, ly + perpY)
        }
        const lHue = (hueShift + hueOff) % 360
        ctx.strokeStyle = `hsla(${lHue|0},40,45,0.12)`
        ctx.lineWidth = lw
        ctx.stroke()
      }
    }
  }

  function draw() {
    if (!offCtx || !offscreen || !canvas.value) return
    t++

    const ctx = canvas.value.getContext('2d')
    if (ctx) {
      renderFrame(ctx, opts.width, opts.height, t)
    }

    animId = requestAnimationFrame(draw)
  }

  function start() {
    if (animId) return
    createCanvas(opts.width, opts.height)
    draw()
  }

  function stop() {
    if (animId) {
      cancelAnimationFrame(animId)
      animId = 0
    }
  }

  function setOptions(newOpts: Partial<MoireOptions>) {
    Object.assign(opts, newOpts)
  }

  return {
    canvas,
    start,
    stop,
    resize,
    setOptions,
    renderFrame,
  }
}
