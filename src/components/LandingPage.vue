<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@/composables/useTauri'
import { Icon } from '@iconify/vue'

const emit = defineEmits<{ (e: 'open-app'): void }>()

const hostname = 'cybermanju'

const quotes = [
  '"The cloud is just someone else\'s computer.\n This one has ML-KEM-1024. Good luck, NSA."',
  '"Google Drive reads your files.\n We just store them. The difference is subtle."',
  '"Dropbox thought folders were revolutionary.\n We thought quantum-safe encryption might be nicer."',
  '"Your data should be yours.\n Not a product. Not a training set. Just yours."',
]

const currentQuote = ref(quotes[Math.floor(Math.random() * quotes.length)])

// ── Rotating Buddha ASCII ──
const buddhaFrames = [
  [
    '      ┌─────┐      ',
    '    ╱  ═════  ╲    ',
    '   │  ╱   ╲  │    ',
    '   │ (  ═  ) │    ',
    '   │  ╲   ╱  │    ',
    '    ╲  ───  ╱    ',
    '      └─────┘      ',
    '     ╱  │  ╲       ',
    '    ╱   │   ╲      ',
    '   │   ╱ ╲   │     ',
    '    ╲ ╱   ╲ ╱     ',
  ],
  [
    '      ┌─────┐      ',
    '    ╱  ═════  ╲    ',
    '   │  ╲   ╱  │    ',
    '   │ (  ═  ) │    ',
    '   │  ╱   ╲  │    ',
    '    ╲  ───  ╱    ',
    '      └─────┘      ',
    '       ╲ │ ╱       ',
    '        ╲│╱        ',
    '       ╱ │ ╲       ',
    '      ╱  │  ╲      ',
  ],
  [
    '      ╱‾‾‾‾‾╲      ',
    '    ╱  ═════  ╲    ',
    '   │  ╱   ╲  │    ',
    '   │ (  ═  ) │    ',
    '   │  ╲   ╱  │    ',
    '    ╲  ───  ╱    ',
    '      ╲_____╱      ',
    '    ╱  ╲   ╱  ╲    ',
    '   ╱    ╲ ╱    ╲   ',
    '  │    ╱ ╲    │   ',
    '   ╲  ╱   ╲  ╱   ',
  ],
  [
    '      ┌─────┐      ',
    '    ╱  ═════  ╲    ',
    '   │  ╱   ╲  │    ',
    '   │ (  ═  ) │    ',
    '   │  ╲   ╱  │    ',
    '    ╲  ───  ╱    ',
    '      └─────┘      ',
    '       ╱ │ ╲       ',
    '      ╱  │  ╲      ',
    '     ╱   │   ╲     ',
    '    ╱    │    ╲    ',
  ],
]

const currentFrame = ref(0)
const frameLines = ref<string[]>(buddhaFrames[0])
const buddhaGlow = ref(0)
let buddhaTimer: ReturnType<typeof setInterval> | null = null
let glowTimer: ReturnType<typeof setInterval> | null = null

const terminalInput = ref('')
const hiddenInput = ref<HTMLInputElement | null>(null)
const terminalHistory = ref<string[]>([])
const commandHist = ref<string[]>([])
const histIdx = ref(-1)
const booting = ref(true)
const bootLines = ref<string[]>([])

const commandRegistry: Record<string, { desc: string; args?: string }> = {
  help:      { desc: 'Show available commands', args: '[command]' },
  launch:    { desc: 'Open file manager' },
  about:     { desc: 'System info' },
  clear:     { desc: 'Clear terminal' },
  quote:     { desc: 'Show wisdom' },
  status:    { desc: 'System status (via Tauri)' },
  search:    { desc: 'Search files via Tantivy', args: '<query>' },
  suggest:   { desc: 'Type-ahead suggestions', args: '<prefix>' },
  diagnostic:{ desc: 'Run full diagnostic' },
  keys:      { desc: 'Manage encryption keys', args: '[list|generate]' },
  db:        { desc: 'Database stats', args: '[check|stats]' },
  tree:      { desc: 'Directory tree' },
  whoami:    { desc: 'Show current user' },
  uname:     { desc: 'System information' },
  date:      { desc: 'Show current date/time' },
}

