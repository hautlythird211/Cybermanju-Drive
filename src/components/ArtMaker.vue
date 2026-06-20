<template>
  <div class="art-maker">
    <div class="am-header">
      <span class="am-title">ART MAKER</span>
      <span class="am-subtitle">REAL-TIME CANVAS CONTROL</span>
    </div>

    <div class="am-scroll">
      <!-- Global Controls -->
      <section class="am-section">
        <div class="am-section-title">GLOBAL</div>
        <div class="am-controls-grid">
          <div class="am-knob-group">
            <label>SPEED</label>
            <input type="range" min="0.1" max="3" step="0.05" :value="settings.globalSpeed" @input="setGlobal('globalSpeed', parseFloat(($event.target as HTMLInputElement).value))" />
            <span class="am-value">{{ settings.globalSpeed.toFixed(2) }}</span>
          </div>
          <div class="am-knob-group">
            <label>HUE SHIFT</label>
            <input type="range" min="0" max="360" step="1" :value="settings.globalHueShift" @input="setGlobal('globalHueShift', parseFloat(($event.target as HTMLInputElement).value))" />
            <span class="am-value">{{ settings.globalHueShift }}°</span>
          </div>
          <div class="am-knob-group">
            <label>SATURATION</label>
            <input type="range" min="0" max="100" step="1" :value="settings.globalSaturation" @input="setGlobal('globalSaturation', parseFloat(($event.target as HTMLInputElement).value))" />
            <span class="am-value">{{ settings.globalSaturation }}%</span>
          </div>
          <div class="am-knob-group">
            <label>BRIGHTNESS</label>
            <input type="range" min="0" max="100" step="1" :value="settings.globalBrightness" @input="setGlobal('globalBrightness', parseFloat(($event.target as HTMLInputElement).value))" />
            <span class="am-value">{{ settings.globalBrightness }}%</span>
          </div>
          <div class="am-knob-group">
            <label>OPACITY</label>
            <input type="range" min="0" max="1" step="0.05" :value="settings.opacity" @input="setGlobal('opacity', parseFloat(($event.target as HTMLInputElement).value))" />
            <span class="am-value">{{ (settings.opacity * 100).toFixed(0) }}%</span>
          </div>
        </div>
      </section>

      <!-- Palette + Mouse Mode -->
      <section class="am-section">
        <div class="am-section-title">PALETTE</div>
        <div class="am-palette-grid">
          <button
            v-for="p in PALETTE_PRESETS"
            :key="p.name"
            class="am-palette-btn"
            :class="{ active: settings.palette === p.name }"
            :style="{ '--accent-color': p.accentColor }"
            @click="settings.palette = p.name"
          >
            {{ p.label }}
          </button>
        </div>
      </section>

      <section class="am-section">
        <div class="am-section-title">MOUSE MODE</div>
        <div class="am-palette-grid">
          <button
            v-for="m in MOUSE_MODES"
            :key="m.id"
            class="am-palette-btn am-mouse-btn"
            :class="{ active: settings.mouseMode === m.id }"
            @click="settings.mouseMode = m.id"
          >
            {{ m.label }}
          </button>
        </div>
      </section>

      <!-- Presets -->
      <section class="am-section">
        <div class="am-section-title">PRESETS</div>
        <div class="am-preset-grid">
          <button class="am-preset-btn" @click="applyPreset('psychedelic')">PSYCHEDELIC</button>
          <button class="am-preset-btn" @click="applyPreset('darkAmbient')">DARK AMBIENT</button>
          <button class="am-preset-btn" @click="applyPreset('matrixMode')">MATRIX MODE</button>
          <button class="am-preset-btn" @click="applyPreset('glitchCore')">GLITCH CORE</button>
          <button class="am-preset-btn" @click="applyPreset('heavenly')">HEAVENLY</button>
          <button class="am-preset-btn" @click="applyPreset('neuralDream')">NEURAL DREAM</button>
          <button class="am-preset-btn am-randomize" @click="randomize">RANDOMIZE</button>
          <button class="am-preset-btn am-reset" @click="resetDefaults">RESET</button>
        </div>
      </section>

      <!-- Layers -->
      <section class="am-section">
        <div class="am-section-title">LAYERS</div>
        <div class="am-layers">
          <div
            v-for="(layer, key) in settings.layers"
            :key="key"
            class="am-layer"
            :class="{ disabled: !layer.enabled }"
          >
            <div class="am-layer-header">
              <button
                class="am-layer-toggle"
                :class="{ on: layer.enabled }"
                @click="layer.enabled = !layer.enabled"
              >
                {{ layer.enabled ? 'ON' : 'OFF' }}
              </button>
              <span class="am-layer-name">{{ LAYER_LABELS[key as string] || key }}</span>
            </div>
            <div class="am-layer-controls" v-if="layer.enabled">
              <div class="am-layer-slider">
                <label>OPACITY</label>
                <input type="range" min="0" max="1" step="0.05" :value="layer.opacity" @input="layer.opacity = parseFloat(($event.target as HTMLInputElement).value)" />
                <span class="am-value-sm">{{ (layer.opacity * 100).toFixed(0) }}%</span>
              </div>
              <div class="am-layer-slider">
                <label>SPEED</label>
                <input type="range" min="0.1" max="3" step="0.05" :value="layer.speed" @input="layer.speed = parseFloat(($event.target as HTMLInputElement).value)" />
                <span class="am-value-sm">{{ layer.speed.toFixed(2) }}x</span>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'
import { PALETTE_PRESETS, MOUSE_MODES, LAYER_LABELS, applyPreset as applyPresetFn, randomizeSettings, DEFAULT_SETTINGS } from '@/configs/artMaker'

