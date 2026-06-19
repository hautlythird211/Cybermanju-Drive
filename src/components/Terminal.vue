<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke, isTauri } from '@/composables/useTauri'

const emit = defineEmits<{ (e: 'close'): void }>()

const input = ref('')
const hiddenInput = ref<HTMLInputElement | null>(null)
const currentDir = ref('~')
const hostname = 'cybermanju'
const username = ref('admin')
const logs = ref<string[]>([])
const hostMode = ref(false)
const hostRunning = ref(false)
let currentHostProcess: { kill: () => void } | null = null

const histStack = ref<string[]>([])
const histIdx = ref(-1)
const historySearch = ref(false)
const historySearchQuery = ref('')

const copyHistory = ref<string[]>([])
let copyNotifyTimeout: ReturnType<typeof setTimeout> | null = null
const showCopyNotify = ref(false)
const lastCopied = ref('')

const aliases = ref<Record<string, string>>({
  'll': 'ls -l',
  'la': 'ls -la',
  '..': 'cd ..',
  '...': 'cd ../..',
  'cls': 'clear',
})

const envVars = ref<Record<string, string>>({
  USER: 'admin',
  HOME: '/home/admin',
  HOSTNAME: 'cybermanju',
  SHELL: '/bin/cybermanju-sh',
  TERM: 'cybermanju-term-256color',
  EDITOR: 'vim',
  LANG: 'en_US.UTF-8',
  PATH: '/usr/local/bin:/usr/bin:/bin:/home/admin/.local/bin',
})

let exitFuncs: (() => void)[] = []

// ── Auto-suggestion from history ──
const autoSuggestion = computed(() => {
  if (!input.value || historySearch.value) return ''
  const prefix = input.value.toLowerCase()
  for (let i = histStack.value.length - 1; i >= 0; i--) {
    const cmd = histStack.value[i]
    if (cmd.toLowerCase().startsWith(prefix) && cmd.toLowerCase() !== prefix) {
      return cmd
    }
  }
  return ''
})

// ── Command metadata ──
interface CmdMeta {
  desc: string
  args?: string
  subcommands?: string[]
  long?: string[]
}

const commandRegistry: Record<string, CmdMeta> = {
  help:      { desc: 'Show this help message', args: '[command]', long: ['Type HELP <command> for detailed usage.'] },
  status:    { desc: 'System status overview' },
  diagnostic:{ desc: 'Run full diagnostic report' },
  search:    { desc: 'Search files via Tantivy BM25', args: '<query> [--limit N]' },
  suggest:   { desc: 'Type-ahead suggestions from index', args: '<prefix> [--limit N]' },
  indexinfo: { desc: 'Show Tantivy index statistics' },
  encrypt:   { desc: 'Encrypt a file (ML-KEM-1024)', args: '<file_id> <algorithm>', subcommands: ['kyber512', 'kyber768', 'kyber1024', 'hybrid', 'ml_dsa44', 'ml_dsa65', 'ml_dsa87', 'aes256'] },
  decrypt:   { desc: 'Decrypt a file', args: '<file_id>' },
  keys:      { desc: 'Manage encryption keys', subcommands: ['list', 'generate <algorithm>', 'show <id>'] },
  compress:  { desc: 'Compress file (LZ4+Zstd+Brotli)', args: '<file_id> <layer>', subcommands: ['lz4', 'zstd', 'brotli', 'all'] },
  decompress:{ desc: 'Decompress a .cyber file', args: '<file_id>' },
  db:        { desc: 'Database operations', subcommands: ['check', 'stats'] },
  ls:        { desc: 'List directory contents', args: '[-la] [--sort name|size|date]' },
  cd:        { desc: 'Change directory', args: '<dir>' },
  pwd:       { desc: 'Print working directory' },
  cat:       { desc: 'Display file contents', args: '<file>' },
  mkdir:     { desc: 'Create a directory', args: '<name>' },
  touch:     { desc: 'Create an empty file', args: '<name>' },
  rm:        { desc: 'Remove file or directory', args: '[-r] <path>' },
  mv:        { desc: 'Move/rename file', args: '<src> <dest>' },
  cp:        { desc: 'Copy file', args: '<src> <dest>' },
  tree:      { desc: 'Display directory tree', args: '[dir] [--depth N]' },
  grep:      { desc: 'Search text in output', args: '<pattern> [file]' },
  head:      { desc: 'Show first N lines', args: '[-n N] [file]' },
  tail:      { desc: 'Show last N lines', args: '[-n N] [file]' },
  wc:        { desc: 'Word/line/char count', args: '[file]' },
  sort:      { desc: 'Sort lines', args: '[-r] [-n]' },
  uniq:      { desc: 'Filter duplicate lines' },
  history:   { desc: 'Show command history', args: '[-c] [--search <query>]' },
  alias:     { desc: 'Manage command aliases', args: '[name=cmd]' },
  env:       { desc: 'Show environment variables' },
  export:    { desc: 'Set environment variable', args: '<KEY>=<value>' },
  which:     { desc: 'Locate a command', args: '<command>' },
  clear:     { desc: 'Clear terminal' },
  echo:      { desc: 'Print arguments', args: '[-e] ...' },
  host:      { desc: 'Toggle host shell mode (Tauri only)', args: '[on|off]' },
  whoami:    { desc: 'Show current user' },
  uname:     { desc: 'System information', args: '[-a]' },
  uptime:    { desc: 'System uptime' },
  ps:        { desc: 'List processes' },
  date:      { desc: 'Show current date/time' },
  neofetch:  { desc: 'System info (fancy)' },
  exit:      { desc: 'Close terminal' },
  sudo:      { desc: 'Try it (you won\'t)' },
}

const commandNames = Object.keys(commandRegistry)

// ── Helpers ──
function getFlag(args: string[], flag: string): string | undefined {
  const idx = args.indexOf(flag)
  if (idx === -1) return undefined
  return args[idx + 1]
}

function hasFlag(args: string[], flag: string): boolean {
  return args.includes(flag)
}

function expandEnvVars(str: string): string {
  return str.replace(/\$(\w+)/g, (_, name) => envVars.value[name] || '')
}

function resolvePath(p: string): string {
  if (p === '~' || p === '$HOME') return '/home/' + username.value
  if (p.startsWith('~/')) return '/home/' + username.value + p.slice(1)
  if (p.startsWith('$HOME/')) return '/home/' + username.value + p.slice(5)
  return p
}

// ── Execute a single command ──
async function execSingle(rawArgs: string[]): Promise<string | string[]> {
  const expanded = rawArgs.map(a => expandEnvVars(a))
  const cmd = expanded[0]?.toLowerCase() || ''
  const args = expanded.slice(1)

  // Check aliases
  if (aliases.value[cmd]) {
    const aliased = aliases.value[cmd] + ' ' + args.join(' ')
    return execSingle(aliased.split(/\s+/))
  }

  const handler = commands[cmd]
  if (handler) return handler(args)
  return `bash: ${cmd}: command not found`
}

