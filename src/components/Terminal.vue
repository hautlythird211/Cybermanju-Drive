<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'

const emit = defineEmits<{ (e: 'close'): void }>()

const history = ref<string[]>([])
const input = ref('')
const currentDir = ref('~')
const hostname = 'cybermanju'
const username = ref('admin')

const histStack = ref<string[]>([])
const histIdx = ref(-1)
const logs = ref<string[]>([])

let exitFuncs: (() => void)[] = []

const commands: Record<string, (args: string[]) => string | string[]> = {
  help: () => [
    'Available commands:',
    '  help       Show this message',
    '  ls         List directory contents',
    '  cd <dir>   Change directory',
    '  pwd        Print working directory',
    '  cat <file> Display file contents',
    '  clear      Clear terminal',
    '  echo ...   Print arguments',
    '  whoami     Show current user',
    '  uname      System information',
    '  uptime     System uptime',
    '  ps         List processes',
    '  date       Show current date/time',
    '  neofetch   System info (fancy)',
    '  exit       Close terminal',
    '  sudo       Try it (you won\'t)',
  ],
  ls: (args) => {
    const files = [
      'Documents/', 'Downloads/', 'Pictures/', 'Music/', 'Videos/',
      '.config/', '.ssh/', '.bashrc', '.profile', 'README.md',
      'projects/', 'cybermanju-db/', '.gitignore', 'todo.txt',
    ]
    if (args.includes('-l')) {
      return files.map(f => {
        const isDir = f.endsWith('/')
        const perms = isDir ? 'drwxr-xr-x' : '-rw-r--r--'
        const size = isDir ? '4096' : Math.floor(Math.random() * 65536).toString()
        return `${perms}  1 ${username.value} ${username.value}  ${size}  Feb ${Math.floor(Math.random() * 28 + 1).toString().padStart(2, ' ')} ${String(Math.floor(Math.random() * 12 + 1)).padStart(2, '0')}:${String(Math.floor(Math.random() * 60)).padStart(2, '0')}  ${f}`
      })
    }
    return files.join('  ')
  },
  cd: (args) => {
    if (args.length === 0 || args[0] === '~' || args[0] === '/home/' + username.value) {
      currentDir.value = '~'
      return ''
    }
    if (args[0] === '..') {
      if (currentDir.value === '~') return ''
      const parts = currentDir.value.split('/')
      parts.pop()
      currentDir.value = parts.join('/') || '~'
      return ''
    }
    currentDir.value = currentDir.value === '~' ? '~/' + args[0] : currentDir.value + '/' + args[0]
    return ''
  },
  pwd: () => `/home/${username.value}${currentDir.value === '~' ? '' : '/' + currentDir.value.slice(1)}`,
  cat: (args) => {
    if (args.length === 0) return 'cat: missing operand'
    const files: Record<string, string> = {
      '.bashrc': '# ~/.bashrc\nalias ll="ls -la"\nexport EDITOR=vim\nexport PS1="\\u@\\h:\\w$ "\n\n# Cybermanju Drive\nalias cd="cd && ls -F"',
      '.profile': '# ~/.profile\nexport PATH=$PATH:$HOME/.local/bin\nexport LANG=en_US.UTF-8\n\n# Start cybermanju-shell\nif [ -z "$DISPLAY" ] && [ "$XDG_VTNR" = "1" ]; then\n  exec cybermanju-shell\nfi',
      'README.md': '# Cybermanju Drive\n\nPost-Quantum Encrypted File System\n\n## Features\n- ML-KEM-1024 encryption\n- ML-DSA-87 signing\n- Triple compression (LZ4+ZSTD+BROTLI)\n- Tantivy BM25 full-text search\n- AI face recognition\n- Geo-tagging\n- Multi-backend sync',
      'todo.txt': '- [ ] Write unit tests for crypto layer\n- [ ] Benchmark search indexing\n- [ ] Add dark mode toggle\n- [ ] Fix WASM build pipeline\n- [ ] Document API endpoints',
    }
    return files[args[0]] || `cat: ${args[0]}: No such file or directory`
  },
  clear: () => { logs.value = []; return '' },
  echo: (args) => args.join(' '),
  whoami: () => username.value,
  uname: () => 'Linux cybermanju 6.8.0-cybermanju #1 SMP PREEMPT_DYNAMIC x86_64 GNU/Linux',
  uptime: () => {
    const u = 86400
    const d = Math.floor(u / 86400)
    const h = Math.floor((u % 86400) / 3600)
    const m = Math.floor((u % 3600) / 60)
    return ` ${d}:${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')} up ${d} day${d !== 1 ? 's' : ''},  1 user,  load average: ${(Math.random() * 2).toFixed(2)}, ${(Math.random() * 2).toFixed(2)}, ${(Math.random() * 2).toFixed(2)}`
  },
  ps: () => {
    return '  PID TTY          TIME CMD\n  123 pts/0    00:00:02 bash\n  456 pts/0    00:00:00 ps\n  789 pts/0    00:00:05 cybermanju-shell\n  321 ?        00:00:12 systemd'
  },
  date: () => new Date().toString(),
  neofetch: () => [
    `           ▄▄▄▄▄▄▄▄▄▄▄  ${username.value}@${hostname}`,
    `        ▄████████████████▄  -------------------`,
    `      ▄██▀▀▀▀████████▀▀▀▀██▄  OS: Cybermanju OS x86_64`,
    `     ██▀         ██        ▀██  Kernel: 6.8.0-cybermanju`,
    `     ██           ██         ██  Uptime: ${(Math.random() * 30 + 1).toFixed(1)} hours`,
    `     ██           ██        ██  Packages: 1423`,
    `     ██▄          ██       ▄██  Shell: bash 5.2.21`,
    `      ████▄▄▄▄████████▄▄▄▄████  Resolution: 1920x1080`,
    `          ▀██████████████▀      DE: Cybermanju Shell`,
    `             ▀████████▀         WM: Wayland (wlroots)`,
    `                   ██           Terminal: cybermanju-term`,
    `                   ██           CPU: PQC-NEON (8) @ 2.80GHz`,
    `                  ██            GPU: NVIDIA RTX 5090`,
    `                  ██            Memory: ${(Math.random() * 16 + 8).toFixed(1)} GiB / 32 GiB`,
  ],
  sudo: (args) => {
    if (args[0] === 'rm' && args.includes('-rf') && args.includes('/')) {
      return `Nice try. This isn't a real shell, cowboy.`
    }
    return `sudo: ${args.join(' ') || '(no command)'}: command not found (permission denied — nice try)`
  },
  exit: () => {
    exitFuncs.push(() => emit('close'))
    return ''
  },
}

