import { ref } from 'vue'
import gsap from 'gsap'

let instance: ReturnType<typeof createFpsMonitor> | null = null

function createFpsMonitor() {
  const isLowFps = ref(false)
  const prefersReducedMotion = ref(false)
  const currentFps = ref(60)
  const isAnimating = ref(true)

  const mm = gsap.matchMedia()
  mm.add('(prefers-reduced-motion: reduce)', () => {
    prefersReducedMotion.value = true
    isAnimating.value = false
    gsap.globalTimeline.timeScale(0)
    return () => {
      prefersReducedMotion.value = false
      isAnimating.value = true
      gsap.globalTimeline.timeScale(1)
    }
  })

  const fpsSamples: number[] = []
  let consecutiveLow = 0
  let consecutiveGood = 0

  const onTicker = gsap.ticker.add((_time: number, deltaTime: number, _frame: number) => {
    if (deltaTime && deltaTime < 1) {
      const fps = Math.round(1 / deltaTime)
      currentFps.value = fps
      fpsSamples.push(fps)

      if (fpsSamples.length >= 30) {
        const avg = fpsSamples.reduce((a, b) => a + b, 0) / fpsSamples.length
        fpsSamples.length = 0

        if (avg < 30) {
          consecutiveLow++
          consecutiveGood = 0
          if (consecutiveLow >= 3) {
            if (isAnimating.value) {
              isLowFps.value = true
              isAnimating.value = false
              gsap.globalTimeline.timeScale(0)
            }
          }
        } else if (avg > 45) {
          consecutiveGood++
          consecutiveLow = 0
          if (consecutiveGood >= 2) {
            if (!isAnimating.value) {
              isLowFps.value = false
              isAnimating.value = true
              gsap.globalTimeline.timeScale(1)
            }
          }
        } else {
          consecutiveLow = 0
          consecutiveGood = 0
        }
      }
    }
  })

  function dispose() {
    gsap.ticker.remove(onTicker)
    gsap.globalTimeline.timeScale(1)
    mm.revert()
  }

  return { isLowFps, prefersReducedMotion, currentFps, isAnimating, dispose }
}

export function useFpsMonitor() {
  if (!instance) instance = createFpsMonitor()
  return instance
}
