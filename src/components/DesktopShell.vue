<template>
  <div class="desktop-shell" @contextmenu.prevent="onDesktopContext">
    <!-- Animated ambient background with glass-orbs -->
    <div class="desktop-ambient" aria-hidden="true">
      <div class="ambient-orb ambient-orb--1"></div>
      <div class="ambient-orb ambient-orb--2"></div>
      <div class="ambient-orb ambient-orb--3"></div>
      <div class="ambient-orb ambient-orb--4"></div>
      <div class="ambient-orb ambient-orb--5"></div>
      <div class="ambient-grid"></div>
    </div>
    <div class="desktop-wallpaper">
      <slot name="wallpaper" />
    </div>

    <TopMenuBar
      class="desktop-menubar"
    />

    <div class="desktop-content">
      <div
        ref="workspaceRef"
        class="desktop-workspace"
        @touchstart="onTouchStart"
        @touchmove="onTouchMove"
        @touchend="onTouchEnd"
        @wheel="onWheel"
      >
      <div v-if="overview" ref="overviewRef" class="overview-overlay" @click.self="exitOverview">
        <div class="overview-header">MISSION CONTROL</div>
        <div class="overview-grid">
          <div
            v-for="(scr, si) in allScreens"
            :key="si"
            class="overview-screen"
            :class="{ hovered: hoverScreen === si, active: scr.x === wm.currentScreen.value.x && scr.y === wm.currentScreen.value.y }"
            @mouseenter="hoverScreen = si"
            @mouseleave="hoverScreen = null"
            @click="jumpToScreen(si)"
          >
            <div class="overview-screen-label">SCREEN {{ si + 1 }}</div>
            <div class="overview-screen-grid">
              <div
                v-for="w in scr.windows"
                :key="w.id"
                class="overview-tile"
                :style="overviewTileBg(w)"
              >
                <Icon :icon="w.icon" width="12" height="12" />
                <span class="overview-tile-label">{{ w.title }}</span>
              </div>
              <div
                v-for="n in Math.max(0, 4 - scr.windows.length)"
                :key="'e' + n"
                class="overview-tile overview-tile-empty"
              />
            </div>
          </div>
        </div>
        <div class="overview-hint">click a screen to jump - 3-finger swipe to close</div>
      </div>

      <div ref="gridRef" class="desktop-grid" :class="{ 'screen-swiping': screenTransitioning }">
        <AppWindow
          v-for="win in visibleWindows"
          :key="win.id"
          :win="win"
          :focused="win.id === wm.activeWindow.value?.id"
          @close="(id: string) => wm.close(id)"
          @minimize="(id: string) => wm.minimize(id)"
          @maximize="(id: string) => wm.maximize(id)"
          @focus="(id: string) => wm.focus(id)"
          @move="(id: string, x: number, y: number) => wm.updatePosition(id, x, y)"
        />
      </div>
      </div>
    </div>

    <Dock class="desktop-dock" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import gsap from 'gsap'
import { Icon } from '@iconify/vue'
import { useAppStore } from '@/stores/app'
import { useWindowManager } from '@/composables/useWindowManager'
import { useGsapAnimation } from '@/composables/useGsapAnimation'
import TopMenuBar from '@/components/TopMenuBar.vue'
import Dock from '@/components/Dock.vue'
import AppWindow from '@/components/AppWindow.vue'

const anim = useGsapAnimation()
const store = useAppStore()
const wm = useWindowManager()

const workspaceRef = ref<HTMLElement | null>(null)
const gridRef = ref<HTMLElement | null>(null)
const overviewRef = ref<HTMLElement | null>(null)
const screenTransitioning = ref(false)
const overview = ref(false)
const hoverScreen = ref<number | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const allScreens = computed(() => wm.getAllScreens())
const currentScreenIdx = computed(() => {
  const screens = allScreens.value
  return screens.findIndex(s => s.x === wm.currentScreen.value.x && s.y === wm.currentScreen.value.y)
})
const visibleWindows = computed(() => wm.getCurrentScreenWindows())

function overviewTileBg(w: { title: string; icon: string }) {
  const colours = ['#ff453a', '#ffd60a', '#30d158', '#5af0ff', '#b388ff', '#ff6b9d']
  const bg = colours[w.title.length % colours.length]
  return { background: `${bg}18`, borderColor: bg }
}

function jumpToScreen(idx: number) {
  const screen = allScreens.value[idx]
  if (screen) {
    wm.currentScreen.value = { x: screen.x, y: screen.y }
  }
  exitOverview()
}

function exitOverview() {
  overview.value = false
  hoverScreen.value = null
}