function processCmd() {
  const raw = input.value.trim()
  input.value = ''
  if (!raw) return

  const parts = raw.split(/\s+/)
  const cmd = parts[0].toLowerCase()
  const args = parts.slice(1)

  histStack.value.push(raw)
  histIdx.value = -1

  logs.value.push(`\x1b[32m${username.value}@${hostname}\x1b[0m:\x1b[34m${currentDir.value}\x1b[0m$ ${raw}`)

  const handler = commands[cmd]
  if (handler) {
    const result = handler(args)
    if (result) {
      const lines = Array.isArray(result) ? result : [result]
      logs.value.push(...lines)
    }
  } else {
    logs.value.push(`bash: ${cmd}: command not found`)
  }

  nextTick(() => {
    const el = document.querySelector('.term-output')
    if (el) el.scrollTop = el.scrollHeight
  })
}

function handleKey(e: KeyboardEvent) {
  if (e.key === 'Enter') processCmd()
  else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (histStack.value.length) {
      histIdx.value = histIdx.value < histStack.value.length - 1 ? histIdx.value + 1 : histIdx.value
      input.value = histStack.value[histStack.value.length - 1 - histIdx.value] || ''
    }
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (histIdx.value > 0) {
      histIdx.value--
      input.value = histStack.value[histStack.value.length - 1 - histIdx.value] || ''
    } else {
      histIdx.value = -1
      input.value = ''
    }
  }
}

