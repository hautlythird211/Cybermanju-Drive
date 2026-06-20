<script setup lang="ts">
import { computed, h, ref, onMounted, watch, nextTick, onUnmounted } from 'vue'
import type { VNode } from 'vue'
import gsap from 'gsap'
import OsIcon from './OsIcon.vue'
import { useGsapAnimation } from '@/composables/useGsapAnimation'

const anim = useGsapAnimation()

export interface TableColumn {
  key: string
  label: string
  sortable?: boolean
  width?: string
  align?: 'left' | 'center' | 'right'
  render?: (row: Record<string, unknown>) => string | number | boolean | null | undefined | VNode
}

const props = withDefaults(defineProps<{
  columns: TableColumn[]
  rows: Record<string, unknown>[]
  variant?: 'default' | 'glass' | 'neon' | 'gothic' | 'cute'
  size?: 'sm' | 'md' | 'lg'
  striped?: boolean
  hover?: boolean
  compact?: boolean
  sortKey?: string
  sortDir?: 'asc' | 'desc'
}>(), {
  variant: 'default',
  size: 'md',
  striped: false,
  hover: false,
  compact: false,
})

const emit = defineEmits<{
  'update:sortKey': [key: string]
  'update:sortDir': [dir: 'asc' | 'desc']
  'rowClick': [row: Record<string, unknown>]
}>()

const tbodyRef = ref<HTMLElement | null>(null)
const gsapCtx = ref<gsap.Context | null>(null)

const cls = computed(() => [
  'os-table',
  `os-table--${props.variant}`,
  `os-table--${props.size}`,
  {
    'os-table--striped': props.striped,
    'os-table--hover': props.hover,
    'os-table--compact': props.compact,
  },
])

function handleSort(col: TableColumn) {
  if (!col.sortable) return
  if (props.sortKey === col.key) {
    emit('update:sortDir', props.sortDir === 'asc' ? 'desc' : 'asc')
  } else {
    emit('update:sortKey', col.key)
    emit('update:sortDir', 'asc')
  }
}

function renderCell(col: TableColumn, row: Record<string, unknown>): VNode {
  const rendered = col.render!(row)
  if (rendered != null && typeof rendered === 'object') {
    return rendered as VNode
  }
  return h('span', undefined, rendered ?? '')
}

watch(() => props.rows, async () => {
  await nextTick()
  if (tbodyRef.value) {
    const rowEls = Array.from(tbodyRef.value.querySelectorAll('tr:not(.os-table__empty-row)'))
    if (rowEls.length > 0) {
      gsapCtx.value?.add(() => {
        anim.staggerIn(rowEls as HTMLElement[])
      })
    }
  }
}, { deep: true })

onMounted(() => {
  gsapCtx.value = gsap.context(() => {})
})

onUnmounted(() => {
  gsapCtx.value?.revert()
})
</script>

<template>
  <div :class="[...cls, 'gpu']">
    <table class="os-table__table" role="table">
      <thead>
        <tr>
          <th
            v-for="col in columns"
            :key="col.key"
            :style="{ width: col.width, textAlign: col.align || 'left' }"
            :class="{ 'os-table__sortable': col.sortable, 'os-table__sorted': sortKey === col.key }"
            role="columnheader"
            :aria-sort="sortKey === col.key ? (sortDir === 'asc' ? 'ascending' : 'descending') : undefined"
            :aria-label="col.sortable ? (col.label + ', sortable') : col.label"
            :scope="'col'"
            @click="handleSort(col)"
          >
            {{ col.label }}
            <span v-if="col.sortable" class="os-table__sort-icon">
              <OsIcon
                v-if="sortKey === col.key"
                :icon="sortDir === 'asc' ? 'mdi:arrow-up' : 'mdi:arrow-down'"
                :size="10"
              />
              <OsIcon v-else icon="mdi:arrow-up-down" :size="10" />
            </span>
          </th>
        </tr>
      </thead>
      <tbody ref="tbodyRef">
        <tr
          v-for="(row, idx) in rows"
          :key="idx"
          v-memo="[row]"
          :class="{ 'os-table__row--clickable': $attrs.onRowClick }"
          role="row"
          :aria-rowindex="idx + 1"
          @click="emit('rowClick', row)"
        >
          <td
            v-for="col in columns"
            :key="col.key"
            :style="{ textAlign: col.align || 'left' }"
            role="cell"
          >
            <template v-if="col.render">
              <component :is="() => renderCell(col, row)" />
            </template>
            <template v-else>{{ row[col.key] }}</template>
          </td>
        </tr>
        <tr v-if="rows.length === 0" class="os-table__empty-row">
          <td :colspan="columns.length" class="os-table__empty" role="cell">NO DATA</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.os-table {
  overflow-x: auto;
  font-family: var(--font-mono);
}

.os-table__table {
  width: 100%;
  border-collapse: collapse;
}

.os-table--default { color: var(--text-secondary); }
.os-table--default th { border-bottom: 1px solid var(--border-subtle); color: var(--text-muted); box-shadow: 0 1px 0 var(--accent-dim); }
.os-table--default td { border-bottom: 1px solid var(--border-subtle); }
.os-table--default tbody tr:hover { background: var(--bg-overlay); transition: background var(--duration-fast) var(--ease-spring); }

.os-table--glass { color: var(--text-secondary); }
.os-table--glass th { border-bottom: 1px solid var(--border-glass); color: var(--text-muted); }
.os-table--glass td { border-bottom: 1px solid var(--border-glass); }
.os-table--glass tbody tr:hover { background: rgba(255,255,255,0.05); }

.os-table--neon { color: var(--text-secondary); }
.os-table--neon th { border-bottom: 1px solid rgba(var(--accent-rgb), 0.15); color: var(--text-muted); }
.os-table--neon td { border-bottom: 1px solid rgba(var(--accent-rgb), 0.08); }
.os-table--neon tbody tr:hover { background: var(--accent-dim); }

.os-table--gothic { color: #d4a0b0; }
.os-table--gothic th { border-bottom: 1px solid #2a1010; color: #886060; }
.os-table--gothic td { border-bottom: 1px solid #2a1010; }
.os-table--gothic tbody tr:hover { background: rgba(255, 107, 157, 0.05); }

.os-table--cute { color: var(--text-secondary); }
.os-table--cute th { border-bottom: 1px solid rgba(255, 107, 157, 0.15); color: var(--text-pink); }
.os-table--cute td { border-bottom: 1px solid rgba(255, 107, 157, 0.08); }
.os-table--cute tbody tr:hover { background: var(--pink-dim); }

.os-table--sm th, .os-table--sm td { padding: 4px 8px; font-size: var(--font-size-xs); }
.os-table--md th, .os-table--md td { padding: 6px 12px; font-size: var(--font-size-base); }
.os-table--lg th, .os-table--lg td { padding: 8px 16px; font-size: var(--font-size-md); }

.os-table--striped tbody tr:nth-child(even) { background: var(--overlay-light); }

.os-table--compact th, .os-table--compact td { padding: 3px 6px; }

.os-table th {
  text-align: left;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  user-select: none;
  white-space: nowrap;
}

.os-table__sortable {
  cursor: pointer;
}
.os-table__sortable:hover {
  color: var(--text-primary);
}

.os-table__sorted {
  color: var(--text-accent);
}

.os-table__sort-icon {
  display: inline-flex;
  vertical-align: middle;
  margin-left: 4px;
}

.os-table__empty {
  text-align: center;
  padding: 24px;
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.os-table__row--clickable {
  cursor: pointer;
}

.os-table tbody tr {
  will-change: transform;
}
</style>
