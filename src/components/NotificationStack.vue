<template>
  <Teleport to="body">
    <div class="notification-stack" role="alert" aria-live="polite">
      <TransitionGroup name="nstack">
        <div
          v-for="n in notifications"
          :key="n.id"
          class="notification-item"
          :class="'notif-' + n.type"
        >
          <span class="notif-icon">{{ ICONS[n.type] }}</span>
          <span class="notif-msg">{{ n.message }}</span>
          <button
            v-if="n.action"
            class="notif-action-btn"
            @click.stop="handleAction(n)"
          >{{ n.action.label }}</button>
          <button class="notif-close" @click.stop="dismiss(n.id)">X</button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useNotifications, type NotificationType, type Notification } from '@/composables/useNotifications'

const { notifications, dismiss } = useNotifications()

const ICONS: Record<NotificationType, string> = {
  success: '[$]',
  error: '[!]',
  warning: '[?]',
  info: '[*]',
}

function handleAction(n: Notification) {
  n.action?.handler()
  dismiss(n.id)
}
</script>

<style scoped>
.notification-stack {
  position: fixed;
  bottom: 36px;
  right: 12px;
  z-index: 10000;
  display: flex;
  flex-direction: column-reverse;
  gap: 6px;
  pointer-events: none;
  max-width: 400px;
}

.notification-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: #FFFFFF;
  border: 2px solid #000000;
  box-shadow: 3px 3px 0 #000000;
  color: #000000;
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  cursor: default;
  pointer-events: auto;
}

.notif-error {
  border-color: #000000;
  border-width: 3px;
}

.notif-success {
  border-color: #000000;
}

.notif-warning {
  border-color: #000000;
}

.notif-icon {
  font-size: 12px;
  flex-shrink: 0;
}

.notif-msg {
  flex: 1;
  line-height: 1.3;
}

.notif-action-btn {
  background: #000000;
  border: 2px solid #000000;
  color: #FFFFFF;
  padding: 4px 10px;
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 700;
  cursor: pointer;
  flex-shrink: 0;
  letter-spacing: 0.5px;
  transition: all 0.1s;
}

.notif-action-btn:hover {
  background: #FFFFFF;
  color: #000000;
}

.notif-close {
  background: none;
  border: 2px solid #000000;
  color: #000000;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  font-weight: 700;
  cursor: pointer;
  flex-shrink: 0;
  font-family: var(--font-mono);
}

.notif-close:hover {
  background: #000000;
  color: #FFFFFF;
}

.nstack-enter-active,
.nstack-leave-active {
  transition: all 0.25s ease;
}

.nstack-enter-from {
  opacity: 0;
  transform: translateX(40px);
}

.nstack-leave-to {
  opacity: 0;
  transform: translateX(40px);
}
</style>
