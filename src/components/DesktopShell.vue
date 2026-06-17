<template>
  <div class="desktop-shell" @contextmenu.prevent="onDesktopContext">
    <!-- Animated ambient background -->
    <div class="desktop-ambient" aria-hidden="true">
      <div class="ambient-orb ambient-orb--1"></div>
      <div class="ambient-orb ambient-orb--2"></div>
      <div class="ambient-orb ambient-orb--3"></div>
    </div>
    <div class="desktop-wallpaper">
      <slot name="wallpaper" />
    </div>

    <TopMenuBar
      class="desktop-menubar"
    />

    <div class="desktop-content gpu-layer">
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
            :class="{ hovered: hoverScreen === si, active: si === currentScreen }"
            @mouseenter="hoverScreen = si"
            @mouseleave="hoverScreen = null"
            @click="jumpToScreen(si)"
          >
            <div class="overview-screen-label">SCREEN {{ si + 1 }}</div>
            <div class="overview-screen-grid">
              <div
                v-for="(w, wi) in scr"
                :key="w.id"
                class="overview-tile"
                :style="overviewTileBg(w)"
              >
                <Icon :icon="w.icon" width="10" height="10" />
                <span class="overview-tile-label">{{ w.title }}</span>
              </div>
              <div
                v-for="n in (4 - scr.length)"
                :key="'e' + n"
                class="overview-tile overview-tile-empty"
              />
            </div>
          </div>
          <div class="overview-screen overview-screen-new" @click="createNewScreen">
            <Icon icon="mdi:plus-circle-outline" width="24" height="24" />
            <span>NEW SCREEN</span>
          </div>
        </div>
        <div class="overview-hint">click a screen to jump · 3-finger swipe to close</div>
      </div>

      <div ref="gridRef" class="desktop-grid" :class="{ 'screen-swiping': screenTransitioning }">
        <AppWindow
          v-for="(win, idx) in visibleWindows"
          :key="win.id"
          :win="win"
          :focused="win.id === wm.activeWindow.value?.id"
          :tile-style="getTileStyle(idx)"
          @close="(id: string) => wm.close(id)"
          @minimize="(id: string) => wm.minimize(id)"
          @focus="(id: string) => wm.focus(id)"
          @move="(id: string, x: number, y: number) => wm.updatePosition(id, x, y)"
        />
      </div>
      </div>
    </div>

    <StatusBar class="desktop-statusbar" />

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
import StatusBar from '@/components/StatusBar.vue'
import Dock from '@/components/Dock.vue'
import AppWindow from '@/components/AppWindow.vue'

const anim = useGsapAnimation()
const store = useAppStore()
const wm = useWindowManager()

const workspaceRef = ref<HTMLElement | null>(null)
const gridRef = ref<HTMLElement | null>(null)
const overviewRef = ref<HTMLElement | null>(null)
const screenTransitioning = ref(false)
const currentScreen = ref(0)
const overview = ref(false)
const hoverScreen = ref<number | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const visibleWindows = computed(() => {
  const windows = wm.windows.value.filter(w => !w.minimized)
  const perScreen = 4
  const start = currentScreen.value * perScreen
  return windows.slice(start, start + perScreen)
})

const allScreens = computed(() => {
  const windows = wm.windows.value.filter(w => !w.minimized)
  const perScreen = 4
  const screens: typeof windows[] = []
  for (let i = 0; i < windows.length; i += perScreen) {
    screens.push(windows.slice(i, i + perScreen))
  }
  if (screens.length === 0) screens.push([])
  return screens
})

function getTileStyle(index: number) {
  const pos = index % 4
  const col = pos % 2
  const row = Math.floor(pos / 2)
  return {
    left: `${col * 50}%`,
    top: `${row * 50}%`,
    width: '50%',
    height: '50%',
  }
}

function overviewTileBg(w: { title: string; icon: string }) {
  const colours = ['#ff5f57', '#febc2e', '#28c840', '#5dade2', '#af7ac5', '#f5b041']
  const bg = colours[w.title.length % colours.length]
  return { background: `${bg}22`, borderColor: bg }
}

function jumpToScreen(si: number) {
  currentScreen.value = Math.max(0, Math.min(si, allScreens.value.length - 1))
  exitOverview()
}

function createNewScreen() {
  const total = allScreens.value.length
  currentScreen.value = total
  overview.value = false
}

function exitOverview() {
  overview.value = false
  hoverScreen.value = null
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

  // 3-finger swipe up from center → toggle overview (mission control)
  if (touchFingerCount >= 3) {
    if (dy < 0 && inCenterZone(touchStartX, touchStartY)) {
      overview.value = true
    } else if (overview.value) {
      exitOverview()
    }
    touchFingerCount = 0
    return
  }

  // 2-finger horizontal swipe → navigate screens
  if (touchFingerCount === 2 && absDx > absDy) {
    navigateScreen(dx > 0 ? -1 : 1)
  }

  touchFingerCount = 0
}

// ── Wheel navigation (Ctrl+Wheel) ──
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
  const max = Math.max(0, allScreens.value.length - 1)
  const next = Math.max(0, Math.min(currentScreen.value + dir, max))
  if (next === currentScreen.value) return
  screenTransitioning.value = true
  if (gridRef.value) {
    gsapCtx.value?.add(() => {
      anim.fadeOut(gridRef.value!, { duration: 0.15 })
    })
  }
  currentScreen.value = next
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

