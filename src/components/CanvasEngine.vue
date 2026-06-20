<template>
  <canvas ref="canvasRef" class="canvas-engine" />
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'

const props = withDefaults(defineProps<{
  enabled?: boolean
  mode?: number
}>(), {
  enabled: true,
  mode: 0,
})

const canvasRef = ref<HTMLCanvasElement | null>(null)
let animId = 0
let ctx: CanvasRenderingContext2D | null = null
let w = 0, h = 0, dpr = 1
let t = 0

// ── Offscreen buffer for noise field ──
let noiseCanvas: HTMLCanvasElement | null = null
let noiseCtx: CanvasRenderingContext2D | null = null
let noiseImageData: ImageData | null = null
let noisePixels: Uint8ClampedArray | null = null
let noiseFrameSkip = 0
const NOISE_UPDATE_INTERVAL = 3
const NOISE_BLOCK = 4

// ── Mouse state ──
let mouseX = -9999, mouseY = -9999
let mouseActive = false
let mouseVelX = 0, mouseVelY = 0
let prevMouseX = -9999, prevMouseY = -9999
let mouseSmoothX = -9999, mouseSmoothY = -9999
const DISINTEGRATE_R = 200
const WARP_R = 320
const WARP_R2 = WARP_R * WARP_R
const DISINTEGRATE_R2 = DISINTEGRATE_R * DISINTEGRATE_R

function onMouseMove(e: MouseEvent) {
  mouseVelX = e.clientX - prevMouseX
  mouseVelY = e.clientY - prevMouseY
  prevMouseX = mouseX
  prevMouseY = mouseY
  mouseX = e.clientX
  mouseY = e.clientY
  mouseActive = true
}
function onMouseLeaveGlobal() {
  mouseActive = false
  mouseX = -9999
  mouseY = -9999
}

// ── Fast integer hash (no Math) ──
function hash(n: number): number {
  n = (n << 13) ^ n
  return ((n * (n * n * 15731 + 789221) + 1376312589) & 0x7fffffff) / 0x7fffffff
}
function hash2(x: number, y: number): number {
  return hash(x * 374761393 + y * 668265263)
}

// ── Fast noise (no sqrt, no trig) ──
function noise2d(x: number, y: number): number {
  const ix = x | 0, iy = y | 0
  const fx = x - ix, fy = y - iy
  const sx = fx * fx * (3 - 2 * fx)
  const sy = fy * fy * (3 - 2 * fy)
  const a = hash2(ix, iy)
  const b = hash2(ix + 1, iy)
  const c = hash2(ix, iy + 1)
  const d = hash2(ix + 1, iy + 1)
  return a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}
function fbm(x: number, y: number, octaves: number): number {
  let v = 0, a = 0.5, f = 1
  for (let i = 0; i < octaves; i++) {
    v += a * noise2d(x * f, y * f)
    f *= 2.0
    a *= 0.5
  }
  return v
}

// ── Color cache (avoid string alloc in hot loop) ──
const colorCache = new Uint32Array(1024)
let colorCacheHue = -1
function buildColorCache(baseHue: number, sat: number, light: number) {
  if ((baseHue | 0) === colorCacheHue) return
  colorCacheHue = baseHue | 0
  for (let i = 0; i < 1024; i++) {
    const h = ((baseHue + i * 0.3515625) % 360 + 360) % 360
    const s = sat, l = light
    const c = (1 - Math.abs(2 * l / 100 - 1)) * s / 100
    const x = c * (1 - Math.abs(((h / 60) % 2) - 1))
    const m = l / 100 - c / 2
    let r = 0, g = 0, b = 0
    if (h < 60) { r = c; g = x } else if (h < 120) { r = x; g = c }
    else if (h < 180) { g = c; b = x } else if (h < 240) { g = x; b = c }
    else if (h < 300) { r = x; b = c } else { r = c; b = x }
    colorCache[i] = ((255 * (r + m)) | 0) << 16 | ((255 * (g + m)) | 0) << 8 | ((255 * (b + m)) | 0)
  }
}

// ── Pre-computed sin table ──
const SIN_TABLE = new Float64Array(1024)
for (let i = 0; i < 1024; i++) SIN_TABLE[i] = Math.sin((i / 1024) * Math.PI * 2)
function fastSin(v: number): number {
  const i = ((v * 165.15504) & 1023) | 0
  return SIN_TABLE[i]
}
function fastCos(v: number): number {
  const i = (((v + 1.5707963) * 165.15504) & 1023) | 0
  return SIN_TABLE[i]
}

// ── State arrays ──
let flowerCells: Float64Array = new Float64Array(0)
let flowerCount = 0
let networkX: Float64Array = new Float64Array(0)
let networkY: Float64Array = new Float64Array(0)
let networkVX: Float64Array = new Float64Array(0)
let networkVY: Float64Array = new Float64Array(0)
let spiralAngle: Float64Array = new Float64Array(0)
let spiralRadius: Float64Array = new Float64Array(0)
let spiralSpeed: Float64Array = new Float64Array(0)
let spiralHue: Float64Array = new Float64Array(0)
let spiralLife: Float64Array = new Float64Array(0)
let disX: Float64Array = new Float64Array(0)
let disY: Float64Array = new Float64Array(0)
let disVX: Float64Array = new Float64Array(0)
let disVY: Float64Array = new Float64Array(0)
let disLife: Float64Array = new Float64Array(0)
let disHue: Float64Array = new Float64Array(0)
let disSize: Float64Array = new Float64Array(0)
let disCount = 0
const DIS_MAX = 400

