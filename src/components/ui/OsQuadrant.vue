<script setup lang="ts">
import { computed, ref, onMounted, nextTick } from 'vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

const props = withDefaults(defineProps<{
  layout?: '2x2' | '3x1' | '1x3' | '2x1' | '1x2' | '3x2' | '2x3'
  gap?: string
  padding?: string
  variant?: 'default' | 'glass' | 'neon' | 'gothic'
}>(), {
  layout: '2x2',
  gap: '8px',
  padding: '8px',
  variant: 'default',
})

const quadrantRef = ref<HTMLElement | null>(null)

const cls = computed(() => [
  'os-quadrant',
  `os-quadrant--${props.layout}`,
  `os-quadrant--${props.variant}`,
])

const style = computed(() => ({
  gap: props.gap,
  padding: props.padding,
}))

const quadrantSlots = computed(() => {
  const map: Record<string, string> = {
    '2x2': '4 quadrants (tl, tr, bl, br)',
    '3x1': '3 horizontal (left, center, right)',
    '1x3': '3 vertical (top, middle, bottom)',
    '2x1': '2 horizontal (left, right)',
    '1x2': '2 vertical (top, bottom)',
    '3x2': '6 cells (tl, tc, tr, bl, bc, br)',
    '2x3': '6 cells (tl, tr, ml, mr, bl, br)',
  }
  return map[props.layout] || '4 quadrants'
})

const ariaLabel = computed(() => `${props.layout} grid layout: ${quadrantSlots.value}`)

onMounted(async () => {
  await nextTick()
  if (quadrantRef.value) {
    anim.fadeIn(quadrantRef.value, { from: { y: 8 } })
    const cells = Array.from(quadrantRef.value.querySelectorAll('.os-quadrant__cell'))
    if (cells.length > 0) {
      anim.staggerIn(cells as HTMLElement[], { stagger: 0.04 })
    }
  }
})
</script>

<template>
  <div
    ref="quadrantRef"
    :class="[...cls, 'gpu']"
    :style="style"
    role="grid"
    :aria-label="ariaLabel"
  >
    <div v-if="$slots.tl || $slots['top-left']" class="os-quadrant__cell os-quadrant__cell--tl" role="gridcell" aria-label="Top left quadrant">
      <slot name="tl" />
      <slot name="top-left" />
    </div>
    <div v-if="$slots.tc || $slots['top-center']" class="os-quadrant__cell os-quadrant__cell--tc" role="gridcell" aria-label="Top center quadrant">
      <slot name="tc" />
      <slot name="top-center" />
    </div>
    <div v-if="$slots.tr || $slots['top-right']" class="os-quadrant__cell os-quadrant__cell--tr" role="gridcell" aria-label="Top right quadrant">
      <slot name="tr" />
      <slot name="top-right" />
    </div>
    <div v-if="$slots.ml || $slots['mid-left']" class="os-quadrant__cell os-quadrant__cell--ml" role="gridcell" aria-label="Mid left quadrant">
      <slot name="ml" />
      <slot name="mid-left" />
    </div>
    <div v-if="$slots.mr || $slots['mid-right']" class="os-quadrant__cell os-quadrant__cell--mr" role="gridcell" aria-label="Mid right quadrant">
      <slot name="mr" />
      <slot name="mid-right" />
    </div>
    <div v-if="$slots.bl || $slots['bottom-left']" class="os-quadrant__cell os-quadrant__cell--bl" role="gridcell" aria-label="Bottom left quadrant">
      <slot name="bl" />
      <slot name="bottom-left" />
    </div>
    <div v-if="$slots.bc || $slots['bottom-center']" class="os-quadrant__cell os-quadrant__cell--bc" role="gridcell" aria-label="Bottom center quadrant">
      <slot name="bc" />
      <slot name="bottom-center" />
    </div>
    <div v-if="$slots.br || $slots['bottom-right']" class="os-quadrant__cell os-quadrant__cell--br" role="gridcell" aria-label="Bottom right quadrant">
      <slot name="br" />
      <slot name="bottom-right" />
    </div>
  </div>
</template>

<style scoped>
.os-quadrant {
  display: grid;
  width: 100%;
  height: 100%;
  will-change: transform, opacity;
}

.os-quadrant--2x2 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  grid-template-areas:
    "tl tr"
    "bl br";
}
.os-quadrant--2x2 .os-quadrant__cell--tl { grid-area: tl; }
.os-quadrant--2x2 .os-quadrant__cell--tr { grid-area: tr; }
.os-quadrant--2x2 .os-quadrant__cell--bl { grid-area: bl; }
.os-quadrant--2x2 .os-quadrant__cell--br { grid-area: br; }

.os-quadrant--3x1 {
  grid-template-columns: 1fr 1fr 1fr;
  grid-template-rows: 1fr;
  grid-template-areas: "l c r";
}
.os-quadrant--3x1 .os-quadrant__cell--tl { grid-area: l; }
.os-quadrant--3x1 .os-quadrant__cell--tc { grid-area: c; }
.os-quadrant--3x1 .os-quadrant__cell--tr { grid-area: r; }

.os-quadrant--1x3 {
  grid-template-columns: 1fr;
  grid-template-rows: 1fr 1fr 1fr;
  grid-template-areas:
    "t"
    "m"
    "b";
}
.os-quadrant--1x3 .os-quadrant__cell--tl { grid-area: t; }
.os-quadrant--1x3 .os-quadrant__cell--ml { grid-area: m; }
.os-quadrant--1x3 .os-quadrant__cell--bl { grid-area: b; }

.os-quadrant--2x1 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr;
  grid-template-areas: "l r";
}
.os-quadrant--2x1 .os-quadrant__cell--tl { grid-area: l; }
.os-quadrant--2x1 .os-quadrant__cell--tr { grid-area: r; }

.os-quadrant--1x2 {
  grid-template-columns: 1fr;
  grid-template-rows: 1fr 1fr;
  grid-template-areas:
    "t"
    "b";
}
.os-quadrant--1x2 .os-quadrant__cell--tl { grid-area: t; }
.os-quadrant--1x2 .os-quadrant__cell--bl { grid-area: b; }

.os-quadrant--3x2 {
  grid-template-columns: 1fr 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  grid-template-areas:
    "tl tc tr"
    "bl bc br";
}
.os-quadrant--2x3 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr 1fr;
  grid-template-areas:
    "tl tr"
    "ml mr"
    "bl br";
}

.os-quadrant__cell {
  overflow: auto;
  position: relative;
  will-change: transform, opacity;
  contain: layout style;
}

.os-quadrant--glass .os-quadrant__cell {
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-md);
}

.os-quadrant--neon .os-quadrant__cell {
  border: 1px solid rgba(0, 255, 65, 0.1);
  border-radius: var(--radius-md);
}

.os-quadrant--gothic .os-quadrant__cell {
  border: 1px solid rgba(60, 10, 20, 0.3);
  border-radius: var(--radius-md);
  background: rgba(20, 5, 5, 0.3);
}
</style>
