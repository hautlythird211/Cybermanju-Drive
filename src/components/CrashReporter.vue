<script setup lang="ts">
import { ref, onMounted } from 'vue'

const crashLog = ref<string | null>(null)
const showReport = ref(false)

async function checkCrash() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const log = await invoke<string | null>('get_crash_log')
    if (log) {
      crashLog.value = log
      showReport.value = true
    }
  } catch {
    // Not running in Tauri — ignore
  }
}

async function clearCrash() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('clear_crash_log')
  } catch { /* ignore */ }
  showReport.value = false
  crashLog.value = null
}

onMounted(checkCrash)
</script>

<template>
  <Teleport to="body">
    <div v-if="showReport" class="crash-overlay" @click.self="clearCrash">
      <div class="crash-panel">
        <div class="crash-header">
          <span class="crash-icon">!</span>
          <span class="crash-title">Recovery — Previous Crash Detected</span>
        </div>
        <div class="crash-body">
          <p class="crash-desc">
            Cybermanju Drive did not shut down cleanly. The application may have
            encountered an error during startup. If this persists, try the following:
          </p>
          <ul class="crash-tips">
            <li>Run <code>cybermanju-debug.sh</code> (Linux) or <code>cybermanju-debug.cmd</code> (Windows) for detailed logs</li>
            <li>Check <code>crash.log</code> in your app data directory</li>
            <li>Ensure WebView2 / WebKit2GTK runtime is installed</li>
            <li>Report the issue at github.com/cybermanju/cybermanju-drive/issues</li>
          </ul>
          <pre class="crash-log">{{ crashLog }}</pre>
        </div>
        <div class="crash-actions">
          <button class="crash-btn crash-btn-primary" @click="clearCrash">Clear &amp; Continue</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.crash-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999999;
  font-family: 'Courier New', 'Fira Code', monospace;
}

.crash-panel {
  width: 560px;
  max-width: 92vw;
  max-height: 80vh;
  background: #0a0a0a;
  border: 1px solid #ff5f57;
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 0 40px rgba(255, 95, 87, 0.1);
}

.crash-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 20px;
  background: #110000;
  border-bottom: 1px solid #2a0000;
}

.crash-icon {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: #ff5f57;
  color: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 900;
}

.crash-title {
  font-size: 13px;
  font-weight: 800;
  color: #ff5f57;
  letter-spacing: 1px;
}

.crash-body {
  padding: 16px 20px;
  overflow-y: auto;
  flex: 1;
}

.crash-desc {
  font-size: 11px;
  color: #999;
  line-height: 1.5;
  margin: 0 0 12px;
}

.crash-tips {
  font-size: 10px;
  color: #666;
  line-height: 1.8;
  margin: 0 0 12px;
  padding-left: 20px;
}

.crash-tips code {
  color: #00ff41;
  background: rgba(0, 255, 65, 0.06);
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 9px;
}

.crash-log {
  background: #050505;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  padding: 12px;
  font-size: 10px;
  color: #ff5f57;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
  margin: 0;
  line-height: 1.4;
}

.crash-actions {
  padding: 12px 20px;
  border-top: 1px solid #1a1a1a;
  display: flex;
  justify-content: flex-end;
}

.crash-btn {
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #888;
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  padding: 8px 24px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.crash-btn-primary {
  border-color: #ff5f57;
  color: #ff5f57;
}

.crash-btn-primary:hover {
  background: rgba(255, 95, 87, 0.1);
  box-shadow: 0 0 12px rgba(255, 95, 87, 0.15);
}
</style>
