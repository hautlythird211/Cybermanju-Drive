<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import gsap from 'gsap'
import OsIcon from './OsIcon.vue'
import OsTooltip from './OsTooltip.vue'

export interface DockItem {
  id: string
  icon: string
  label: string
  color?: string
  badge?: string | number
  active?: boolean
  action: () => void
}

const props = withDefaults(defineProps<{
  items: DockItem[]
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  position?: 'bottom' | 'left' | 'right'
  magnification?: boolean
}>(), {
  variant: 'default',
  position: 'bottom',
  magnification: true,
})

const dockRefs = ref<(HTMLElement | null)[]>([])
const magnifyRAF = ref<number | null>(null)
const isAnimating = ref(false)
const gsapCtx = ref<gsap.Context | null>(null)
const dockTweens = ref<Map<number, gsap.core.Tween>>(new Map())

const cls = computed(() => [
  'os-dock',
  `os-dock--${props.variant}`,
  `os-dock--${props.position}`,
])

const dockStyle = computed(() => {
  if (props.position === 'left') return { flexDirection: 'column' as const }
  if (props.position === 'right') return { flexDirection: 'column' as const }
  return { flexDirection: 'row' as const }
})

function setDockRef(el: HTMLElement | null, idx: number) {
  dockRefs.value[idx] = el
}

function onDockItemEnter(idx: number) {
  if (!props.magnification || isAnimating.value) return
  isAnimating.value = true
  if (magnifyRAF.value !== null) cancelAnimationFrame(magnifyRAF.value)
  magnifyRAF.value = requestAnimationFrame(() => {
    dockRefs.value.forEach((itemEl, i) => {
      if (itemEl) {
        const dist = Math.abs(i - idx)
        let scale = 1
        if (dist === 0) scale = 1.3
        else if (dist === 1) scale = 1.1
        dockTweens.value.get(i)?.kill()
        const t = gsap.to(itemEl, { scale, duration: 0.35, ease: 'back.out(2)', force3D: true, overwrite: 'auto' })
        dockTweens.value.set(i, t)
      }
    })
    magnifyRAF.value = null
    requestAnimationFrame(() => { isAnimating.value = false })
  })
}

function onDockItemLeave() {
  if (!props.magnification) return
  if (magnifyRAF.value !== null) cancelAnimationFrame(magnifyRAF.value)
  dockRefs.value.forEach((itemEl, i) => {
    if (itemEl) {
      dockTweens.value.get(i)?.kill()
      const t = gsap.to(itemEl, { scale: 1, duration: 0.2, ease: 'cubic-bezier(0.22, 1, 0.36, 1)', force3D: true, overwrite: 'auto' })
      dockTweens.value.set(i, t)
    }
  })
}

onMounted(() => {
  gsapCtx.value = gsap.context(() => {}, dockRefs.value[0] || undefined)
})

onUnmounted(() => {
  if (magnifyRAF.value !== null) cancelAnimationFrame(magnifyRAF.value)
  dockTweens.value.forEach(t => t.kill())
  dockTweens.value.clear()
  gsapCtx.value?.revert()
})
</script>

<template>
  <div :class="cls" :style="dockStyle" role="toolbar" aria-label="Application Dock">
    <div
      v-for="(item, idx) in items"
      :key="item.id"
      :ref="(el: any) => setDockRef(el as HTMLElement | null, idx)"
      class="os-dock__item"
      :class="{ 'os-dock__item--active': item.active }"
      role="button"
      :aria-label="item.label"
      tabindex="0"
      @mouseenter="onDockItemEnter(idx)"
      @mouseleave="onDockItemLeave"
      @click="item.action"
      @keydown.enter="item.action"
      @keydown.space.prevent="item.action"
    >
      <OsTooltip :text="item.label" :position="position === 'bottom' ? 'top' : 'right'">
        <div class="os-dock__icon-wrap" :style="{ color: item.color }">
          <OsIcon :icon="item.icon" :size="22" />
          <span v-if="item.badge" class="os-dock__badge">{{ item.badge }}</span>
        </div>
      </OsTooltip>
    </div>
  </div>
