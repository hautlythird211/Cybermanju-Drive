<template>
  <div class="plugin-creator">
    <div class="pc-sidebar">
      <div class="pc-sidebar-header">
        <span class="pc-brand">PLUGINS</span>
        <button class="pc-add-btn" @click="createPlugin" title="New Plugin">+</button>
      </div>
      <div class="pc-list">
        <div
          v-for="plugin in plugins"
          :key="plugin.id"
          class="pc-item"
          :class="{ active: selectedPluginId === plugin.id }"
          @click="selectPlugin(plugin.id)"
        >
          <span class="pc-item-icon">{{ plugin.icon }}</span>
          <div class="pc-item-info">
            <div class="pc-item-name">{{ plugin.name || 'Untitled Plugin' }}</div>
            <div class="pc-item-type">{{ plugin.type }}</div>
          </div>
        </div>
        <div v-if="plugins.length === 0" class="pc-empty">
          <p>No plugins yet</p>
        </div>
      </div>
    </div>

    <div class="pc-editor">
      <div v-if="selectedPlugin" class="pc-editor-inner">
        <div class="pc-editor-header">
          <input v-model="selectedPlugin.name" class="pc-name-input" placeholder="Plugin name..." @blur="savePlugins" />
          <div class="pc-editor-actions">
            <button class="pc-action-btn" @click="exportPlugin" title="Export .plugin file">Export</button>
            <button class="pc-action-btn pc-action-delete" @click="deletePlugin" title="Delete">Delete</button>
          </div>
        </div>

        <div class="pc-config">
          <div class="pc-config-row">
            <label class="pc-label">Type</label>
            <select v-model="selectedPlugin.type" class="pc-select" @change="onTypeChange">
              <option value="action">Action Button</option>
              <option value="panel">Custom Panel</option>
              <option value="sidebar">Sidebar Item</option>
              <option value="tool">Quick Tool</option>
            </select>
          </div>
          <div class="pc-config-row">
            <label class="pc-label">Icon</label>
            <input v-model="selectedPlugin.icon" class="pc-input-sm" placeholder="emoji or mdi:icon" @blur="savePlugins" />
          </div>
          <div class="pc-config-row">
            <label class="pc-label">Color</label>
            <input v-model="selectedPlugin.color" type="color" class="pc-color" @input="savePlugins" />
          </div>
          <div class="pc-config-row">
            <label class="pc-label">Description</label>
            <input v-model="selectedPlugin.description" class="pc-input" placeholder="What does this plugin do?" @blur="savePlugins" />
          </div>
        </div>

        <div class="pc-code-section">
          <div class="pc-code-header">
            <span class="pc-code-label">LUA CODE</span>
            <button class="pc-code-btn" @click="copyCode">Copy</button>
          </div>
          <textarea v-model="selectedPlugin.code" class="pc-code" placeholder="-- Write your Lua plugin code here..." @input="savePlugins"></textarea>
        </div>

        <div class="pc-preview">
          <div class="pc-preview-label">PREVIEW</div>
          <div class="pc-preview-card" :style="{ borderColor: selectedPlugin.color }">
            <span class="pc-preview-icon">{{ selectedPlugin.icon }}</span>
            <span class="pc-preview-name">{{ selectedPlugin.name || 'Untitled' }}</span>
            <span class="pc-preview-desc">{{ selectedPlugin.description || 'No description' }}</span>
          </div>
        </div>
      </div>
      <div v-else class="pc-no-selection">
        <p>Select or create a plugin</p>
        <button class="pc-create-btn" @click="createPlugin">+ New Plugin</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

interface Plugin {
  id: string
  name: string
  icon: string
  color: string
  type: string
  description: string
  code: string
  createdAt: string
}

const plugins = ref<Plugin[]>([])
const selectedPluginId = ref<string | null>(null)
const saveDebounce = ref<ReturnType<typeof setTimeout> | null>(null)

const selectedPlugin = computed(() =>
  plugins.value.find(p => p.id === selectedPluginId.value) || null
)