// ── Sync store booleans ↔ window manager ──
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

onMounted(async () => {
  gsapCtx.value = gsap.context(() => {
    if (workspaceRef.value) {
      anim.fadeIn(workspaceRef.value, { from: { opacity: 0 } })
    }
  })
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
  gsapCtx.value?.revert()
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
  contain: layout style;
}

.desktop-wallpaper {
  position: fixed;
  inset: 0;
  z-index: 0;
  pointer-events: none;
}

.desktop-menubar {
  position: relative;
  z-index: 100;
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
  width: 100%;
  height: 100%;
  padding: 6px;
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
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 20px;
}

@keyframes overview-fadein {
  0% { opacity: 0; backdrop-filter: blur(0px); }
  100% { opacity: 1; backdrop-filter: blur(16px); }
}

.overview-header {
  font-family: 'Courier New', monospace;
  font-size: 13px;
  font-weight: 700;
  color: #00ff41;
  letter-spacing: 4px;
  text-shadow: 0 0 20px rgba(0, 255, 65, 0.3);
}

.overview-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  justify-content: center;
  align-items: center;
  max-width: 90vw;
}

.overview-screen {
  width: 200px;
  height: 140px;
  border: 2px solid #333;
  border-radius: 10px;
  padding: 8px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition: all 0.2s cubic-bezier(0.22, 1, 0.36, 1);
  position: relative;
  background: rgba(20, 20, 20, 0.8);
  backdrop-filter: blur(8px);
}

.overview-screen:hover {
  border-color: #555;
  transform: translateY(-4px) scale(1.03);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.overview-screen.hovered {
  border-color: #00ff41;
  box-shadow:
    0 0 20px rgba(0, 255, 65, 0.15),
    inset 0 0 30px rgba(0, 255, 65, 0.03);
}

.overview-screen.active {
  border-color: #00ff4188;
}

.overview-screen-label {
  font-family: 'Courier New', monospace;
  font-size: 8px;
  font-weight: 700;
  color: #888;
  letter-spacing: 1px;
}

.overview-screen-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  gap: 3px;
}

.overview-tile {
  border: 1px solid #333;
  border-radius: 4px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  font-size: 7px;
  color: #ccc;
  overflow: hidden;
  transition: all 0.15s;
}

.overview-screen:hover .overview-tile {
  border-color: #444;
}

.overview-tile-empty {
  opacity: 0.15;
  border-style: dashed;
}

.overview-tile-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 90%;
  font-weight: 600;
}

.overview-screen-new {
  border-style: dashed;
  border-color: #444;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #666;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  letter-spacing: 1px;
}

.overview-screen-new:hover {
  border-color: #00ff41;
  color: #00ff41;
}

.overview-hint {
  font-family: 'Courier New', monospace;
  font-size: 9px;
  color: #444;
  letter-spacing: 1px;
}

.desktop-statusbar {
  position: relative;
  z-index: 100;
}

.desktop-dock {
  position: fixed;
  bottom: 8px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 200;
}

/* ── Ambient background ── */
.desktop-ambient {
  position: fixed;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
}

.ambient-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  will-change: transform;
}

.ambient-orb--1 {
  width: clamp(300px, 50vw, 600px);
  height: clamp(300px, 50vw, 600px);
  background: radial-gradient(circle, rgba(0, 255, 65, 0.12), transparent 70%);
  top: -10%;
  left: -10%;
  animation: ambient-drift 20s ease-in-out infinite;
}

.ambient-orb--2 {
  width: clamp(400px, 60vw, 700px);
  height: clamp(400px, 60vw, 700px);
  background: radial-gradient(circle, rgba(90, 240, 255, 0.08), transparent 70%);
  bottom: -15%;
  right: -10%;
  animation: ambient-drift 25s ease-in-out infinite reverse;
}

.ambient-orb--3 {
  width: clamp(200px, 30vw, 400px);
  height: clamp(200px, 30vw, 400px);
  background: radial-gradient(circle, rgba(255, 107, 157, 0.06), transparent 70%);
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation: ambient-drift 30s ease-in-out infinite;
  animation-delay: -10s;
}

.desktop-content {
  position: relative;
  z-index: 1;
  isolation: isolate;
}

@keyframes ambient-drift {
  0%, 100% { transform: translate(0, 0) scale(1); }
  25% { transform: translate(5%, 3%) scale(1.05); }
  50% { transform: translate(-3%, 5%) scale(0.95); }
  75% { transform: translate(4%, -2%) scale(1.02); }
}

/* ── Screen transitions ── */
.screen-enter-active,
.screen-leave-active {
  transition: opacity var(--duration-slow) cubic-bezier(0.22, 1, 0.36, 1),
              transform var(--duration-slow) cubic-bezier(0.22, 1, 0.36, 1),
              filter var(--duration-slow) cubic-bezier(0.22, 1, 0.36, 1);
}

.screen-enter-from {
  opacity: 0;
  transform: scale(0.97) translateY(8px);
  filter: blur(4px);
}

.screen-leave-to {
  opacity: 0;
  transform: scale(0.97) translateY(-8px);
  filter: blur(4px);
}
</style>
