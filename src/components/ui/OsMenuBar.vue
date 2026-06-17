<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import gsap from 'gsap'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

interface MenuItem {
  id: string
  label: string
  shortcut?: string
  disabled?: boolean
  divider?: boolean
  checked?: boolean
  children?: MenuItem[]
  action?: () => void
}

interface MenuGroup {
  id: string
  label: string
  children: MenuItem[]
}

const props = withDefaults(defineProps<{
  items: MenuGroup[]
  variant?: 'default' | 'glass' | 'neon' | 'gothic'
  height?: number
  trayContent?: boolean
}>(), {
  variant: 'default',
  height: 28,
  trayContent: false,
})

const emit = defineEmits<{
  action: [item: MenuItem]
}>()

const menubarRef = ref<HTMLElement | null>(null)
const dropdownRefs = ref<Map<string, HTMLElement>>(new Map())
const activeMenu = ref<string | null>(null)
const activeSubmenu = ref<string | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)
let focusedGroupIndex = 0

function setDropdownRef(el: HTMLElement | null, id: string) {
  if (el) dropdownRefs.value.set(id, el)
}

function toggleMenu(id: string) {
  const wasOpen = activeMenu.value === id
  if (!wasOpen) {
    activeMenu.value = id
    activeSubmenu.value = null
    nextTick(() => {
      const dropdown = dropdownRefs.value.get(id)
      if (dropdown) {
        gsapCtx.value?.add(() => {
          anim.dropdownEnter(dropdown)
        })
      }
    })
  } else {
    closeCurrent()
  }
}

function closeCurrent() {
  if (activeMenu.value) {
    const dropdown = dropdownRefs.value.get(activeMenu.value)
    if (dropdown) {
      gsapCtx.value?.add(() => {
        anim.dropdownLeave(dropdown).then(() => {
          activeMenu.value = null
          activeSubmenu.value = null
        })
      })
    } else {
      activeMenu.value = null
      activeSubmenu.value = null
    }
  }
}

function handleClick(item: MenuItem) {
  if (item.disabled) return
  if (item.children) {
    activeSubmenu.value = activeSubmenu.value === item.id ? null : item.id
    return
  }
  emit('action', item)
  item.action?.()
  closeCurrent()
}