const store = useAppStore()
const settings = computed(() => store.artSettings)

function setGlobal(key: string, value: number) {
  ;(store.artSettings as any)[key] = value
}

function applyPreset(name: string) {
  store.artSettings = applyPresetFn(store.artSettings, name)
}

function randomize() {
  store.artSettings = randomizeSettings(store.artSettings)
}

function resetDefaults() {
  store.artSettings = { ...DEFAULT_SETTINGS, layers: { ...DEFAULT_SETTINGS.layers } }
}
</script>

<style scoped>
.art-maker {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: rgba(8, 8, 10, 0.92);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  overflow: hidden;
}

.am-header {
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.am-title {
  font-size: 13px;
  font-weight: 800;
  letter-spacing: 3px;
  color: #b388ff;
  text-shadow: 0 0 12px rgba(179, 136, 255, 0.15);
  display: block;
}

.am-subtitle {
  font-size: 8px;
  color: rgba(255, 255, 255, 0.3);
  letter-spacing: 2px;
  margin-top: 2px;
  display: block;
}

.am-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px 24px;
}

.am-scroll::-webkit-scrollbar { width: 4px; }
.am-scroll::-webkit-scrollbar-track { background: transparent; }
.am-scroll::-webkit-scrollbar-thumb { background: rgba(179, 136, 255, 0.2); border-radius: 2px; }

.am-section {
  margin-bottom: 16px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.02);
}

.am-section-title {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 2px;
  color: rgba(255, 255, 255, 0.25);
  margin-bottom: 8px;
}

.am-controls-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.am-knob-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.am-knob-group label {
  font-size: 8px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.35);
  letter-spacing: 1px;
}

.am-knob-group input[type="range"] {
  width: 100%;
  height: 3px;
  -webkit-appearance: none;
  appearance: none;
  background: rgba(179, 136, 255, 0.15);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.am-knob-group input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #b388ff;
  box-shadow: 0 0 6px rgba(179, 136, 255, 0.4);
  cursor: pointer;
}

.am-value {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.4);
  text-align: right;
}

.am-palette-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.am-palette-btn {
  padding: 4px 10px;
  font-family: 'Courier New', monospace;
  font-size: 8px;
  font-weight: 700;
  letter-spacing: 1px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.03);
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: all 0.2s ease;
}

.am-palette-btn:hover {
  border-color: rgba(179, 136, 255, 0.3);
  color: rgba(255, 255, 255, 0.7);
}

.am-palette-btn.active {
  border-color: var(--accent-color, #b388ff);
  background: rgba(179, 136, 255, 0.1);
  color: var(--accent-color, #b388ff);
  box-shadow: 0 0 8px color-mix(in srgb, var(--accent-color, #b388ff) 15%, transparent);
}

.am-mouse-btn {
  min-width: 60px;
}

.am-preset-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.am-preset-btn {
  padding: 5px 12px;
  font-family: 'Courier New', monospace;
  font-size: 8px;
  font-weight: 800;
  letter-spacing: 1.5px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.02);
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  transition: all 0.2s ease;
}

.am-preset-btn:hover {
  border-color: rgba(179, 136, 255, 0.25);
  color: rgba(255, 255, 255, 0.7);
  background: rgba(179, 136, 255, 0.03);
}

.am-randomize {
  border-color: rgba(255, 107, 157, 0.2);
  color: rgba(255, 107, 157, 0.5);
}

.am-randomize:hover {
  border-color: rgba(255, 107, 157, 0.4);
  color: rgba(255, 107, 157, 0.8);
  background: rgba(255, 107, 157, 0.05);
}

.am-reset {
  border-color: rgba(255, 69, 58, 0.15);
  color: rgba(255, 69, 58, 0.4);
}

.am-reset:hover {
  border-color: rgba(255, 69, 58, 0.35);
  color: rgba(255, 69, 58, 0.7);
  background: rgba(255, 69, 58, 0.04);
}

.am-layers {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.am-layer {
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 6px;
  padding: 6px 8px;
  background: rgba(255, 255, 255, 0.01);
  transition: all 0.2s ease;
}

.am-layer.disabled {
  opacity: 0.35;
  border-color: rgba(255, 255, 255, 0.02);
}

.am-layer-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.am-layer-toggle {
  font-family: 'Courier New', monospace;
  font-size: 7px;
  font-weight: 800;
  letter-spacing: 1px;
  padding: 2px 6px;
  border-radius: 3px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  cursor: pointer;
  background: rgba(255, 255, 255, 0.02);
  color: rgba(255, 255, 255, 0.2);
  min-width: 28px;
  transition: all 0.2s ease;
}

.am-layer-toggle.on {
  border-color: rgba(179, 136, 255, 0.3);
  background: rgba(179, 136, 255, 0.1);
  color: #b388ff;
}

.am-layer-toggle:hover {
  border-color: rgba(179, 136, 255, 0.2);
}

.am-layer-name {
  font-size: 9px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 0.5px;
}

.am-layer-controls {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  padding-left: 36px;
}

.am-layer-slider {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 4px;
}

.am-layer-slider label {
  font-size: 7px;
  color: rgba(255, 255, 255, 0.2);
  min-width: 32px;
}

.am-layer-slider input[type="range"] {
  flex: 1;
  height: 2px;
  -webkit-appearance: none;
  appearance: none;
  background: rgba(179, 136, 255, 0.1);
  border-radius: 1px;
  outline: none;
  cursor: pointer;
}

.am-layer-slider input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #b388ff;
  cursor: pointer;
}

.am-value-sm {
  font-size: 7px;
  color: rgba(255, 255, 255, 0.25);
  min-width: 28px;
  text-align: right;
}
</style>