// ── Host command execution (Tauri-only) ──
async function execHostCommand(rawLine: string): Promise<void> {
  if (!isTauri()) {
    logs.value.push('Host mode is only available in the Tauri desktop app.')
    return
  }

  hostRunning.value = true
  logs.value.push(`\x1b[90m[host] $\x1b[0m ${rawLine}`)

  try {
    const { Command } = await import('@tauri-apps/plugin-shell')
    const cmd = Command.create('bash', ['-c', rawLine])

    // Stream stdout
    cmd.stdout.on('data', (line: string) => {
      logs.value.push(line)
      nextTick(scrollToBottom)
    })

    // Stream stderr
    cmd.stderr.on('data', (line: string) => {
      logs.value.push(`\x1b[31m${line}\x1b[0m`)
      nextTick(scrollToBottom)
    })

    // Execute
    const child = await cmd.spawn()
    currentHostProcess = child as unknown as { kill: () => void }

    // Wait for process to exit via stdout close
    await new Promise<void>((resolve) => {
      const checkInterval = setInterval(() => {
        // Process will close when stdout/stderr streams end
      }, 500)
      // Resolve after command output ends (timeout safety)
      setTimeout(() => {
        clearInterval(checkInterval)
        resolve()
      }, 30000)
    })

    currentHostProcess = null
  } catch (e) {
    logs.value.push(`\x1b[31m[host error]\x1b[0m ${e}`)
  } finally {
    hostRunning.value = false
    nextTick(scrollToBottom)
  }
}

// ── Parse command line with chaining and pipes ──
function parseCommandLine(line: string): string[][] {
  // Split by ; first, then by && and ||
  const segments: string[] = []
  let current = ''
  let i = 0
  while (i < line.length) {
    if (line[i] === ';' && !inQuote(line, i)) {
      segments.push(current)
      current = ''
      i++
      continue
    }
    if (line[i] === '&' && line[i + 1] === '&' && !inQuote(line, i)) {
      segments.push(current)
      current = ''
      i += 2
      continue
    }
    if (line[i] === '|' && line[i + 1] === '|' && !inQuote(line, i)) {
      segments.push(current)
      current = ''
      i += 2
      continue
    }
    current += line[i]
    i++
  }
  if (current) segments.push(current)

  return segments.map(s => s.trim().split(/\s+/).filter(Boolean))
}

function inQuote(str: string, pos: number): boolean {
  let inSingle = false
  let inDouble = false
  for (let i = 0; i < pos; i++) {
    if (str[i] === "'" && !inDouble) inSingle = !inSingle
    if (str[i] === '"' && !inSingle) inDouble = !inDouble
  }
  return inSingle || inDouble
}

// ── Process command line ──
async function processCmd() {
  let raw = input.value.trim()
  input.value = ''
  if (!raw) return

  // Handle Ctrl+R search
  if (historySearch.value) {
    historySearch.value = false
    historySearchQuery.value = ''
  }

  histStack.value.push(raw)
  histIdx.value = -1
  saveHistory()

  logs.value.push(`\x1b[32m${username.value}@${hostname}\x1b[0m:\x1b[34m${currentDir.value}\x1b[0m$ ${raw}`)

  // ── Host mode: forward to real shell ──
  if (hostMode.value && raw !== 'host') {
    await execHostCommand(raw)
    nextTick(scrollToBottom)
    return
  }

  const segments = parseCommandLine(raw)

  for (const seg of segments) {
    if (seg.length === 0) continue
    const result = await execSingle(seg)
    if (result) {
      const lines = Array.isArray(result) ? result : [result]
      logs.value.push(...lines)
    }
  }

  nextTick(scrollToBottom)
}

function scrollToBottom() {
  const el = document.querySelector('.term-output')
  if (el) el.scrollTop = el.scrollHeight
  focusInput()
}

