<template>
  <div class="collections-panel">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-col">[*]</span>
        <h2 class="panel-title">COLLECTIONS</h2>
      </div>
    </div>

    <div class="section" v-if="collections.length === 0">
      <p class="text-muted">NO COLLECTIONS YET. CREATE ONE TO GROUP FILES.</p>
    </div>

    <div class="collections-list">
      <div
        v-for="col in collections"
        :key="col.id"
        class="collection-card"
        :class="{ 'drop-target': dragOverCollectionId === col.id }"
        @dragover.prevent="dragOverCollectionId = col.id"
        @dragleave.prevent="dragOverCollectionId = null"
        @drop.prevent="handleDrop(col.id)"
        @click="openCollection(col)"
      >
        <div class="col-header">
          <span class="col-name">{{ col.name }}</span>
          <span class="col-type text-muted">{{ col.collectionType }}</span>
        </div>
        <div class="col-meta text-muted">
          {{ col.itemIds.length }} ITEMS
        </div>
      </div>
    </div>

    <div class="section create-section">
      <h3 class="section-title">[+] CREATE COLLECTION</h3>
      <input v-model="newName" class="bw-input" placeholder="COLLECTION NAME" @keyup.enter="handleCreate" />
      <select v-model="newType" class="bw-input" style="appearance:none;">
        <option value="custom">CUSTOM</option>
        <option value="highlights">HIGHLIGHTS</option>
        <option value="best_moments">BEST MOMENTS</option>
      </select>
      <button class="bw-btn" style="width:100%;" @click="handleCreate">[CREATE]</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAppStore } from '@/stores/app'
import type { CollectionType } from '@/types'

const store = useAppStore()
const collections = computed(() => store.collections)
const newName = ref('')
const newType = ref<CollectionType>('custom')
const dragOverCollectionId = ref<string | null>(null)

async function handleCreate() {
  if (!newName.value.trim()) return
  await store.createCollection(newName.value.trim(), newType.value, '#00ff41')
  newName.value = ''
}

function handleDrop(collectionId: string) {
  dragOverCollectionId.value = null
  const fileId = store.selectedFileId
  if (fileId) {
    store.addToCollection(collectionId, fileId)
  }
}

function openCollection(col: { id: string; name: string; itemIds: string[] }) {
  if (col.itemIds && col.itemIds.length > 0) {
    store.selectedFileId = col.itemIds[0]
    store.currentPanel = 'files'
  } else {
    store.notifySuccess(`Collection: ${col.name} (empty)`)
  }
}
</script>

<style scoped>
.collections-panel {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
  margin-bottom: 16px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-col { font-size: 14px; color: var(--text-accent); }

.panel-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  margin: 0;
  color: var(--text-primary);
}

.section { margin-bottom: 16px; }

.collections-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.collection-card {
  border: 1px solid var(--border-glass);
  padding: 12px;
  transition: all var(--transition-fast);
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-md);
  cursor: pointer;
}

.collection-card:hover {
  border-color: var(--border-accent);
  background: var(--accent-dim);
}

.collection-card.drop-target {
  border-color: var(--border-accent);
  background: var(--accent-dim);
}

.col-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.col-name { font-size: 13px; font-weight: 600; color: var(--text-primary); }
.col-type { font-size: 9px; color: var(--text-muted); }
.col-meta { font-size: 10px; color: var(--text-muted); }

.section-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0 0 8px;
  font-family: var(--font-mono);
}

.create-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.bw-input {
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  padding: 7px 10px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
  border-radius: var(--radius-sm);
}

.bw-input:focus {
  border-color: var(--border-accent);
  outline: none;
}

.bw-btn {
  padding: 7px 14px;
  background: var(--accent);
  color: var(--text-inverse);
  border: 1px solid var(--accent);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.bw-btn:hover { background: #00cc35; border-color: #00cc35; }

.text-muted { color: var(--text-muted) !important; }
</style>