</template>

<style scoped>
.os-dock {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 6px 12px;
  z-index: 100;
  pointer-events: auto;
}

.os-dock--bottom {
  flex-direction: row;
  border-radius: var(--radius-xl) var(--radius-xl) 0 0;
}

.os-dock--left {
  flex-direction: column;
  border-radius: 0 var(--radius-xl) var(--radius-xl) 0;
}

.os-dock--right {
  flex-direction: column;
  border-radius: var(--radius-xl) 0 0 var(--radius-xl);
}

.os-dock--default {
  background: var(--bg-elevated);
  border-top: 1px solid var(--border-subtle);
}
.os-dock--left.os-dock--default { border-top: none; border-right: 1px solid var(--border-subtle); }
.os-dock--right.os-dock--default { border-top: none; border-left: 1px solid var(--border-subtle); }

.os-dock--glass {
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border: 1px solid var(--border-glass);
  box-shadow: var(--shadow-glass), var(--panel-inset);
}

.os-dock--neon {
  background: rgba(0, 0, 0, 0.7);
  border: 1px solid rgba(var(--accent-rgb), 0.2);
  box-shadow: 0 0 12px var(--accent-dim);
}

.os-dock--gothic {
  background: rgba(15, 5, 5, 0.85);
  border: 1px solid #2a1010;
}

.os-dock--cute {
  background: rgba(255, 107, 157, 0.06);
  border: 1px solid rgba(255, 107, 157, 0.15);
  border-radius: var(--radius-2xl);
}

.os-dock__item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  cursor: pointer;
  border-radius: var(--radius-lg);
  transition: filter 0.25s cubic-bezier(0.22, 1, 0.36, 1);
  position: relative;
  will-change: transform;
}
.os-dock__item:hover .os-dock__icon-wrap {
  filter: brightness(1.4) drop-shadow(0 0 6px var(--accent-glow));
}

.os-dock__item--active {
  background: var(--accent-dim);
  box-shadow: 0 0 16px var(--accent-glow);
  transform: scale(1.06);
}
.os-dock__item--active::after {
  content: '';
  position: absolute;
  bottom: -4px;
  left: 50%;
  transform: translateX(-50%);
  width: 20px;
  height: 3px;
  border-radius: 2px;
  background: var(--accent);
  box-shadow: 0 0 6px var(--accent-glow), 0 0 24px color-mix(in srgb, var(--accent) 25%, transparent);
  animation: dock-active-in 0.35s cubic-bezier(0.22, 1, 0.36, 1) both;
}

.os-dock--left .os-dock__item--active::after {
  bottom: auto;
  right: -4px;
  left: auto;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  border-radius: 2px;
  animation: dock-active-in-vert 0.35s cubic-bezier(0.22, 1, 0.36, 1) both;
}

.os-dock--right .os-dock__item--active::after {
  bottom: auto;
  left: -4px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  border-radius: 2px;
  animation: dock-active-in-vert 0.35s cubic-bezier(0.22, 1, 0.36, 1) both;
}

@keyframes dock-active-in {
  from { opacity: 0; transform: translateX(-50%) scaleX(0); }
}
@keyframes dock-active-in-vert {
  from { opacity: 0; transform: translateY(-50%) scaleY(0); }
}

.os-dock__icon-wrap {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.os-dock__badge {
  position: absolute;
  top: -4px;
  right: -6px;
  background: var(--danger);
  color: #fff;
  font-size: 8px;
  font-weight: 700;
  font-family: var(--font-mono);
  padding: 0 4px;
  border-radius: var(--radius-full);
  line-height: 14px;
  min-width: 14px;
  text-align: center;
}
</style>
