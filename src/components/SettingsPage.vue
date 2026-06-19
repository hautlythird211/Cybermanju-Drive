<template>
  <OsPanel variant="default" padding="md">
    <OsSection :ref="(el) => setSectionRef(0, el)" title="SETTINGS" icon="mdi:cog-outline" variant="neon" spaced>
      <OsSection :ref="(el) => setSectionRef(1, el)" title="DISPLAY" icon="mdi:monitor-screenshot" collapsible>
        <OsCard variant="glass" padding="md">
          <div class="setting-row">
            <span class="setting-label">DEFAULT VIEW</span>
            <OsSelect
              v-model="store.viewMode"
              :options="viewModeOptions"
              variant="neon"
              size="sm"
              style="width:120px;"
            />
          </div>
          <OsDivider spacing="sm" />
          <div class="setting-row">
            <span class="setting-label">MATRIX RAIN</span>
            <OsToggle v-model="store.matrixRainEnabled" variant="neon" size="md" />
          </div>
          <OsDivider spacing="sm" />
          <div class="setting-row">
            <span class="setting-label">SIDEBAR</span>
            <OsToggle v-model="store.sidebarCollapsed" variant="neon" size="md" label="COLLAPSED" />
          </div>
        </OsCard>
      </OsSection>

      <OsDivider />

      <OsSection :ref="(el) => setSectionRef(7, el)" title="WINDOW TILING" icon="mdi:view-grid-outline" collapsible>
        <OsCard variant="glass" padding="md">
          <div class="setting-row">
            <span class="setting-label">AUTO-ARRANGE</span>
            <OsToggle v-model="store.autoArrange" variant="neon" size="md" label="4-QUADRANT" />
          </div>
          <OsDivider spacing="sm" />
          <p class="text-muted">WHEN ENABLED, WINDOWS AUTO-TILE INTO 4 EQUAL QUADRANTS PER VIRTUAL SCREEN. THE SCREEN IS SPLIT: TOP-LEFT, TOP-RIGHT, BOTTOM-LEFT, BOTTOM-RIGHT.</p>
          <OsDivider spacing="sm" />
          <div class="setting-row">
            <span class="setting-label">ARRANGE NOW</span>
            <OsButton variant="neon" size="sm" @click="wm.arrangeWindows()">RE-TILE ALL</OsButton>
          </div>
        </OsCard>
      </OsSection>

      <OsDivider />

      <OsSection :ref="(el) => setSectionRef(2, el)" title="ABOUT" icon="mdi:information-outline" collapsible>
        <OsCard variant="glass" padding="md">
          <div class="info-row"><span class="info-key">VERSION</span><span class="info-value">0.1.0</span></div>
          <div class="info-row"><span class="info-key">FRAMEWORK</span><span class="info-value">VUE 3 + PINIA</span></div>
          <div class="info-row"><span class="info-key">DESKTOP</span><span class="info-value">TAURI V2</span></div>
          <div class="info-row"><span class="info-key">SEARCH</span><span class="info-value">TANTIVY BM25</span></div>
          <div class="info-row"><span class="info-key">ENCRYPTION</span><span class="info-value">RUSTPQ (PQC)</span></div>
          <div class="info-row"><span class="info-key">DATABASE</span><span class="info-value">REDB</span></div>
        </OsCard>
      </OsSection>

      <OsDivider />

      <OsSection :ref="(el) => setSectionRef(3, el)" title="CONNECTION" icon="mdi:server" collapsible>
        <OsCard variant="glass" padding="md">
          <div class="info-row"><span class="info-key">MODE</span><span class="info-value">{{ isWebMode() ? 'WEB / REST' : 'TAURI DESKTOP' }}</span></div>
          <div class="info-row"><span class="info-key">API URL</span><span class="info-value mono">HTTP://LOCALHOST:3456</span></div>
        </OsCard>
      </OsSection>

      <OsDivider />

      <OsSection :ref="(el) => setSectionRef(4, el)" title="AUTO-REFRESH" icon="mdi:refresh" collapsible>
        <OsCard variant="glass" padding="md">
          <div class="setting-row">
            <span class="setting-label">INTERVAL</span>
            <OsSelect
              :modelValue="String(store.autoRefreshInterval)"
              @update:modelValue="store.autoRefreshInterval = Number($event)"
              :options="refreshIntervalOptions"
              variant="neon"
              size="sm"
              style="width:120px;"
            />
          </div>
        </OsCard>
      </OsSection>

      <OsDivider />

      <OsSection :ref="(el) => setSectionRef(5, el)" title="DATA MANAGEMENT" icon="mdi:database" collapsible>
        <OsCard variant="glass" padding="md">
          <OsButton variant="neon" block @click="handleRefresh">REFRESH ALL DATA</OsButton>
          <p class="text-muted" style="margin-top:4px;">RE-FETCH FILES, ACCOUNTS, COLLECTIONS, FACE GROUPS, AND SYNC CONFIGS.</p>
        </OsCard>
      </OsSection>

      <OsDivider v-if="touchConfig" />

      <OsSection v-if="touchConfig" :ref="(el) => setSectionRef(6, el)" title="TOUCH GESTURES" icon="mdi:gesture-tap" collapsible>
        <OsCard variant="glass" padding="md">
          <p class="text-muted">DEVICE: {{ (touchConfig as any).state?.touchSupported ? 'TOUCH ENABLED' : 'NO TOUCH' }} | {{ (touchConfig as any).state?.isMobile ? 'MOBILE' : 'DESKTOP' }}</p>
          <div class="gesture-table" style="margin-top:8px;">
            <div v-for="gesture in touchConfig.getAllGestures()" :key="gesture" class="gesture-row">
              <span class="gesture-label">{{ touchConfig.getGestureLabel(gesture) }}</span>
              <select
                class="bw-input"
                style="flex:1;"
                :value="touchConfig.getAction(gesture)"
                @change="onGestureChange(gesture, ($event.target as HTMLSelectElement).value)"
              >
                <option v-for="a in touchConfig.getAllActions()" :key="a" :value="a">{{ touchConfig.getActionLabel(a) }}</option>
              </select>
              <OsButton variant="ghost" size="xs" @click="touchConfig.resetGesture(gesture)">R</OsButton>
            </div>
          </div>
        </OsCard>
      </OsSection>
    </OsSection>
  </OsPanel>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import { useAppStore } from '@/stores/app'