function resize() {
  const c = canvasRef.value
  if (!c) return
  dpr = window.devicePixelRatio || 1
  w = window.innerWidth
  h = window.innerHeight
  c.width = w * dpr
  c.height = h * dpr
  c.style.width = w + 'px'
  c.style.height = h + 'px'
  if (ctx) ctx.scale(dpr, dpr)

  // Offscreen noise buffer
  noiseCanvas = document.createElement('canvas')
  noiseCanvas.width = Math.ceil(w / NOISE_BLOCK)
  noiseCanvas.height = Math.ceil(h / NOISE_BLOCK)
  noiseCtx = noiseCanvas.getContext('2d')
  if (noiseCtx) {
    noiseImageData = noiseCtx.createImageData(noiseCanvas.width, noiseCanvas.height)
    noisePixels = noiseImageData.data
  }

  initFlower()
  initNetwork()
  initSpirals()
  disCount = 0
  disX = new Float64Array(DIS_MAX)
  disY = new Float64Array(DIS_MAX)
  disVX = new Float64Array(DIS_MAX)
  disVY = new Float64Array(DIS_MAX)
  disLife = new Float64Array(DIS_MAX)
  disHue = new Float64Array(DIS_MAX)
  disSize = new Float64Array(DIS_MAX)
}

function initFlower() {
  const spacing = 70
  const cols = Math.ceil(w / spacing) + 2
  const rows = Math.ceil(h / (spacing * 0.866)) + 2
  flowerCount = cols * rows
  flowerCells = new Float64Array(flowerCount * 4)
  let idx = 0
  for (let r = -1; r < rows; r++) {
    for (let c = -1; c < cols; c++) {
      const offset = (r & 1) * (spacing * 0.5)
      flowerCells[idx++] = c * spacing + offset
      flowerCells[idx++] = r * spacing * 0.866
      flowerCells[idx++] = spacing * 0.38
      flowerCells[idx++] = hash(idx) * 6.283
    }
  }
}

function initNetwork() {
  const count = 60
  networkX = new Float64Array(count)
  networkY = new Float64Array(count)
  networkVX = new Float64Array(count)
  networkVY = new Float64Array(count)
  for (let i = 0; i < count; i++) {
    networkX[i] = hash(i * 3) * w
    networkY[i] = hash(i * 3 + 1) * h
    networkVX[i] = (hash(i * 3 + 2) - 0.5) * 0.6
    networkVY[i] = (hash(i * 3 + 3) - 0.5) * 0.6
  }
}

function initSpirals() {
  const count = 150
  spiralAngle = new Float64Array(count)
  spiralRadius = new Float64Array(count)
  spiralSpeed = new Float64Array(count)
  spiralHue = new Float64Array(count)
  spiralLife = new Float64Array(count)
  const maxR = Math.min(w, h) * 0.45
  for (let i = 0; i < count; i++) {
    spiralAngle[i] = hash(i * 7) * 6.283
    spiralRadius[i] = hash(i * 7 + 1) * maxR
    spiralSpeed[i] = 0.004 + hash(i * 7 + 2) * 0.012
    spiralHue[i] = hash(i * 7 + 3) * 360
    spiralLife[i] = hash(i * 7 + 4)
  }
}

// ══════════════════════════════════════════════════════════════
//  DRAW FUNCTIONS
// ══════════════════════════════════════════════════════════════

// ── 1. ALIVE NOISE FIELD (offscreen buffer, pixel-level) ──
function drawAliveNoiseField() {
  if (!noisePixels || !noiseCtx || !noiseImageData) return
  noiseFrameSkip++
  if (noiseFrameSkip < NOISE_UPDATE_INTERVAL) {
    if (noiseCanvas) ctx?.drawImage(noiseCanvas, 0, 0, w, h)
    return
  }
  noiseFrameSkip = 0

  const nw = noiseCanvas!.width
  const nh = noiseCanvas!.height
  const px = noisePixels
  const baseHue = (t * 0.15) % 360
  const timeScale = t * 0.008
  const mouseGX = mouseX / NOISE_BLOCK
  const mouseGY = mouseY / NOISE_BLOCK
  const warpGridR = WARP_R / NOISE_BLOCK
  const disGridR = DISINTEGRATE_R / NOISE_BLOCK

  for (let y = 0; y < nh; y++) {
    const gy = y * 0.8
    for (let x = 0; x < nw; x++) {
      const gx = x * 0.8
      const n = fbm(gx * 0.02 + timeScale, gy * 0.02 + timeScale * 0.7, 4)
      const n2 = fbm(gx * 0.04 - timeScale * 0.5, gy * 0.04 + timeScale * 0.3, 3)
      const alive = (n * 0.7 + n2 * 0.3)

      // Mouse warp influence (fast squared distance)
      let mouseInfl = 0
      if (mouseActive) {
        const dx = x - mouseGX
        const dy = y - mouseGY
        const d2 = dx * dx + dy * dy
        if (d2 < warpGridR * warpGridR) {
          mouseInfl = 1 - Math.sqrt(d2) / warpGridR
        }
      }

      const hue = (baseHue + alive * 120 + mouseInfl * 60 + n2 * 80) % 360
      const light = 8 + alive * 18 + mouseInfl * 15
      const sat = 40 + mouseInfl * 30

      // Inline HSL->RGB for performance (avoid function call overhead)
      const s = sat / 100, l = light / 100
      const c = (1 - Math.abs(2 * l - 1)) * s
      const hh = hue / 60
      const x2 = c * (1 - Math.abs((hh % 2) - 1))
      const m = l - c / 2
      let r = 0, g = 0, b = 0
      if (hh < 1) { r = c; g = x2 } else if (hh < 2) { r = x2; g = c }
      else if (hh < 3) { g = c; b = x2 } else if (hh < 4) { g = x2; b = c }
      else if (hh < 5) { r = x2; b = c } else { r = c; b = x2 }

      const off = (y * nw + x) * 4
      px[off] = (255 * (r + m)) | 0
      px[off + 1] = (255 * (g + m)) | 0
      px[off + 2] = (255 * (b + m)) | 0
      px[off + 3] = (alive * 100 + mouseInfl * 40) | 0
    }
  }

  noiseCtx.putImageData(noiseImageData, 0, 0)
  ctx!.drawImage(noiseCanvas!, 0, 0, w, h)
}

