<script setup lang="ts">
import { computed, ref, watch, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

export interface SidebarSection {
  id: string
  label: string
  icon?: string
  active?: boolean
  badge?: string | number
  disabled?: boolean
  children?: SidebarSection[]
  action: () => void
}

const props = withDefaults(defineProps<{
  sections: SidebarSection[]
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  width?: number
  collapsible?: boolean
  collapsed?: boolean
}>(), {
  variant: 'default',
  width: 200,
  collapsible: false,
  collapsed: false,
})

const emit = defineEmits<{
  'update:collapsed': [value: boolean]
  action: [section: SidebarSection]
}>()

const sidebarRef = ref<HTMLElement | null>(null)
const contentRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const cls = computed(() => [
  'os-sidebar',
  `os-sidebar--${props.variant}`,
  {
    'os-sidebar--collapsed': props.collapsed,
    'os-sidebar--collapsible': props.collapsible,
  },
])

const style = computed(() => ({
  width: props.collapsed ? '40px' : `${props.width}px`,
}))

function toggleCollapse() {
  emit('update:collapsed', !props.collapsed)
}

function onSectionKeydown(e: KeyboardEvent, section: SidebarSection) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault()
    if (!section.disabled) {
      section.action()
      emit('action', section)
    }
  }
}

function onToggleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault()
    toggleCollapse()
  }
}

watch(() => props.collapsed, async (collapsed) => {
  await nextTick()
  if (sidebarRef.value) {
    gsapCtx.value?.add(() => {
      if (collapsed) {
        anim.slideOut(sidebarRef.value!, 'left', props.width - 40)
      } else {
        anim.slideIn(sidebarRef.value!, 'left', props.width - 40)
      }
    })
  }
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div
    ref="sidebarRef"
    :class="cls"
    :style="style"
    role="navigation"
    :aria-label="collapsed ? 'Collapsed sidebar' : 'Sidebar navigation'"
  >
    <div
      v-if="collapsible"
      class="os-sidebar__toggle"
      :aria-expanded="!collapsed"
      aria-controls="sidebar-content"
      role="button"
      tabindex="0"
      @click="toggleCollapse"
      @keydown="onToggleKeydown"
    >
      <OsIcon :icon="collapsed ? 'mdi:chevron-right' : 'mdi:chevron-left'" :size="14" />
    </div>
    <div id="sidebar-content" ref="contentRef">
      <template v-if="!collapsed">
        <div
          v-for="section in sections"
          :key="section.id"
          class="os-sidebar__group"
        >
          <div class="os-sidebar__group-label">{{ section.label }}</div>
          <div
            v-for="child in section.children || [section]"
            :key="child.id"
            class="os-sidebar__item"
            :class="{
              'os-sidebar__item--active': child.active,
              'os-sidebar__item--disabled': child.disabled,
            }"
            role="button"
            :tabindex="child.disabled ? -1 : 0"
            :aria-disabled="child.disabled"
            :aria-current="child.active ? 'page' : undefined"
            @click="!child.disabled && (child.action(), emit('action', child))"
            @keydown="onSectionKeydown($event, child)"
          >
            <OsIcon v-if="child.icon" :icon="child.icon" :size="14" class="os-sidebar__item-icon" />
            <span class="os-sidebar__item-label">{{ child.label }}</span>
            <span v-if="child.badge" class="os-sidebar__item-badge">{{ child.badge }}</span>
          </div>
        </div>
      </template>
      <div v-else class="os-sidebar__icons">
        <div
          v-for="section in sections"
          :key="section.id"
          class="os-sidebar__icon-item"
          :class="{ 'os-sidebar__icon-item--active': section.active }"
          :title="section.label"
          role="button"
          tabindex="0"
          :aria-label="section.label"
          @click="section.action"
          @keydown="onSectionKeydown($event, section)"
        >
          <OsIcon v-if="section.icon" :icon="section.icon" :size="16" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.os-sidebar {
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  flex-shrink: 0;
  transition: width var(--transition-normal);
  position: relative;
  will-change: transform, width;
}

.os-sidebar--default {
  background: var(--bg-surface);
  border-right: 1px solid var(--border-subtle);
  color: var(--text-secondary);
}

.os-sidebar--glass {
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border-right: 1px solid var(--border-glass);
}

.os-sidebar--neon {
  background: var(--bg-surface);
  border-right: 1px solid rgba(0, 255, 65, 0.1);
}

.os-sidebar--gothic {
  background: #0f0505;
  border-right: 1px solid #2a1010;
}

.os-sidebar--cute {
  background: rgba(255, 107, 157, 0.03);
  border-right: 1px solid rgba(255, 107, 157, 0.08);
}

.os-sidebar__toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 28px;
  cursor: pointer;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border-subtle);
  transition: all var(--transition-fast);
}
.os-sidebar__toggle:hover {
  background: var(--bg-overlay);
  color: var(--text-primary);
}

.os-sidebar__group {
  padding: 4px 0;
}

.os-sidebar__group-label {
  padding: 6px 12px;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}

.os-sidebar__item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
  border-radius: var(--radius-sm);
  margin: 0 4px;
  transition: all var(--transition-fast);
}
.os-sidebar__item:hover {
  background: var(--bg-overlay);
  color: var(--text-primary);
}

.os-sidebar__item--active {
  background: var(--accent-dim);
  color: var(--text-accent);
}

.os-sidebar__item--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.os-sidebar__item-icon { flex-shrink: 0; }
.os-sidebar__item-label { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.os-sidebar__item-badge {
  font-size: var(--font-size-xs);
  background: var(--bg-overlay);
  padding: 0 6px;
  border-radius: var(--radius-full);
  line-height: 16px;
  flex-shrink: 0;
}

.os-sidebar__icons {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 0;
  gap: 4px;
}

.os-sidebar__icon-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  cursor: pointer;
  color: var(--text-muted);
  transition: all var(--transition-fast);
}
.os-sidebar__icon-item:hover {
  background: var(--bg-overlay);
  color: var(--text-primary);
}
.os-sidebar__icon-item--active {
  color: var(--text-accent);
  background: var(--accent-dim);
}
</style>
