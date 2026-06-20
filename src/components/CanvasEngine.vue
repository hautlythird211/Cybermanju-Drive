<template>
  <canvas ref="canvasRef" class="canvas-engine" />
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import type { ArtMakerSettings } from '@/configs/artMaker'

const props = withDefaults(defineProps<{
  enabled?: boolean
  mode?: number
  settings?: ArtMakerSettings
}>(), {
  enabled: true,
  mode: 0,
})

const canvasRef = ref<HTMLCanvasElement | null>(null)
let animId = 0
let ctx: CanvasRenderingContext2D | null = null
let w = 0, h = 0, dpr = 1
let t = 0
let speedMul = 1
let hueShift = 0
let globalSat = 50
let globalBright = 50
function gs(base: number): number { return Math.min(100, Math.max(0, Math.round(base + globalSat - 50))) }
function gb(base: number): number { return Math.min(100, Math.max(0, Math.round(base + globalBright - 50))) }
const BG_COLOR = 'rgb(6, 6, 10)'

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

function hash(n: number): number {
  n = (n << 13) ^ n
  return ((n * (n * n * 15731 + 789221) + 1376312589) & 0x7fffffff) / 0x7fffffff
}
function hash2(x: number, y: number): number {
  return hash(x * 374761393 + y * 668265263)
}

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
let spiralTrail: Float64Array = new Float64Array(0)
let disX: Float64Array = new Float64Array(0)
let disY: Float64Array = new Float64Array(0)
let disVX: Float64Array = new Float64Array(0)
let disVY: Float64Array = new Float64Array(0)
let disLife: Float64Array = new Float64Array(0)
let disHue: Float64Array = new Float64Array(0)
let disSize: Float64Array = new Float64Array(0)
let disCount = 0
const DIS_MAX = 400

// Neural network data
let neuralPacketX: Float64Array = new Float64Array(0)
let neuralPacketY: Float64Array = new Float64Array(0)
let neuralPacketTarget: Float64Array = new Float64Array(0)
let neuralPacketLife: Float64Array = new Float64Array(0)
let neuralPacketSpeed: Float64Array = new Float64Array(0)
let neuralPacketHue: Float64Array = new Float64Array(0)
const NEURAL_PACKET_COUNT = 60

// Matrix rain column data
let rainHeads: Float64Array = new Float64Array(0)
let rainTrails: Float64Array = new Float64Array(0)
let rainHue: Float64Array = new Float64Array(0)
let rainSpeed: Float64Array = new Float64Array(0)
const RAIN_CHARS = 'アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン0123456789ABCDEF@#$%&*{}[]|;:<>?/~'

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
  initFlower()
  initNetwork()
  initNeuralPackets()
  initSpirals()
  initRain()
  initStars()
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
  const count = 220
  networkX = new Float64Array(count)
  networkY = new Float64Array(count)
  networkVX = new Float64Array(count)
  networkVY = new Float64Array(count)
  for (let i = 0; i < count; i++) {
    networkX[i] = hash(i * 3) * w
    networkY[i] = hash(i * 3 + 1) * h
    networkVX[i] = (hash(i * 3 + 2) - 0.5) * 0.8
    networkVY[i] = (hash(i * 3 + 3) - 0.5) * 0.8
  }
}

function initNeuralPackets() {
  neuralPacketX = new Float64Array(NEURAL_PACKET_COUNT)
  neuralPacketY = new Float64Array(NEURAL_PACKET_COUNT)
  neuralPacketTarget = new Float64Array(NEURAL_PACKET_COUNT)
  neuralPacketLife = new Float64Array(NEURAL_PACKET_COUNT)
  neuralPacketSpeed = new Float64Array(NEURAL_PACKET_COUNT)
  neuralPacketHue = new Float64Array(NEURAL_PACKET_COUNT)
  for (let i = 0; i < NEURAL_PACKET_COUNT; i++) {
    spawnNeuralPacket(i)
  }
}

function spawnNeuralPacket(i: number, depth = 0) {
  if (depth > 10) return
  const source = (hash(i * 37 + t * 3) * networkX.length) | 0
  const target = (hash(i * 41 + t * 7) * networkX.length) | 0
  if (source === target) return spawnNeuralPacket((i + 1) % NEURAL_PACKET_COUNT, depth + 1)
  neuralPacketX[i] = networkX[source]
  neuralPacketY[i] = networkY[source]
  neuralPacketTarget[i] = target
  neuralPacketLife[i] = 0
  neuralPacketSpeed[i] = 0.008 + hash(i * 43) * 0.015
  neuralPacketHue[i] = hash(i * 47) * 360
}

function initSpirals() {
  const count = 150
  spiralAngle = new Float64Array(count)
  spiralRadius = new Float64Array(count)
  spiralSpeed = new Float64Array(count)
  spiralHue = new Float64Array(count)
  spiralLife = new Float64Array(count)
  spiralTrail = new Float64Array(count)
  const maxR = Math.min(w, h) * 0.45
  for (let i = 0; i < count; i++) {
    spiralAngle[i] = hash(i * 7) * 6.283
    spiralRadius[i] = 20 + hash(i * 7 + 1) * maxR
    spiralSpeed[i] = 0.003 + hash(i * 7 + 2) * 0.01
    spiralHue[i] = hash(i * 7 + 3) * 360
    spiralLife[i] = 0.5 + hash(i * 7 + 4) * 0.5
    spiralTrail[i] = 0
  }
}

// ══════════════════════════════════════════════════════════════
//  DRAW FUNCTIONS
// ══════════════════════════════════════════════════════════════