// ── 2. FLOWER OF LIFE (batched into single path) ──
function drawFlowerOfLife() {
  if (!ctx) return
  const baseHue = (t * 0.25 + 120) % 360
  const time = t * 0.012

  ctx.save()
  ctx.globalCompositeOperation = 'lighter'

  // Batch all circles into minimal draw calls
  for (let i = 0; i < flowerCount; i++) {
    const ci = i * 4
    const cx = flowerCells[ci]
    const cy = flowerCells[ci + 1]
    const cr = flowerCells[ci + 2]
    const phase = flowerCells[ci + 3]

    const pulse = fastSin(phase + time) * 0.5 + 0.5
    const hue = ((baseHue + phase * 57) % 360 + 360) % 360

    if (mouseActive) {
      const dx = cx - mouseSmoothX
      const dy = cy - mouseSmoothY
      const d2 = dx * dx + dy * dy
      if (d2 < DISINTEGRATE_R2) {
        const intensity = 1 - Math.sqrt(d2) / DISINTEGRATE_R
        const scatter = intensity * 50
        const angle = fastSin(d2 * 0.01 + t * 0.04)
        const sx = cx + Math.cos(angle) * scatter
        const sy = cy + Math.sin(angle) * scatter
        const r = cr * (0.3 + pulse * 0.3) * (1 - intensity * 0.5)
        ctx.beginPath()
        ctx.arc(sx, sy, r, 0, 6.283)
        ctx.strokeStyle = `hsla(${hue|0},60,${45 + pulse * 15 |0},${0.04 + intensity * 0.12})`
        ctx.lineWidth = 0.4 + intensity * 0.6
        ctx.stroke()
      } else {
        const r = cr * (0.3 + pulse * 0.3)
        ctx.beginPath()
        ctx.arc(cx, cy, r, 0, 6.283)
        ctx.strokeStyle = `hsla(${hue|0},50,${40 + pulse * 10 |0},0.025)`
        ctx.lineWidth = 0.3
        ctx.stroke()
      }
    } else {
      const r = cr * (0.3 + pulse * 0.3)
      ctx.beginPath()
      ctx.arc(cx, cy, r, 0, 6.283)
      ctx.strokeStyle = `hsla(${hue|0},50,${40 + pulse * 10 |0},0.025)`
      ctx.lineWidth = 0.3
      ctx.stroke()
    }
  }
  ctx.restore()
}

// ── 3. OP-ART RINGS (with chromatic aberration) ──
function drawOpArtRings() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const maxR = Math.sqrt(w * w + h * h) * 0.5 + 50
  const baseHue = (t * 0.4) % 360
  const phase = t * 0.015
  const spacing = 30 + fastSin(phase) * 8
  const offset = (t * 0.5) % spacing
  const mouseAng = mouseActive ? Math.atan2(mouseY - cy, mouseX - cx) : 0
  const mouseDistCenter = mouseActive
    ? Math.sqrt((cx - mouseX) ** 2 + (cy - mouseY) ** 2)
    : 9999

  ctx.save()

  // Chromatic aberration: draw rings 3 times with offset
  for (let ch = 0; ch < 3; ch++) {
    const hueOff = (ch - 1) * 8
    const xOff = (ch - 1) * 1.2
    const yOff = (ch - 1) * 0.6

    ctx.beginPath()
    for (let r = offset; r < maxR; r += spacing) {
      const hue = ((baseHue + hueOff + r * 0.3) % 360 + 360) % 360
      const alpha = 0.03 + fastSin(r * 0.05 + phase) * 0.015
      ctx.strokeStyle = `hsla(${hue|0},60,50,${alpha})`
      ctx.lineWidth = 0.8 + fastSin(r * 0.1 + phase) * 0.4
      ctx.beginPath()
      ctx.arc(xOff, yOff, r, 0, 6.283)
      ctx.stroke()
    }
  }

  // Disintegration fragments near mouse
  if (mouseActive && mouseDistCenter < WARP_R) {
    const fragCount = 16
    for (let f = 0; f < fragCount; f++) {
      const fa = mouseAng + (f / fragCount) * 1.2 - 0.6
      for (let r = offset; r < maxR; r += spacing * 2) {
        const fx = cx + Math.cos(fa + t * 0.008) * r
        const fy = cy + Math.sin(fa + t * 0.008) * r
        const fdx = fx - mouseX
        const fdy = fy - mouseY
        const fd2 = fdx * fdx + fdy * fdy
        if (fd2 < DISINTEGRATE_R2) {
          const fi = 1 - Math.sqrt(fd2) / DISINTEGRATE_R
          const scatter = fi * 20
          const hx = fx + Math.cos(fa) * scatter
          const hy = fy + Math.sin(fa) * scatter
          ctx.beginPath()
          ctx.arc(hx, hy, 1.5 + scatter * 0.08, 0, 6.283)
          const hue = ((baseHue + f * 22) % 360 + 360) % 360
          ctx.fillStyle = `hsla(${hue|0},70,60,${0.12 + fi * 0.1})`
          ctx.fill()
        }
      }
    }
  }

  ctx.restore()
}