function renderLine(line: string): string {
  return line
    .replace(/\x1b\[32m/g, '<span style="color:#00ff41">')
    .replace(/\x1b\[34m/g, '<span style="color:#5af0ff">')
    .replace(/\x1b\[31m/g, '<span style="color:#ff5f57">')
    .replace(/\x1b\[33m/g, '<span style="color:#febc2e">')
    .replace(/\x1b\[0m/g, '</span>')
}

onMounted(() => {
  const saved = localStorage.getItem('cybermanju_username')
  if (saved) username.value = saved
  logs.value = [
    `Welcome to Cybermanju OS 1.0 (GNU/Linux 6.8.0-cybermanju x86_64)`,
    ``,
    ` * Documentation:  https://docs.cybermanju.dev`,
    ` * Management:     https://dashboard.cybermanju.dev:3456`,
    ` * Support:        https://support.cybermanju.dev`,
    ``,
    `${Math.floor(Math.random() * 99) + 1} update(s) available.`,
    `Last login: ${new Date().toLocaleString()} from 127.0.0.1`,
    ``,
  ]
})

onUnmounted(() => {
  exitFuncs.forEach(f => f())
})
</script>

<template>
  <div class="terminal" @keydown="handleKey" tabindex="0" autofocus>
    <div class="term-header">
      <div class="term-dots">
        <span class="term-dot term-dot--close" @click="$emit('close')"></span>
        <span class="term-dot term-dot--min"></span>
        <span class="term-dot term-dot--max"></span>
      </div>
      <div class="term-title">{{ username }}@{{ hostname }}: {{ currentDir }}</div>
    </div>
    <div class="term-body">
      <div class="term-output">
        <div v-for="(line, i) in logs" :key="i" class="term-line" v-html="renderLine(line)"></div>
      </div>
      <div class="term-input-line">
        <span class="term-prompt">
          <span style="color:#00ff41">{{ username }}@{{ hostname }}</span>:<span style="color:#5af0ff">{{ currentDir }}</span>$
        </span>
        <span class="term-input-text">{{ input }}</span>
        <span class="term-cursor">▊</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.terminal {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  overflow: hidden;
  font-family: 'Courier New', 'Fira Code', monospace;
  contain: layout style;
}

.term-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  background: #111;
  border-bottom: 1px solid #1a1a1a;
}

.term-dots {
  display: flex;
  gap: 6px;
}

.term-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  cursor: pointer;
}

.term-dot--close { background: #ff5f57; }
.term-dot--close:hover { filter: brightness(1.3); }
.term-dot--min { background: #febc2e; }
.term-dot--max { background: #00ff41; }

.term-title {
  flex: 1;
  font-size: 10px;
  color: #555;
  letter-spacing: 1px;
  text-align: center;
}

.term-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 10px 14px;
  background: #050505;
}

.term-output {
  flex: 1;
  overflow-y: auto;
  contain: paint;
}

.term-output::-webkit-scrollbar { width: 4px; }
.term-output::-webkit-scrollbar-track { background: transparent; }
.term-output::-webkit-scrollbar-thumb { background: #1a1a1a; border-radius: 2px; }

.term-line {
  font-size: 11px;
  line-height: 1.6;
  color: #bbb;
  white-space: pre-wrap;
  word-break: break-all;
}

.term-input-line {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  flex-shrink: 0;
}

.term-prompt {
  font-size: 11px;
  white-space: nowrap;
}

.term-input-text {
  font-size: 11px;
  color: #ddd;
}

.term-cursor {
  font-size: 11px;
  color: #00ff41;
  animation: term-blink 0.8s step-end infinite;
}

@keyframes term-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}
</style>