// ── 1. CLOUD CHAMBER (was Noise Field) ──
function drawCloudChamber() {
  if (!ctx) return
  const baseHue = (t * 0.15 + hueShift) % 360
  const time = t * 0.006
  const step = 6

  for (let y = 0; y < h; y += step) {
    for (let x = 0; x < w; x += step) {
      const nx = x * 0.015, ny = y * 0.015
      const n = fbm(nx + time * 0.5, ny + time * 0.3, 3)
      const n2 = noise2d(x * 0.02 + time * 0.5, y * 0.02 - time * 0.3)

      let mouseDist = 0
      if (mouseActive) {
        const dx = x - mouseX, dy = y - mouseY
        mouseDist = Math.max(0, 1 - Math.sqrt(dx * dx + dy * dy) / WARP_R)
      }

      const density = (n * 0.6 + n2 * 0.4)
      const alpha = 0.15 + density * 0.5 + mouseDist * 0.25
      const hue = (baseHue + density * 80 + n2 * 40) % 360
      const light = 35 + density * 25 + mouseDist * 10

      ctx.fillStyle = `hsla(${hue|0},${gs(50)},${light|0},${Math.min(alpha, 0.85)})`
      ctx.fillRect(x, y, step + 1, step + 1)
    }
  }
}

// ── 2. MOIRÉ GRID (was Checkerboard) ──
function drawMoireGrid() {
  if (!ctx) return
  const baseHue = (t * 0.12 + hueShift + 80) % 360
  const time = t * 0.004
  const spacing = 24
  const layers = 4

  for (let l = 0; l < layers; l++) {
    const angle = 0.15 + (l / layers) * Math.PI / 3 + time * 0.05 * (l + 1) * 0.3
    const layerSpacing = spacing * (1 + l * 0.25)
    const cosA = fastCos(angle)
    const sinA = fastSin(angle)
    const offset = (t * 0.01 * (1 + l * 0.2)) % layerSpacing
    const hueOff = l * 30

    ctx.beginPath()
    for (let line = -50; line < Math.max(w, h) * 2 + 50; line++) {
      const proj = line * layerSpacing + offset
      const lx = cosA * proj
      const ly = sinA * proj
      if (lx < -layerSpacing || lx > w + layerSpacing || ly < -layerSpacing || ly > h + layerSpacing) continue

      const perpX = -sinA * 9999
      const perpY = cosA * 9999
      ctx.moveTo(lx - perpX, ly - perpY)
      ctx.lineTo(lx + perpX, ly + perpY)
    }
    const lHue = (baseHue + hueOff) % 360
    ctx.strokeStyle = `hsla(${lHue|0},${gs(45)},${gb(50)},0.15)`
    ctx.lineWidth = 0.6
    ctx.stroke()
  }

  // Moiré interference spots
  for (let i = 0; i < 20; i++) {
    const sx = (fastSin(t * 0.005 + i * 1.7) * 0.5 + 0.5) * w
    const sy = (fastCos(t * 0.007 + i * 2.3) * 0.5 + 0.5) * h
    const sr = 30 + fastSin(t * 0.01 + i * 0.9) * 15 + 15
    const sh = (baseHue + i * 18 + t * 0.3) % 360
    const grad = ctx.createRadialGradient(sx, sy, 0, sx, sy, sr)
    grad.addColorStop(0, `hsla(${sh|0},${gs(60)},${gb(55)},0.12)`)
    grad.addColorStop(1, 'rgba(0,0,0,0)')
    ctx.fillStyle = grad
    ctx.fillRect(sx - sr, sy - sr, sr * 2, sr * 2)
  }
}

// ── 3. FLOWER OF LIFE ──
function drawFlowerOfLife() {
  if (!ctx) return
  const baseHue = (t * 0.25 + 120 + hueShift) % 360
  const time = t * 0.012

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
        ctx.strokeStyle = `hsla(${hue|0},${gs(55)},${gb(45)},${0.25 + intensity * 0.35})`
        ctx.lineWidth = 0.4 + intensity * 0.6
        ctx.stroke()
        continue
      }
    }
    const r = cr * (0.3 + pulse * 0.3)
    ctx.beginPath()
    ctx.arc(cx, cy, r, 0, 6.283)
    ctx.strokeStyle = `hsla(${hue|0},${gs(50)},${gb(40)},0.2)`
    ctx.lineWidth = 0.3
    ctx.stroke()
  }
}

// ── 4. OP-ART RINGS ──
function drawOpArtRings() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const maxR = Math.sqrt(w * w + h * h) * 0.5 + 50
  const baseHue = (t * 0.4 + hueShift) % 360
  const phase = t * 0.015
  const spacing = 30 + fastSin(phase) * 8
  const offset = (t * 0.5) % spacing

  for (let ch = 0; ch < 3; ch++) {
    const hueOff = (ch - 1) * 8
    const xOff = (ch - 1) * 1.2
    const yOff = (ch - 1) * 0.6

    ctx.beginPath()
    for (let r = offset; r < maxR; r += spacing) {
      const hue = ((baseHue + hueOff + r * 0.3) % 360 + 360) % 360
      const alpha = 0.1 + fastSin(r * 0.05 + phase) * 0.06
      ctx.strokeStyle = `hsla(${hue|0},${gs(55)},${gb(45)},${alpha})`
      ctx.lineWidth = 0.8 + fastSin(r * 0.1 + phase) * 0.4
      ctx.beginPath()
      ctx.arc(xOff, yOff, r, 0, 6.283)
      ctx.stroke()
    }
  }

  if (mouseActive) {
    const mouseDistCenter = Math.sqrt((cx - mouseX) ** 2 + (cy - mouseY) ** 2)
    const mouseAng = Math.atan2(mouseY - cy, mouseX - cx)
    if (mouseDistCenter < WARP_R) {
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
            ctx.fillStyle = `hsla(${hue|0},${gs(65)},${gb(55)},${0.25 + fi * 0.25})`
            ctx.fill()
          }
        }
      }
    }
  }
}

