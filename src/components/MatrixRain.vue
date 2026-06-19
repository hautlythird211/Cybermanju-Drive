<template>
  <canvas
    ref="canvasRef"
    class="matrix-rain-canvas"
    :style="{ opacity: enabled ? opacity : 0, transition: 'opacity 0.5s ease' }"
  />
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'

const props = withDefaults(defineProps<{
  enabled?: boolean
  opacity?: number
}>(), {
  enabled: true,
  opacity: 0.08,
})

const canvasRef = ref<HTMLCanvasElement | null>(null)
let animationId: number | null = null
let columns: Float64Array = new Float64Array(0)
let columnHue: Float64Array = new Float64Array(0)
let columnSpeed: Float64Array = new Float64Array(0)
let ctx: CanvasRenderingContext2D | null = null
let w = 0, h = 0, t = 0

// Mouse
let mouseX = -9999, mouseY = -9999, mouseActive = false
function onMouseMove(e: MouseEvent) { mouseX = e.clientX; mouseY = e.clientY; mouseActive = true }
function onMouseLeave() { mouseActive = false; mouseX = -9999; mouseY = -9999 }

// Katakana + symbols
const chars = 'アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*{}[]|;:<>?/~'
const charArr: string[] = []
for (let i = 0; i < chars.length; i++) charArr.push(chars[i])

function getRandomChar(): string {
  return charArr[(Math.random() * charArr.length) | 0]
}

function hash(n: number): number {
  n = (n << 13) ^ n
  return ((n * (n * n * 15731 + 789221) + 1376312589) & 0x7fffffff) / 0x7fffffff
}

function resize(): void {
  const canvas = canvasRef.value
  if (!canvas) return
  w = window.innerWidth
  h = window.innerHeight
  canvas.width = w
  canvas.height = h
  const colWidth = 16
  const colCount = Math.ceil(w / colWidth)
  columns = new Float64Array(colCount)
  columnHue = new Float64Array(colCount)
  columnSpeed = new Float64Array(colCount)
  for (let i = 0; i < colCount; i++) {
    columns[i] = Math.random() * h / 18 * -1
    columnHue[i] = hash(i * 7) * 360
    columnSpeed[i] = 0.6 + hash(i * 11) * 1.2
  }
}

function draw(): void {
  const canvas = canvasRef.value
  if (!canvas || !ctx) return
  t++

  // Fade trail (darker = more visible characters)
  ctx.fillStyle = 'rgba(5, 5, 10, 0.08)'
  ctx.fillRect(0, 0, w, h)

  const colWidth = 16
  const fontSize = 14
  ctx.font = `${fontSize}px "Courier New", monospace`

  const timeHue = (t * 0.5) % 360
  const colCount = columns.length

  for (let i = 0; i < colCount; i++) {
    const x = i * colWidth
    const y = columns[i] * 18
    const char = getRandomChar()

    // Per-column hue cycling (psychedelic)
    columnHue[i] = (columnHue[i] + 0.3 + hash(i + t) * 0.2) % 360
    const hue = columnHue[i]

    // Distance to mouse for chromatic glow boost
    let mouseBoost = 0
    if (mouseActive) {
      const dx = x - mouseX
      const dy = y - mouseY
      const d2 = dx * dx + dy * dy
      if (d2 < 40000) mouseBoost = 1 - Math.sqrt(d2) / 200
    }

    // Brightness variation
    const brightness = Math.random()

    if (brightness > 0.95) {
      // Head of stream — brightest, full color
      const headHue = (hue + timeHue) % 360
      ctx.fillStyle = `hsla(${headHue|0}, 80%, ${75 + mouseBoost * 15 |0}%, 0.95)`
      ctx.shadowBlur = 14 + mouseBoost * 10
      ctx.shadowColor = `hsla(${headHue|0}, 90%, 60%, 0.8)`
    } else if (brightness > 0.8) {
      const midHue = (hue + timeHue * 0.5) % 360
      ctx.fillStyle = `hsla(${midHue|0}, 70%, ${55 + mouseBoost * 20 |0}%, 0.85)`
      ctx.shadowBlur = 8 + mouseBoost * 6
      ctx.shadowColor = `hsla(${midHue|0}, 80%, 50%, 0.6)`
    } else {
      const dimHue = (hue + timeHue * 0.3) % 360
      const lightness = 20 + Math.random() * 25 + mouseBoost * 10
      ctx.fillStyle = `hsla(${dimHue|0}, ${30 + mouseBoost * 20 |0}%, ${lightness|0}, ${0.3 + Math.random() * 0.35})`
      ctx.shadowBlur = 0
      ctx.shadowColor = 'transparent'
    }

    ctx.fillText(char, x, y)

    // Reset shadow
    if (brightness > 0.8) {
      ctx.shadowBlur = 0
      ctx.shadowColor = 'transparent'
    }

    // Chromatic aberration ghost (vintage)
    if (brightness > 0.9 && Math.random() > 0.5) {
      ctx.fillStyle = `hsla(${((hue + 120) % 360)|0}, 60%, 50%, 0.08)`
      ctx.fillText(char, x + 2, y - 1)
    }

    // Move column
    columns[i] += columnSpeed[i]
    if (y > h && Math.random() > 0.975) {
      columns[i] = 0
      columnHue[i] = hash(i * 13 + t) * 360
      columnSpeed[i] = 0.6 + hash(i * 17 + t) * 1.2
    }
  }

  // CRT scanline sweep
  const sweepY = (t * 1.5) % (h + 100) - 50
  ctx.fillStyle = 'rgba(0, 255, 65, 0.01)'
  ctx.fillRect(0, sweepY, w, 3)

  animationId = requestAnimationFrame(draw)
}

function startAnimation(): void {
  if (animationId) return
  resize()
  draw()
}

function stopAnimation(): void {
  if (animationId) {
    cancelAnimationFrame(animationId)
    animationId = null
  }
}

onMounted(() => {
  const canvas = canvasRef.value
  if (canvas) ctx = canvas.getContext('2d')
  if (props.enabled) startAnimation()
  window.addEventListener('resize', resize)
  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseleave', onMouseLeave)
})

onUnmounted(() => {
  stopAnimation()
  window.removeEventListener('resize', resize)
  window.removeEventListener('mousemove', onMouseMove)
  window.removeEventListener('mouseleave', onMouseLeave)
})

watch(() => props.enabled, (val) => {
  val ? startAnimation() : stopAnimation()
})
</script>

<style scoped>
.matrix-rain-canvas {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 0;
  pointer-events: none;
}
</style>
