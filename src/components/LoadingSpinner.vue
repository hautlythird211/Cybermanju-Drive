<template>
  <div class="loading-spinner" role="status" aria-label="Loading">
    <Icon v-if="size === 'sm'" icon="svg-spinners:eclipse" width="16" height="16" />
    <Icon v-else-if="size === 'lg'" icon="svg-spinners:180-ring" width="36" height="36" />
    <Icon v-else icon="svg-spinners:180-ring-with-bg" width="24" height="24" />
    <span v-if="label" class="spinner-label">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
import { Icon } from '@iconify/vue'

const props = withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg'
  label?: string
}>(), {
  size: 'md',
})
</script>

<style scoped>
.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.spinner-label {
  font-family: 'Courier New', monospace;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.5);
}
</style>