// ── 5. NEURAL NETWORK (was Network Grid - 200+ nodes, data packets) ──
function drawNeuralNetwork() {
  if (!ctx) return
  const baseHue = (t * 0.2 + 300 + hueShift) % 360
  const maxDist = 160
  const maxDist2 = maxDist * maxDist
  const nodeCount = networkX.length

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

  ctx.lineWidth = 0.3

  for (let i = 0; i < nodeCount; i++) {
    for (let j = i + 1; j < nodeCount; j++) {
      const dx = networkX[i] - networkX[j]
      const dy = networkY[i] - networkY[j]
      const d2 = dx * dx + dy * dy
      if (d2 < maxDist2) {
        const d = Math.sqrt(d2)
        const alpha = (1 - d / maxDist) * 0.12
        const hue = ((baseHue + (networkX[i] + networkX[j]) * 0.1) % 360 + 360) % 360
        ctx.beginPath()
        ctx.moveTo(networkX[i], networkY[i])
        ctx.lineTo(networkX[j], networkY[j])
        ctx.strokeStyle = `hsla(${hue|0},${gs(50)},${gb(45)},${alpha})`
        ctx.stroke()
      }
    }
  }

  // Node dots
  ctx.beginPath()
  for (let i = 0; i < nodeCount; i++) {
    let highlight = 0
    if (mouseActive) {
      const dx = networkX[i] - mouseX
      const dy = networkY[i] - mouseY
      const d2 = dx * dx + dy * dy
      if (d2 < WARP_R2) highlight = 1 - Math.sqrt(d2) / WARP_R
    }
    const r = 1.2 + highlight * 2.5
    ctx.moveTo(networkX[i] + r, networkY[i])
    ctx.arc(networkX[i], networkY[i], r, 0, 6.283)
  }
  ctx.fillStyle = `hsla(${baseHue|0},${gs(55)},${gb(50)},0.35)`
  ctx.fill()

  // Data packets traveling between nodes
  for (let i = 0; i < NEURAL_PACKET_COUNT; i++) {
    const targetIdx = neuralPacketTarget[i] | 0
    if (targetIdx >= nodeCount) { spawnNeuralPacket(i); continue }

    const tx = networkX[targetIdx]
    const ty = networkY[targetIdx]
    const dx = tx - neuralPacketX[i]
    const dy = ty - neuralPacketY[i]
    const d = Math.sqrt(dx * dx + dy * dy)

    neuralPacketLife[i] += neuralPacketSpeed[i]
    if (neuralPacketLife[i] >= 1 || d < 5) {
      spawnNeuralPacket(i)
      continue
    }

    const progress = neuralPacketLife[i]
    const px = neuralPacketX[i] + dx * progress
    const py = neuralPacketY[i] + dy * progress
    const ph = (neuralPacketHue[i] + t * 0.5) % 360

    ctx.beginPath()
    ctx.arc(px, py, 2.5, 0, 6.283)
    ctx.fillStyle = `hsla(${ph|0},${gs(70)},${gb(60)},0.7)`
    ctx.fill()

    // Glow trail
    ctx.beginPath()
    ctx.arc(px, py, 5, 0, 6.283)
    ctx.fillStyle = `hsla(${ph|0},${gs(60)},${gb(55)},0.15)`
    ctx.fill()
  }
}

// ── 6. VORTEX ──
function drawVortex() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const baseHue = (t * 0.7 + 200 + hueShift) % 360
  const maxDim = Math.min(w, h) * 0.42

  let warpCx = cx, warpCy = cy
  if (mouseActive) {
    warpCx = cx + (mouseX - cx) * 0.06
    warpCy = cy + (mouseY - cy) * 0.06
  }

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
    ctx.strokeStyle = `hsla(${armHue|0},${gs(60)},${gb(45)},0.2)`
    ctx.stroke()
  }
}

// ── 7. LISSAJOUS ──
function drawLissajous() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const r = Math.min(w, h) * 0.26
  const baseHue = (t * 0.6 + 180 + hueShift) % 360
  const phase = t * 0.02

  const curves = [
    { a: 3, b: 4, s: 1, ho: 0 },
    { a: 5, b: 6, s: 0.6, ho: 60 },
    { a: 7, b: 8, s: 0.35, ho: 120 },
  ]

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
    ctx.strokeStyle = `hsla(${ch|0},${gs(50)},${gb(45)},0.18)`
    ctx.lineWidth = 0.5
    ctx.stroke()
  }
}

// ── 8. MANDALA ──
function drawMandala() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const baseHue = (t * 0.5 + hueShift) % 360
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

  for (let layer = 0; layer < 5; layer++) {
    const layerHue = ((baseHue + layer * 40) % 360 + 360) % 360
    const layerR = 50 + layer * 65
    const pts = 14 + layer * 5
    const lr = rot * (layer & 1 ? -1 : 1) * (1 + layer * 0.25)

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
    ctx.strokeStyle = `hsla(${layerHue|0},${gs(60)},${gb(50)},${0.25 + layer * 0.04})`
    ctx.lineWidth = 0.7
    ctx.stroke()

    ctx.beginPath()
    for (let i = 0; i < pts; i++) {
      const a1 = (i / pts) * 6.283 + lr
      const a3 = ((i + 2) / pts) * 6.283 + lr
      const r1 = layerR + fastSin(a1 * pts + t * 0.018) * 12
      const r2 = layerR * 0.45 + fastSin(a3 * pts + t * 0.018) * 8
      ctx.moveTo(cx + warpX + fastCos(a1) * r1, cy + warpY + fastSin(a1) * r1)
      ctx.lineTo(cx + warpX + fastCos(a3) * r2, cy + warpY + fastSin(a3) * r2)
    }
    ctx.strokeStyle = `hsla(${((layerHue + 30) % 360)|0},55,45,0.1)`
    ctx.lineWidth = 0.4
    ctx.stroke()
  }
}

