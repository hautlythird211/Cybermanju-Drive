<template>
  <div class="code-panel">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-code">[T]</span>
        <h2 class="panel-title">CODE INTELLIGENCE</h2>
      </div>
      <div class="header-actions">
        <button
          v-if="parseResult"
          class="bw-btn-sm"
          @click="showAst = !showAst"
          title="TOGGLE AST VIEW"
        >{{ showAst ? '[TREE]' : '[AST]' }}</button>
        <button
          v-if="parseResult"
          class="bw-btn-sm"
          @click="showSource = !showSource"
          title="TOGGLE SOURCE VIEW"
        >{{ showSource ? '[SRC]' : '[SRC]' }}</button>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[PARSE] ANALYZE FILE</h3>
      <div v-if="selectedFile" class="selected-file">
        <span>{{ selectedFile.name }}</span>
        <div class="go-to-line" v-if="parseResult">
          <input
            v-model="gotoLine"
            class="goto-input"
            placeholder="LINE #"
            type="number"
            min="1"
            :max="parseResult.totalLines"
            @keyup.enter="handleGoToLine"
            aria-label="GO TO LINE NUMBER"
          />
          <button class="bw-btn-sm" @click="handleGoToLine">[GO]</button>
        </div>
        <button class="bw-btn" style="margin-top:6px;" @click="handleParse">[PARSE WITH TREE-SITTER]</button>
      </div>
      <p v-else class="text-muted">SELECT A TEXT/CODE FILE TO PARSE</p>
    </div>

    <div class="section" v-if="parseResult">
      <h3 class="section-title">[DATA] PARSE RESULT</h3>
      <div class="parse-meta">
        <div class="meta-row"><span class="meta-key text-muted">LANGUAGE</span><span class="meta-value language-badge">{{ parseResult.language }}</span></div>
        <div class="meta-row"><span class="meta-key text-muted">LINES</span><span class="meta-value">{{ parseResult.totalLines }}</span></div>
        <div class="meta-row"><span class="meta-key text-muted">SYMBOLS</span><span class="meta-value">{{ parseResult.symbols.length }}</span></div>
        <div class="meta-row"><span class="meta-key text-muted">PARSE TIME</span><span class="meta-value">{{ parseResult.parseTimeMs }}ms</span></div>
      </div>
    </div>

    <div class="section" v-if="parseResult && showAst">
      <h3 class="section-title">[TREE] SYMBOL TREE</h3>
      <div class="symbol-tree">
        <div v-for="sym in parseResult.symbols" :key="sym.name + sym.startLine" class="symbol-row" @click="gotoLineNum = sym.startLine">
          <span class="symbol-kind">{{ sym.kind }}</span>
          <span class="symbol-name">{{ sym.name }}</span>
          <span class="symbol-lines text-muted">:{{ sym.startLine }}</span>
        </div>
      </div>
    </div>

    <div class="section" v-if="parseResult && showSource">
      <h3 class="section-title">[SOURCE] SYNTAX VIEW</h3>
      <div class="source-view">
        <div
          v-for="(line, li) in sourceLines"
          :key="li"
          class="source-line"
          :class="{ 'goto-highlight': li + 1 === gotoLineNum }"
          :ref="(el) => { if (li + 1 === gotoLineNum && el) highlightLine(el as HTMLElement) }"
        >
          <span class="source-ln">{{ li + 1 }}</span>
          <span class="source-code" v-html="highlightSyntax(line, parseResult!.language)"></span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const selectedFile = computed(() => store.selectedFile)
const parseResult = computed(() => store.parseResult)
const showAst = ref(true)
const showSource = ref(false)
const gotoLine = ref('')
const gotoLineNum = ref(0)

const sourceLines = computed(() => {
  if (!selectedFile.value?.contentText) return []
  return selectedFile.value.contentText.split('\n')
})

async function handleParse() {
  if (!store.selectedFile?.path) return
  await store.parseFileCode(store.selectedFile.path)
  showSource.value = true
}

function handleGoToLine() {
  const n = parseInt(gotoLine.value)
  if (n > 0 && parseResult.value && n <= parseResult.value.totalLines) {
    gotoLineNum.value = n
    showSource.value = true
  }
}

function highlightLine(el: HTMLElement) {
  nextTick(() => el.scrollIntoView({ block: 'center', behavior: 'smooth' }))
}