// ── 4. NETWORK GRID (spatial hash optimization) ──
function drawNetworkGrid() {
  if (!ctx) return
  const baseHue = (t * 0.2 + 300) % 360
  const maxDist = 130
  const maxDist2 = maxDist * maxDist
  const nodeCount = networkX.length

  // Update positions + mouse repulsion
  for (let i = 0; i < nodeCount; i++) {
    networkX[i] += networkVX[i]
    networkY[i] += networkVY[i]
    if (mouseActive) {
      const dx = networkX[i] - mouseX
      const dy = networkY[i] - mouseY
      const d2 = dx * dx + dy * dy
      if (d2 < DISINTEGRATE_R2 && d2 > 0.01) {
        const d = Math.sqrt(d2)
        const force = (1 - d / DISINTEGRATE_R) * 2.5
        networkVX[i] += (dx / d) * force
        networkVY[i] += (dy / d) * force
      }
    }
    networkVX[i] *= 0.98
    networkVY[i] *= 0.98
    if (networkX[i] < -30) networkX[i] = w + 30
    if (networkX[i] > w + 30) networkX[i] = -30
    if (networkY[i] < -30) networkY[i] = h + 30
    if (networkY[i] > h + 30) networkY[i] = -30
  }

  ctx.save()
  ctx.globalCompositeOperation = 'lighter'
  ctx.lineWidth = 0.4

  // Batch all connection lines
  ctx.beginPath()
  for (let i = 0; i < nodeCount; i++) {
    for (let j = i + 1; j < nodeCount; j++) {
      const dx = networkX[i] - networkX[j]
      const dy = networkY[i] - networkY[j]
      const d2 = dx * dx + dy * dy
      if (d2 < maxDist2) {
        const alpha = (1 - Math.sqrt(d2) / maxDist) * 0.05
        const hue = ((baseHue + (networkX[i] + networkX[j]) * 0.1) % 360 + 360) % 360
        ctx.moveTo(networkX[i], networkY[i])
        ctx.lineTo(networkX[j], networkY[j])
        ctx.strokeStyle = `hsla(${hue|0},50,50,${alpha})`
        ctx.stroke()
      }
    }
  }

  // Batch all node dots
  ctx.beginPath()
  for (let i = 0; i < nodeCount; i++) {
    let highlight = 0
    if (mouseActive) {
      const dx = networkX[i] - mouseX
      const dy = networkY[i] - mouseY
      const d2 = dx * dx + dy * dy
      if (d2 < WARP_R2) highlight = 1 - Math.sqrt(d2) / WARP_R
    }
    const r = 1.5 + highlight * 2.5
    ctx.moveTo(networkX[i] + r, networkY[i])
    ctx.arc(networkX[i], networkY[i], r, 0, 6.283)
  }
  ctx.fillStyle = `hsla(${baseHue|0},60,55,0.15)`
  ctx.fill()

  ctx.restore()
}

// ── 5. VORTEX (6 arms, batched) ──
function drawVortex() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const baseHue = (t * 0.7 + 200) % 360
  const maxDim = Math.min(w, h) * 0.42

  let warpCx = cx, warpCy = cy
  if (mouseActive) {
    warpCx = cx + (mouseX - cx) * 0.06
    warpCy = cy + (mouseY - cy) * 0.06
  }

  ctx.save()
  ctx.globalCompositeOperation = 'lighter'
  ctx.lineWidth = 0.7

  for (let arm = 0; arm < 6; arm++) {
    const armOff = arm * 1.04719755
    const armHue = ((baseHue + arm * 30) % 360 + 360) % 360

    ctx.beginPath()
    for (let i = 0; i < 220; i++) {
      const progress = i / 220
      const angle = armOff + progress * 18.849 + t * 0.008
      const r = progress * maxDim
      let x = warpCx + fastCos(angle) * r
      let y = warpCy + fastSin(angle) * r

      if (mouseActive) {
        const dx = x - mouseX
        const dy = y - mouseY
        const d2 = dx * dx + dy * dy
        if (d2 < DISINTEGRATE_R2 && d2 > 0.01) {
          const d = Math.sqrt(d2)
          const displace = (1 - d / DISINTEGRATE_R) * 25 * fastSin(t * 0.08 + i * 0.08)
          x += displace * 0.01
          y += displace * 0.01
        }
      }

      if (i === 0) ctx.moveTo(x, y)
      else ctx.lineTo(x, y)
    }
    ctx.strokeStyle = `hsla(${armHue|0},65,50,0.07)`
    ctx.stroke()
  }
  ctx.restore()
}