// ── 9. SPIRAL PARTICLES (trails, radial drift, size variation) ──
function drawSpiralParticles() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const count = spiralAngle.length
  const maxR = Math.min(w, h) * 0.45

  for (let i = 0; i < count; i++) {
    spiralAngle[i] += spiralSpeed[i]
    spiralLife[i] -= 0.001
    spiralTrail[i] += 0.02
    if (spiralTrail[i] > 1) spiralTrail[i] = 0

    // Radial drift
    spiralRadius[i] += (hash(i * 17 + t) - 0.5) * 0.3
    if (spiralRadius[i] < 10) spiralRadius[i] = 10
    if (spiralRadius[i] > maxR) spiralRadius[i] = maxR

    if (spiralLife[i] <= 0) {
      spiralLife[i] = 1
      spiralRadius[i] = 20 + hash(i * 11 + t) * maxR
      spiralHue[i] = hash(i * 13 + t) * 360
      spiralSpeed[i] = 0.003 + hash(i * 7 + 2) * 0.01
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

    const alpha = spiralLife[i] * 0.55
    const size = 1 + (spiralHue[i] % 4) + fastSin(t * 0.02 + i) * 0.5
    const hue = (spiralHue[i] + t * 0.3) % 360

    // Trail - draw behind particle
    const trailAngle = spiralAngle[i] - spiralSpeed[i] * spiralTrail[i] * 5
    const trailR = spiralRadius[i] * (1 - spiralTrail[i] * 0.08)
    const tx = cx + fastCos(trailAngle) * trailR
    const ty = cy + fastSin(trailAngle) * trailR
    const trailAlpha = alpha * (1 - spiralTrail[i]) * 0.3
    if (trailAlpha > 0.02) {
      ctx.beginPath()
      ctx.arc(tx, ty, size * 0.8, 0, 6.283)
      ctx.fillStyle = `hsla(${hue|0},${gs(50)},${gb(45)},${trailAlpha})`
      ctx.fill()
    }

    // Glow
    ctx.beginPath()
    ctx.arc(x, y, size * 3, 0, 6.283)
    ctx.fillStyle = `hsla(${hue|0},${gs(50)},${gb(45)},${alpha * 0.15})`
    ctx.fill()

    // Core
    ctx.beginPath()
    ctx.arc(x, y, size, 0, 6.283)
    ctx.fillStyle = `hsla(${hue|0},${gs(60)},${gb(55)},${alpha + 0.15})`
    ctx.fill()
  }
}

// ── 10. PULSATING CORE (larger, with ray beams) ──
function drawCore() {
  if (!ctx) return
  const cx = w * 0.5
  const cy = h * 0.5
  const baseHue = (t * 0.8 + hueShift) % 360
  const pulse = fastSin(t * 0.018) * 0.5 + 0.5
  const coreR = 40 + pulse * 30

  // Ray beams
  for (let ray = 0; ray < 12; ray++) {
    const rayAngle = (ray / 12) * 6.283 + t * 0.003
    const rayLen = coreR * (3 + fastSin(t * 0.015 + ray * 1.2) * 1.5)
    const rayEndX = cx + fastCos(rayAngle) * rayLen
    const rayEndY = cy + fastSin(rayAngle) * rayLen
    const rayHue = (baseHue + ray * 15 + 30) % 360

    const grad = ctx.createRadialGradient(cx, cy, 0, cx, cy, rayLen)
    grad.addColorStop(0, `hsla(${rayHue|0},${gs(65)},${gb(55)},0.08)`)
    grad.addColorStop(0.3, `hsla(${rayHue|0},${gs(60)},${gb(50)},0.04)`)
    grad.addColorStop(1, 'rgba(0,0,0,0)')
    ctx.fillStyle = grad

    ctx.beginPath()
    ctx.moveTo(cx, cy)
    ctx.lineTo(
      cx + fastCos(rayAngle - 0.08) * rayLen * 0.5,
      cy + fastSin(rayAngle - 0.08) * rayLen * 0.5
    )
    ctx.lineTo(rayEndX, rayEndY)
    ctx.lineTo(
      cx + fastCos(rayAngle + 0.08) * rayLen * 0.5,
      cy + fastSin(rayAngle + 0.08) * rayLen * 0.5
    )
    ctx.closePath()
    ctx.fill()
  }

  // Outer glow
  const grad = ctx.createRadialGradient(cx, cy, 0, cx, cy, coreR * 5)
  grad.addColorStop(0, `hsla(${baseHue|0},${gs(65)},${gb(55)},${0.25 + pulse * 0.15})`)
  grad.addColorStop(0.3, `hsla(${((baseHue+30)%360)|0},55,45,0.1)`)
  grad.addColorStop(1, 'rgba(0,0,0,0)')
  ctx.fillStyle = grad
  ctx.fillRect(cx - coreR * 5, cy - coreR * 5, coreR * 10, coreR * 10)

  // Hexagonal shape
  ctx.beginPath()
  for (let i = 0; i <= 6; i++) {
    const angle = (i / 6) * 6.283 + t * 0.008
    const r = coreR + fastSin(angle * 3 + t * 0.025) * 6
    const x = cx + fastCos(angle) * r
    const y = cy + fastSin(angle) * r
    if (i === 0) ctx.moveTo(x, y)
    else ctx.lineTo(x, y)
  }
  ctx.closePath()
  ctx.strokeStyle = `hsla(${baseHue|0},${gs(55)},${gb(50)},0.3)`
  ctx.lineWidth = 1.2
  ctx.stroke()

  // Inner core
  ctx.beginPath()
  ctx.arc(cx, cy, coreR * 0.3, 0, 6.283)
  ctx.fillStyle = `hsla(${baseHue|0},${gs(70)},${gb(60)},${0.35 + pulse * 0.2})`
  ctx.fill()
}

// ── 11. WARP FIELD (mouse aura) ──
function drawWarpField() {
  if (!ctx || !mouseActive) return
  const baseHue = (t * 1.5 + 60 + hueShift) % 360

  for (let i = 0; i < 10; i++) {
    const r = 18 + i * 28 + fastSin(t * 0.04 + i) * 8
    const alpha = (1 - i / 10) * 0.2
    ctx.beginPath()
    ctx.arc(mouseSmoothX, mouseSmoothY, r, 0, 6.283)
    ctx.strokeStyle = `hsla(${((baseHue + i * 18) % 360)|0},55,50,${alpha})`
    ctx.lineWidth = 0.5
    ctx.stroke()
  }

  ctx.beginPath()
  for (let i = 0; i < 28; i++) {
    const angle = (i / 28) * 6.283 + t * 0.008
    const innerR = 14 + fastSin(t * 0.07 + i * 0.4) * 4
    const outerR = DISINTEGRATE_R * 0.75 + fastSin(t * 0.025 + i) * 18
    ctx.moveTo(mouseSmoothX + fastCos(angle) * innerR, mouseSmoothY + fastSin(angle) * innerR)
    ctx.lineTo(mouseSmoothX + fastCos(angle + 0.04) * outerR, mouseSmoothY + fastSin(angle + 0.04) * outerR)
  }
  ctx.strokeStyle = `hsla(${baseHue|0},${gs(45)},${gb(45)},0.08)`
  ctx.lineWidth = 0.3
  ctx.stroke()
}