const commandNames = Object.keys(commandRegistry)

// ── Boot sequence ──
async function runBootSequence() {
  const lines = [
    '\x1b[33mCybermanju BIOS v1.0\x1b[0m',
    '',
    'Initializing POST...',
    '  CPU:    PQC-NEON x86_64 @ 2.80GHz \x1b[32m[OK]\x1b[0m',
    '  RAM:    32768 MB DDR5 \x1b[32m[OK]\x1b[0m',
    '  Disk:   NVMe 2TB \x1b[32m[OK]\x1b[0m',
    '',
  ]

  for (const line of lines) {
    bootLines.value.push(line)
    await new Promise(r => setTimeout(r, 60 + Math.random() * 40))
  }

  // Try to get real system info
  try {
    const info = await invoke<{
      osName: string; osVersion: string; hostname: string;
      cpuBrand: string; cpuCores: number; totalMemoryMb: number;
    }>('get_system_info')
    bootLines.value.push(`  OS:     ${info.osName} ${info.osVersion}`)
    bootLines.value.push(`  Host:   ${info.hostname}`)
    bootLines.value.push(`  CPU:    ${info.cpuBrand} (${info.cpuCores} cores)`)
    bootLines.value.push(`  Memory: ${info.totalMemoryMb} MB`)
  } catch {
    bootLines.value.push('  OS:     Cybermanju OS x86_64')
    bootLines.value.push('  Host:   cybermanju')
  }

  bootLines.value.push('')
  bootLines.value.push('\x1b[32mPOST complete. Loading kernel...\x1b[0m')
  bootLines.value.push('')

  await new Promise(r => setTimeout(r, 400))

  // Transition to terminal
  booting.value = false
  terminalHistory.value = [
    `\x1b[33mCybermanju Shell 1.0\x1b[0m (GNU/Linux 6.8.0-cybermanju x86_64)`,
    '',
    ` * Type \x1b[33mHELP\x1b[0m for all commands`,
    ` * Type \x1b[33mLAUNCH\x1b[0m or click [ENTER CYBERMANJU] to open the file manager`,
    ` * TAB to auto-complete`,
    '',
  ]
  nextTick(() => focusInput())
}