import { useTouchConfig } from '@/composables/useTouchConfig'
import { useWindowManager } from '@/composables/useWindowManager'
import { isWebMode } from '@/composables/useTauri'
import { useGsapAnimation } from '@/composables/useGsapAnimation'
import { OsPanel, OsSection, OsCard, OsButton, OsInput, OsSelect, OsToggle, OsBadge, OsDivider } from '@/components/ui'
import type { SelectOption } from '@/components/ui'

const anim = useGsapAnimation()
const store = useAppStore()
const wm = useWindowManager()
const touchConfig = useTouchConfig({ autoDetect: true })

const sectionRefs = ref<(HTMLElement | null)[]>([])
const gsapCtx = ref<gsap.Context | null>(null)

function setSectionRef(idx: number, el: any) {
  sectionRefs.value[idx] = el as HTMLElement
}

const viewModeOptions = computed<SelectOption[]>(() => [
  { value: 'grid', label: 'GRID' },
  { value: 'list', label: 'LIST' },
  { value: 'masonry', label: 'MASONRY' },
])

const refreshIntervalOptions = computed<SelectOption[]>(() => [
  { value: '0', label: 'DISABLED' },
  { value: '10', label: '10 SECONDS' },
  { value: '30', label: '30 SECONDS' },
  { value: '60', label: '1 MINUTE' },
  { value: '300', label: '5 MINUTES' },
])

function handleRefresh() {
  store.fetchFiles()
  store.fetchAccounts()
  store.fetchCollections()
  store.fetchFaceGroups()
  store.fetchSyncConfigs()
}

function onGestureChange(gesture: string, action: string) {
  touchConfig.setAction(gesture as any, action as any)
}

onMounted(async () => {
  await nextTick()
  gsapCtx.value = gsap.context(() => {
    const sections = sectionRefs.value.filter(Boolean) as HTMLElement[]
    if (sections.length > 0) {
      anim.staggerIn(sections, { stagger: 0.08, from: 'start', duration: 0.3 })
    }
  })
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<style scoped>
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 4px 0;
}

.setting-label {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 0;
  border-bottom: 1px solid var(--border-glass);
}

.info-row:last-child { border-bottom: none; }

.info-key {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-transform: uppercase;
}

.info-value {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  color: var(--text-accent);
  font-weight: 600;
}

.info-value.mono { font-family: var(--font-mono); }

.gesture-table {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.gesture-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.gesture-label {
  flex: 1;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
}

.gs-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.gs-label {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  min-width: 110px;
}

.text-muted { color: var(--text-muted) !important; }

/* Enhanced glass sections */
.settings-section {
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  background: var(--bg-glass-heavy);
  border: 1px solid var(--border-subtle);
  box-shadow: var(--shadow-glass), var(--panel-inset);
  transition: transform var(--duration-fast) cubic-bezier(0.22, 1, 0.36, 1),
              box-shadow var(--duration-fast) cubic-bezier(0.22, 1, 0.36, 1);
  contain: layout style;
}

.settings-section:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-elevated), var(--panel-inset-strong);
}

.settings-section-header {
  position: relative;
  border-bottom: 1px solid var(--border-subtle);
}

.settings-section-header::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 0;
  width: 60px;
  height: 2px;
  background: var(--accent);
  box-shadow: 0 0 8px var(--accent-glow);
  border-radius: 1px;
}

/* Setting rows */
.setting-row {
  transition: background var(--duration-fast) cubic-bezier(0.22, 1, 0.36, 1);
  border-radius: 6px;
}

.setting-row:hover {
  background: var(--overlay-light);
}

/* Select/slider enhancements */
.setting-select, .setting-slider {
  transition: all var(--duration-fast) cubic-bezier(0.22, 1, 0.36, 1);
}

/* Menu settings with hover lift */
.menu-setting-card {
  transition: transform var(--duration-fast) cubic-bezier(0.22, 1, 0.36, 1),
              box-shadow var(--duration-fast) cubic-bezier(0.22, 1, 0.36, 1);
  contain: layout style;
}

.menu-setting-card:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-elevated);
}
</style>