// ── 12. DISINTEGRATION ──
function drawDisintegration() {
  if (!ctx || !mouseActive) return

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

  for (let i = 0; i < disCount; i++) {
    disX[i] += disVX[i]
    disY[i] += disVY[i]
    disVX[i] *= 0.97
    disVY[i] *= 0.97
    disLife[i] -= 0.012

    if (disLife[i] <= 0) continue

    const alpha = disLife[i] * 0.55
    const sz = disSize[i] * disLife[i]
    const hue = disHue[i]

    ctx.beginPath()
    ctx.arc(disX[i], disY[i], sz, 0, 6.283)
    ctx.fillStyle = `hsla(${hue|0},${gs(65)},${gb(55)},${alpha + 0.15})`
    ctx.fill()

    ctx.beginPath()
    ctx.arc(disX[i] - 2, disY[i] - 1, sz * 0.6, 0, 6.283)
    ctx.fillStyle = `hsla(${((hue+40)%360)|0},55,50,${alpha * 0.35})`
    ctx.fill()

    ctx.beginPath()
    ctx.arc(disX[i] + 2, disY[i] + 1, sz * 0.6, 0, 6.283)
    ctx.fillStyle = `hsla(${((hue-40+360)%360)|0},55,50,${alpha * 0.35})`
    ctx.fill()
  }
}

// ── 13. SCREEN TEAR ──
function drawScreenTear() {
  if (!ctx) return

  if (Math.random() < 0.08) {
    const tearY = Math.random() * h
    const tearH = 2 + Math.random() * 12
    const tearShift = (Math.random() - 0.5) * 40
    const tearHue = Math.random() * 360

    ctx.save()
    ctx.beginPath()
    ctx.rect(0, tearY, w, tearH)
    ctx.clip()
    ctx.translate(tearShift, 0)

    for (let i = 0; i < 20; i++) {
      const x = Math.random() * w
      const y = tearY + Math.random() * tearH
      ctx.fillStyle = `hsla(${(tearHue + i * 18)|0},55,50,0.25)`
      ctx.fillRect(x, y, 10 + Math.random() * 80, 1 + Math.random() * 2)
    }
    ctx.restore()
  }

  if (Math.random() < 0.02) {
    const gy = Math.random() * h
    const gh = 30 + Math.random() * 80
    const hue = Math.random() * 360
    ctx.fillStyle = `hsla(${hue|0},${gs(35)},${gb(50)},0.08)`
    ctx.fillRect(0, gy, w, gh)
  }

  if (Math.random() < 0.015) {
    const splitX = Math.random() * w
    const splitW = 60 + Math.random() * 150
    ctx.fillStyle = `hsla(0,${gs(60)},${gb(55)},0.06)`
    ctx.fillRect(splitX, 0, splitW, h)
    ctx.fillStyle = `hsla(120,${gs(60)},${gb(55)},0.06)`
    ctx.fillRect(splitX + 4, 0, splitW, h)
    ctx.fillStyle = `hsla(240,${gs(60)},${gb(55)},0.06)`
    ctx.fillRect(splitX + 8, 0, splitW, h)
  }
}

// ── 14. CRT OVERLAY ──
let sweepGradCache: CanvasGradient | null = null
let vigGradCache: CanvasGradient | null = null
let cachedVigR = 0