// ── Async command handlers ──
async function runAsyncCmd(cmd: string, args: string[]): Promise<string[]> {
  try {
    if (cmd === 'status') {
      const info = await invoke<{
        osName: string; osVersion: string; osArch: string; hostname: string;
        cpuBrand: string; cpuCores: number; totalMemoryMb: number; usedMemoryMb: number;
        totalDiskGb: number; usedDiskGb: number; kernelVersion: string; uptimeSeconds: number;
      }>('get_system_info')
      const uptimeH = Math.floor(info.uptimeSeconds / 3600)
      const uptimeM = Math.floor((info.uptimeSeconds % 3600) / 60)
      return [
        '\x1b[33mSystem Status\x1b[0m',
        '',
        `  OS:       ${info.osName} ${info.osVersion} (${info.osArch})`,
        `  Host:     ${info.hostname}`,
        `  Kernel:   ${info.kernelVersion}`,
        `  CPU:      ${info.cpuBrand} (${info.cpuCores} cores)`,
        `  Memory:   ${info.usedMemoryMb} MB / ${info.totalMemoryMb} MB`,
        `  Disk:     ${info.usedDiskGb.toFixed(1)} GB / ${info.totalDiskGb.toFixed(1)} GB`,
        `  Uptime:   ${uptimeH}h ${uptimeM}m`,
      ]
    }
    if (cmd === 'diagnostic') {
      const info = await invoke<{
        osName: string; osVersion: string; osArch: string; hostname: string;
        cpuBrand: string; cpuCores: number; totalMemoryMb: number; usedMemoryMb: number;
        totalDiskGb: number; usedDiskGb: number; kernelVersion: string;
      }>('get_system_info')
      return [
        '\x1b[33mDiagnostic\x1b[0m',
        '',
        `  Platform:  ${info.osName} ${info.osVersion}`,
        `  Arch:      ${info.osArch}`,
        `  Host:      ${info.hostname}`,
        `  Kernel:    ${info.kernelVersion}`,
        `  CPU:       ${info.cpuBrand} (${info.cpuCores} cores)`,
        `  Memory:    ${info.usedMemoryMb} / ${info.totalMemoryMb} MB`,
        `  Disk:      ${info.usedDiskGb.toFixed(1)} / ${info.totalDiskGb.toFixed(1)} GB`,
        '',
        '  \x1b[32mAll systems nominal.\x1b[0m',
      ]
    }
    if (cmd === 'search') {
      if (args.length === 0) return ['Usage: search <query>']
      const query = args.join(' ')
      const results = await invoke<Array<{ fileName: string; score: number; matchType?: string }>>('search_files', { query, limit: 10, offset: 0 })
      if (results.length === 0) return [`No results for "${query}"`]
      const lines: string[] = [`\x1b[33mResults for "${query}"\x1b[0m (${results.length}):`, '']
      for (const r of results) {
        const type = r.matchType ? ` [${r.matchType}]` : ''
        lines.push(`  ${r.fileName.padEnd(30)} score=${r.score.toFixed(2)}${type}`)
      }
      return lines
    }
    if (cmd === 'suggest') {
      if (args.length === 0) return ['Usage: suggest <prefix>']
      const suggestions = await invoke<string[]>('suggest', { prefix: args[0], limit: 8 })
      if (suggestions.length === 0) return [`No suggestions for "${args[0]}"`]
      return [`\x1b[33mSuggestions for "${args[0]}":\x1b[0m`, ...suggestions.map(s => `  ${s}`)]
    }
    if (cmd === 'keys') {
      const action = args[0] || 'list'
      if (action === 'list') {
        const keys = await invoke<Array<{ id: string; algorithm: string; algorithmDisplay: string }>>('list_keys')
        if (keys.length === 0) return ['No keys found. Generate in the Encryption panel.']
        const lines: string[] = ['\x1b[33mEncryption Keys\x1b[0m', '']
        for (const k of keys) {
          lines.push(`  ${k.id}  ${k.algorithmDisplay}`)
        }
        return lines
      }
      return ['Usage: keys list']
    }
    if (cmd === 'db') {
      const info = await invoke<{
        hostname: string; totalDiskGb: number; usedDiskGb: number;
      }>('get_system_info')
      return [
        '\x1b[33mDatabase Info\x1b[0m',
        `  Engine:   redb (embedded)`,
        `  Host:     ${info.hostname}`,
        `  Disk:     ${info.usedDiskGb.toFixed(1)} / ${info.totalDiskGb.toFixed(1)} GB`,
      ]
    }
    if (cmd === 'tree') {
      return [
        '\x1b[34m.\x1b[0m',
        '├── \x1b[34mDocuments/\x1b[0m',
        '├── \x1b[34mDownloads/\x1b[0m',
        '├── \x1b[34mPictures/\x1b[0m',
        '├── \x1b[34m.config/\x1b[0m',
        '├── README.md',
        '├── .bashrc',
        '└── .gitignore',
        '',
        '3 directories, 3 files',
      ]
    }
    if (cmd === 'whoami') return [username.value]
    if (cmd === 'uname') return ['Linux cybermanju 6.8.0-cybermanju #1 SMP PREEMPT_DYNAMIC x86_64 GNU/Linux']
    if (cmd === 'date') return [new Date().toString()]
    return [`Unknown command: ${cmd}`]
  } catch (e) {
    return [`${cmd} failed: ${e}`]
  }
}

