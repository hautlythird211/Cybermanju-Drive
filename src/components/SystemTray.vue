<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

const time = ref('')
const date = ref('')
const showCalendar = ref(false)
const showNetwork = ref(false)
const showVolume = ref(false)
const showBattery = ref(false)
const volume = ref(68)
const batteryPercent = ref(83)
const batteryCharging = ref(true)
const wifiConnected = ref(true)
const wifiSignal = ref(4)
const notifications = ref(3)

let timer: ReturnType<typeof setInterval> | null = null

function updateClock() {
  const now = new Date()
  time.value = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  date.value = now.toLocaleDateString([], { weekday: 'short', month: 'short', day: 'numeric' })
}

onMounted(() => {
  updateClock()
  timer = setInterval(updateClock, 10000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div class="system-tray">
    <!-- Notifications -->
    <div class="tray-item" title="Notifications" @click="showCalendar = !showCalendar">
      <span class="tray-icon">🔔</span>
      <span v-if="notifications > 0" class="tray-badge">{{ notifications }}</span>
    </div>

    <!-- Network -->
    <div class="tray-item" title="Network" @click="showNetwork = !showNetwork">
      <span class="tray-icon">{{ wifiConnected ? '📶' : '📡' }}</span>
    </div>

    <!-- Volume -->
    <div class="tray-item" title="Volume" @click="showVolume = !showVolume">
      <span class="tray-icon">{{ volume > 50 ? '🔊' : volume > 0 ? '🔉' : '🔇' }}</span>
    </div>
    <div v-if="showVolume" class="tray-popup" @click.self="showVolume = false">
      <div class="popup-panel">
        <div class="popup-header">Volume</div>
        <input type="range" v-model.number="volume" min="0" max="100" class="tray-slider" />
        <div class="popup-value">{{ volume }}%</div>
      </div>
    </div>

    <!-- Battery -->
    <div class="tray-item" title="Battery" @click="showBattery = !showBattery">
      <span class="tray-icon">{{ batteryCharging ? '⚡' : batteryPercent > 20 ? '🔋' : '🪫' }}</span>
      <span class="tray-percent">{{ batteryPercent }}%</span>
    </div>

    <!-- Separator -->
    <div class="tray-separator"></div>

    <!-- Clock -->
    <div class="tray-item tray-clock" title="Calendar" @click="showCalendar = !showCalendar">
      <span class="tray-time">{{ time }}</span>
      <span class="tray-date">{{ date }}</span>
    </div>

    <!-- Calendar Popup -->
    <div v-if="showCalendar" class="tray-popup" @click.self="showCalendar = false">
      <div class="popup-panel popup-calendar">
        <div class="popup-header">{{ new Date().toLocaleDateString([], { month: 'long', year: 'numeric' }) }}</div>
        <div class="calendar-grid">
          <span v-for="d in ['M','T','W','T','F','S','S']" :key="d" class="cal-day-header">{{ d }}</span>
          <span v-for="n in 31" :key="n" class="cal-day"
            :class="{ 'cal-today': n === new Date().getDate() }">{{ n }}</span>
        </div>
        <div class="calendar-events">
          <div class="event-row"><span class="event-dot"></span> System update available</div>
          <div class="event-row"><span class="event-dot"></span> Sync pending (3 files)</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.system-tray {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 100%;
  position: relative;
}

.tray-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  height: 100%;
  cursor: pointer;
  transition: background 0.15s;
  position: relative;
}

.tray-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.tray-icon {
  font-size: 12px;
  line-height: 1;
}

.tray-badge {
  position: absolute;
  top: 2px;
  right: 2px;
  background: #ff5f57;
  color: #fff;
  font-size: 7px;
  font-weight: 700;
  padding: 1px 4px;
  border-radius: 6px;
  line-height: 1;
  min-width: 14px;
  text-align: center;
}

.tray-percent {
  font-size: 9px;
  color: #888;
  font-family: 'Courier New', monospace;
}

.tray-separator {
  width: 1px;
  height: 16px;
  background: #1a1a1a;
  margin: 0 4px;
}

.tray-clock {
  flex-direction: column;
  align-items: flex-end;
  gap: 0;
  line-height: 1.2;
}

.tray-time {
  font-size: 11px;
  font-weight: 700;
  color: #ddd;
  font-family: 'Courier New', monospace;
}

.tray-date {
  font-size: 8px;
  color: #555;
  letter-spacing: 0.5px;
}

.tray-popup {
  position: fixed;
  inset: 0;
  z-index: 99999;
}

.popup-panel {
  position: absolute;
  right: 60px;
  bottom: 48px;
  width: 240px;
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 10px;
  padding: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
  z-index: 100000;
}

.popup-header {
  font-size: 11px;
  font-weight: 700;
  color: #ddd;
  letter-spacing: 1px;
  margin-bottom: 12px;
}

.tray-slider {
  width: 100%;
  -webkit-appearance: none;
  height: 4px;
  background: #1a1a1a;
  border-radius: 2px;
  outline: none;
}

.tray-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #00ff41;
  cursor: pointer;
  box-shadow: 0 0 6px rgba(0, 255, 65, 0.3);
}

.popup-value {
  font-size: 10px;
  color: #555;
  text-align: center;
  margin-top: 6px;
  font-family: 'Courier New', monospace;
}

.popup-calendar {
  width: 280px;
}

.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
  margin-bottom: 12px;
}

.cal-day-header {
  font-size: 8px;
  color: #444;
  text-align: center;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 4px 0;
}

.cal-day {
  font-size: 10px;
  color: #666;
  text-align: center;
  padding: 4px 0;
  border-radius: 4px;
}

.cal-today {
  color: #00ff41;
  font-weight: 700;
  background: rgba(0, 255, 65, 0.08);
  box-shadow: 0 0 4px rgba(0, 255, 65, 0.15);
}

.calendar-events {
  border-top: 1px solid #1a1a1a;
  padding-top: 8px;
}

.event-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 9px;
  color: #555;
  padding: 4px 0;
}

.event-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: #febc2e;
  flex-shrink: 0;
}
</style>