function createPlugin() {
  const plugin: Plugin = {
    id: `plugin-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
    name: '',
    icon: '🔧',
    color: '#28c840',
    type: 'action',
    description: '',
    code: `-- Cybermanju OS Plugin\n-- This plugin runs when triggered\n\nfunction on_init()\n  -- Called when plugin is loaded\n  print("Plugin loaded!")\nend\n\nfunction on_action()\n  -- Called when user triggers the action\n  print("Plugin action triggered!")\nend\n\nfunction on_shutdown()\n  -- Called when plugin is unloaded\n  print("Plugin unloaded!")\nend\n`,
    createdAt: new Date().toISOString(),
  }
  plugins.value.unshift(plugin)
  selectedPluginId.value = plugin.id
  savePlugins()
}

function selectPlugin(id: string) {
  selectedPluginId.value = id
}

function deletePlugin() {
  if (!selectedPluginId.value) return
  plugins.value = plugins.value.filter(p => p.id !== selectedPluginId.value)
  selectedPluginId.value = plugins.value.length > 0 ? plugins.value[0].id : null
  savePlugins()
}

function onTypeChange() {
  if (!selectedPlugin.value) return
  const t = selectedPlugin.value.type
  if (t === 'action') {
    selectedPlugin.value.code = `-- Action Plugin\nfunction on_init()\n  print("Action plugin loaded")\nend\n\nfunction on_action()\n  -- Your action code here\nend\n`
  } else if (t === 'panel') {
    selectedPlugin.value.code = `-- Panel Plugin\nfunction on_init()\n  print("Panel plugin loaded")\nend\n\nfunction render()\n  -- Return HTML content for the panel\n  return [[\n    <div class="plugin-panel">\n      <h2>My Plugin Panel</h2>\n    </div>\n  ]]\nend\n`
  } else if (t === 'sidebar') {
    selectedPlugin.value.code = `-- Sidebar Plugin\nfunction on_init()\n  print("Sidebar plugin loaded")\nend\n\nfunction get_sidebar_items()\n  return {\n    { label = "My Plugin", action = "open_panel" }\n  }\nend\n`
  } else if (t === 'tool') {
    selectedPlugin.value.code = `-- Quick Tool Plugin\nfunction on_init()\n  print("Tool plugin loaded")\nend\n\nfunction on_tool_click()\n  -- Quick tool action\nend\n`
  }
  savePlugins()
}

async function copyCode() {
  if (!selectedPlugin.value) return
  try { await navigator.clipboard.writeText(selectedPlugin.value.code) } catch {}
}

function exportPlugin() {
  if (!selectedPlugin.value) return
  const data = {
    name: selectedPlugin.value.name,
    icon: selectedPlugin.value.icon,
    color: selectedPlugin.value.color,
    type: selectedPlugin.value.type,
    description: selectedPlugin.value.description,
    code: selectedPlugin.value.code,
    version: '1.0.0',
    author: 'Cybermanju User',
    exportedAt: new Date().toISOString(),
  }
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${selectedPlugin.value.name || 'plugin'}.plugin`
  a.click()
  URL.revokeObjectURL(url)
}

function savePlugins() {
  if (saveDebounce.value) clearTimeout(saveDebounce.value)
  saveDebounce.value = setTimeout(() => {
    try { localStorage.setItem('cybermanju_plugins', JSON.stringify(plugins.value)) } catch {}
  }, 500)
}

function loadPlugins() {
  try {
    const raw = localStorage.getItem('cybermanju_plugins')
    if (raw) {
      plugins.value = JSON.parse(raw)
      if (plugins.value.length > 0) selectedPluginId.value = plugins.value[0].id
    }
  } catch {}
}

onMounted(loadPlugins)
</script>

<style scoped>
.plugin-creator {
  display: flex;
  height: 100%;
  background: var(--bg-surface);
  color: #e0e0e0;
  font-family: var(--font-mono);
}
.pc-sidebar {
  width: 200px;
  background: #111;
  border-right: 1px solid #2a2a2a;
  display: flex;
  flex-direction: column;
}
.pc-sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid #2a2a2a;
  background: #1a1a1a;
}
.pc-brand { font-size: 10px; font-weight: 700; color: #28c840; letter-spacing: 1px; flex: 1; }
.pc-add-btn {
  background: transparent;
  border: 1px solid #333;
  color: #28c840;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
}
.pc-add-btn:hover { border-color: #28c840; background: rgba(40, 200, 64, 0.1); }
.pc-list { flex: 1; overflow-y: auto; padding: 4px; }
.pc-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid transparent;
  margin-bottom: 2px;
  transition: background 0.1s;
}
.pc-item:hover { background: #1a1a1a; border-color: #2a2a2a; }
.pc-item.active { background: rgba(40, 200, 64, 0.05); border-color: rgba(40, 200, 64, 0.2); }
.pc-item-icon { font-size: 14px; }
.pc-item-info { flex: 1; min-width: 0; }
.pc-item-name { font-size: 10px; font-weight: 700; color: #e0e0e0; }
.pc-item-type { font-size: 8px; color: #555; text-transform: uppercase; }
.pc-empty { text-align: center; padding: 20px; color: #555; font-size: 9px; }
.pc-editor { flex: 1; display: flex; flex-direction: column; }
.pc-editor-inner { flex: 1; display: flex; flex-direction: column; overflow-y: auto; }
.pc-editor-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid #2a2a2a;
  background: #111;
}
.pc-name-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #28c840;
  font-size: 13px;
  font-weight: 700;
  font-family: var(--font-mono);
  outline: none;
  padding: 4px 0;
}
.pc-name-input::placeholder { color: #444; }
.pc-editor-actions { display: flex; gap: 4px; }
.pc-action-btn {
  background: transparent;
  border: 1px solid #333;
  color: #aaa;
  padding: 2px 8px;
  font-size: 8px;
  font-family: var(--font-mono);
  font-weight: 700;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.1s;
}
.pc-action-btn:hover { border-color: #555; color: #e0e0e0; background: #1a1a1a; }
.pc-action-delete:hover { border-color: #ff5f57; color: #ff5f57; }
.pc-config {
  padding: 10px 12px;
  border-bottom: 1px solid #2a2a2a;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.pc-config-row { display: flex; align-items: center; gap: 6px; }
.pc-label { font-size: 8px; font-weight: 700; color: #555; letter-spacing: 0.5px; min-width: 60px; }
.pc-select {
  background: #1a1a1a;
  border: 1px solid #333;
  color: #ccc;
  font-size: 9px;
  padding: 3px 6px;
  border-radius: 4px;
  font-family: var(--font-mono);
  cursor: pointer;
}
.pc-input-sm {
  background: #1a1a1a;
  border: 1px solid #333;
  color: #ccc;
  font-size: 9px;
  padding: 3px 6px;
  border-radius: 4px;
  font-family: var(--font-mono);
  width: 80px;
}
.pc-input {
  background: #1a1a1a;
  border: 1px solid #333;
  color: #ccc;
  font-size: 9px;
  padding: 3px 6px;
  border-radius: 4px;
  font-family: var(--font-mono);
  flex: 1;
  min-width: 150px;
}
.pc-color { width: 28px; height: 24px; border: 1px solid #333; border-radius: 4px; cursor: pointer; background: transparent; padding: 0; }
.pc-code-section { flex: 1; display: flex; flex-direction: column; min-height: 200px; }
.pc-code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  border-bottom: 1px solid #2a2a2a;
}
.pc-code-label { font-size: 8px; font-weight: 700; color: #555; letter-spacing: 0.5px; }
.pc-code-btn {
  background: transparent;
  border: 1px solid #333;
  color: #aaa;
  padding: 2px 8px;
  font-size: 8px;
  font-family: var(--font-mono);
  font-weight: 700;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.1s;
}
.pc-code-btn:hover { border-color: #555; color: #e0e0e0; }
.pc-code {
  flex: 1;
  background: #0d0d0d;
  border: none;
  color: #28c840;
  font-size: 11px;
  font-family: var(--font-mono);
  line-height: 1.6;
  resize: none;
  outline: none;
  padding: 12px;
  tab-size: 2;
}
.pc-code::placeholder { color: #333; }
.pc-preview {
  padding: 10px 12px;
  border-top: 1px solid #2a2a2a;
  background: #111;
}
.pc-preview-label { font-size: 8px; font-weight: 700; color: #555; letter-spacing: 0.5px; margin-bottom: 6px; }
.pc-preview-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 6px;
  border-left: 3px solid;
}
.pc-preview-icon { font-size: 16px; }
.pc-preview-name { font-size: 11px; font-weight: 700; color: #e0e0e0; }
.pc-preview-desc { font-size: 9px; color: #666; flex: 1; }
.pc-no-selection { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; }
.pc-no-selection p { font-size: 10px; color: #555; }
.pc-create-btn {
  background: transparent;
  border: 1px solid #333;
  color: #28c840;
  padding: 6px 12px;
  font-size: 9px;
  font-family: var(--font-mono);
  font-weight: 700;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.1s;
}
.pc-create-btn:hover { border-color: #28c840; background: rgba(40, 200, 64, 0.1); }
</style>