// ── Process command ──
function processCmd() {
  const raw = terminalInput.value.trim().toLowerCase()
  terminalInput.value = ''
  if (!raw) return

  const parts = raw.split(/\s+/)
  const base = parts[0]
  const args = parts.slice(1)

  commandHist.value.push(raw)
  histIdx.value = -1
  terminalHistory.value.push(`\x1b[32m${username.value}@${hostname}\x1b[0m:\x1b[34m~\x1b[0m$ ${raw}`)

  if (base === 'clear') {
    terminalHistory.value = []
    return
  }
  if (base === 'launch') {
    terminalHistory.value.push('Launching Cybermanju Drive...')
    setTimeout(() => emit('open-app'), 600)
    return
  }
  if (base === 'help') {
    if (args.length > 0) {
      const meta = commandRegistry[args[0]]
      if (meta) {
        terminalHistory.value.push(`  ${args[0]} — ${meta.desc}`)
        if (meta.args) terminalHistory.value.push(`  Usage: ${args[0]} ${meta.args}`)
        return
      }
      terminalHistory.value.push(`  No such command: ${args[0]}`)
      return
    }
    terminalHistory.value.push(...[
      '',
      '  \x1b[33mQuick Commands:\x1b[0m',
      '    help       Show this message',
      '    launch     Open file manager',
      '    about      System version info',
      '    status     System status (Tauri)',
      '    diagnostic Full diagnostic',
      '    search     Tantivy search <query>',
      '    suggest    Type-ahead <prefix>',
      '    keys       Encryption keys',
      '    db         Database info',
      '    tree       Directory tree',
      '    quote      Random wisdom',
      '    whoami     Current user',
      '    uname      System info',
      '    date       Current date/time',
      '    clear      Clear terminal',
      '',
      '  For the full terminal, open it from the Dock or use LAUNCH.',
      '  TAB to auto-complete.',
      '',
    ])
    return
  }
  if (base === 'quote') {
    terminalHistory.value.push(quotes[Math.floor(Math.random() * quotes.length)])
    return
  }
  if (base === 'about') {
    terminalHistory.value.push(...[
      '\x1b[33mCybermanju Drive v0.0.1\x1b[0m',
      'Post-Quantum Encrypted File System',
      'ML-KEM-1024 | ML-DSA-87 | Triple Compression',
      'Tantivy BM25 Full-Text Search',
      'https://github.com/hautlythird211/Cybermanju-Drive',
    ])
    return
  }

  runAsyncCmd(base, args).then(lines => {
    terminalHistory.value.push(...lines)
  })
}

// ── TAB completion ──
function tabComplete() {
  const raw = terminalInput.value.trim()
  if (!raw) return

  const parts = raw.split(/\s+/)
  if (parts.length === 1) {
    const prefix = parts[0].toLowerCase()
    const matches = commandNames.filter(c => c.startsWith(prefix))
    if (matches.length === 1) {
      terminalInput.value = matches[0] + ' '
    } else if (matches.length > 1) {
      terminalHistory.value.push(`\x1b[90m${matches.join('  ')}\x1b[0m`)
    }
    return
  }

  const base = parts[0].toLowerCase()
  if (base === 'keys' && parts.length === 2) {
    const subMatches = ['list'].filter(s => s.startsWith(parts[1]))
    if (subMatches.length === 1) terminalInput.value = `keys ${subMatches[0]}`
  }
  if (base === 'db' && parts.length === 2) {
    const subMatches = ['check', 'stats'].filter(s => s.startsWith(parts[1]))
    if (subMatches.length === 1) terminalInput.value = `db ${subMatches[0]}`
  }
}

// ── Keyboard handlers ──
function handleKey(e: KeyboardEvent) {
  if (e.key === 'Enter') processCmd()
  if (e.key === 'Tab') { e.preventDefault(); tabComplete() }
  if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (commandHist.value.length) {
      histIdx.value = Math.min(histIdx.value + 1, commandHist.value.length - 1)
      terminalInput.value = commandHist.value[commandHist.value.length - 1 - histIdx.value]
    }
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (histIdx.value > 0) {
      histIdx.value--
      terminalInput.value = commandHist.value[commandHist.value.length - 1 - histIdx.value]
    } else {
      histIdx.value = -1
      terminalInput.value = ''
    }
  }
}