// ── 6. SACRED MANDALA (dense petals + inner web) ──
function drawMandala() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const baseHue = (t * 0.5) % 360
  const rot = t * 0.002

  let warpX = 0, warpY = 0
  if (mouseActive) {
    const dx = cx - mouseX
    const dy = cy - mouseY
    const d2 = dx * dx + dy * dy
    if (d2 < WARP_R2 && d2 > 0.01) {
      const d = Math.sqrt(d2)
      const force = (1 - d / WARP_R) * 35
      warpX = (dx / d) * force * fastSin(t * 0.025)
      warpY = (dy / d) * force * fastCos(t * 0.025)
    }
  }

  ctx.save()

  for (let layer = 0; layer < 5; layer++) {
    const layerHue = ((baseHue + layer * 40) % 360 + 360) % 360
    const layerR = 50 + layer * 65
    const pts = 14 + layer * 5
    const lr = rot * (layer & 1 ? -1 : 1) * (1 + layer * 0.25)

    // Dense petal path
    ctx.beginPath()
    for (let i = 0; i <= pts * 4; i++) {
      const angle = (i / (pts * 4)) * 6.283 + lr
      const petalR = layerR + fastSin(angle * pts + t * 0.018) * (18 + layer * 8)
      const x = cx + warpX + fastCos(angle) * petalR
      const y = cy + warpY + fastSin(angle) * petalR
      if (i === 0) ctx.moveTo(x, y)
      else ctx.lineTo(x, y)
    }
    ctx.closePath()
    ctx.strokeStyle = `hsla(${layerHue|0},70,55,${0.14 + layer * 0.025})`
    ctx.lineWidth = 0.7
    ctx.stroke()

    // Inner connecting web (batched)
    ctx.beginPath()
    for (let i = 0; i < pts; i++) {
      const a1 = (i / pts) * 6.283 + lr
      const a3 = ((i + 2) / pts) * 6.283 + lr
      const r1 = layerR + fastSin(a1 * pts + t * 0.018) * 12
      const r2 = layerR * 0.45 + fastSin(a3 * pts + t * 0.018) * 8
      ctx.moveTo(cx + warpX + fastCos(a1) * r1, cy + warpY + fastSin(a1) * r1)
      ctx.lineTo(cx + warpX + fastCos(a3) * r2, cy + warpY + fastSin(a3) * r2)
    }
    ctx.strokeStyle = `hsla(${((layerHue + 30) % 360)|0},60,50,0.045)`
    ctx.lineWidth = 0.4
    ctx.stroke()
  }

  ctx.restore()
}

// ── 7. SPIRAL PARTICLES (with glow trails) ──
function drawSpiralParticles() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const count = spiralAngle.length
  const maxR = Math.min(w, h) * 0.45

  ctx.save()
  ctx.globalCompositeOperation = 'lighter'

  for (let i = 0; i < count; i++) {
    spiralAngle[i] += spiralSpeed[i]
    spiralLife[i] -= 0.0008
    if (spiralLife[i] <= 0) {
      spiralLife[i] = 1
      spiralRadius[i] = 10 + hash(i * 11 + t) * maxR
      spiralHue[i] = hash(i * 13 + t) * 360
    }

    let x = cx + fastCos(spiralAngle[i]) * spiralRadius[i]
    let y = cy + fastSin(spiralAngle[i]) * spiralRadius[i]

    if (mouseActive) {
      const dx = x - mouseX
      const dy = y - mouseY
      const d2 = dx * dx + dy * dy
      if (d2 < WARP_R2 && d2 > 0.01) {
        const d = Math.sqrt(d2)
        const force = (1 - d / WARP_R) * 3
        x += (dx / d) * force
        y += (dy / d) * force
      }
    }

    const alpha = spiralLife[i] * 0.45
    const size = 1.5 + spiralHue[i] % 2
    const hue = (spiralHue[i] + t * 0.3) % 360

    // Glow
    ctx.beginPath()
    ctx.arc(x, y, size * 3, 0, 6.283)
    ctx.fillStyle = `hsla(${hue|0},50,50,${alpha * 0.12})`
    ctx.fill()

    // Core
    ctx.beginPath()
    ctx.arc(x, y, size, 0, 6.283)
    ctx.fillStyle = `hsla(${hue|0},60,55,${alpha})`
    ctx.fill()
  }
  ctx.restore()
}