let resizeTimeout: ReturnType<typeof setTimeout> | null = null

function retileFromResize() {
  if (resizeTimeout) clearTimeout(resizeTimeout)
  resizeTimeout = setTimeout(() => {
    if (gridRef.value && store.autoArrange) {
      const rect = gridRef.value.getBoundingClientRect()
      wm.retileAll(rect.width, rect.height)
    }
    resizeTimeout = null
  }, 80)
}

function forceRetile() {
  if (gridRef.value) {
    const rect = gridRef.value.getBoundingClientRect()
    wm.retileAll(rect.width, rect.height)
  }
}

// ── Touch handling ──
let touchStartX = 0
let touchStartY = 0
let touchStartTime = 0
let touchFingerCount = 0
let touchMoved = false

function onTouchStart(e: TouchEvent) {
  touchFingerCount = e.touches.length
  const avgX = [...e.touches].reduce((s, t) => s + t.clientX, 0) / e.touches.length
  const avgY = [...e.touches].reduce((s, t) => s + t.clientY, 0) / e.touches.length
  touchStartX = avgX
  touchStartY = avgY
  touchStartTime = Date.now()
  touchMoved = false
}

function onTouchMove(e: TouchEvent) {
  e.preventDefault()
  touchMoved = true
}

function inCenterZone(x: number, y: number): boolean {
  const w = window.innerWidth
  const h = window.innerHeight
  const marginX = w * 0.2
  const marginY = h * 0.25
  return x > marginX && x < w - marginX && y > marginY && y < h - marginY
}

function onTouchEnd(e: TouchEvent) {
  if (touchFingerCount === 0) return
  const dt = Date.now() - touchStartTime
  if (dt > 600) { touchFingerCount = 0; return }
  if (e.changedTouches.length === 0) { touchFingerCount = 0; return }

  const endX = [...e.changedTouches].reduce((s, t) => s + t.clientX, 0) / e.changedTouches.length
  const endY = [...e.changedTouches].reduce((s, t) => s + t.clientY, 0) / e.changedTouches.length
  const dx = endX - touchStartX
  const dy = endY - touchStartY
  const absDx = Math.abs(dx)
  const absDy = Math.abs(dy)
  const minDist = 40

  if (absDx < minDist && absDy < minDist) { touchFingerCount = 0; return }

  if (touchFingerCount >= 3) {
    if (dy < 0 && inCenterZone(touchStartX, touchStartY)) {
      overview.value = true
    } else if (overview.value) {
      exitOverview()
    }
    touchFingerCount = 0
    return
  }

  if (touchFingerCount === 2 && absDx > absDy) {
    navigateScreen(dx > 0 ? -1 : 1)
  }

  touchFingerCount = 0
}

function onWheel(e: WheelEvent) {
  if (!e.ctrlKey && !e.metaKey) return
  e.preventDefault()
  if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
    navigateScreen(e.deltaY > 0 ? 1 : -1)
  } else {
    navigateScreen(e.deltaX > 0 ? 1 : -1)
  }
}

function navigateScreen(dir: number) {
  const screens = allScreens.value
  const idx = currentScreenIdx.value
  const nextIdx = Math.max(0, Math.min(idx + dir, screens.length - 1))
  if (nextIdx === idx) return
  const next = screens[nextIdx]
  if (!next) return

  screenTransitioning.value = true
  if (gridRef.value) {
    gsapCtx.value?.add(() => {
      anim.fadeOut(gridRef.value!, { duration: 0.15 })
    })
  }
  wm.currentScreen.value = { x: next.x, y: next.y }
  setTimeout(() => {
    if (gridRef.value) {
      gsapCtx.value?.add(() => {
        anim.fadeIn(gridRef.value!, { from: { opacity: 0 } })
      })
    }
    screenTransitioning.value = false
  }, 200)
}

function onDesktopContext(e: MouseEvent) {
  const ctx = (window as any).__contextMenu
  if (ctx) ctx.open('file_grid_bg', { x: e.clientX, y: e.clientY })
}

// ── Sync store booleans - window manager ──
watch(() => store.showEncryptionPanel, (val) => {
  if (val) wm.open('encryption')
  else { const w = wm.windows.value.find(win => win.panelType === 'encryption'); if (w) wm.close(w.id) }
})
watch(() => store.showCompressionPanel, (val) => {
  if (val) wm.open('compression')
  else { const w = wm.windows.value.find(win => win.panelType === 'compression'); if (w) wm.close(w.id) }
})
watch(() => store.showPermissionsPanel, (val) => {
  if (val) wm.open('permissions', { fileId: store.selectedFileId })
  else { const w = wm.windows.value.find(win => win.panelType === 'permissions'); if (w) wm.close(w.id) }
})

