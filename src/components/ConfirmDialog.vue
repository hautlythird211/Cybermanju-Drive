<template>
  <OsModal
    :visible="visible"
    :title="title"
    variant="glass"
    size="sm"
    :closable="true"
    @update:visible="$emit('update:visible', $event)"
    @close="$emit('cancel')"
  >
    <div class="confirm-body">{{ message }}</div>
    <template #footer>
      <OsButton variant="ghost" size="sm" @click="handleCancel">{{ cancelText }}</OsButton>
      <OsButton variant="neon" size="sm" @click="handleConfirm">{{ confirmText }}</OsButton>
    </template>
  </OsModal>
</template>

<script setup lang="ts">
import { OsModal, OsButton } from '@/components/ui'

const props = withDefaults(defineProps<{
  visible: boolean
  title?: string
  message?: string
  confirmText?: string
  cancelText?: string
}>(), {
  title: 'CONFIRM',
  message: 'ARE YOU SURE?',
  confirmText: '[YES]',
  cancelText: '[CANCEL]',
})

const emit = defineEmits<{
  confirm: []
  cancel: []
  'update:visible': [value: boolean]
}>()

function handleConfirm() { emit('confirm'); emit('update:visible', false) }
function handleCancel() { emit('cancel'); emit('update:visible', false) }
</script>

<style scoped>
.confirm-body {
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
  color: var(--text-secondary);
  line-height: 1.5;
  padding: 8px 0;
}

.dialog-overlay {
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
}

.dialog-panel {
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  background: var(--bg-glass-heavy);
  border: 1px solid var(--border-subtle);
  box-shadow: var(--shadow-elevated), var(--glow-accent), var(--panel-inset);
  contain: layout style;
}

.dialog-title {
  background: linear-gradient(135deg, var(--accent), var(--accent-dim));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.dialog-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