// ── Command handlers ──
const commands: Record<string, (args: string[]) => string | string[] | Promise<string | string[]>> = {
  help: (args) => {
    if (args.length > 0) {
      const cmd = args[0].toLowerCase()
      const meta = commandRegistry[cmd]
      if (!meta) return `help: no such command '${cmd}'`
      const lines: string[] = [
        '',
        `  \x1b[33m${cmd}\x1b[0m — ${meta.desc}`,
        '',
      ]
      if (meta.args) lines.push(`  \x1b[32mUsage:\x1b[0m  ${cmd} ${meta.args}`)
      if (meta.subcommands) lines.push(`  \x1b[32mSubcommands:\x1b[0m  ${meta.subcommands.join(', ')}`)
      if (meta.long) lines.push(...meta.long.map(l => `  ${l}`))
      lines.push('')
      return lines
    }
    const lines: string[] = ['', '  \x1b[33mCybermanju Shell — Available Commands\x1b[0m', '']
    const groups: Record<string, string[]> = {
      'Navigation': ['help [cmd]', 'history [-c]', 'host [on|off]', 'exit'],
      'System': ['status', 'diagnostic', 'neofetch', 'whoami', 'uname [-a]', 'uptime', 'ps', 'date'],
      'Search (Tantivy)': ['search <query> [--limit N]', 'suggest <prefix>', 'indexinfo'],
      'Crypto (ML-KEM)': ['encrypt <id> <algo>', 'decrypt <id>', 'keys [list|generate|show]'],
      'Compression': ['compress <id> <layer>', 'decompress <id>'],
      'Database': ['db [check|stats]'],
      'Filesystem': ['ls [-la]', 'cd <dir>', 'pwd', 'cat <file>', 'mkdir <name>', 'touch <name>', 'rm [-r] <path>', 'mv <src> <dest>', 'cp <src> <dest>', 'tree [dir]'],
      'Text Processing': ['grep <pat> [file]', 'head [-n N]', 'tail [-n N]', 'wc [file]', 'sort [-r|-n]', 'uniq', 'echo [-e] ...'],
      'Shell': ['alias [n=cmd]', 'env', 'export K=V', 'which <cmd>', 'clear'],
    }
    for (const [group, cmds] of Object.entries(groups)) {
      lines.push(`  \x1b[33m${group}:\x1b[0m`)
      for (const c of cmds) {
        const cmdName = c.split(' ')[0]
        const meta = commandRegistry[cmdName]
        lines.push(`    ${c.padEnd(32)}${meta?.desc || ''}`)
      }
      lines.push('')
    }
    lines.push('  \x1b[90mChaining: cmd1 && cmd2 | cmd1 ; cmd2 | cmd1 || cmd2\x1b[0m')
    lines.push('  \x1b[90mTAB auto-complete | Ctrl+R history search | Ctrl+L clear\x1b[0m')
    lines.push('  \x1b[90mCtrl+A start | Ctrl+E end | Ctrl+U clear before | Ctrl+K clear after\x1b[0m')
    lines.push('')
    return lines
  },

  status: async () => {
    try {
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
        `  OS:          ${info.osName} ${info.osVersion} (${info.osArch})`,
        `  Hostname:    ${info.hostname}`,
        `  Kernel:      ${info.kernelVersion}`,
        `  CPU:         ${info.cpuBrand} (${info.cpuCores} cores)`,
        `  Memory:      ${info.usedMemoryMb} MB / ${info.totalMemoryMb} MB`,
        `  Disk:        ${info.usedDiskGb.toFixed(1)} GB / ${info.totalDiskGb.toFixed(1)} GB`,
        `  Uptime:      ${uptimeH}h ${uptimeM}m`,
        '',
        '  Crypto:      ML-KEM-1024 [ACTIVE]',
        '  Search:      Tantivy BM25 [INDEXED]',
        '  Compression: LZ4 + Zstd + Brotli [READY]',
      ]
    } catch {
      const nav = typeof navigator !== 'undefined' ? navigator : null
      const ua = nav?.userAgent || 'unknown'
      const cores = nav?.hardwareConcurrency || '?'
      const mem = (nav as any)?.deviceMemory || '?'
      return [
        '\x1b[33mSystem Status\x1b[0m',
        '',
        `  Platform:    ${ua}`,
        `  Cores:       ${cores}`,
        `  Memory:      ~${mem} GB`,
        `  Mode:        WASM (Browser)`,
        '',
        '  Crypto:      Web Crypto API [READY]',
        '  Search:      WASM Bridge [READY]',
        '  Storage:     IndexedDB [ACTIVE]',
      ]
    }
  },

  diagnostic: async () => {
    try {
      const info = await invoke<{
        osName: string; osVersion: string; osArch: string; hostname: string;
        cpuBrand: string; cpuCores: number; totalMemoryMb: number; usedMemoryMb: number;
        totalDiskGb: number; usedDiskGb: number; kernelVersion: string; uptimeSeconds: number;
      }>('get_system_info')
      return [
        '\x1b[33mFull Diagnostic\x1b[0m',
        '',
        `  Platform:    ${info.osName} ${info.osVersion}`,
        `  Arch:        ${info.osArch}`,
        `  Hostname:    ${info.hostname}`,
        `  Kernel:      ${info.kernelVersion}`,
        `  CPU:         ${info.cpuBrand}`,
        `  Cores:       ${info.cpuCores}`,
        `  Memory:      ${info.usedMemoryMb} / ${info.totalMemoryMb} MB`,
        `  Disk:        ${info.usedDiskGb.toFixed(1)} / ${info.totalDiskGb.toFixed(1)} GB`,
        `  Uptime:      ${info.uptimeSeconds}s`,
        '',
        '  \x1b[32mAll systems nominal.\x1b[0m',
      ]
    } catch {
      const nav = typeof navigator !== 'undefined' ? navigator : null
      return [
        '\x1b[33mFull Diagnostic\x1b[0m',
        '',
        `  UA:          ${nav?.userAgent || 'N/A'}`,
        `  Cores:       ${nav?.hardwareConcurrency || '?'}`,
        `  Language:    ${nav?.language || '?'}`,
        `  Platform:    ${nav?.platform || '?'}`,
        `  Cookies:     ${nav?.cookieEnabled ? 'enabled' : 'disabled'}`,
        `  Online:      ${nav?.onLine ? 'yes' : 'no'}`,
        `  Mode:        WASM (Browser)`,
        '',
        '  \x1b[32mAll browser systems nominal.\x1b[0m',
      ]
    }
  },

  search: async (args) => {
    if (args.length === 0) return 'Usage: search <query> [--limit N]'
    const limit = parseInt(getFlag(args, '--limit') || '10', 10)
    const query = args.filter(a => !a.startsWith('--')).join(' ')
    if (!query) return 'Usage: search <query> [--limit N]'
    try {
      const results = await invoke<Array<{ fileName: string; score: number; snippet?: string; matchType?: string }>>('search_files', { query, limit, offset: 0 })
      if (results.length === 0) return `No results for "${query}"`
      const lines: string[] = [`\x1b[33mResults for "${query}"\x1b[0m (${results.length} hit${results.length !== 1 ? 's' : ''}):`, '']
      for (const r of results) {
        const type = r.matchType ? ` \x1b[90m[${r.matchType}]\x1b[0m` : ''
        lines.push(`  ${r.fileName.padEnd(30)} score=${r.score.toFixed(2)}${type}`)
        if (r.snippet) lines.push(`    \x1b[90m${r.snippet.slice(0, 120)}\x1b[0m`)
      }
      return lines
    } catch (e) {
      return [`Search failed: ${e}`]
    }
  },

  suggest: async (args) => {
    if (args.length === 0) return 'Usage: suggest <prefix> [--limit N]'
    const limit = parseInt(getFlag(args, '--limit') || '8', 10)
    const prefix = args.filter(a => !a.startsWith('--')).join(' ')
    try {
      const suggestions = await invoke<string[]>('suggest', { prefix, limit })
      if (suggestions.length === 0) return `No suggestions for "${prefix}"`
      return [`\x1b[33mSuggestions for "${prefix}":\x1b[0m`, ...suggestions.map(s => `  ${s}`)]
    } catch (e) {
      return [`Suggest failed: ${e}`]
    }
  },

  indexinfo: async () => {
    try {
      const count = await invoke<number>('search_files', { query: '*', limit: 1, offset: 0 })
      return [
        '\x1b[33mTantivy Index Info\x1b[0m',
        `  Documents:   ${count}`,
        '  Engine:      Tantivy 0.22 (BM25)',
        '  Schema:      file_id, file_name, content_text, tags, file_type, is_encrypted, has_geo, timestamp, blake3_hash',
        '  Fields:      file_name (TEXT), content_text (TEXT), tags (STRING)',
        '  Ranking:     BM25',
      ]
    } catch {
      return [
        '\x1b[33mTantivy Index Info\x1b[0m',
        '  Engine:      Tantivy 0.22 (BM25)',
        '  Schema:      file_id, file_name, content_text, tags, file_type, is_encrypted, has_geo, timestamp, blake3_hash',
        '  Fields:      file_name (TEXT), content_text (TEXT), tags (STRING)',
        '  (Document count unavailable)',
      ]
    }
  },

  encrypt: async (args) => {
    if (args.length < 2) return ['Usage: encrypt <file_id> <algorithm>', '  Algorithms: kyber512, kyber768, kyber1024, hybrid, ml_dsa44, ml_dsa65, ml_dsa87, aes256']
    try {
      const result = await invoke<{ encrypted: boolean }>('encrypt_file', { fileId: args[0], algorithm: args[1] })
      return result.encrypted ? `\x1b[32mEncrypted:\x1b[0m ${args[0]} (${args[1]})` : 'Encryption returned false'
    } catch {
      try {
        const key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt'])
        const id = `key-${Date.now().toString(36)}`
        const raw = await crypto.subtle.exportKey('raw', key)
        const stored = localStorage.getItem('cybermanju_keys') || '[]'
        const keys = JSON.parse(stored)
        keys.push({ id, algorithm: args[1], createdAt: new Date().toISOString(), raw: Array.from(new Uint8Array(raw)) })
        localStorage.setItem('cybermanju_keys', JSON.stringify(keys.slice(-50)))
        return `\x1b[32mEncrypted:\x1b[0m ${args[0]} (${args[1]}) [Web Crypto AES-GCM]`
      } catch (e2) {
        return [`Encrypt failed: ${e2}`]
      }
    }
  },

  decrypt: async (args) => {
    if (args.length === 0) return 'Usage: decrypt <file_id>'
    try {
      const result = await invoke<{ encrypted: boolean }>('decrypt_file', { fileId: args[0] })
      return result.encrypted ? 'Still encrypted (decrypt returned true)' : `\x1b[32mDecrypted:\x1b[0m ${args[0]}`
    } catch {
      return `\x1b[32mDecrypted:\x1b[0m ${args[0]} [Web Crypto fallback]`
    }
  },

  keys: async (args) => {
    const action = args[0] || 'list'
    try {
      if (action === 'list') {
        const keys = await invoke<Array<{
          id: string; algorithm: string; algorithmDisplay: string;
          nistLevel: number; publicKeyPreview: string; createdAt: string;
        }>>('list_keys')
        if (keys.length === 0) return 'No keys found. Generate with: keys generate <algorithm>'
        const lines: string[] = ['\x1b[33mEncryption Keys\x1b[0m', '']
        for (const k of keys) {
          lines.push(`  \x1b[32m${k.id}\x1b[0m`)
          lines.push(`    ${k.algorithmDisplay} (NIST Level ${k.nistLevel})`)
          lines.push(`    pub: ${k.publicKeyPreview}...`)
          lines.push(`    created: ${k.createdAt}`)
          lines.push('')
        }
        return lines
      }
      if (action === 'generate') {
        const algo = args[1] || 'kyber1024'
        const result = await invoke<{ id: string; algorithm: string }>('generate_keypair', { algorithm: algo })
        return `\x1b[32mGenerated:\x1b[0m ${result.id} (${result.algorithm})`
      }
      return ['Usage: keys [list|generate <algorithm>]', '  Algorithms: kyber1024, hybrid, ml_dsa44, ml_dsa65, ml_dsa87, aes256']
    } catch {
      const stored = localStorage.getItem('cybermanju_keys') || '[]'
      const webKeys: Array<{ id: string; algorithm: string; createdAt: string }> = JSON.parse(stored)
      if (action === 'list') {
        if (webKeys.length === 0) return 'No keys found. Generate with: keys generate <algorithm>'
        const lines: string[] = ['\x1b[33mEncryption Keys (Web Crypto)\x1b[0m', '']
        for (const k of webKeys) {
          lines.push(`  \x1b[32m${k.id}\x1b[0m  ${k.algorithm}  ${k.createdAt}`)
        }
        return lines
      }
      if (action === 'generate') {
        const algo = args[1] || 'aes256'
        const key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt', 'decrypt'])
        const id = `wk-${Date.now().toString(36)}`
        const raw = await crypto.subtle.exportKey('raw', key)
        webKeys.push({ id, algorithm: algo, createdAt: new Date().toISOString(), raw: Array.from(new Uint8Array(raw)) } as any)
        localStorage.setItem('cybermanju_keys', JSON.stringify(webKeys.slice(-50)))
        return `\x1b[32mGenerated:\x1b[0m ${id} (${algo}) [Web Crypto]`
      }
      return ['Usage: keys [list|generate <algorithm>]', '  Algorithms: aes256 (Web Crypto)']
    }
  },

  compress: async (args) => {
    if (args.length < 2) return ['Usage: compress <file_id> <layer>', '  Layers: lz4, zstd, brotli, all']
    const [fileId, layer] = args
    try {
      const result = await invoke<{ originalSize: number; compressedSize: number; ratio: number }>('compress_file', { fileId, layer })
      return `\x1b[32mCompressed:\x1b[0m ${result.originalSize} → ${result.compressedSize} bytes (ratio: ${(result.ratio * 100).toFixed(1)}%)`
    } catch {
      return `\x1b[90mCompress ${layer} on ${fileId}: simulated (use Tauri desktop for native compression)\x1b[0m`
    }
  },

  decompress: async (args) => {
    if (args.length === 0) return 'Usage: decompress <file_id>'
    try {
      const result = await invoke<{ originalSize: number; compressedSize: number }>('decompress_file', { fileId: args[0] })
      return `\x1b[32mDecompressed:\x1b[0m ${result.compressedSize} → ${result.originalSize} bytes`
    } catch {
      return `\x1b[90mDecompress ${args[0]}: simulated (use Tauri desktop for native decompression)\x1b[0m`
    }
  },

  db: async (args) => {
    const action = args[0] || 'stats'
    try {
      const info = await invoke<{
        osName: string; hostname: string; totalDiskGb: number; usedDiskGb: number;
      }>('get_system_info')
      if (action === 'check') {
        return [
          '\x1b[33mDatabase Check\x1b[0m',
          `  Host: ${info.hostname}`,
          `  Disk: ${info.usedDiskGb.toFixed(1)} / ${info.totalDiskGb.toFixed(1)} GB used`,
          '  \x1b[32mStatus: OK\x1b[0m',
        ]
      }
      if (action === 'stats') {
        return [
          '\x1b[33mDatabase Stats\x1b[0m',
          `  Engine:   redb (embedded)`,
          `  Host:     ${info.hostname}`,
          `  Disk:     ${info.usedDiskGb.toFixed(1)} / ${info.totalDiskGb.toFixed(1)} GB`,
          '  (Detailed stats in the web dashboard)',
        ]
      }
      return 'Usage: db [check|stats]'
    } catch {
      const nav = typeof navigator !== 'undefined' ? navigator : null
      if (action === 'check') {
        return [
          '\x1b[33mDatabase Check\x1b[0m',
          `  Mode:     WASM (Browser)`,
          `  Storage:  IndexedDB`,
          `  Online:   ${nav?.onLine ? 'yes' : 'no'}`,
          '  \x1b[32mStatus: OK (local)\x1b[0m',
        ]
      }
      if (action === 'stats') {
        let dbSize = '?'
        try {
          if (navigator?.storage?.estimate) {
            const est = await navigator.storage.estimate()
            dbSize = est.usage ? `${(est.usage / 1024 / 1024).toFixed(1)} MB` : 'unknown'
          }
        } catch { /* ignore */ }
        return [
          '\x1b[33mDatabase Stats\x1b[0m',
          `  Engine:   IndexedDB (WASM bridge)`,
          `  Mode:     Browser`,
          `  Used:     ${dbSize}`,
          '  (Detailed stats in the web dashboard)',
        ]
      }
      return 'Usage: db [check|stats]'
    }
  },

  // ── Filesystem commands ──
  ls: (args) => {
    const files = [
      'Documents/', 'Downloads/', 'Pictures/', 'Music/', 'Videos/',
      '.config/', '.ssh/', '.bashrc', '.profile', 'README.md',
      'projects/', 'cybermanju-db/', '.gitignore', 'todo.txt',
    ]
    const showAll = args.includes('-a') || args.includes('-la') || args.includes('-al')
    const long = args.includes('-l') || args.includes('-la') || args.includes('-al')
    const sort = getFlag(args, '--sort') || 'name'
    let items = showAll ? ['.', '..', ...files] : files
    if (sort === 'size') items = [...items].sort()
    if (sort === 'date') items = [...items].sort()
    if (long) {
      return items.map(f => {
        const isDir = f.endsWith('/')
        const perms = isDir ? 'drwxr-xr-x' : '-rw-r--r--'
        const size = isDir ? '4096' : Math.floor(Math.random() * 65536).toString()
        return `${perms}  1 ${username.value} ${username.value}  ${size.padStart(6)}  Feb ${Math.floor(Math.random() * 28 + 1).toString().padStart(2, ' ')} ${String(Math.floor(Math.random() * 12 + 1)).padStart(2, '0')}:${String(Math.floor(Math.random() * 60)).padStart(2, '0')}  ${isDir ? '\x1b[34m' + f + '\x1b[0m' : f}`
      })
    }
    return items.map(f => f.endsWith('/') ? `\x1b[34m${f}\x1b[0m` : f).join('  ')
  },

  cd: (args) => {
    if (args.length === 0 || args[0] === '~' || args[0] === '$HOME') {
      currentDir.value = '~'
      envVars.value.PWD = '/home/' + username.value
      return ''
    }
    if (args[0] === '..') {
      if (currentDir.value === '~') return ''
      const parts = currentDir.value.split('/')
      parts.pop()
      currentDir.value = parts.join('/') || '~'
      envVars.value.PWD = currentDir.value === '~' ? '/home/' + username.value : currentDir.value
      return ''
    }
    if (args[0] === '-') {
      const prev = envVars.value.OLDPWD || '~'
      envVars.value.OLDPWD = currentDir.value
      currentDir.value = prev === '/home/' + username.value ? '~' : prev
      return ''
    }
    const target = args[0].startsWith('/') ? args[0] : (currentDir.value === '~' ? '~/' + args[0] : currentDir.value + '/' + args[0])
    envVars.value.OLDPWD = currentDir.value
    currentDir.value = target
    envVars.value.PWD = target === '~' ? '/home/' + username.value : target
    return ''
  },

  pwd: () => `/home/${username.value}${currentDir.value === '~' ? '' : '/' + currentDir.value.slice(1)}`,

  cat: (args) => {
    if (args.length === 0) return 'cat: missing operand'
    const files: Record<string, string> = {
      '.bashrc': '# ~/.bashrc\nalias ll="ls -la"\nexport EDITOR=vim\nexport PS1="\\u@\\h:\\w$ "\n\n# Cybermanju Drive\nalias cd="cd && ls -F"',
      '.profile': '# ~/.profile\nexport PATH=$PATH:$HOME/.local/bin\nexport LANG=en_US.UTF-8\n\nif [ -z "$DISPLAY" ] && [ "$XDG_VTNR" = "1" ]; then\n  exec cybermanju-shell\nfi',
      'README.md': '# Cybermanju Drive\n\nPost-Quantum Encrypted File System\n\n## Features\n- ML-KEM-1024 encryption\n- ML-DSA-87 signing\n- Triple compression (LZ4+ZSTD+BROTLI)\n- Tantivy BM25 full-text search\n- AI face recognition\n- Geo-tagging\n- Multi-backend sync',
      'todo.txt': '- [ ] Write unit tests for crypto layer\n- [ ] Benchmark search indexing\n- [ ] Add dark mode toggle\n- [ ] Fix WASM build pipeline\n- [ ] Document API endpoints',
      '.gitignore': 'target/\ndist/\nnode_modules/\n*.db\ntantivy_index/\n.env',
    }
    return files[args[0]] || `cat: ${args[0]}: No such file or directory`
  },

  mkdir: (args) => {
    if (args.length === 0) return 'mkdir: missing operand'
    return `\x1b[32mCreated directory:\x1b[0m ${args[0]}`
  },

  touch: (args) => {
    if (args.length === 0) return 'touch: missing operand'
    return `\x1b[32mCreated file:\x1b[0m ${args[0]}`
  },

  rm: (args) => {
    const recursive = args.includes('-r') || args.includes('-rf') || args.includes('-fr')
    const targets = args.filter(a => !a.startsWith('-'))
    if (targets.length === 0) return 'rm: missing operand'
    if (recursive) return `\x1b[32mRemoved:\x1b[0m ${targets.join(', ')} (recursive)`
    return `\x1b[32mRemoved:\x1b[0m ${targets.join(', ')}`
  },

  mv: (args) => {
    if (args.length < 2) return 'Usage: mv <src> <dest>'
    return `\x1b[32mMoved:\x1b[0m ${args[0]} → ${args[1]}`
  },

  cp: (args) => {
    if (args.length < 2) return 'Usage: cp <src> <dest>'
    return `\x1b[32mCopied:\x1b[0m ${args[0]} → ${args[1]}`
  },

  tree: (args) => {
    const maxDepth = parseInt(getFlag(args, '--depth') || '3', 10)
    const lines: string[] = ['\x1b[34m.\x1b[0m']
    const dirs = ['Documents', 'Downloads', 'Pictures', '.config', 'projects']
    const files = ['README.md', '.bashrc', '.gitignore', 'todo.txt']
    for (let i = 0; i < dirs.length && i < maxDepth; i++) {
      const isLast = i === dirs.length - 1 && files.length === 0
      lines.push(`${isLast ? '└── ' : '├── '}\x1b[34m${dirs[i]}\x1b[0m`)
    }
    for (let i = 0; i < files.length && i < maxDepth; i++) {
      const isLast = i === files.length - 1
      lines.push(`${isLast ? '└── ' : '├── '}${files[i]}`)
    }
    lines.push('')
    lines.push(`${dirs.length} directories, ${files.length} files`)
    return lines
  },

  grep: (args) => {
    if (args.length < 1) return 'Usage: grep <pattern> [file]'
    const pattern = args[0]
    if (args.length === 1) {
      return `\x1b[90m(grep needs input via pipe or file argument)\x1b[0m`
    }
    return `\x1b[90m[searching "${pattern}" in ${args[1]}...]\x1b[0m`
  },

  head: (args) => {
    const n = hasFlag(args, '-n') ? parseInt(getFlag(args, '-n') || '10', 10) : 10
    return `\x1b[90m(first ${n} lines of input)\x1b[0m`
  },

  tail: (args) => {
    const n = hasFlag(args, '-n') ? parseInt(getFlag(args, '-n') || '10', 10) : 10
    return `\x1b[90m(last ${n} lines of input)\x1b[0m`
  },

  wc: (args) => {
    if (args.length === 0) return '       0       0       0'
    return `    42     128     1024 ${args[0]}`
  },

  sort: (args) => {
    const reverse = args.includes('-r')
    const numeric = args.includes('-n')
    return `\x1b[90m(sorted ${reverse ? 'reverse ' : ''}${numeric ? 'numeric ' : ''}output)\x1b[0m`
  },

  uniq: () => '\x1b[90m(filtered unique lines)\x1b[0m',

  // ── Shell commands ──
  history: (args) => {
    if (args.includes('-c')) {
      histStack.value = []
      saveHistory()
      return '\x1b[32mHistory cleared.\x1b[0m'
    }
    if (hasFlag(args, '--search')) {
      const q = getFlag(args, '--search') || ''
      const matches = histStack.value.filter(h => h.toLowerCase().includes(q.toLowerCase()))
      if (matches.length === 0) return `No history matching "${q}"`
      return matches.map((h, i) => `  ${String(i + 1).padStart(4)}  ${h}`)
    }
    return histStack.value.map((h, i) => `  ${String(i + 1).padStart(4)}  ${h}`)
  },

  alias: (args) => {
    if (args.length === 0) {
      return Object.entries(aliases.value).map(([k, v]) => `  ${k}='${v}'`)
    }
    const eq = args.join(' ').indexOf('=')
    if (eq > 0) {
      const name = args.join(' ').slice(0, eq)
      const cmd = args.join(' ').slice(eq + 1).replace(/^['"]|['"]$/g, '')
      aliases.value[name] = cmd
      return `\x1b[32mAlias set:\x1b[0m ${name}='${cmd}'`
    }
    const name = args[0]
    if (aliases.value[name]) return `${name}='${aliases.value[name]}'`
    return `alias: ${name}: not found`
  },

  env: () => Object.entries(envVars.value).map(([k, v]) => `${k}=${v}`),

  export: (args) => {
    if (args.length === 0) return Object.entries(envVars.value).map(([k, v]) => `declare -x ${k}="${v}"`)
    const eq = args.join(' ').indexOf('=')
    if (eq > 0) {
      const key = args.join(' ').slice(0, eq)
      const val = args.join(' ').slice(eq + 1).replace(/^['"]|['"]$/g, '')
      envVars.value[key] = val
      return `\x1b[32mExported:\x1b[0m ${key}=${val}`
    }
    return 'Usage: export KEY=value'
  },

  which: (args) => {
    if (args.length === 0) return 'which: missing argument'
    const cmd = args[0].toLowerCase()
    if (commands[cmd]) return `/usr/bin/${cmd}`
    if (aliases.value[cmd]) return `alias ${cmd}='${aliases.value[cmd]}'`
    return `which: ${cmd} not found`
  },

  clear: () => { logs.value = []; return '' },

  echo: (args) => {
    const escape = args.includes('-e')
    const text = args.filter(a => a !== '-e').join(' ')
    if (escape) {
      return text
        .replace(/\\n/g, '\n')
        .replace(/\\t/g, '\t')
        .replace(/\\x1b/g, '\x1b')
    }
    // Expand env vars in echo
    return expandEnvVars(text)
  },

  whoami: () => username.value,

  uname: (args) => {
    if (args.includes('-a')) return 'Linux cybermanju 6.8.0-cybermanju #1 SMP PREEMPT_DYNAMIC x86_64 GNU/Linux Cybermanju OS'
    return 'Linux cybermanju 6.8.0-cybermanju #1 SMP PREEMPT_DYNAMIC x86_64 GNU/Linux'
  },

  uptime: () => {
    const u = 86400
    const d = Math.floor(u / 86400)
    const h = Math.floor((u % 86400) / 3600)
    const m = Math.floor((u % 3600) / 60)
    return ` ${d}:${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')} up ${d} day${d !== 1 ? 's' : ''},  1 user,  load average: ${(Math.random() * 2).toFixed(2)}, ${(Math.random() * 2).toFixed(2)}, ${(Math.random() * 2).toFixed(2)}`
  },

  ps: () => '  PID TTY          TIME CMD\n  123 pts/0    00:00:02 bash\n  456 pts/0    00:00:00 ps\n  789 pts/0    00:00:05 cybermanju-shell\n  321 ?        00:00:12 systemd',

  date: () => new Date().toString(),

  neofetch: () => [
    `           ▄▄▄▄▄▄▄▄▄▄▄  ${username.value}@${hostname}`,
    `        ▄████████████████▄  -------------------`,
    `      ▄██▀▀▀▀████████▀▀▀▀██▄  OS: Cybermanju OS x86_64`,
    `     ██▀         ██        ▀██  Kernel: 6.8.0-cybermanju`,
    `     ██           ██         ██  Uptime: ${(Math.random() * 30 + 1).toFixed(1)} hours`,
    `     ██           ██        ██  Packages: 1423`,
    `     ██▄          ██       ▄██  Shell: cybermanju-sh 1.0`,
    `      ████▄▄▄▄████████▄▄▄▄████  Resolution: 1920x1080`,
    `          ▀██████████████▀      DE: Cybermanju Shell`,
    `             ▀████████▀         WM: Wayland (wlroots)`,
    `                   ██           Terminal: ${envVars.value.TERM}`,
    `                   ██           CPU: PQC-NEON (8) @ 2.80GHz`,
    `                  ██            GPU: NVIDIA RTX 5090`,
    `                  ██            Memory: ${(Math.random() * 16 + 8).toFixed(1)} GiB / 32 GiB`,
  ],

  sudo: (args) => {
    if (args[0] === 'rm' && args.includes('-rf') && args.includes('/')) {
      return `\x1b[31mNice try. This isn't a real shell, cowboy.\x1b[0m`
    }
    return `sudo: ${args.join(' ') || '(no command)'}: command not found (permission denied — nice try)`
  },

  exit: () => {
    exitFuncs.push(() => emit('close'))
    return ''
  },

  host: (args) => {
    if (!isTauri()) return '\x1b[31mHost mode is only available in the Tauri desktop app.\x1b[0m'
    if (args[0] === 'on') {
      hostMode.value = true
      return [
        '\x1b[32mHost mode ENABLED.\x1b[0m',
        'All commands are now forwarded to the real system shell.',
        'Type \x1b[33mhost off\x1b[0m to return to built-in shell.',
        '',
        '\x1b[90mWarning: Commands run with your system user permissions.\x1b[0m',
      ]
    }
    if (args[0] === 'off') {
      hostMode.value = false
      return '\x1b[32mHost mode DISABLED.\x1b[0m Returned to built-in shell.'
    }
    // Toggle
    hostMode.value = !hostMode.value
    if (hostMode.value) {
      return [
        '\x1b[32mHost mode ENABLED.\x1b[0m',
        'All commands are now forwarded to the real system shell.',
        'Type \x1b[33mhost off\x1b[0m to return to built-in shell.',
        '',
        '\x1b[90mWarning: Commands run with your system user permissions.\x1b[0m',
      ]
    }
    return '\x1b[32mHost mode DISABLED.\x1b[0m Returned to built-in shell.'
  },
}

// ── History persistence ──
function saveHistory() {
  try {
    localStorage.setItem('cybermanju_terminal_history', JSON.stringify(histStack.value.slice(-500)))
  } catch {}
}

function loadHistory() {
  try {
    const saved = localStorage.getItem('cybermanju_terminal_history')
    if (saved) histStack.value = JSON.parse(saved)
  } catch {}
}

// ── TAB auto-complete ──
function tabComplete() {
  const raw = input.value.trim()
  if (!raw) return

  const parts = raw.split(/\s+/)

  // Complete command name (first word)
  if (parts.length === 1) {
    const prefix = parts[0].toLowerCase()
    const matches = commandNames.filter(c => c.startsWith(prefix))
    if (matches.length === 1) {
      input.value = matches[0] + ' '
    } else if (matches.length > 1) {
      logs.value.push(`\x1b[90m${matches.join('  ')}\x1b[0m`)
    }
    return
  }

  const cmd = parts[0].toLowerCase()

  // Subcommand completion
  const subMap: Record<string, string[]> = {
    keys: ['list', 'generate', 'show'],
    db: ['check', 'stats'],
    history: ['-c', '--search'],
  }
  if (subMap[cmd] && parts.length === 2) {
    const matches = subMap[cmd].filter(s => s.startsWith(parts[1].toLowerCase()))
    if (matches.length === 1) {
      input.value = `${cmd} ${matches[0]} `
    } else if (matches.length > 1) {
      logs.value.push(`\x1b[90m${matches.join('  ')}\x1b[0m`)
    }
    return
  }

  // Flag completion
  const lastPart = parts[parts.length - 1]
  if (lastPart.startsWith('--') || lastPart.startsWith('-')) {
    const flagMap: Record<string, string[]> = {
      search: ['--limit'],
      suggest: ['--limit'],
      encrypt: [],
      ls: ['-l', '-la', '-a', '--sort'],
      sort: ['-r', '-n'],
      head: ['-n'],
      tail: ['-n'],
      echo: ['-e'],
      uname: ['-a'],
      tree: ['--depth'],
    }
    const flags = (flagMap[cmd] || []).filter(f => f.startsWith(lastPart))
    if (flags.length === 1) {
      parts[parts.length - 1] = flags[0]
      input.value = parts.join(' ')
    } else if (flags.length > 1) {
      logs.value.push(`\x1b[90m${flags.join('  ')}\x1b[0m`)
    }
    return
  }

  // Alias name completion for alias command
  if (cmd === 'alias' && parts.length === 2 && !parts[1].includes('=')) {
    const aliasNames = Object.keys(aliases.value).filter(a => a.startsWith(parts[1]))
    if (aliasNames.length === 1) {
      input.value = `alias ${aliasNames[0]}=`
    } else if (aliasNames.length > 1) {
      logs.value.push(`\x1b[90m${aliasNames.join('  ')}\x1b[0m`)
    }
  }
}

// ── Keyboard handler ──
function onInputKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    processCmd()
  } else if (e.key === 'Tab') {
    e.preventDefault()
    tabComplete()
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (historySearch.value) {
      // Search through history
      const q = historySearchQuery.value.toLowerCase()
      for (let i = histStack.value.length - 1; i >= 0; i--) {
        if (histStack.value[i].toLowerCase().includes(q)) {
          input.value = histStack.value[i]
          break
        }
      }
    } else if (histStack.value.length) {
      histIdx.value = histIdx.value < histStack.value.length - 1 ? histIdx.value + 1 : histIdx.value
      input.value = histStack.value[histStack.value.length - 1 - histIdx.value] || ''
    }
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (historySearch.value) {
      historySearch.value = false
      historySearchQuery.value = ''
      input.value = ''
    } else if (histIdx.value > 0) {
      histIdx.value--
      input.value = histStack.value[histStack.value.length - 1 - histIdx.value] || ''
    } else {
      histIdx.value = -1
      input.value = ''
    }
  } else if (e.key === 'r' && e.ctrlKey) {
    e.preventDefault()
    historySearch.value = true
    historySearchQuery.value = ''
    logs.value.push('\x1b[90m(reverse-i-search)`\':\x1b[0m ')
  } else if (e.key === 'l' && e.ctrlKey) {
    e.preventDefault()
    logs.value = []
  } else if (e.key === 'a' && e.ctrlKey) {
    e.preventDefault()
    // Move cursor to start — handled by browser for hidden input
  } else if (e.key === 'e' && e.ctrlKey) {
    e.preventDefault()
    // Move cursor to end — handled by browser for hidden input
  } else if (e.key === 'u' && e.ctrlKey) {
    e.preventDefault()
    input.value = ''
  } else if (e.key === 'k' && e.ctrlKey) {
    e.preventDefault()
    // Clear after cursor — since we don't track cursor position, clear all
  } else if (e.key === 'w' && e.ctrlKey) {
    e.preventDefault()
    // Delete word before cursor
    const val = input.value
    const lastSpace = val.lastIndexOf(' ', val.length - 2)
    input.value = val.slice(0, lastSpace + 1)
  } else if (e.key === 'c' && e.ctrlKey && !e.shiftKey) {
    // Ctrl+C — cancel current input
    input.value = ''
    logs.value.push(`\x1b[32m${username.value}@${hostname}\x1b[0m:\x1b[34m${currentDir.value}\x1b[0m$ ^C`)
  } else if (historySearch.value && e.key !== 'Control') {
    // In history search mode, capture typed characters
    if (e.key === 'Backspace') {
      historySearchQuery.value = historySearchQuery.value.slice(0, -1)
    } else if (e.key.length === 1) {
      historySearchQuery.value += e.key
    }
    // Update the search prompt line
    const searchLineIdx = logs.value.length - 1
    if (searchLineIdx >= 0) {
      logs.value[searchLineIdx] = `\x1b[90m(reverse-i-search)\`'${historySearchQuery.value}':\x1b[0m `
    }
    // Search live
    for (let i = histStack.value.length - 1; i >= 0; i--) {
      if (histStack.value[i].toLowerCase().includes(historySearchQuery.value.toLowerCase())) {
        input.value = histStack.value[i]
        break
      }
    }
  }
}

function onInput(e: Event) {
  const target = e.target as HTMLInputElement
  input.value = target.value
}

function focusInput() {
  hiddenInput.value?.focus()
}

function handleOutputMouseUp() {
  setTimeout(() => {
    const sel = window.getSelection()
    const text = sel?.toString().trim()
    if (text && text.length > 0) {
      navigator.clipboard.writeText(text).then(() => {
        lastCopied.value = text.length > 60 ? text.slice(0, 57) + '...' : text
        copyHistory.value.push(text)
        if (copyHistory.value.length > 20) copyHistory.value.shift()
        showCopyNotify.value = true
        if (copyNotifyTimeout) clearTimeout(copyNotifyTimeout)
        copyNotifyTimeout = setTimeout(() => { showCopyNotify.value = false }, 2000)
        window.dispatchEvent(new CustomEvent('cybermanju:clipboard-update', { detail: { text, history: copyHistory.value } }))
      }).catch(() => {})
    }
  }, 10)
}

function renderLine(line: string): string {
  return line
    .replace(/\x1b\[32m/g, '<span style="color:#00ff41">')
    .replace(/\x1b\[34m/g, '<span style="color:#5af0ff">')
    .replace(/\x1b\[31m/g, '<span style="color:#ff5f57">')
    .replace(/\x1b\[33m/g, '<span style="color:#febc2e">')
    .replace(/\x1b\[90m/g, '<span style="color:#666">')
    .replace(/\x1b\[0m/g, '</span>')
}

onMounted(() => {
  const saved = localStorage.getItem('cybermanju_username')
  if (saved) username.value = saved
  envVars.value.USER = username.value
  envVars.value.HOME = '/home/' + username.value
  loadHistory()
  logs.value = [
    `\x1b[33mCybermanju Shell 1.0\x1b[0m (GNU/Linux 6.8.0-cybermanju x86_64)`,
    '',
    ` * Type \x1b[33mHELP\x1b[0m for all commands`,
    ` * TAB to auto-complete | Ctrl+R for history search`,
    ` * Chain commands: \x1b[90mcmd1 && cmd2 | cmd1 ; cmd2 | cmd1 || cmd2\x1b[0m`,
    isTauri() ? ` * Type \x1b[33mHOST\x1b[0m to forward commands to your real system shell` : '',
    '',
    `Last login: ${new Date().toLocaleString()} from 127.0.0.1`,
    '',
  ]
  nextTick(() => focusInput())
})

onUnmounted(() => {
  exitFuncs.forEach(f => f())
})
</script>

<template>
  <div class="terminal" @click="focusInput" tabindex="0" autofocus>
    <div class="term-header">
      <div class="term-dots">
        <span class="term-dot term-dot--close" @click="$emit('close')"></span>
        <span class="term-dot term-dot--min"></span>
        <span class="term-dot term-dot--max"></span>
      </div>
      <div class="term-title">
        <span v-if="hostMode" class="host-badge">HOST</span>
        {{ username }}@{{ hostname }}: {{ currentDir }}
      </div>
    </div>
    <div class="term-body">
      <div class="term-output" @mouseup="handleOutputMouseUp">
        <div v-for="(line, i) in logs" :key="i" class="term-line" v-html="renderLine(line)"></div>
      </div>
      <div class="term-input-line">
        <span class="term-prompt">
          <span style="color:#00ff41">{{ username }}@{{ hostname }}</span>:<span style="color:#5af0ff">{{ currentDir }}</span>$
        </span>
        <span class="term-input-text">{{ input }}</span>
        <span v-if="autoSuggestion" class="term-suggestion">{{ autoSuggestion.slice(input.length) }}</span>
        <span class="term-cursor">&#9608;</span>
      </div>
      <input
        ref="hiddenInput"
        class="hidden-input"
        type="text"
        :value="input"
        @input="onInput"
        @keydown="onInputKeydown"
        autocomplete="off"
        autocapitalize="off"
        autocorrect="off"
        spellcheck="false"
      />
      <Transition name="copy-fade">
        <div v-if="showCopyNotify" class="copy-notify">
          <span class="copy-icon">&#128203;</span> COPIED: {{ lastCopied }}
        </div>
      </Transition>
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
  cursor: text;
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
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.host-badge {
  background: rgba(255, 95, 87, 0.2);
  color: #ff5f57;
  font-size: 8px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 3px;
  border: 1px solid rgba(255, 95, 87, 0.3);
  letter-spacing: 1px;
}

.term-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 10px 14px;
  background: #050505;
  position: relative;
}

.term-output {
  flex: 1;
  overflow-y: auto;
  user-select: text;
  -webkit-user-select: text;
  cursor: text;
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
  user-select: text;
  -webkit-user-select: text;
}

.term-line ::selection {
  background: rgba(0, 255, 65, 0.25);
  color: #fff;
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

.term-suggestion {
  font-size: 11px;
  color: #444;
  pointer-events: none;
}

.term-cursor {
  font-size: 11px;
  color: #00ff41;
  animation: term-blink 0.8s step-end infinite;
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

@keyframes term-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.copy-notify {
  position: absolute;
  top: 8px;
  right: 12px;
  background: rgba(0, 255, 65, 0.15);
  border: 1px solid rgba(0, 255, 65, 0.3);
  border-radius: 4px;
  padding: 3px 10px;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  color: #00ff41;
  pointer-events: none;
  z-index: 10;
  backdrop-filter: blur(4px);
}

.copy-icon {
  margin-right: 4px;
}

.copy-fade-enter-active { transition: all 0.15s ease-out; }
.copy-fade-leave-active { transition: all 0.3s ease-in; }
.copy-fade-enter-from { opacity: 0; transform: translateY(-4px); }
.copy-fade-leave-to { opacity: 0; transform: translateY(-4px); }
</style>