function drawVintageOverlay() {
  if (!ctx) return

  ctx.fillStyle = 'rgba(0,0,0,0.04)'
  const scanOff = (t * 0.5) % 4
  for (let y = scanOff; y < h; y += 4) {
    ctx.fillRect(0, y, w, 1)
  }

  const sweepY = (t * 2) % (h + 200) - 100
  sweepGradCache = ctx.createLinearGradient(0, sweepY - 60, 0, sweepY + 60)
  sweepGradCache.addColorStop(0, 'rgba(0,255,65,0)')
  sweepGradCache.addColorStop(0.5, 'rgba(0,255,65,0.025)')
  sweepGradCache.addColorStop(1, 'rgba(0,255,65,0)')
  ctx.fillStyle = sweepGradCache
  ctx.fillRect(0, sweepY - 60, w, 120)

  const vigR = Math.max(w, h) * 0.72
  if (vigR !== cachedVigR) {
    cachedVigR = vigR
    vigGradCache = ctx.createRadialGradient(w * 0.5, h * 0.5, vigR * 0.25, w * 0.5, h * 0.5, vigR)
    vigGradCache.addColorStop(0, 'rgba(0,0,0,0)')
    vigGradCache.addColorStop(0.65, 'rgba(0,0,0,0)')
    vigGradCache.addColorStop(1, 'rgba(0,0,0,0.5)')
  }
  if (vigGradCache) {
    ctx.fillStyle = vigGradCache
    ctx.fillRect(0, 0, w, h)
  }

  if (Math.random() < 0.4) {
    for (let i = 0; i < 8; i++) {
      ctx.fillStyle = `rgba(255,255,255,${0.015 + Math.random() * 0.03})`
      ctx.fillRect(Math.random() * w, Math.random() * h, 1, 1)
    }
  }

  ctx.fillStyle = 'rgba(0,0,0,0.12)'
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

// ── 15. MATRIX RAIN (trailing columns with bright head + fading tail) ──
function initRain() {
  const colWidth = 16
  const colCount = Math.ceil(w / colWidth)
  rainHeads = new Float64Array(colCount)
  rainTrails = new Float64Array(colCount)
  rainHue = new Float64Array(colCount)
  rainSpeed = new Float64Array(colCount)
  for (let i = 0; i < colCount; i++) {
    const start = Math.random() * h * -1
    rainHeads[i] = start
    rainTrails[i] = 8 + Math.random() * 20
    rainHue[i] = hash(i * 7) * 360
    rainSpeed[i] = 0.6 + hash(i * 11) * 1.2
  }
}

function drawMatrixRain() {
  if (!ctx) return
  const hs = hueShift
  const colWidth = 16
  const fontSize = 14
  ctx.font = `${fontSize}px "Courier New", monospace`
  const colCount = rainHeads.length

  for (let i = 0; i < colCount; i++) {
    const x = i * colWidth
    const headY = rainHeads[i] * 18
    rainHue[i] = (rainHue[i] + 0.3 + hash(i + t) * 0.2) % 360
    const hue = (rainHue[i] + hs) % 360
    const ch = RAIN_CHARS[(Math.random() * RAIN_CHARS.length) | 0]

    // Bright head character
    ctx.shadowBlur = 10
    ctx.shadowColor = `hsla(${hue|0},${gs(90)},${gb(60)},0.5)`
    ctx.fillStyle = `hsla(${hue|0},${gs(80)},${gb(80)},0.85)`
    ctx.fillText(ch, x, headY)

    // Fading trail characters
    ctx.shadowBlur = 0
    const trailLen = rainTrails[i] | 0
    for (let t2 = 1; t2 <= trailLen; t2++) {
      const trailY = headY - t2 * 18
      if (trailY < -18) break
      const trailAlpha = 0.4 * (1 - t2 / trailLen) * (1 - t2 / trailLen)
      const trailChar = RAIN_CHARS[(Math.random() * RAIN_CHARS.length) | 0]
      ctx.fillStyle = `hsla(${hue|0},${gs(70)},${gb(55 - t2 * 2|0)},${Math.min(trailAlpha, 0.4)})`
      ctx.fillText(trailChar, x, trailY)
    }

    rainHeads[i] += rainSpeed[i] * speedMul
    if (headY > h + trailLen * 18 && Math.random() > 0.975) {
      rainHeads[i] = 0
      rainTrails[i] = 8 + Math.random() * 20
      rainHue[i] = hash(i * 13 + t) * 360
      rainSpeed[i] = 0.6 + hash(i * 17 + t) * 1.2
    }
  }
}

// ── 16. FRACTAL TREE (spread decreases with depth, leaf endpoints, higher alpha) ──
function drawFractalTree() {
  if (!ctx) return
  const cx = mouseActive ? mouseX : w * 0.5
  const cy = h - 30
  const baseHue = (t * 0.3 + hueShift) % 360
  const time = t * 0.015 * speedMul

  ctx.lineCap = 'round'

  const trunkLen = 70
  const trunkAngle = -Math.PI / 2
  const hue = (baseHue) % 360
  ctx.strokeStyle = `hsla(${hue|0},${gs(55)},${gb(45)},0.5)`
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(cx, cy)
  const sway = fastSin(time) * 0.12
  const trunkEndX = cx + Math.cos(trunkAngle + sway * 0.2) * trunkLen
  const trunkEndY = cy + Math.sin(trunkAngle + sway * 0.2) * trunkLen
  ctx.lineTo(trunkEndX, trunkEndY)
  ctx.stroke()

  const branches: Array<{ x: number; y: number; angle: number; len: number; depth: number }> = [
    { x: trunkEndX, y: trunkEndY, angle: trunkAngle + sway * 0.2, len: 50, depth: 1 },
  ]

  for (let i = 0; i < branches.length && i < 300; i++) {
    const b = branches[i]
    if (b.depth > 8 || b.len < 2) continue

    // Spread decreases with depth
    const spread = Math.max(0.15, 0.5 - b.depth * 0.04)
    const swayB = fastSin(time + b.depth * 0.7) * 0.1

    for (let dir = -1; dir <= 1; dir += 2) {
      const a = b.angle + dir * spread + swayB
      const l = b.len * (0.62 + fastSin(time + b.depth * 2 + dir) * 0.04)
      const ex = b.x + Math.cos(a) * l
      const ey = b.y + Math.sin(a) * l

      const bh = ((baseHue + b.depth * 22 + (dir + 1) * 15) % 360 + 360) % 360

      // Higher alpha for branches
      const branchAlpha = 0.5 - b.depth * 0.04
      ctx.strokeStyle = `hsla(${bh|0},${gs(50)},${gb(45)},${Math.max(branchAlpha, 0.2)})`
      ctx.lineWidth = Math.max(0.3, 1.5 - b.depth * 0.15)
      ctx.beginPath()
      ctx.moveTo(b.x, b.y)
      ctx.lineTo(ex, ey)
      ctx.stroke()

      // Leaf endpoints
      if (b.depth >= 5 && l > 3) {
        ctx.beginPath()
        ctx.arc(ex, ey, 1.5 + (8 - b.depth) * 0.3, 0, 6.283)
        const leafHue = (bh + 60) % 360
        ctx.fillStyle = `hsla(${leafHue|0},${gs(60)},${gb(55)},${0.4 + (8 - b.depth) * 0.05})`
        ctx.fill()
      }

      branches.push({ x: ex, y: ey, angle: a, len: l, depth: b.depth + 1 })
    }
  }
}

// ── 17. PLASMA WAVE ──
function drawPlasmaWave() {
  if (!ctx) return
  const hs = hueShift
  const step = 8
  const cols = Math.ceil(w / step)
  const rows = Math.ceil(h / step)
  const time = t * 0.015 * speedMul

  for (let y = 0; y < rows; y++) {
    for (let x = 0; x < cols; x++) {
      const px = x * step
      const py = y * step
      let dist = 0
      if (mouseActive) {
        const dx = px - mouseX
        const dy = py - mouseY
        dist = Math.sqrt(dx * dx + dy * dy)
      }
      const v = fastSin(px * 0.008 + time) + fastSin(py * 0.01 + time * 0.7) + fastSin((px + py) * 0.006 + time * 0.5) + fastSin(Math.sqrt(px * px + py * py) * 0.005 - time * 0.8)
      const norm = (v / 4 + 0.5)
      const hue = ((norm * 360 + hs + dist * 0.2) % 360 + 360) % 360
      const alpha = 0.12 + norm * 0.18
      ctx.fillStyle = `hsla(${hue|0},${gs(55)},${gb(45)},${alpha})`
      ctx.fillRect(px, py, step + 1, step + 1)
    }
  }
}

// ── 18. STARDUST ──
interface Star { x: number; y: number; z: number; size: number; twinkleSpeed: number; twinkleOffset: number }
let stars: Star[] = []
let starsInit = false
function initStars() {
  stars = []
  for (let i = 0; i < 200; i++) {
    stars.push({
      x: Math.random() * w, y: Math.random() * h,
      z: 0.3 + Math.random() * 0.7,
      size: 0.5 + Math.random() * 2,
      twinkleSpeed: 0.5 + Math.random() * 2,
      twinkleOffset: Math.random() * 6.283,
    })
  }
  starsInit = true
}

function drawStardust() {
  if (!ctx) return
  if (!starsInit) initStars()

  const hs = hueShift
  const baseHue = (t * 0.1 + hs + 240) % 360

  const nebulaCount = 5
  for (let n = 0; n < nebulaCount; n++) {
    const nx = w * 0.5 + fastSin(t * 0.005 * speedMul + n * 1.2) * w * 0.3
    const ny = h * 0.5 + fastCos(t * 0.004 * speedMul + n * 1.5) * h * 0.25
    const nr = 60 + fastSin(t * 0.01 * speedMul + n * 0.8) * 30 + 40
    const nh = ((baseHue + n * 50 + fastSin(t * 0.006 * speedMul + n) * 30) % 360 + 360) % 360
    const grad = ctx.createRadialGradient(nx, ny, 0, nx, ny, nr)
    grad.addColorStop(0, `hsla(${nh|0},${gs(45)},${gb(45)},0.06)`)
    grad.addColorStop(0.5, `hsla(${((nh + 40) % 360)|0},35,35,0.035)`)
    grad.addColorStop(1, 'rgba(0,0,0,0)')
    ctx.fillStyle = grad
    ctx.fillRect(nx - nr, ny - nr, nr * 2, nr * 2)
  }

  const mouseInflR = 120
  for (let i = 0; i < stars.length; i++) {
    const s = stars[i]
    const twinkle = fastSin(t * 0.03 * s.twinkleSpeed + s.twinkleOffset) * 0.5 + 0.5
    const alpha = twinkle * 0.4 * s.z

    let sx = s.x, sy = s.y
    if (mouseActive) {
      const dx = s.x - mouseX
      const dy = s.y - mouseY
      const d2 = dx * dx + dy * dy
      if (d2 < mouseInflR * mouseInflR && d2 > 0.01) {
        const force = (1 - Math.sqrt(d2) / mouseInflR) * 15
        const ang = Math.atan2(dy, dx) + fastSin(t * 0.02 + i) * 0.5
        sx += Math.cos(ang) * force
        sy += Math.sin(ang) * force
      }
    }

    const hue = ((baseHue + i * 3 + twinkle * 20) % 360 + 360) % 360
    ctx.beginPath()
    ctx.arc(sx, sy, s.size * twinkle, 0, 6.283)
    ctx.fillStyle = `hsla(${hue|0},${gs(55)},${${gb(45)} + twinkle * 20|0},${alpha})`
    ctx.fill()

    ctx.beginPath()
    ctx.arc(sx, sy, s.size * twinkle * 2.5, 0, 6.283)
    ctx.fillStyle = `hsla(${hue|0},${gs(45)},${gb(40)},${alpha * 0.2})`
    ctx.fill()
  }
}

// ── 19. AURORA BOREALIS ──
function drawAurora() {
  if (!ctx) return
  const baseHue = (t * 0.08 + hueShift + 180) % 360
  const time = t * 0.004

  for (let x = 0; x < w; x += 4) {
    const v = fastSin(x * 0.003 + time * 0.5) * 0.6
      + fastSin(x * 0.007 - time * 0.3) * 0.4
      + fastSin(x * 0.001 + time * 0.8) * 0.3
    const height = (v * 0.5 + 0.5) * h * 0.35 + h * 0.05

    const hue = (baseHue + x * 0.1 + v * 40) % 360
    const alpha = 0.08 + (v * 0.5 + 0.5) * 0.15

    ctx.beginPath()
    ctx.moveTo(x, h)
    ctx.lineTo(x, h - height)
    ctx.strokeStyle = `hsla(${hue|0},${gs(55)},${gb(50)},${alpha})`
    ctx.lineWidth = 3
    ctx.stroke()
  }

  // Aurora glow bands
  for (let band = 0; band < 3; band++) {
    const bandY = h * (0.15 + band * 0.08) + fastSin(t * 0.005 + band * 1.5) * h * 0.04
    const bandH = 20 + fastSin(t * 0.007 + band * 2) * 10 + 10
    const bh = (baseHue + band * 30) % 360
    const bAlpha = 0.04 + fastSin(t * 0.003 + band * 0.8) * 0.02

    ctx.beginPath()
    for (let x = 0; x < w; x += 2) {
      const wave = fastSin(x * 0.005 + time + band * 1.2) * 30
      const y = bandY + wave
      if (x === 0) ctx.moveTo(x, y)
      else ctx.lineTo(x, y)
    }
    ctx.strokeStyle = `hsla(${bh|0},${gs(50)},${gb(48)},${bAlpha})`
    ctx.lineWidth = bandH
    ctx.stroke()
  }
}

// ── 20. HEX GRID ──
function drawHexGrid() {
  if (!ctx) return
  const baseHue = (t * 0.15 + hueShift + 60) % 360
  const hexSize = 35
  const hexH = hexSize * Math.sqrt(3)
  const time = t * 0.006

  for (let row = -1; row < Math.ceil(h / hexH) + 1; row++) {
    for (let col = -1; col < Math.ceil(w / hexSize) + 1; col++) {
      const offsetX = (row & 1) * hexSize * 0.5
      const cx = col * hexSize * 1.5 + offsetX
      const cy = row * hexH + hexSize

      const pulse = fastSin(time + col * 0.5 + row * 0.7) * 0.5 + 0.5
      const hue = (baseHue + col * 8 + row * 12 + pulse * 30) % 360
      const alpha = 0.08 + pulse * 0.15

      // Hexagon
      ctx.beginPath()
      for (let i = 0; i <= 6; i++) {
        const angle = (i / 6) * 6.283 - Math.PI * 0.5
        const r = hexSize * (0.5 + pulse * 0.2)
        const hx = cx + Math.cos(angle) * r
        const hy = cy + Math.sin(angle) * r
        if (i === 0) ctx.moveTo(hx, hy)
        else ctx.lineTo(hx, hy)
      }
      ctx.closePath()
      ctx.strokeStyle = `hsla(${hue|0},${gs(50)},${gb(45)},${alpha})`
      ctx.lineWidth = 0.4 + pulse * 0.4
      ctx.stroke()
    }
  }
}

// ══════════════════════════════════════════════════════════════
//  MAIN LOOP
// ══════════════════════════════════════════════════════════════

function draw() {
  if (!ctx) return
  t++

  const s = props.settings
  speedMul = s?.globalSpeed ?? 1
  hueShift = s?.globalHueShift ?? 0
  globalSat = s?.globalSaturation ?? 50
  globalBright = s?.globalBrightness ?? 50

  mouseSmoothX += (mouseX - mouseSmoothX) * 0.12
  mouseSmoothY += (mouseY - mouseSmoothY) * 0.12

  // Solid background fill (removed screen blend)
  ctx.fillStyle = BG_COLOR
  ctx.fillRect(0, 0, w, h)

  // ── Background landscape layer: Moiré Grid ──
  // Always drawn first as the deep background
  if (!s || s.layers.moireGrid.enabled) { ctx.globalAlpha = s?.layers.moireGrid.opacity ?? 0.6; drawMoireGrid(); ctx.globalAlpha = 1 }

  // Aurora Borealis - atmospheric background
  if (!s || s.layers.aurora.enabled) { ctx.globalAlpha = s?.layers.aurora.opacity ?? 0.5; drawAurora(); ctx.globalAlpha = 1 }

  // Cloud Chamber
  if (!s || s.layers.cloudChamber.enabled) { ctx.globalAlpha = s?.layers.cloudChamber.opacity ?? 0.7; drawCloudChamber(); ctx.globalAlpha = 1 }

  // Flower of Life
  if (!s || s.layers.flowerOfLife.enabled) { ctx.globalAlpha = s?.layers.flowerOfLife.opacity ?? 0.5; drawFlowerOfLife(); ctx.globalAlpha = 1 }

  // Op-Art Rings
  if (!s || s.layers.opArtRings.enabled) { ctx.globalAlpha = s?.layers.opArtRings.opacity ?? 0.5; drawOpArtRings(); ctx.globalAlpha = 1 }

  // Hex Grid
  if (!s || s.layers.hexGrid.enabled) { ctx.globalAlpha = s?.layers.hexGrid.opacity ?? 0.4; drawHexGrid(); ctx.globalAlpha = 1 }

  // Neural Network
  if (!s || s.layers.neuralNetwork.enabled) { ctx.globalAlpha = s?.layers.neuralNetwork.opacity ?? 0.6; drawNeuralNetwork(); ctx.globalAlpha = 1 }

  // Vortex
  if (!s || s.layers.vortex.enabled) { ctx.globalAlpha = s?.layers.vortex.opacity ?? 0.5; drawVortex(); ctx.globalAlpha = 1 }

  // Lissajous curves
  if (!s || s.layers.lissajous.enabled) { ctx.globalAlpha = s?.layers.lissajous.opacity ?? 0.5; drawLissajous(); ctx.globalAlpha = 1 }

  // Mandala
  if (!s || s.layers.mandala.enabled) { ctx.globalAlpha = s?.layers.mandala.opacity ?? 0.5; drawMandala(); ctx.globalAlpha = 1 }

  // Spiral Particles
  if (!s || s.layers.spiralParticles.enabled) { ctx.globalAlpha = s?.layers.spiralParticles.opacity ?? 0.6; drawSpiralParticles(); ctx.globalAlpha = 1 }

  // Pulsating Core
  if (!s || s.layers.core.enabled) { ctx.globalAlpha = s?.layers.core.opacity ?? 0.6; drawCore(); ctx.globalAlpha = 1 }

  // Warp Field (mouse)
  if (!s || s.layers.warpField.enabled) { ctx.globalAlpha = s?.layers.warpField.opacity ?? 0.5; drawWarpField(); ctx.globalAlpha = 1 }

  // Disintegration (mouse)
  if (!s || s.layers.disintegration.enabled) { ctx.globalAlpha = s?.layers.disintegration.opacity ?? 0.6; drawDisintegration(); ctx.globalAlpha = 1 }

  // Screen Tear
  if (!s || s.layers.screenTear.enabled) { ctx.globalAlpha = s?.layers.screenTear.opacity ?? 0.4; drawScreenTear(); ctx.globalAlpha = 1 }

  // Matrix Rain
  if (!s || s.layers.matrixRain.enabled) { ctx.globalAlpha = s?.layers.matrixRain.opacity ?? 0.6; drawMatrixRain(); ctx.globalAlpha = 1 }

  // Fractal Tree
  if (!s || s.layers.fractalTree.enabled) { ctx.globalAlpha = s?.layers.fractalTree.opacity ?? 0.6; drawFractalTree(); ctx.globalAlpha = 1 }

  // Plasma Wave
  if (!s || s.layers.plasmaWave.enabled) { ctx.globalAlpha = s?.layers.plasmaWave.opacity ?? 0.5; drawPlasmaWave(); ctx.globalAlpha = 1 }

  // Stardust / Nebula
  if (!s || s.layers.stardust.enabled) { ctx.globalAlpha = s?.layers.stardust.opacity ?? 0.5; drawStardust(); ctx.globalAlpha = 1 }

  // CRT Overlay (always on top)
  if (!s || s.layers.crtOverlay.enabled) { ctx.globalAlpha = s?.layers.crtOverlay.opacity ?? 0.4; drawVintageOverlay(); ctx.globalAlpha = 1 }

  animId = requestAnimationFrame(draw)
}

function start() {
  if (animId) return
  resize()
  initFlower()
  initNetwork()
  initNeuralPackets()
  initSpirals()
  initRain()
  initStars()
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
}
</style>
