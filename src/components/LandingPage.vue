<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import TopMenuBar from './TopMenuBar.vue'
import Dock from './Dock.vue'

const emit = defineEmits<{ (e: 'open-app'): void }>()

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
const terminalHistory = ref<string[]>([
  'System ready. Type HELP for commands.',
  '',
])
const commandHist = ref<string[]>([])
const histIdx = ref(-1)

const commands: Record<string, { out: string[]; desc: string }> = {
  help: {
    desc: 'Show available commands',
    out: [
      '  HELP     This message',
      '  LAUNCH   Open file manager',
      '  ABOUT    System info',
      '  CLEAR    Clear terminal',
      '  QUOTE    Show wisdom',
      '  STATUS   System status',
    ],
  },
  launch: {
    desc: 'Launch app',
    out: ['Launching Cybermanju Drive...'],
  },
  about: {
    desc: 'System info',
    out: [
      'Cybermanju Drive v0.0.1',
      'Post-Quantum Encrypted File System',
      'ML-KEM-1024 | ML-DSA-87 | Triple Compression',
      'https://github.com/hautlythird211/Cybermanju-Drive',
    ],
  },
  clear: {
    desc: 'Clear terminal',
    out: [],
  },
  quote: {
    desc: 'Random wisdom',
    out: [quotes[Math.floor(Math.random() * quotes.length)]],
  },
  status: {
    desc: 'System status',
    out: [
      'STATUS: ONLINE',
      'CRYPTO: ML-KEM-1024 [ACTIVE]',
      'SEARCH: Tantivy BM25 [INDEXED]',
      'SYNC:   [CONNECTED]',
      'FACE:   ONNX Runtime [WARM]',
      'WEB:    :3456 [SERVING]',
    ],
  },
}

function processCmd() {
  const cmd = terminalInput.value.trim().toLowerCase()
  terminalInput.value = ''
  if (!cmd) return
  commandHist.value.push(cmd)
  histIdx.value = -1
  terminalHistory.value.push(`> ${cmd}`)
  const c = commands[cmd]
  if (c) {
    terminalHistory.value.push(...c.out)
    if (cmd === 'clear') terminalHistory.value = []
    if (cmd === 'launch') {
      setTimeout(() => emit('open-app'), 800)
    }
  } else {
    terminalHistory.value.push(`Unknown: ${cmd}. Try HELP.`)
  }
}

function handleKey(e: KeyboardEvent) {
  if (e.key === 'Enter') processCmd()
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

onMounted(() => {
  startBuddhaAnimation()
})

onUnmounted(() => {
  stopAnimations()
})
</script>

<template>
  <div class="landing-os" @keydown="handleKey" tabindex="0">
    <TopMenuBar />

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

        <div class="terminal-window">
          <div class="terminal-log">
            <div v-for="(line, i) in terminalHistory" :key="i" class="term-line"
              :class="{ 'term-prompt': line.startsWith('>'), 'term-system': !line.startsWith('>') }">
              {{ line }}
            </div>
            <div class="term-input-line">
              <span class="term-prompt-sign">&gt;</span>
              <span class="term-input-text">{{ terminalInput }}</span>
              <span class="term-cursor">&#9608;</span>
            </div>
          </div>
        </div>

        <div class="launch-hint">
          <button class="launch-button" @click="emit('open-app')">
            [ ENTER CYBERMANJU ]
          </button>
        </div>
      </div>
    </div>

    <Dock />
  </div>
</template>

<style scoped>
.landing-os {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: #000;
  font-family: 'Courier New', 'Fira Code', monospace;
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

/* ── Desktop Landing ── */
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

.buddha-line {
  white-space: pre;
}

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
  background: rgba(0, 0, 0, 0.85);
  border: 1px solid rgba(0, 255, 65, 0.2);
  border-radius: 8px;
  padding: 16px 20px;
  backdrop-filter: blur(4px);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.6);
}

.terminal-log {
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow-y: auto;
  max-height: 30vh;
}

.term-line {
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.term-system {
  color: rgba(0, 255, 65, 0.7);
}

.term-prompt {
  color: #00ff41;
  font-weight: 700;
}

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

.term-cursor.hide {
  opacity: 0;
}

/* ── Launch Buttons ── */
.launch-hint {
  position: relative;
  z-index: 2;
  display: flex;
  gap: 12px;
}

.launch-button, .reboot-button {
  background: transparent;
  border: 1px solid rgba(0, 255, 65, 0.3);
  border-radius: 6px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
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

@media (max-width: 768px) {
  .terminal-window {
    padding: 12px 14px;
    width: 96vw;
  }
  .term-line {
    font-size: 10px;
  }
  .ascii-buddha {
    font-size: 8px;
  }
  .launch-button {
    font-size: 10px;
    padding: 8px 16px;
  }
}
</style>