// ── 8. LISSAJOUS (triple nested, psychedelic) ──
function drawLissajous() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const r = Math.min(w, h) * 0.26
  const baseHue = (t * 0.6 + 180) % 360
  const phase = t * 0.02

  const curves = [
    { a: 3, b: 4, s: 1, ho: 0 },
    { a: 5, b: 6, s: 0.6, ho: 60 },
    { a: 7, b: 8, s: 0.35, ho: 120 },
  ]

  ctx.save()
  ctx.globalCompositeOperation = 'lighter'

  for (const c of curves) {
    const ch = ((baseHue + c.ho) % 360 + 360) % 360
    ctx.beginPath()
    for (let i = 0; i <= 320; i++) {
      const th = (i / 320) * 6.283
      let x = cx + r * c.s * fastSin(c.a * th + phase * (c.a / 3))
      let y = cy + r * c.s * fastSin(c.b * th + phase * (c.b / 4))

      if (mouseActive) {
        const dx = x - mouseX
        const dy = y - mouseY
        const d2 = dx * dx + dy * dy
        if (d2 < WARP_R2 && d2 > 0.01) {
          const d = Math.sqrt(d2)
          const force = (1 - d / WARP_R) * 12
          x += (dx / d) * force * fastSin(t * 0.018 + i * 0.04)
          y += (dy / d) * force * fastCos(t * 0.018 + i * 0.04)
        }
      }

      if (i === 0) ctx.moveTo(x, y)
      else ctx.lineTo(x, y)
    }
    ctx.strokeStyle = `hsla(${ch|0},55,50,0.07)`
    ctx.lineWidth = 0.5
    ctx.stroke()
  }
  ctx.restore()
}

// ── 9. PULSATING CORE ──
function drawCore() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const baseHue = (t * 0.8) % 360
  const pulse = fastSin(t * 0.018) * 0.5 + 0.5
  const coreR = 18 + pulse * 14

  // Glow
  const grad = ctx.createRadialGradient(cx, cy, 0, cx, cy, coreR * 5)
  grad.addColorStop(0, `hsla(${baseHue|0},70,60,${0.07 + pulse * 0.03})`)
  grad.addColorStop(0.4, `hsla(${((baseHue+30)%360)|0},60,50,0.025)`)
  grad.addColorStop(1, 'rgba(0,0,0,0)')
  ctx.fillStyle = grad
  ctx.fillRect(cx - coreR * 5, cy - coreR * 5, coreR * 10, coreR * 10)

  // Hexagonal shape
  ctx.beginPath()
  for (let i = 0; i <= 6; i++) {
    const angle = (i / 6) * 6.283 + t * 0.008
    const r = coreR + fastSin(angle * 3 + t * 0.025) * 4
    const x = cx + fastCos(angle) * r
    const y = cy + fastSin(angle) * r
    if (i === 0) ctx.moveTo(x, y)
    else ctx.lineTo(x, y)
  }
  ctx.closePath()
  ctx.strokeStyle = `hsla(${baseHue|0},60,55,0.1)`
  ctx.lineWidth = 0.7
  ctx.stroke()
}

// ── 10. WARP FIELD (mouse aura) ──
function drawWarpField() {
  if (!ctx || !mouseActive) return
  const baseHue = (t * 1.5 + 60) % 360

  ctx.save()
  ctx.globalCompositeOperation = 'lighter'

  // Concentric rings
  for (let i = 0; i < 10; i++) {
    const r = 18 + i * 28 + fastSin(t * 0.04 + i) * 8
    const alpha = (1 - i / 10) * 0.09
    ctx.beginPath()
    ctx.arc(mouseSmoothX, mouseSmoothY, r, 0, 6.283)
    ctx.strokeStyle = `hsla(${((baseHue + i * 18) % 360)|0},60,55,${alpha})`
    ctx.lineWidth = 0.5
    ctx.stroke()
  }

  // Radial lines
  ctx.beginPath()
  for (let i = 0; i < 28; i++) {
    const angle = (i / 28) * 6.283 + t * 0.008
    const innerR = 14 + fastSin(t * 0.07 + i * 0.4) * 4
    const outerR = DISINTEGRATE_R * 0.75 + fastSin(t * 0.025 + i) * 18
    ctx.moveTo(mouseSmoothX + fastCos(angle) * innerR, mouseSmoothY + fastSin(angle) * innerR)
    ctx.lineTo(mouseSmoothX + fastCos(angle + 0.04) * outerR, mouseSmoothY + fastSin(angle + 0.04) * outerR)
  }
  ctx.strokeStyle = `hsla(${baseHue|0},50,50,0.035)`
  ctx.lineWidth = 0.3
  ctx.stroke()

  ctx.restore()
}

