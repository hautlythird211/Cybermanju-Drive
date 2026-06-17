<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import gsap from 'gsap'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

export interface Notification {
  id: string
  title?: string
  message: string
  variant?: 'default' | 'accent' | 'danger' | 'success' | 'warning' | 'info' | 'pink' | 'gothic' | 'cute'
  icon?: string
  duration?: number
  action?: { label: string; onClick: () => void }
}

const props = withDefaults(defineProps<{
  notification: Notification
  index?: number
}>(), {
  index: 0,
})

const emit = defineEmits<{
  dismiss: [id: string]
}>()

const notificationRef = ref<HTMLElement | null>(null)
const isVisible = ref(false)
const isExiting = ref(false)
const gsapCtx = ref<gsap.Context | null>(null)

const cls = computed(() => [
  'os-notification',
  `os-notification--${props.notification.variant || 'default'}`,
  { 'os-notification--visible': isVisible.value, 'os-notification--exiting': isExiting.value },
])

const offsetStyle = computed(() => ({
  marginTop: props.index > 0 ? `${props.index * 8}px` : undefined,
}))

onMounted(() => {
  gsapCtx.value = gsap.context(() => {
    requestAnimationFrame(() => {
      isVisible.value = true
      if (notificationRef.value) anim.slideIn(notificationRef.value, 'right')
    })
    const dur = props.notification.duration ?? 4000
    if (dur > 0) {
      setTimeout(() => dismiss(), dur)
    }
  })
})

async function dismiss() {
  isExiting.value = true
  if (notificationRef.value) {
    gsapCtx.value?.add(() => {
      anim.slideOut(notificationRef.value!, 'right')
    })
  }
  emit('dismiss', props.notification.id)
}

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div
    ref="notificationRef"
    :class="cls"
    :style="offsetStyle"
    role="alert"
    aria-live="polite"
  >
    <OsIcon v-if="notification.icon" :icon="notification.icon" :size="16" class="os-notification__icon" />
    <div class="os-notification__body">
      <div v-if="notification.title" class="os-notification__title">{{ notification.title }}</div>
      <div class="os-notification__message">{{ notification.message }}</div>
    </div>
    <button v-if="notification.action" class="os-notification__action" @click="notification.action.onClick">
      {{ notification.action.label }}
    </button>
    <button class="os-notification__close" @click="dismiss" aria-label="Dismiss notification">
      <OsIcon icon="mdi:close" :size="12" />
    </button>
  </div>
</template>

<style scoped>
.os-notification {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  border-radius: var(--radius-lg);
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
  box-shadow: var(--shadow-dropdown);
  transition: all 0.2s ease;
  transform: translateX(100%);
  opacity: 0;
  max-width: 380px;
  pointer-events: auto;
  will-change: transform, opacity;
}

.os-notification--visible {
  transform: translateX(0);
  opacity: 1;
}

.os-notification--exiting {
  transform: translateX(100%);
  opacity: 0;
}

.os-notification--default {
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
}

.os-notification--accent {
  background: var(--accent-dim);
  border: 1px solid rgba(0, 255, 65, 0.2);
  color: var(--text-accent);
}

.os-notification--danger {
  background: var(--danger-dim);
  border: 1px solid rgba(255, 95, 87, 0.2);
  color: var(--text-danger);
}

.os-notification--success {
  background: var(--success-dim);
  border: 1px solid rgba(40, 200, 64, 0.2);
  color: var(--text-success);
}

.os-notification--warning {
  background: var(--warning-dim);
  border: 1px solid rgba(254, 188, 46, 0.2);
  color: var(--text-warning);
}

.os-notification--info {
  background: var(--info-dim);
  border: 1px solid rgba(90, 240, 255, 0.2);
  color: var(--text-info);
}

.os-notification--pink {
  background: var(--pink-dim);
  border: 1px solid rgba(255, 107, 157, 0.2);
  color: var(--text-pink);
}

.os-notification--gothic {
  background: rgba(30, 5, 10, 0.9);
  border: 1px solid #3a1a1a;
  color: #d4a0b0;
}

.os-notification--cute {
  background: linear-gradient(135deg, rgba(255, 107, 157, 0.1), rgba(179, 136, 255, 0.1));
  border: 1px solid rgba(255, 107, 157, 0.2);
  color: var(--text-pink);
  border-radius: var(--radius-xl);
}

.os-notification__icon { flex-shrink: 0; margin-top: 1px; }

.os-notification__body { flex: 1; min-width: 0; }

.os-notification__title {
  font-weight: 700;
  margin-bottom: 2px;
}

.os-notification__message {
  line-height: 1.4;
}

.os-notification__action {
  flex-shrink: 0;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  font-weight: 700;
  text-transform: uppercase;
  padding: 3px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  background: var(--bg-overlay);
  border: 1px solid var(--border-subtle);
  color: inherit;
  transition: all var(--transition-fast);
}
.os-notification__action:hover {
  background: var(--border-medium);
}

.os-notification__close {
  flex-shrink: 0;
  cursor: pointer;
  color: var(--text-muted);
  background: none;
  border: none;
  padding: 2px;
  transition: color var(--transition-fast);
}
.os-notification__close:hover { color: var(--text-primary); }
</style>