function onMenuKeydown(e: KeyboardEvent) {
  if (!activeMenu.value) {
    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
      e.preventDefault()
      const dir = e.key === 'ArrowRight' ? 1 : -1
      const len = props.items.length
      focusedGroupIndex = (focusedGroupIndex + dir + len) % len
      const group = props.items[focusedGroupIndex]
      if (group) toggleMenu(group.id)
    }
    return
  }
  if (e.key === 'Escape') {
    e.preventDefault()
    if (activeSubmenu.value) {
      activeSubmenu.value = null
    } else {
      closeCurrent()
    }
  }
  if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
    e.preventDefault()
    const dir = e.key === 'ArrowRight' ? 1 : -1
    const len = props.items.length
    focusedGroupIndex = (focusedGroupIndex + dir + len) % len
    closeCurrent()
    const group = props.items[focusedGroupIndex]
    if (group) toggleMenu(group.id)
  }
}

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div
    ref="menubarRef"
    :class="['os-menubar', `os-menubar--${props.variant}`]"
    :style="{ height: height + 'px' }"
    role="menubar"
    :aria-label="'Menu bar'"
    tabindex="0"
    @keydown="onMenuKeydown"
  >
    <div class="os-menubar__left">
      <div
        v-for="(group, gIdx) in items"
        :key="group.id"
        class="os-menubar__group"
        @mouseenter="activeMenu = group.id"
        @click="toggleMenu(group.id)"
      >
        <span
          class="os-menubar__label"
          :class="{ 'os-menubar__label--active': activeMenu === group.id }"
          role="menuitem"
          :aria-expanded="activeMenu === group.id"
          aria-haspopup="menu"
          tabindex="-1"
        >
          {{ group.label }}
        </span>
        <div
          v-if="activeMenu === group.id"
          :ref="(el: any) => setDropdownRef(el as HTMLElement | null, group.id)"
          class="os-menubar__dropdown"
          role="menu"
          :aria-label="group.label"
          @mouseleave="activeMenu = null"
        >
          <template v-for="item in group.children" :key="item.id">
            <div v-if="item.divider" class="os-menubar__divider" role="separator" />
            <div
              v-else
              class="os-menubar__item"
              :class="{
                'os-menubar__item--disabled': item.disabled,
                'os-menubar__item--checked': item.checked,
                'os-menubar__item--parent': item.children,
              }"
              role="menuitem"
              :aria-disabled="item.disabled"
              :aria-checked="item.checked || undefined"
              tabindex="-1"
              @click.stop="handleClick(item)"
              @mouseenter="activeSubmenu = item.id"
              @keydown.enter="handleClick(item)"
              @keydown.space.prevent="handleClick(item)"
            >
              <span class="os-menubar__item-label">{{ item.label }}</span>
              <span v-if="item.shortcut" class="os-menubar__shortcut">{{ item.shortcut }}</span>
              <OsIcon v-if="item.children" icon="mdi:chevron-right" :size="12" />
              <div v-if="item.children && activeSubmenu === item.id" class="os-menubar__submenu" role="menu">
                <template v-for="sub in item.children" :key="sub.id">
                  <div v-if="sub.divider" class="os-menubar__divider" role="separator" />
                  <div
                    v-else
                    class="os-menubar__item"
                    :class="{ 'os-menubar__item--disabled': sub.disabled }"
                    role="menuitem"
                    :aria-disabled="sub.disabled"
                    tabindex="-1"
                    @click.stop="handleClick(sub)"
                    @keydown.enter="handleClick(sub)"
                    @keydown.space.prevent="handleClick(sub)"
                  >
                    <span class="os-menubar__item-label">{{ sub.label }}</span>
                    <span v-if="sub.shortcut" class="os-menubar__shortcut">{{ sub.shortcut }}</span>
                  </div>
                </template>
              </div>
            </div>
          </template>
        </div>
      </div>
    </div>
    <div v-if="trayContent" class="os-menubar__right">
      <slot name="tray" />
    </div>
  </div>
</template>

<style scoped>
.os-menubar {
  display: flex;
  align-items: center;
  padding: 0 8px;
  user-select: none;
  flex-shrink: 0;
  font-family: var(--font-mono);
  font-size: var(--font-size-base);
}

.os-menubar--default {
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border-subtle);
  color: var(--text-secondary);
}

.os-menubar--glass {
  background: var(--bg-glass);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border-bottom: 1px solid var(--border-glass);
}

.os-menubar--neon {
  background: var(--bg-surface);
  border-bottom: 1px solid rgba(0, 255, 65, 0.15);
}

.os-menubar--gothic {
  background: #150808;
  border-bottom: 1px solid #2a1010;
}

.os-menubar__left {
  display: flex;
  align-items: center;
  gap: 2px;
  flex: 1;
}

.os-menubar__right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.os-menubar__group {
  position: relative;
}

.os-menubar__label {
  display: block;
  padding: 4px 8px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}
.os-menubar__label:hover,
.os-menubar__label--active {
  background: var(--bg-overlay);
  color: var(--text-primary);
}

.os-menubar__dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 200px;
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-lg);
  padding: 4px;
  box-shadow: var(--shadow-dropdown);
  z-index: 1000;
  will-change: transform, opacity;
}

.os-menubar__item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  position: relative;
  transition: all var(--transition-fast);
}
.os-menubar__item:hover {
  background: var(--bg-overlay);
  color: var(--text-primary);
}

.os-menubar__item--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.os-menubar__item--disabled:hover {
  background: transparent;
  color: var(--text-secondary);
}

.os-menubar__item-label { flex: 1; white-space: nowrap; }
.os-menubar__shortcut { font-size: var(--font-size-xs); color: var(--text-muted); margin-left: 16px; }

.os-menubar__divider {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 8px;
}

.os-menubar__submenu {
  position: absolute;
  left: 100%;
  top: -4px;
  min-width: 180px;
  background: var(--bg-glass-heavy);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-glass);
  border-radius: var(--radius-lg);
  padding: 4px;
  box-shadow: var(--shadow-dropdown);
  will-change: transform, opacity;
}
</style>
