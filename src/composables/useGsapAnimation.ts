import gsap from 'gsap'
import { useFpsMonitor } from './useFpsMonitor'

const SPRING_EASE = 'cubic-bezier(0.22, 1, 0.36, 1)'
const FAST_DURATION = 0.3

const elCache = new Map<Element | string, Element | null>()

function resolveEl(el: Element | string): Element | null {
  if (el instanceof Element) return el
  const cached = elCache.get(el)
  if (cached !== undefined) return cached
  const result = document.querySelector(el)
  elCache.set(el, result)
  return result
}

function resolveEls(els: Element[] | string): Element[] {
  if (Array.isArray(els)) return els
  return Array.from(document.querySelectorAll(els))
}

export function useGsapAnimation() {
  const { isAnimating } = useFpsMonitor()

  function shouldAnimate(): boolean {
    return isAnimating.value
  }

  function ctx() {
    return gsap.context(() => {})
  }

  async function fadeIn(el: Element | string, options?: {
    duration?: number; delay?: number; from?: { y?: number; x?: number; scale?: number; opacity?: number }
  }): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    const { duration = FAST_DURATION, delay = 0, from } = options || {}
    const vars: gsap.TweenVars = {
      autoAlpha: 1,
      duration,
      delay,
      ease: SPRING_EASE,
      force3D: true,
    }
    if (from) {
      gsap.set(resolved, {
        autoAlpha: from.opacity ?? 0,
        y: from.y ?? 0,
        x: from.x ?? 0,
        scale: from.scale ?? 1,
      })
    } else {
      gsap.set(resolved, { autoAlpha: 0 })
    }
    return new Promise(resolve => gsap.to(resolved, { ...vars, onComplete: resolve }))
  }

  async function fadeOut(el: Element | string, options?: {
    duration?: number; delay?: number; to?: { y?: number; x?: number; scale?: number; opacity?: number }
  }): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    const { duration = FAST_DURATION, delay = 0, to } = options || {}
    const vars: gsap.TweenVars = {
      autoAlpha: 0,
      duration,
      delay,
      ease: SPRING_EASE,
      force3D: true,
    }
    if (to) {
      if (to.y !== undefined) vars.y = to.y
      if (to.x !== undefined) vars.x = to.x
      if (to.scale !== undefined) vars.scale = to.scale
      if (to.opacity !== undefined) {
        delete vars.autoAlpha
        vars.opacity = to.opacity
      }
    }
    return new Promise(resolve => gsap.to(resolved, { ...vars, onComplete: resolve }))
  }

  async function staggerIn(els: Element[] | string, options?: {
    duration?: number; stagger?: number; from?: 'start' | 'end' | 'center'; axis?: 'x' | 'y'
  }): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEls(els)
    if (resolved.length === 0) return
    const { duration = FAST_DURATION, stagger = 0.05, from = 'start', axis } = options || {}
    gsap.set(resolved, { autoAlpha: 0, y: 20 })
    const vars: gsap.TweenVars = {
      autoAlpha: 1,
      y: 0,
      duration,
      ease: SPRING_EASE,
      force3D: true,
      stagger: { each: stagger, from },
    }
    if (axis) {
      vars.stagger = { each: stagger, from, axis }
    }
    return new Promise(resolve => gsap.to(resolved, { ...vars, onComplete: resolve }))
  }

  async function modalEnter(backdrop: Element | string, panel: Element | string): Promise<void> {
    if (!shouldAnimate()) return
    const bd = resolveEl(backdrop)
    const pn = resolveEl(panel)
    if (!bd || !pn) return
    const tl = gsap.timeline({ force3D: true })
    tl.fromTo(bd, { autoAlpha: 0 }, { autoAlpha: 1, duration: 0.2, ease: SPRING_EASE })
      .fromTo(pn, { autoAlpha: 0, scale: 0.95, y: 20 }, { autoAlpha: 1, scale: 1, y: 0, duration: 0.3, ease: SPRING_EASE }, '-=0.1')
    return new Promise(resolve => tl.eventCallback('onComplete', resolve))
  }

  async function modalLeave(backdrop: Element | string, panel: Element | string): Promise<void> {
    if (!shouldAnimate()) return
    const bd = resolveEl(backdrop)
    const pn = resolveEl(panel)
    if (!bd || !pn) return
    const tl = gsap.timeline({ force3D: true })
    tl.to(pn, { autoAlpha: 0, scale: 0.95, y: 10, duration: 0.2, ease: SPRING_EASE })
      .to(bd, { autoAlpha: 0, duration: 0.15, ease: SPRING_EASE }, '-=0.1')
    return new Promise(resolve => tl.eventCallback('onComplete', resolve))
  }

  async function toggleOn(el: Element | string): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    gsap.set(resolved, { scale: 0.85, autoAlpha: 0 })
    return new Promise(resolve => gsap.to(resolved, { scale: 1, autoAlpha: 1, duration: 0.25, ease: SPRING_EASE, force3D: true, onComplete: resolve }))
  }

  async function toggleOff(el: Element | string): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    return new Promise(resolve => gsap.to(resolved, { scale: 0.85, autoAlpha: 0, duration: 0.2, ease: SPRING_EASE, force3D: true, onComplete: resolve }))
  }

  async function slideIn(el: Element | string, direction: 'left' | 'right' | 'top' | 'bottom' = 'right', distance = 300): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    const from: Record<string, number> = {}
    const to: Record<string, number> = {}
    if (direction === 'left') { from.x = -distance; to.x = 0 }
    else if (direction === 'right') { from.x = distance; to.x = 0 }
    else if (direction === 'top') { from.y = -distance; to.y = 0 }
    else { from.y = distance; to.y = 0 }
    gsap.set(resolved, { ...from, autoAlpha: 0 })
    return new Promise(resolve => gsap.to(resolved, { ...to, autoAlpha: 1, duration: FAST_DURATION, ease: SPRING_EASE, force3D: true, onComplete: resolve }))
  }

  async function slideOut(el: Element | string, direction: 'left' | 'right' | 'top' | 'bottom' = 'right', distance = 300): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    const to: Record<string, number> = {}
    if (direction === 'left') to.x = -distance
    else if (direction === 'right') to.x = distance
    else if (direction === 'top') to.y = -distance
    else to.y = distance
    return new Promise(resolve => gsap.to(resolved, { ...to, autoAlpha: 0, duration: FAST_DURATION, ease: SPRING_EASE, force3D: true, onComplete: resolve }))
  }

  async function countUp(el: Element | string, target: number, options?: { duration?: number; prefix?: string; suffix?: string }): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    const { duration = 1, prefix = '', suffix = '' } = options || {}
    const setText = gsap.quickSetter(resolved, 'textContent')
    const obj = { val: 0 }
    return new Promise(resolve => {
      gsap.to(obj, {
        val: target,
        duration,
        ease: SPRING_EASE,
        onUpdate: () => setText(`${prefix}${Math.round(obj.val)}${suffix}`),
        onComplete: resolve,
      })
    })
  }

  function magnify(el: Element | string, scale = 1.5) {
    const resolved = resolveEl(el)
    const tl = gsap.timeline({ paused: true, force3D: true })
    if (resolved) tl.to(resolved, { scale, duration: 0.2, ease: SPRING_EASE, force3D: true })
    return {
      enter: () => { if (shouldAnimate()) tl.play() },
      leave: () => { if (shouldAnimate()) tl.reverse() },
      kill: () => tl.kill(),
    }
  }

  async function dropdownEnter(el: Element | string): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    gsap.set(resolved, { autoAlpha: 0, y: -8, scaleY: 0.8, transformOrigin: 'top center' })
    return new Promise(resolve => gsap.to(resolved, { autoAlpha: 1, y: 0, scaleY: 1, duration: 0.2, ease: SPRING_EASE, force3D: true, onComplete: resolve }))
  }

  async function dropdownLeave(el: Element | string): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    return new Promise(resolve => gsap.to(resolved, { autoAlpha: 0, y: -8, scaleY: 0.8, duration: 0.15, ease: SPRING_EASE, force3D: true, onComplete: resolve }))
  }

  async function animateProgress(el: Element | string, from: number, to: number, options?: { duration?: number }): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    const { duration = 0.6 } = options || {}
    gsap.set(resolved, { width: `${from}%` })
    return new Promise(resolve => gsap.to(resolved, { width: `${to}%`, duration, ease: SPRING_EASE, onComplete: resolve }))
  }

  async function pulse(el: Element | string): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    const tl = gsap.timeline({ force3D: true })
    tl.to(resolved, { scale: 1.08, duration: 0.15, ease: SPRING_EASE })
      .to(resolved, { scale: 1, duration: 0.25, ease: SPRING_EASE })
    return new Promise(resolve => tl.eventCallback('onComplete', resolve))
  }

  async function shake(el: Element | string): Promise<void> {
    if (!shouldAnimate()) return
    const resolved = resolveEl(el)
    if (!resolved) return
    const tl = gsap.timeline()
    tl.to(resolved, { x: -6, duration: 0.05 })
      .to(resolved, { x: 6, duration: 0.05 })
      .to(resolved, { x: -4, duration: 0.05 })
      .to(resolved, { x: 4, duration: 0.05 })
      .to(resolved, { x: -2, duration: 0.05 })
      .to(resolved, { x: 2, duration: 0.05 })
      .to(resolved, { x: 0, duration: 0.05 })
    return new Promise(resolve => tl.eventCallback('onComplete', resolve))
  }

  function killTweens(el: Element | string): void {
    const resolved = resolveEl(el)
    if (!resolved) return
    gsap.killTweensOf(resolved)
  }

  return {
    fadeIn, fadeOut, staggerIn, modalEnter, modalLeave, toggleOn, toggleOff,
    slideIn, slideOut, countUp, pulse, shake, magnify, dropdownEnter, dropdownLeave,
    animateProgress, killTweens, isAnimating,
  }
}