function highlightSyntax(code: string, lang: string): string {
  const escaped = code.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  const keywords = new Set([
    'if', 'else', 'for', 'while', 'do', 'switch', 'case', 'break', 'continue',
    'return', 'function', 'class', 'struct', 'enum', 'interface', 'type',
    'var', 'let', 'const', 'import', 'export', 'from', 'def', 'async', 'await',
    'try', 'catch', 'throw', 'new', 'this', 'super', 'extends', 'implements',
    'pub', 'fn', 'let', 'mut', 'const', 'use', 'mod', 'impl', 'trait', 'where',
    'package', 'void', 'int', 'float', 'double', 'char', 'bool', 'string',
    'null', 'undefined', 'true', 'false', 'static', 'private', 'public',
    'protected', 'readonly', 'abstract', 'virtual', 'override',
  ])
  const strRe = /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)/g
  const commentRe = /(\/\/.*$|\/\*[\s\S]*?\*\/)/g
  let highlighted = escaped
  highlighted = highlighted.replace(commentRe, '<span class="syn-comment">$1</span>')
  highlighted = highlighted.replace(strRe, '<span class="syn-string">$1</span>')
  highlighted = highlighted.replace(/\b(\d+(?:\.\d+)?)\b/g, '<span class="syn-number">$1</span>')
  highlighted = highlighted.replace(/\b([a-zA-Z_]\w*)\b/g, (match) => {
    if (keywords.has(match)) return `<span class="syn-keyword">${match}</span>`
    if (match[0] === match[0].toUpperCase() && match[0] !== match[0].toLowerCase()) {
      return `<span class="syn-type">${match}</span>`
    }
    return match
  })
  return highlighted
}
</script>

<style scoped>
.code-panel {
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

.header-left { display: flex; align-items: center; gap: 8px; }
.header-actions { display: flex; gap: 4px; }
.icon-code { font-size: 14px; color: var(--text-accent); }
.panel-title { font-size: 13px; font-weight: 700; letter-spacing: 1px; margin: 0; color: var(--text-primary); }

.section { margin-bottom: 16px; }

.section-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0 0 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-glass);
  font-family: var(--font-mono);
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

.bw-btn-sm {
  padding: 3px 8px;
  background: transparent;
  border: 1px solid var(--border-medium);
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 600;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.bw-btn-sm:hover { background: var(--accent-dim); border-color: var(--border-accent); color: var(--text-accent); }

.selected-file {
  font-size: 12px;
  background: var(--bg-elevated);
  padding: 6px 10px;
  border: 1px solid var(--border-subtle);
  color: var(--text-primary);
  border-radius: var(--radius-sm);
}

.go-to-line {
  display: flex;
  gap: 4px;
  margin-top: 6px;
  align-items: center;
}

.goto-input {
  width: 80px;
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 10px;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
}

.goto-input:focus {
  border-color: var(--border-accent);
  outline: none;
}

.parse-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
}

.meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.meta-key { font-size: 10px; color: var(--text-muted); }
.meta-value { font-size: 10px; font-family: var(--font-mono); font-weight: 600; color: var(--text-primary); }

.language-badge {
  border: 1px solid var(--border-accent);
  padding: 1px 6px;
  font-size: 9px;
  color: var(--text-accent);
  border-radius: var(--radius-sm);
}

.symbol-tree {
  border: 1px solid var(--border-glass);
  overflow: hidden;
  border-radius: var(--radius-md);
}

.symbol-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  font-size: 10px;
  border-bottom: 1px solid var(--border-subtle);
  cursor: pointer;
  color: var(--text-primary);
}

.symbol-row:hover {
  background: var(--accent-dim);
}

.symbol-kind {
  font-size: 8px;
  color: var(--text-muted);
  text-transform: uppercase;
  flex-shrink: 0;
  min-width: 40px;
}

.symbol-name { color: var(--text-accent); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
.symbol-lines { flex-shrink: 0; font-size: 9px; color: var(--text-muted); }

.source-view {
  border: 1px solid var(--border-glass);
  font-family: var(--font-mono);
  font-size: 10px;
  max-height: 400px;
  overflow-y: auto;
  line-height: 1.5;
  background: var(--bg-elevated);
  border-radius: var(--radius-md);
}

.source-line {
  display: flex;
  padding: 0 4px;
}

.source-line:hover {
  background: var(--bg-glass-light);
}

.source-line.goto-highlight {
  background: var(--accent-dim);
  border-left: 3px solid var(--border-accent);
}

.source-ln {
  flex-shrink: 0;
  width: 32px;
  text-align: right;
  padding-right: 8px;
  color: var(--text-muted);
  opacity: 0.5;
  user-select: none;
}

.source-code {
  flex: 1;
  white-space: pre;
  overflow-x: auto;
}

.text-muted { color: var(--text-muted) !important; }
</style>

<style>
.syn-keyword { color: var(--text-primary); font-weight: 700; }
.syn-string { color: var(--text-secondary); }
.syn-comment { color: var(--text-muted); font-style: italic; }
.syn-number { color: var(--text-primary); }
.syn-type { color: var(--text-primary); text-decoration: underline; }
</style>