watch(() => wm.windows.value.map(w => w.id).join(','), () => {
  if (!wm.windows.value.some(w => w.panelType === 'encryption')) store.showEncryptionPanel = false
  if (!wm.windows.value.some(w => w.panelType === 'compression')) store.showCompressionPanel = false
  if (!wm.windows.value.some(w => w.panelType === 'permissions')) store.showPermissionsPanel = false
})

watch(() => store.autoArrange, (enabled) => {
  if (enabled) nextTick(forceRetile)
})

onMounted(async () => {
  gsapCtx.value = gsap.context(() => {
    if (workspaceRef.value) {
      anim.fadeIn(workspaceRef.value, { from: { opacity: 0 } })
    }
  })
  nextTick(forceRetile)
  window.addEventListener('resize', retileFromResize)
})

watch(overview, async (val) => {
  if (val) {
    await nextTick()
    if (overviewRef.value) {
      gsapCtx.value?.add(() => {
        anim.fadeIn(overviewRef.value!, { from: { opacity: 0 } })
      })
    }
  }
})

onUnmounted(() => {
  if (resizeTimeout) clearTimeout(resizeTimeout)
  gsapCtx.value?.revert()
  window.removeEventListener('resize', retileFromResize)
})
</script>

<style scoped>
.desktop-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  position: relative;
  overflow: hidden;
  isolation: isolate;
  background: var(--bg-deep);
}

.desktop-shell::before {
  content: '';
  position: absolute;
  inset: 0;
  z-index: 0;
  pointer-events: none;
  background:
    repeating-linear-gradient(
      0deg,
      transparent,
      transparent 2px,
      rgba(0, 255, 65, 0.012) 2px,
      rgba(0, 255, 65, 0.012) 4px
    );
}

.desktop-wallpaper {
  position: absolute;
  inset: 0;
  z-index: 0;
  pointer-events: none;
}

.desktop-menubar {
  position: relative;
  z-index: 1000;
}

.desktop-workspace {
  flex: 1;
  position: relative;
  z-index: 1;
  overflow: hidden;
  will-change: transform, opacity;
}

.desktop-grid {
  position: relative;
  z-index: 1;
  width: 100%;
  height: 100%;
  padding: 0;
  transition: opacity 0.2s, filter 0.2s;
  will-change: transform, opacity;
}

.desktop-grid.screen-swiping {
  opacity: 1;
}

/* ── Overview / Mission Control ── */
.overview-overlay {
  position: absolute;
  inset: 0;
  z-index: 50;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
}

.overview-header {
  font-family: var(--font-mono);
  font-size: 14px;
  font-weight: 800;
  color: var(--text-accent);
  letter-spacing: 4px;
  text-shadow: 0 0 20px rgba(var(--accent-rgb), 0.2);
}

.overview-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  justify-content: center;
  align-items: center;
  max-width: 90vw;
}

.overview-screen {
  width: 220px;
  height: 150px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 10px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 8px;
  transition: all 0.25s cubic-bezier(0.22, 1, 0.36, 1);
  position: relative;
  background: rgba(14, 14, 18, 0.7);
  backdrop-filter: blur(12px) saturate(1.4);
  -webkit-backdrop-filter: blur(12px) saturate(1.4);
}

.overview-screen:hover {
  border-color: rgba(var(--accent-rgb), 0.2);
  transform: translateY(-4px) scale(1.03);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 0 20px rgba(0, 255, 65, 0.04);
}

.overview-screen.hovered {
  border-color: rgba(var(--accent-rgb), 0.3);
  box-shadow:
    0 0 20px rgba(var(--accent-rgb), 0.08),
    inset 0 0 30px rgba(0, 255, 65, 0.02);
}

.overview-screen.active {
  border-color: rgba(var(--accent-rgb), 0.2);
}

.overview-screen-label {
  font-family: var(--font-mono);
  font-size: 8px;
  font-weight: 700;
  color: var(--text-muted);
  letter-spacing: 1.5px;
}

.overview-screen-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  gap: 4px;
}

.overview-tile {
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  font-size: 7px;
  color: rgba(255, 255, 255, 0.7);
  overflow: hidden;
  transition: all 0.15s;
  font-family: var(--font-mono);
}

.overview-screen:hover .overview-tile {
  border-color: rgba(255, 255, 255, 0.1);
}

.overview-tile-empty {
  opacity: 0.12;
  border-style: dashed;
}

.overview-tile-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 90%;
  font-weight: 600;
}