function onInput(e: Event) {
  terminalInput.value = (e.target as HTMLInputElement).value
}

function onInputKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') processCmd()
  if (e.key === 'Tab') { e.preventDefault(); tabComplete() }
  if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (commandHist.value.length) {
      histIdx.value = Math.min(histIdx.value + 1, commandHist.value.length - 1)
      terminalInput.value = commandHist.value[commandHist.value.length - 1 - histIdx.value]
    }
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (histIdx.value > 0) {
      histIdx.value--
      terminalInput.value = commandHist.value[commandHist.value.length - 1 - histIdx.value]
    } else {
      histIdx.value = -1
      terminalInput.value = ''
    }
  }
}

function focusInput() {
  hiddenInput.value?.focus()
}

function renderLine(line: string): string {
  return line
    .replace(/\x1b\[32m/g, '<span style="color:#00ff41">')
    .replace(/\x1b\[34m/g, '<span style="color:#5af0ff">')
    .replace(/\x1b\[33m/g, '<span style="color:#febc2e">')
    .replace(/\x1b\[90m/g, '<span style="color:#666">')
    .replace(/\x1b\[0m/g, '</span>')
}

function startBuddhaAnimation() {
  buddhaTimer = setInterval(() => {
    currentFrame.value = (currentFrame.value + 1) % buddhaFrames.length
    frameLines.value = buddhaFrames[currentFrame.value]
  }, 280)
  glowTimer = setInterval(() => {
    buddhaGlow.value = Math.sin(Date.now() / 800) * 0.3 + 0.6
  }, 50)
}

function stopAnimations() {
  if (buddhaTimer) clearInterval(buddhaTimer)
  if (glowTimer) clearInterval(glowTimer)
  buddhaTimer = null
  glowTimer = null
}

const username = ref('admin')

onMounted(() => {
  const saved = localStorage.getItem('cybermanju_username')
  if (saved) username.value = saved
  startBuddhaAnimation()
  runBootSequence()
})

onUnmounted(() => {
  stopAnimations()
})
</script>

<template>
  <div class="landing-os" @click="focusInput" tabindex="0">
    <div class="landing-content">
      <div class="desktop-landing">
        <div class="ascii-background">
          <div class="ascii-buddha" :style="{ opacity: buddhaGlow }">
            <div v-for="(line, i) in frameLines" :key="i" class="buddha-line">{{ line }}</div>
          </div>
          <div class="ascii-particles">
            <div v-for="n in 20" :key="n" class="particle" :style="{
              left: Math.random() * 100 + '%',
              top: Math.random() * 100 + '%',
              animationDelay: Math.random() * 5 + 's',
              animationDuration: (3 + Math.random() * 4) + 's',
            }">.</div>
          </div>
        </div>

        <!-- Boot sequence -->
        <div v-if="booting" class="terminal-window boot-terminal" @click.stop="focusInput">
          <div class="terminal-log">
            <div v-for="(line, i) in bootLines" :key="i" class="term-line term-system">
              <span v-if="line.includes('\x1b[')" v-html="renderLine(line)"></span>
              <template v-else>{{ line }}</template>
            </div>
            <div class="boot-cursor-line">
              <span class="term-cursor">&#9608;</span>
            </div>
          </div>
        </div>

        <!-- Interactive terminal -->
        <div v-else class="terminal-window" @click.stop="focusInput">
          <div class="terminal-log">
            <div v-for="(line, i) in terminalHistory" :key="i" class="term-line"
              :class="{ 'term-prompt': line.includes('$'), 'term-system': !line.includes('$') }">
              <span v-if="line.includes('\x1b[')" v-html="renderLine(line)"></span>
              <template v-else>{{ line }}</template>
            </div>
            <div class="term-input-line">
              <span class="term-prompt-sign">
                <span style="color:#00ff41">{{ username }}@{{ hostname }}</span>:<span style="color:#5af0ff">~</span>$
              </span>
              <span class="term-input-text">{{ terminalInput }}</span>
              <span class="term-cursor">&#9608;</span>
            </div>
          </div>
          <input
            ref="hiddenInput"
            class="hidden-input"
            type="text"
            :value="terminalInput"
            @input="onInput"
            @keydown="onInputKeydown"
            autocomplete="off"
            autocapitalize="off"
            autocorrect="off"
            spellcheck="false"
          />
        </div>

        <div class="launch-hint" :class="{ 'fade-in': !booting }">
          <button class="launch-button" @click="emit('open-app')">
            [ ENTER CYBERMANJU ]
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.landing-os {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: #030308;
  font-family: 'SF Mono', 'Fira Code', 'JetBrains Mono', var(--font-mono);
  outline: none;
  overflow: hidden;
  z-index: 999;
}

.landing-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.desktop-landing {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
}

.ascii-background {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 0;
}

.ascii-buddha {
  text-align: center;
  font-size: 11px;
  line-height: 1.15;
  color: #00ff41;
  text-shadow: 0 0 6px rgba(0, 255, 65, 0.25);
  letter-spacing: 1px;
  transition: opacity 0.05s;
  user-select: none;
}

.buddha-line { white-space: pre; }

.ascii-particles {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.particle {
  position: absolute;
  color: rgba(0, 255, 65, 0.15);
  font-size: 8px;
  animation: float 4s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0) scale(1); opacity: 0; }
  50% { transform: translateY(-20px) scale(1.5); opacity: 0.8; }
}

/* ── Terminal Window ── */
.terminal-window {
  position: relative;
  z-index: 2;
  width: 92vw;
  max-width: 640px;
  max-height: 40vh;
  background: rgba(3, 3, 8, 0.82);
  border: 1px solid rgba(0, 255, 65, 0.12);
  border-radius: 12px;
  padding: 16px 20px;
  backdrop-filter: blur(20px) saturate(1.4);
  -webkit-backdrop-filter: blur(20px) saturate(1.4);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6), 0 0 1px rgba(0, 255, 65, 0.2);
  cursor: text;
  transition: opacity 0.3s ease;
}

.boot-terminal {
  border-color: rgba(0, 255, 65, 0.1);
}

.terminal-log {
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow-y: auto;
  max-height: 34vh;
}

.term-line {
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.term-system { color: rgba(0, 255, 65, 0.7); }
.term-prompt { color: #00ff41; font-weight: 700; }

.term-input-line {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
}

.term-prompt-sign {
  color: #00ff41;
  font-weight: 700;
  opacity: 0.8;
  font-size: 12px;
  white-space: nowrap;
}

.term-input-text {
  color: #00ff41;
  font-size: 12px;
}

.term-cursor {
  color: #00ff41;
  font-size: 11px;
  animation: blink 500ms step-end infinite;
}

.boot-cursor-line {
  margin-top: 4px;
}

.hidden-input {
  position: absolute;
  left: -9999px;
  top: -9999px;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}

/* ── Launch Button ── */
.launch-hint {
  position: relative;
  z-index: 2;
  opacity: 0;
  transition: opacity 0.5s ease 0.3s;
}

.launch-hint.fade-in {
  opacity: 1;
}

.launch-button {
  background: transparent;
  border: 1px solid rgba(0, 255, 65, 0.3);
  border-radius: 6px;
  color: #00ff41;
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 700;
  padding: 10px 24px;
  cursor: pointer;
  text-shadow: 0 0 4px rgba(0, 255, 65, 0.2);
  transition: all 0.15s;
  letter-spacing: 1px;
}

.launch-button:hover {
  background: rgba(0, 255, 65, 0.1);
  border-color: #00ff41;
  box-shadow: 0 0 16px rgba(0, 255, 65, 0.2);
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

@media (max-width: 768px) {
  .terminal-window { padding: 12px 14px; width: 96vw; }
  .term-line { font-size: 10px; }
  .ascii-buddha { font-size: 8px; }
  .launch-button { font-size: 10px; padding: 8px 16px; }
}
</style>
