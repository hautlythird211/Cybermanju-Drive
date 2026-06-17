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
</style>