.overview-hint {
  font-family: var(--font-mono);
  font-size: 9px;
  color: rgba(255, 255, 255, 0.2);
  letter-spacing: 1px;
}

.desktop-dock {
  position: absolute;
  bottom: 14px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 200;
}

/* ── Ambient background ── */
.desktop-ambient {
  position: absolute;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
}

.ambient-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(255, 255, 255, 0.012) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 255, 255, 0.012) 1px, transparent 1px);
  background-size: 60px 60px;
  mask-image: radial-gradient(ellipse at center, rgba(0,0,0,0.4) 0%, transparent 70%);
  -webkit-mask-image: radial-gradient(ellipse at center, rgba(0,0,0,0.4) 0%, transparent 70%);
}

.ambient-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  will-change: transform;
}

.ambient-orb--1 {
  width: clamp(300px, 50vw, 700px);
  height: clamp(300px, 50vw, 700px);
  background: radial-gradient(circle, rgba(var(--accent-rgb), 0.12), rgba(0, 200, 255, 0.04) 50%, transparent 70%);
  top: -10%;
  left: -10%;
  animation: ambient-drift-1 12s ease-in-out infinite;
}

.ambient-orb--2 {
  width: clamp(400px, 60vw, 800px);
  height: clamp(400px, 60vw, 800px);
  background: radial-gradient(circle, rgba(255, 107, 157, 0.08), rgba(179, 136, 255, 0.05) 50%, transparent 70%);
  bottom: -15%;
  right: -10%;
  animation: ambient-drift-2 15s ease-in-out infinite reverse;
}

.ambient-orb--3 {
  width: clamp(200px, 35vw, 500px);
  height: clamp(200px, 35vw, 500px);
  background: radial-gradient(circle, rgba(255, 215, 0, 0.07), rgba(255, 153, 51, 0.04) 50%, transparent 70%);
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation: ambient-drift-3 18s ease-in-out infinite;
}

.ambient-orb--4 {
  width: clamp(250px, 40vw, 550px);
  height: clamp(250px, 40vw, 550px);
  background: radial-gradient(circle, rgba(90, 240, 255, 0.07), rgba(0, 255, 65, 0.03) 50%, transparent 70%);
  top: 30%;
  right: 10%;
  animation: ambient-drift-4 20s ease-in-out infinite reverse;
}

.ambient-orb--5 {
  width: clamp(180px, 25vw, 350px);
  height: clamp(180px, 25vw, 350px);
  background: radial-gradient(circle, rgba(179, 136, 255, 0.06), rgba(255, 107, 157, 0.04) 50%, transparent 70%);
  bottom: 20%;
  left: 15%;
  animation: ambient-drift-5 22s ease-in-out infinite;
}

.desktop-content {
  position: relative;
  z-index: 1;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  isolation: isolate;
}

@keyframes ambient-drift-1 {
  0%, 100% { transform: translate(0, 0) scale(1) rotate(0deg); }
  25% { transform: translate(6%, 4%) scale(1.08) rotate(2deg); }
  50% { transform: translate(-4%, 6%) scale(0.95) rotate(-1deg); }
  75% { transform: translate(5%, -3%) scale(1.04) rotate(3deg); }
}

@keyframes ambient-drift-2 {
  0%, 100% { transform: translate(0, 0) scale(1) rotate(0deg); }
  20% { transform: translate(-5%, -3%) scale(1.05) rotate(-2deg); }
  50% { transform: translate(3%, -5%) scale(0.96) rotate(1deg); }
  80% { transform: translate(-2%, 4%) scale(1.06) rotate(-3deg); }
}

@keyframes ambient-drift-3 {
  0%, 100% { transform: translate(-50%, -50%) scale(1) rotate(0deg); }
  33% { transform: translate(calc(-50% + 5%), calc(-50% - 3%)) scale(1.1) rotate(4deg); }
  66% { transform: translate(calc(-50% - 4%), calc(-50% + 5%)) scale(0.9) rotate(-2deg); }
}

@keyframes ambient-drift-4 {
  0%, 100% { transform: translate(0, 0) scale(1) rotate(0deg); }
  25% { transform: translate(-5%, 5%) scale(1.05) rotate(-4deg); }
  50% { transform: translate(4%, -4%) scale(1.08) rotate(2deg); }
  75% { transform: translate(-3%, -2%) scale(0.95) rotate(-1deg); }
}

@keyframes ambient-drift-5 {
  0%, 100% { transform: translate(0, 0) scale(1) rotate(0deg); }
  30% { transform: translate(4%, -5%) scale(1.06) rotate(3deg); }
  60% { transform: translate(-5%, 3%) scale(0.93) rotate(-4deg); }
}
</style>