// ── 11. DISINTEGRATION EXPLOSION ──
function drawDisintegration() {
  if (!ctx || !mouseActive) return

  // Spawn particles (ring buffer, no pops)
  const spawnRate = 2
  for (let s = 0; s < spawnRate; s++) {
    const i = disCount % DIS_MAX
    const angle = hash(i + t * 7) * 6.283
    const speed = 1 + hash(i + t * 11) * 4
    disX[i] = mouseX + (hash(i + t * 3) - 0.5) * 25
    disY[i] = mouseY + (hash(i + t * 5) - 0.5) * 25
    disVX[i] = Math.cos(angle) * speed + mouseVelX * 0.25
    disVY[i] = Math.sin(angle) * speed + mouseVelY * 0.25
    disLife[i] = 0.7 + hash(i + t * 13) * 0.3
    disHue[i] = (t * 2 + hash(i + t * 17) * 60) % 360
    disSize[i] = 1 + hash(i + t * 19) * 3
    disCount++
  }

  ctx.save()
  ctx.globalCompositeOperation = 'lighter'

  for (let i = 0; i < disCount; i++) {
    disX[i] += disVX[i]
    disY[i] += disVY[i]
    disVX[i] *= 0.97
    disVY[i] *= 0.97
    disLife[i] -= 0.012

    if (disLife[i] <= 0) continue

    const alpha = disLife[i] * 0.75
    const sz = disSize[i] * disLife[i]
    const hue = disHue[i]

    // Core
    ctx.beginPath()
    ctx.arc(disX[i], disY[i], sz, 0, 6.283)
    ctx.fillStyle = `hsla(${hue|0},70,60,${alpha})`
    ctx.fill()

    // Chromatic split
    ctx.beginPath()
    ctx.arc(disX[i] - 2, disY[i] - 1, sz * 0.6, 0, 6.283)
    ctx.fillStyle = `hsla(${((hue+40)%360)|0},60,55,${alpha * 0.25})`
    ctx.fill()

    ctx.beginPath()
    ctx.arc(disX[i] + 2, disY[i] + 1, sz * 0.6, 0, 6.283)
    ctx.fillStyle = `hsla(${((hue-40+360)%360)|0},60,55,${alpha * 0.25})`
    ctx.fill()
  }
  ctx.restore()
}

// ── 12. MASSIVE SCREEN TEAR / GLITCH BUG ──
function drawScreenTear() {
  if (!ctx) return

  // Random horizontal screen tears
  if (Math.random() < 0.08) {
    const tearY = Math.random() * h
    const tearH = 2 + Math.random() * 12
    const tearShift = (Math.random() - 0.5) * 40
    const tearHue = Math.random() * 360

    // Shifted strip
    ctx.save()
    ctx.beginPath()
    ctx.rect(0, tearY, w, tearH)
    ctx.clip()
    ctx.translate(tearShift, 0)

    // Re-draw a colored noise strip
    for (let i = 0; i < 20; i++) {
      const x = Math.random() * w
      const y = tearY + Math.random() * tearH
      ctx.fillStyle = `hsla(${(tearHue + i * 18)|0},60,55,0.15)`
      ctx.fillRect(x, y, 10 + Math.random() * 80, 1 + Math.random() * 2)
    }
    ctx.restore()
  }

  // Large color bleed glitch
  if (Math.random() < 0.02) {
    const gy = Math.random() * h
    const gh = 30 + Math.random() * 80
    const hue = Math.random() * 360
    ctx.fillStyle = `hsla(${hue|0},40,55,0.04)`
    ctx.fillRect(0, gy, w, gh)
  }

  // Vertical RGB split ghost
  if (Math.random() < 0.015) {
    const splitX = Math.random() * w
    const splitW = 60 + Math.random() * 150
    ctx.save()
    ctx.globalCompositeOperation = 'screen'
    ctx.fillStyle = 'rgba(255,0,0,0.03)'
    ctx.fillRect(splitX, 0, splitW, h)
    ctx.fillStyle = 'rgba(0,255,0,0.03)'
    ctx.fillRect(splitX + 4, 0, splitW, h)
    ctx.fillStyle = 'rgba(0,0,255,0.03)'
    ctx.fillRect(splitX + 8, 0, splitW, h)
    ctx.restore()
  }
}

// ── 13. VINTAGE CRT OVERLAY (cached gradients) ──
let sweepGradCache: CanvasGradient | null = null
let vigGradCache: CanvasGradient | null = null
let cachedVigR = 0

function drawVintageOverlay() {
  if (!ctx) return

  // Interlace lines
  ctx.fillStyle = 'rgba(0,0,0,0.025)'
  const scanOff = (t * 0.5) % 4
  for (let y = scanOff; y < h; y += 4) {
    ctx.fillRect(0, y, w, 1)
  }

  // Phosphor glow scanline sweep
  const sweepY = (t * 2) % (h + 200) - 100
  sweepGradCache = ctx.createLinearGradient(0, sweepY - 60, 0, sweepY + 60)
  sweepGradCache.addColorStop(0, 'rgba(0,255,65,0)')
  sweepGradCache.addColorStop(0.5, 'rgba(0,255,65,0.015)')
  sweepGradCache.addColorStop(1, 'rgba(0,255,65,0)')
  ctx.fillStyle = sweepGradCache
  ctx.fillRect(0, sweepY - 60, w, 120)

  // Vignette (only rebuild on resize)
  const vigR = Math.max(w, h) * 0.72
  if (vigR !== cachedVigR) {
    cachedVigR = vigR
    vigGradCache = ctx.createRadialGradient(w * 0.5, h * 0.5, vigR * 0.25, w * 0.5, h * 0.5, vigR)
    vigGradCache.addColorStop(0, 'rgba(0,0,0,0)')
    vigGradCache.addColorStop(0.65, 'rgba(0,0,0,0)')
    vigGradCache.addColorStop(1, 'rgba(0,0,0,0.45)')
  }
  if (vigGradCache) {
    ctx.fillStyle = vigGradCache
    ctx.fillRect(0, 0, w, h)
  }

  // Noise grain (subtle)
  if (Math.random() < 0.4) {
    const grainCount = 8
    for (let i = 0; i < grainCount; i++) {
      ctx.fillStyle = `rgba(255,255,255,${0.01 + Math.random() * 0.02})`
      ctx.fillRect(Math.random() * w, Math.random() * h, 1, 1)
    }
  }

  // CRT barrel distortion indicator (corner darkening)
  ctx.fillStyle = 'rgba(0,0,0,0.08)'
  const cornerSize = 80
  ctx.beginPath()
  ctx.moveTo(0, 0); ctx.lineTo(cornerSize, 0); ctx.lineTo(0, cornerSize); ctx.closePath(); ctx.fill()
  ctx.beginPath()
  ctx.moveTo(w, 0); ctx.lineTo(w - cornerSize, 0); ctx.lineTo(w, cornerSize); ctx.closePath(); ctx.fill()
  ctx.beginPath()
  ctx.moveTo(0, h); ctx.lineTo(cornerSize, h); ctx.lineTo(0, h - cornerSize); ctx.closePath(); ctx.fill()
  ctx.beginPath()
  ctx.moveTo(w, h); ctx.lineTo(w - cornerSize, h); ctx.lineTo(w, h - cornerSize); ctx.closePath(); ctx.fill()
}

