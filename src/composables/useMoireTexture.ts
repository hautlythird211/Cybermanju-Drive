import { ref, shallowRef, onUnmounted, type Ref } from 'vue'

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
}

const DEFAULT_OPTS: MoireOptions = {
  width: 1920,
  height: 1080,
  scale: 0.25,
  speed: 1,
  layers: 3,
  baseSpacing: 18,
  baseRotation: 0.15,
  contrast: 0.6,
  hueShift: 0,
  saturation: 50,
  brightness: 45,
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
    canvas.value = c
  }

  function resize(w: number, h: number) {
    opts.width = w
    opts.height = h
    createCanvas(w, h)
  }

  function draw() {
    if (!offCtx || !offscreen || !canvas.value) return
    t++

    const iw = internalW
    const ih = internalH
    const spacing = opts.baseSpacing * (opts.scale ?? 0.25)
    const speed = (opts.speed ?? 1) * 0.005
    const layers = opts.layers ?? 3
    const rot = opts.baseRotation ?? 0.15
    const hueShift = opts.hueShift ?? 0
    const sat = opts.saturation ?? 50
    const bright = opts.brightness ?? 45
    const contrast = opts.contrast ?? 0.6

    offCtx.fillStyle = `hsl(${hueShift}, ${sat}%, ${bright}%)`
    offCtx.fillRect(0, 0, iw, ih)

    for (let l = 0; l < layers; l++) {
      const angle = rot + (l / layers) * Math.PI / 3
      const layerSpacing = spacing * (1 + l * 0.3)
      const cosA = Math.cos(angle)
      const sinA = Math.sin(angle)
      const offset = t * speed * (1 + l * 0.2) * layerSpacing

      for (let line = -50; line < Math.max(iw, ih) * 2 + 50; line++) {
        const proj = line * layerSpacing + offset
        const x = cosA * proj
        const y = sinA * proj

        // Clamp to extended bounds
        if (x < -layerSpacing || x > iw + layerSpacing || y < -layerSpacing || y > ih + layerSpacing) continue

        const hue = (hueShift + l * 40 + line * 3 + t * 0.3) % 360
        const alpha = (0.15 + (Math.sin(line * 0.3) * 0.5 + 0.5) * 0.35) * contrast
        offCtx.strokeStyle = `hsla(${hue|0}, ${sat}%, ${bright + 20 + Math.sin(line * 0.2) * 15|0}%, ${alpha})`
        offCtx.lineWidth = 0.5 + (Math.sin(line * 0.4 + l) * 0.5 + 0.5) * 1.2
        offCtx.beginPath()

        // Draw line across the full diagonal extent
        const perpX = -sinA * 9999
        const perpY = cosA * 9999
        offCtx.moveTo(x - perpX, y - perpY)
        offCtx.lineTo(x + perpX, y + perpY)
        offCtx.stroke()
      }
    }

    // Scale up to full resolution with smoothing
    const ctx = canvas.value.getContext('2d')
    if (ctx) {
      ctx.imageSmoothingEnabled = false
      ctx.clearRect(0, 0, opts.width, opts.height)
      ctx.drawImage(offscreen, 0, 0, opts.width, opts.height)
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
  }
}