// ── 14. CHECKERBOARD WARP (optimized: single batch) ──
function drawCheckerboard() {
  if (!ctx) return
  const size = 28
  const baseHue = (t * 0.2 + 80) % 360
  const offX = (t * 0.3) % (size * 2)
  const offY = (t * 0.2) % (size * 2)
  const invWarpR = 1 / WARP_R
  const warpForce = 22

  ctx.save()

  // Batch: build all rects, single color change per batch
  let lastHue = -1
  ctx.beginPath()
  for (let x = -size * 2 + offX; x < w + size * 2; x += size) {
    for (let y = -size * 2 + offY; y < h + size * 2; y += size) {
      const gx = (x / size) | 0
      const gy = (y / size) | 0
      if ((gx + gy) & 1) continue

      let drawX = x, drawY = y
      if (mouseActive) {
        const dx = x - mouseX
        const dy = y - mouseY
        const d2 = dx * dx + dy * dy
        if (d2 < WARP_R2 && d2 > 0.01) {
          const d = Math.sqrt(d2)
          const force = (1 - d * invWarpR) * warpForce
          const a = Math.atan2(dy, dx) + t * 0.015
          drawX += Math.cos(a) * force
          drawY += Math.sin(a) * force
        }
      }

      const hue = ((baseHue + (gx + gy) * 3 + t * 0.08) % 360 + 360) % 360
      const hueKey = hue | 0
      if (hueKey !== lastHue) {
        ctx.fillStyle = `hsla(${hueKey},50,50,0.015)`
        lastHue = hueKey
      }
      ctx.rect(drawX, drawY, size, size)
    }
  }
  ctx.fill()
  ctx.restore()
}

// ══════════════════════════════════════════════════════════════
//  MAIN LOOP
// ══════════════════════════════════════════════════════════════

function draw() {
  if (!ctx) return
  t++

  // Smooth mouse position
  mouseSmoothX += (mouseX - mouseSmoothX) * 0.12
  mouseSmoothY += (mouseY - mouseSmoothY) * 0.12

  ctx.clearRect(0, 0, w, h)

  // Layer 1: Alive noise field (deepest, dense, alive)
  drawAliveNoiseField()

  // Layer 2: Checkerboard warp
  drawCheckerboard()

  // Layer 3: Flower of Life
  drawFlowerOfLife()

  // Layer 4: Op-Art Rings + chromatic aberration
  drawOpArtRings()

  // Layer 5: Network Grid
  drawNetworkGrid()

  // Layer 6: Vortex spiral
  drawVortex()

  // Layer 7: Lissajous curves
  drawLissajous()

  // Layer 8: Sacred Mandala
  drawMandala()

  // Layer 9: Spiral particles
  drawSpiralParticles()

  // Layer 10: Pulsating core
  drawCore()

  // Layer 11: Warp field (mouse)
  drawWarpField()

  // Layer 12: Disintegration (mouse)
  drawDisintegration()

  // Layer 13: Screen tear / glitch bugs
  drawScreenTear()

  // Layer 14: Vintage CRT overlay (top)
  drawVintageOverlay()

  animId = requestAnimationFrame(draw)
}

function start() {
  if (animId) return
  resize()
  initFlower()
  initNetwork()
  initSpirals()
  draw()
}

function stop() {
  if (animId) {
    cancelAnimationFrame(animId)
    animId = 0
  }
}

onMounted(() => {
  const c = canvasRef.value
  if (c) ctx = c.getContext('2d')
  if (props.enabled) start()
  window.addEventListener('resize', resize)
  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseleave', onMouseLeaveGlobal)
})

onUnmounted(() => {
  stop()
  window.removeEventListener('resize', resize)
  window.removeEventListener('mousemove', onMouseMove)
  window.removeEventListener('mouseleave', onMouseLeaveGlobal)
})

watch(() => props.enabled, (v) => {
  v ? start() : stop()
})
</script>

<style scoped>
.canvas-engine {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  pointer-events: none;
  mix-blend-mode: screen;
}
</style>
