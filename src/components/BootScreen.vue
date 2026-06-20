<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useLogin } from '@/composables/useLogin'
import { useAppStore } from '@/stores/app'
import type { OAuthProvider } from '@/wasm/oauth'

const emit = defineEmits<{ (e: 'complete', user: string): void }>()

const { login: doLogin } = useLogin()
const store = useAppStore()

const bootLogs = ref<string[]>([])
const progress = ref(0)
const showGlitch = ref(false)
const crtFlicker = ref(false)
const ready = ref(false)
const dismissing = ref(false)
const bootError = ref(false)
const usernameInput = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const bootPhase = ref<'post' | 'kernel' | 'done'>('post')
const skipRequested = ref(false)

/* --- Gaussian psychedelic spots --- */
const spots = Array.from({ length: 18 }, (_, i) => ({
  id: i,
  x: Math.random() * 100,
  y: Math.random() * 100,
  size: 120 + Math.random() * 280,
  hue: Math.random() * 360,
  delay: Math.random() * 4,
  duration: 6 + Math.random() * 8,
  drift: -30 + Math.random() * 60,
}))

/* --- Bizarre geometric figures --- */
const figures = Array.from({ length: 8 }, (_, i) => ({
  id: i,
  type: ['eye', 'triangle', 'spiral', 'diamond', 'cross', 'ring', 'hexagon', 'star'][i],
  x: 5 + Math.random() * 90,
  y: 5 + Math.random() * 90,
  size: 20 + Math.random() * 60,
  rotation: Math.random() * 360,
  hue: Math.random() * 360,
  delay: Math.random() * 3,
  duration: 4 + Math.random() * 6,
}))

/* --- Storage providers --- */
const PROVIDER_META = [
  { id: 'google', label: 'Google (Drive + Photos)', icon: 'logos:google-icon', color: '#4285F4' },
  { id: 'github', label: 'GitHub', icon: 'logos:github-icon', color: '#333' },
  { id: 'gitlab', label: 'GitLab', icon: 'logos:gitlab', color: '#FC6D26' },
  { id: 'telegram', label: 'Telegram', icon: 'logos:telegram', color: '#0088CC' },
  { id: 'mega', label: 'Mega.nz', icon: 'logos:mega', color: '#D9272E' },
]

const PROVIDER_TO_OAUTH: Record<string, OAuthProvider> = {
  google: 'googleDrive',
  github: 'github',
  gitlab: 'gitlab',
  telegram: 'telegram',
}

const connectedProviders = ref<string[]>([])
const connectingProvider = ref<string | null>(null)
const providerError = ref('')
const showClientIdForm = ref<string | null>(null)
const clientIdInput = ref('')

const showMegaModal = ref(false)
const megaEmail = ref('')
const megaPassword = ref('')
const megaLabel = ref('Mega')
const mega2FACode = ref('')
const megaVerifying = ref(false)
const megaVerifyError = ref('')

let timer: ReturnType<typeof setTimeout> | null = null
let glitchTimer: ReturnType<typeof setInterval> | null = null
let bootTimeout: ReturnType<typeof setTimeout> | null = null

interface SysInfo {
  os_name: string
  os_version: string
  os_arch: string
  hostname: string
  cpu_brand: string
  cpu_cores: number
  cpu_threads: number
  total_memory_mb: number
  used_memory_mb: number
  total_disk_gb: number
  used_disk_gb: number
  kernel_version: string
  uptime_seconds: number
}

/* ─── Merged boot sequence: POST → KERNEL → LOGIN ─── */
function buildMergedBootLines(s: SysInfo | null): Array<{ msg: string; delay: number; pct: number }> {
  const cpu = s?.cpu_brand || 'Unknown CPU'
  const mem = s?.total_memory_mb || 0
  const cores = s?.cpu_cores || 1
  const threads = s?.cpu_threads || 1
  const kernel = s?.kernel_version || 'unknown'
  const os = s?.os_name || 'Unknown'
  const arch = s?.os_arch || 'unknown'
  const hostname = s?.hostname || 'localhost'
  const disk = s?.total_disk_gb || 0
  const now = new Date()
  const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')} ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}`
  const memChannels = Math.max(2, Math.ceil(cores / 4))
  const memPerChannel = Math.round(mem / memChannels)

  return [
    /* ═══ PHASE 1: BIOS POST ═══ */
    { msg: `\x1b[36mCYBERMANJU UEFI BIOS v2.4.1\x1b[0m (Build ${dateStr})`, delay: 400, pct: 1 },
    { msg: `CPU: ${cpu} — ${cores}C/${threads}T [PASS]`, delay: 200, pct: 2 },
    { msg: `CPU Features: AES-NI, SHA-NI, AVX2, AVX-512, VAES, VPCLMULQDQ`, delay: 180, pct: 3 },
    { msg: `MEM: Testing ${mem}MB DDR5...`, delay: 200, pct: 4 },
    ...Array.from({ length: memChannels }, (_, i) => ({
      msg: `MEM: Channel ${String.fromCharCode(65 + i)}: ${memPerChannel}MB [OK]`,
      delay: 120,
      pct: 4 + (i + 1),
    })),
    { msg: `MEM: POST-QUANTUM CRYPTO ZONES: ${Math.max(4, Math.floor(cores / 2))} GUARD REGIONS [ACTIVE]`, delay: 250, pct: 9 },
    { msg: `PCH: Chipset — DMI 4.0 x8 [DETECTED]`, delay: 180, pct: 10 },
    { msg: `PCI: Bus enumeration... devices found [COMPLETE]`, delay: 200, pct: 11 },
    { msg: `USB: XHCI Controller at 0xFE800000 (irq 16)`, delay: 150, pct: 12 },
    { msg: `USB: hubs, devices enumerated [OK]`, delay: 150, pct: 13 },
    { msg: `NET: Network interface [DETECTED]`, delay: 180, pct: 14 },
    { msg: `NET: Wireless adapter [DETECTED]`, delay: 180, pct: 15 },
    { msg: `SND: Audio Controller [INITIALIZED]`, delay: 150, pct: 16 },
    { msg: `TPM: 2.0 Security Module [ACTIVE]`, delay: 180, pct: 17 },
    { msg: `RTC: System Clock — ${dateStr} UTC [SYNCED]`, delay: 150, pct: 18 },
    { msg: `ACPI: DSDT loaded [PARSED]`, delay: 180, pct: 19 },
    { msg: `SYS: CMOS checksum OK [NOMINAL]`, delay: 180, pct: 20 },
    { msg: `\x1b[32mPOST complete\x1b[0m — transitioning to kernel...`, delay: 300, pct: 21 },

    /* ═══ PHASE 2: KERNEL BOOT ═══ */
    { msg: `[BOOT] Cybermanju Drive Kernel v4.2.0-RELEASE (${arch})`, delay: 70, pct: 23 },
    { msg: `[BOOT] CPU: ${cpu}, ${cores}C/${threads}T`, delay: 55, pct: 25 },
    { msg: `[BOOT] MEM: ${mem}MB POST-QUANTUM CRYPTO RAM (ECC)`, delay: 60, pct: 27 },
    { msg: '[BIOS] ACPI: IRQ routing table loaded', delay: 50, pct: 28 },
    { msg: '[KERN] Initializing memory protection — NX, ASLR, SMEP', delay: 65, pct: 30 },
    { msg: '[KERN] CRYPTO: ChaCha20-Poly1305 hardware acceleration ENABLED', delay: 55, pct: 32 },
    { msg: '[KERN] CRYPTO: Kyber-1024 key encapsulation module loaded', delay: 70, pct: 34 },
    { msg: '[KERN] CRYPTO: Dilithium-5 signature verification online', delay: 60, pct: 36 },
    { msg: '[KERN] VFS: Mounting root filesystem (ext4, encrypted)', delay: 75, pct: 38 },
    { msg: '[KERN] VFS: /dev/sda1 — LUKS2 (Argon2id) unlocked', delay: 65, pct: 40 },
    { msg: '[KERN] VFS: /dev/sdb1 — XFS, journal replay OK', delay: 50, pct: 41 },
    { msg: `[KERN] NET: eth0 — link UP`, delay: 55, pct: 43 },
    { msg: '[KERN] NET: wlan0 — scan complete', delay: 60, pct: 45 },
    { msg: '[KERN] NET: IPv6 stack ready — SLAAC configured', delay: 45, pct: 46 },
    { msg: '[KERN] USB: OHCI controller #1 at 0xFE800000 (irq 16)', delay: 50, pct: 48 },
    { msg: '[KERN] USB: hubs, devices enumerated', delay: 45, pct: 49 },
    { msg: '[KERN] DRM: efifb — display active', delay: 55, pct: 51 },
    { msg: '[KERN] SND: Audio controller detected', delay: 45, pct: 52 },
    { msg: '[KERN] SND: ALSA device list loaded', delay: 45, pct: 54 },
    { msg: '[KERN] RNG: crng init done — entropy pool seeded', delay: 55, pct: 56 },
    { msg: '[KERN] RTC: system clock synced to hardware (UTC)', delay: 40, pct: 57 },
    { msg: '[INIT] Starting init daemon (PID 1): openrc-0.52', delay: 50, pct: 59 },
    { msg: '[INIT] Mounting pseudo-filesystems: proc, sysfs, tmpfs, devpts', delay: 55, pct: 61 },
    { msg: '[INIT] Loading kernel modules: cryptodev, ipsec, wireguard', delay: 50, pct: 62 },
    { msg: '[INIT] Starting udev: device manager online', delay: 40, pct: 64 },
    { msg: '[INIT] Starting syslog-ng: logging daemon active', delay: 45, pct: 66 },
    { msg: '[INIT] Starting sshd: OpenSSH_9.4 (port 2222)', delay: 50, pct: 67 },
    { msg: '[INIT] Starting nginx: HTTPS reverse proxy online', delay: 45, pct: 69 },
    { msg: '[INIT] Starting postgresql: database cluster ready', delay: 55, pct: 71 },
    { msg: '[INIT] Starting redis: cache layer initialized', delay: 40, pct: 72 },
    { msg: '[DAEMON] cybermanju-syncd — sync orchestrator starting...', delay: 50, pct: 74 },
    { msg: '[DAEMON] cybermanju-syncd — 6 backends registered', delay: 45, pct: 76 },
    { msg: '[DAEMON] cybermanju-cryptd — quantum-safe tunnel established', delay: 55, pct: 78 },
    { msg: '[DAEMON] cybermanju-watchd — file watcher active (inotify)', delay: 40, pct: 79 },
    { msg: '[DAEMON] cybermanju-indexd — full-text index rebuilt', delay: 50, pct: 81 },
    { msg: '[DAEMON] cybermanju-faced — facial recognition model loaded', delay: 60, pct: 83 },
    { msg: '[DAEMON] cybermanju-geod — geotag index initialized', delay: 45, pct: 85 },
    { msg: '[SHELL] Starting Cybermanju Drive Session Manager (SDM)', delay: 50, pct: 87 },
    { msg: '[SHELL] SDM: policykit authority acquired', delay: 40, pct: 89 },
    { msg: '[SHELL] SDM: D-Bus session bus listening', delay: 45, pct: 91 },
    { msg: `[SHELL] SDM: ${os} ${kernel} (${arch})`, delay: 55, pct: 93 },
    { msg: `[SHELL] SDM: ${hostname} — desktop environment ready`, delay: 50, pct: 95 },
    { msg: `[SHELL] ${disk > 0 ? `Disk ${disk.toFixed(0)}GB mounted` : 'Storage mounted'} — startup sequence complete.`, delay: 70, pct: 98 },
    { msg: '\x1b[36m[LOGIN]\x1b[0m Cybermanju Drive — system ready. awaiting authentication...', delay: 100, pct: 100 },
  ]
}

function addBootLine(line: string) {
  bootLogs.value.push(line)
  const el = document.querySelector('.boot-terminal-text')
  if (el) el.scrollTop = el.scrollHeight
}

function triggerGlitch() {
  showGlitch.value = true
  crtFlicker.value = true
  setTimeout(() => { showGlitch.value = false }, 150 + Math.random() * 300)
  setTimeout(() => { crtFlicker.value = false }, 30 + Math.random() * 60)
}

async function runBoot() {
  let sysInfo: SysInfo | null = null
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    sysInfo = await invoke<SysInfo>('get_system_info')
  } catch {}

  const bootLines = buildMergedBootLines(sysInfo)

  for (const entry of bootLines) {
    if (skipRequested.value) {
      progress.value = 100
      break
    }
    await new Promise(r => setTimeout(r, entry.delay + Math.random() * 30))
    addBootLine(entry.msg)
    progress.value = entry.pct

    if (Math.random() < 0.12) triggerGlitch()

    if (entry.msg.includes('RNG') || entry.msg.includes('dilithium')) {
      addBootLine('  \x1b[33mWARN\x1b[0m: entropy level marginal — using hybrid seed')
    }
    if (entry.msg.includes('NET:') && Math.random() < 0.5) {
      addBootLine('  \x1b[33mWARN\x1b[0m: interface rx buffer adjusted (4096 -> 8192)')
    }
    if (Math.random() < 0.06) {
      await new Promise(r => setTimeout(r, 150 + Math.random() * 300))
      triggerGlitch()
      addBootLine(`  \x1b[31mPANIC\x1b[0m: ... recovering via watchdog ... OK`)
    }

    if (entry.pct >= 21 && bootPhase.value === 'post') {
      bootPhase.value = 'kernel'
    }
  }

  await new Promise(r => setTimeout(r, 400))
  triggerGlitch()
  await new Promise(r => setTimeout(r, 500))
  bootPhase.value = 'done'
  ready.value = true
}

async function submitLogin() {
  if (dismissing.value) return
  const name = usernameInput.value.trim()
  if (!name) return
  dismissing.value = true
  triggerGlitch()
  await doLogin(name)
  setTimeout(() => emit('complete', name), 400)
}

function handleUsernameKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') submitLogin()
}

/* --- Storage provider connection --- */
async function connectProvider(pid: string) {
  if (pid === 'mega') { openMegaModal(); return }
  if (connectingProvider.value) return
  providerError.value = ''

  const oauthKey = PROVIDER_TO_OAUTH[pid]
  if (!oauthKey) return

  connectingProvider.value = pid
  try {
    const { oauth } = await import('@/wasm')
    const data = await import('@/wasm/data')
    oauth.loadClientIdsFromEnv()

    const clientId = oauth.getProviderClientId(oauthKey)
    if (!clientId) {
      showClientIdForm.value = pid
      connectingProvider.value = null
      return
    }

    const existingToken = await oauth.loadTokenFromStorage(oauthKey)
    let token = existingToken ? await oauth.getValidToken(existingToken) : null
    if (token) {
      await oauth.saveTokenToStorage(token)
    } else {
      token = await oauth.authenticateWithPopup(oauthKey)
      await oauth.saveTokenToStorage(token)
    }

    await data.upsertOAuthAccount(oauthKey, token)
    if (pid === 'google') {
      await data.upsertOAuthAccount('googlePhotos' as OAuthProvider, token)
    }

    await store.fetchAccounts()
    if (!connectedProviders.value.includes(pid)) {
      connectedProviders.value.push(pid)
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    if (msg.includes('Client ID not configured')) {
      showClientIdForm.value = pid
    } else if (msg.includes('Popup blocked')) {
      providerError.value = `${pid.toUpperCase()}: POPUP BLOCKED — allow popups for this site`
    } else if (msg.includes('timed out') || msg.includes('closed by the user')) {
      providerError.value = `${pid.toUpperCase()}: AUTHORIZATION ${msg.includes('timed out') ? 'TIMED OUT' : 'CANCELLED'}`
    } else {
      providerError.value = `${pid.toUpperCase()}: ${msg}`
    }
  } finally {
    connectingProvider.value = null
  }
}

function providerIcon(pid: string): string {
  return PROVIDER_META.find(p => p.id === pid)?.icon || 'mdi:cloud-outline'
}

async function saveClientId() {
  if (!showClientIdForm.value || !clientIdInput.value.trim()) return
  const pid = showClientIdForm.value
  const oauthKey = PROVIDER_TO_OAUTH[pid]
  if (!oauthKey) return

  const { oauth } = await import('@/wasm')
  oauth.setProviderClientId(oauthKey, clientIdInput.value.trim())
  if (pid === 'google') {
    oauth.setProviderClientId('googlePhotos' as OAuthProvider, clientIdInput.value.trim())
  }

  showClientIdForm.value = null
  clientIdInput.value = ''
  await connectProvider(pid)
}

function openMegaModal() {
  megaEmail.value = ''
  megaPassword.value = ''
  megaLabel.value = 'Mega'
  megaVerifyError.value = ''
  mega2FACode.value = ''
  showMegaModal.value = true
}

function closeMegaModal() {
  showMegaModal.value = false
  megaVerifying.value = false
  megaVerifyError.value = ''
  mega2FACode.value = ''
}

async function verifyAndConnectMega() {
  megaVerifyError.value = ''
  if (!megaEmail.value.trim() || !megaPassword.value.trim()) {
    megaVerifyError.value = 'EMAIL AND PASSWORD ARE REQUIRED'
    return
  }

  megaVerifying.value = true
  connectingProvider.value = 'mega'
  try {
    const label = megaLabel.value.trim() || 'Mega'
    const token = `${megaEmail.value.trim()}|${megaPassword.value}`

    const testConfig: any = {
      id: '', backendType: 'mega', enabled: true, name: label,
      basePath: '/', token, secondFactorCode: mega2FACode.value.trim() || undefined,
      autoSync: false, compressBeforeUpload: false, createPreviews: false,
      deleteRawAfterSync: false, maxConcurrentUploads: 1,
    }

    await store.testSyncConnection(testConfig)

    const { saveTokenToStorage } = await import('@/wasm/oauth')
    await saveTokenToStorage({
      accessToken: token, refreshToken: null, expiresAt: null,
      tokenType: 'mega', scope: null, provider: 'mega' as any,
    })

    const data = await import('@/wasm/data')
    await data.upsertMegaAccount(label, token)
    await store.fetchAccounts()

    if (!connectedProviders.value.includes('mega')) {
      connectedProviders.value.push('mega')
    }
    closeMegaModal()
  } catch (e) {
    megaVerifyError.value = e instanceof Error ? e.message : 'CONNECTION FAILED'
  } finally {
    megaVerifying.value = false
    connectingProvider.value = null
  }
}

function handleSkipKey(e: KeyboardEvent) {
  if (e.code === 'Space' && !ready.value && !dismissing.value && !skipRequested.value) {
    e.preventDefault()
    skipRequested.value = true
  }
}

onMounted(() => {
  usernameInput.value = localStorage.getItem('cybermanju_username') || ''
  window.addEventListener('keydown', handleSkipKey)

  glitchTimer = setInterval(() => {
    if (Math.random() < 0.04) triggerGlitch()
    crtFlicker.value = Math.random() < 0.003
  }, 1000)

  bootTimeout = setTimeout(() => {
    if (!ready.value && !dismissing.value) {
      bootError.value = true
      addBootLine('[KERN] \x1b[31mWATCHDOG\x1b[0m: boot sequence stalled — forcing completion')
    }
  }, 30000)

  runBoot().catch((err) => {
    bootError.value = true
    addBootLine(`[KERN] \x1b[31mFATAL\x1b[0m: ${err instanceof Error ? err.message : String(err)}`)
  })
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleSkipKey)
  if (timer) clearTimeout(timer)
  if (glitchTimer) clearInterval(glitchTimer)
  if (bootTimeout) clearTimeout(bootTimeout)
})
</script>

<template>
  <div class="boot-screen" :class="{ 'glitch-active': showGlitch, 'crt-flicker': crtFlicker, ready, dismissing }">
    <!-- ═══ Gaussian psychedelic spots ═══ -->
    <div class="gaussian-layer">
      <div
        v-for="spot in spots"
        :key="spot.id"
        class="gaussian-spot"
        :style="{
          left: spot.x + '%',
          top: spot.y + '%',
          width: spot.size + 'px',
          height: spot.size + 'px',
          '--hue': spot.hue,
          '--drift': spot.drift + 'px',
          animationDelay: spot.delay + 's',
          animationDuration: spot.duration + 's',
        }"
      />
    </div>

    <!-- ═══ Bizarre floating figures ═══ -->
    <div class="figures-layer">
      <svg v-for="fig in figures" :key="fig.id" class="bizarre-figure"
        :style="{
          left: fig.x + '%',
          top: fig.y + '%',
          width: fig.size + 'px',
          height: fig.size + 'px',
          '--hue': fig.hue,
          animationDelay: fig.delay + 's',
          animationDuration: fig.duration + 's',
        }"
        viewBox="0 0 100 100"
      >
        <!-- Eye -->
        <template v-if="fig.type === 'eye'">
          <ellipse cx="50" cy="50" rx="45" ry="25" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4"/>
          <circle cx="50" cy="50" r="12" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.5"/>
          <circle cx="50" cy="50" r="4" fill="currentColor" opacity="0.6"/>
        </template>
        <!-- Triangle -->
        <template v-else-if="fig.type === 'triangle'">
          <polygon points="50,5 95,90 5,90" fill="none" stroke="currentColor" stroke-width="1.2" opacity="0.35"/>
          <polygon points="50,25 78,78 22,78" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.25"/>
        </template>
        <!-- Spiral -->
        <template v-else-if="fig.type === 'spiral'">
          <path d="M50,50 Q55,30 70,35 Q85,40 80,55 Q75,70 60,65 Q45,60 50,50 Q55,40 65,45" fill="none" stroke="currentColor" stroke-width="1" opacity="0.35"/>
        </template>
        <!-- Diamond -->
        <template v-else-if="fig.type === 'diamond'">
          <polygon points="50,5 95,50 50,95 5,50" fill="none" stroke="currentColor" stroke-width="1.2" opacity="0.35"/>
          <polygon points="50,20 80,50 50,80 20,50" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.2"/>
        </template>
        <!-- Cross -->
        <template v-else-if="fig.type === 'cross'">
          <line x1="50" y1="10" x2="50" y2="90" stroke="currentColor" stroke-width="1.2" opacity="0.3"/>
          <line x1="10" y1="50" x2="90" y2="50" stroke="currentColor" stroke-width="1.2" opacity="0.3"/>
          <line x1="22" y1="22" x2="78" y2="78" stroke="currentColor" stroke-width="0.8" opacity="0.2"/>
          <line x1="78" y1="22" x2="22" y2="78" stroke="currentColor" stroke-width="0.8" opacity="0.2"/>
        </template>
        <!-- Ring -->
        <template v-else-if="fig.type === 'ring'">
          <circle cx="50" cy="50" r="42" fill="none" stroke="currentColor" stroke-width="1.2" opacity="0.3"/>
          <circle cx="50" cy="50" r="30" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.2"/>
          <circle cx="50" cy="50" r="18" fill="none" stroke="currentColor" stroke-width="0.6" opacity="0.15"/>
        </template>
        <!-- Hexagon -->
        <template v-else-if="fig.type === 'hexagon'">
          <polygon points="50,5 90,27.5 90,72.5 50,95 10,72.5 10,27.5" fill="none" stroke="currentColor" stroke-width="1.2" opacity="0.3"/>
          <polygon points="50,20 75,35 75,65 50,80 25,65 25,35" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.2"/>
        </template>
        <!-- Star -->
        <template v-else-if="fig.type === 'star'">
          <polygon points="50,5 61,35 95,35 68,57 79,90 50,70 21,90 32,57 5,35 39,35" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3"/>
        </template>
      </svg>
    </div>

    <!-- ═══ CRT effects ═══ -->
    <div class="crt-scanlines"></div>
    <div class="crt-vignette"></div>
    <div class="glitch-slice" v-for="n in 5" :key="n" :style="{ top: `${10 + Math.random() * 80}%`, height: `${2 + Math.random() * 6}px`, animationDelay: `${Math.random() * 2}s` }"></div>
    <div class="static-overlay" :style="{ opacity: Math.random() * 0.04 }"></div>

    <!-- ═══ Boot terminal ═══ -->
    <div class="boot-terminal">
      <div class="boot-terminal-header">
        <span class="boot-title">CYBERMANJU DRIVE — INIT SEQUENCE</span>
        <span class="boot-version">v4.2.0</span>
      </div>
      <div class="boot-terminal-text">
        <div v-for="(line, i) in bootLogs" :key="i" class="boot-line" :class="{
          'error-line': line.includes('ERR'),
          'warn-line': line.includes('WARN'),
          'panic-line': line.includes('PANIC'),
          'phase-post': bootPhase === 'post' && i < 22,
          'phase-kernel': bootPhase === 'kernel' && i >= 22,
        }">
          <span class="line-arrow">></span>
          <span class="line-text" v-html='line.replace(/\x1b\[33m/g, "<span class=\"warn\">").replace(/\x1b\[31m/g, "<span class=\"err\">").replace(/\x1b\[36m/g, "<span class=\"cyan\">").replace(/\x1b\[32m/g, "<span class=\"green\">").replace(/\x1b\[0m/g, "</span>")'></span>
        </div>
        <div v-if="progress < 100" class="boot-line boot-cursor">
          <span class="line-arrow">></span>
          <span class="cursor-blink">_</span>
        </div>
      </div>
      <div class="boot-progress-track">
        <div class="boot-progress-fill" :style="{ width: progress + '%' }"></div>
        <div class="boot-progress-label">{{ progress }}%</div>
      </div>
      <div v-if="!ready && !skipRequested" class="boot-skip-hint">
        <span class="skip-text">SPACE to skip</span>
      </div>
      <div v-else-if="skipRequested && !ready" class="boot-skip-hint">
        <span class="skip-text skip-active">Skipping...</span>
      </div>
      <div class="boot-hints">
        <span v-if="bootError" class="boot-error-hint">BOOT ERROR — CHECK LOGS</span>
        <span v-else-if="progress < 21">BIOS POST — HARDWARE DETECTION...</span>
        <span v-else-if="progress < 40">KERNEL — LOADING CRYPTO MODULES...</span>
        <span v-else-if="progress < 65">KERNEL — STARTING SERVICES...</span>
        <span v-else-if="progress < 85">DAEMONS — INITIALIZING SUBSYSTEMS...</span>
        <span v-else-if="progress < 100">FINALIZING BOOT...</span>
        <span v-else class="login-hint">SYSTEM READY — ENTER YOUR NAME</span>
      </div>
      <div v-if="bootError" class="boot-emergency">
        <button class="emergency-btn" @click.stop="submitLogin">[ FORCE BOOT ]</button>
      </div>
    </div>

    <!-- ═══ Login overlay ═══ -->
    <div v-if="ready" class="login-overlay" tabindex="0" autofocus>
      <div class="login-backdrop"></div>
      <div class="login-container">
        <div class="login-panel">
          <div class="login-brand">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="#00ff41" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" class="login-logo">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
              <circle cx="12" cy="16" r="1" fill="#00ff41"/>
            </svg>
            <div class="login-brand-text">
              <div class="login-brand-title">CyberManju Drive</div>
              <div class="login-brand-sub">Post-Quantum Encrypted OS</div>
            </div>
          </div>

          <div class="login-section">
            <div class="section-label">USERNAME</div>
            <div class="username-input-wrap">
              <input
                ref="inputRef"
                v-model="usernameInput"
                class="username-input"
                placeholder="ENTER YOUR NAME"
                maxlength="32"
                @keydown="handleUsernameKeydown"
              />
              <button class="login-go-btn" :disabled="!usernameInput.trim() || dismissing" @click="submitLogin">
                [ {{ dismissing ? 'BOOTING...' : 'GO' }} ]
              </button>
            </div>
          </div>

          <div class="login-divider"><span>CONNECT STORAGE (OPTIONAL)</span></div>

          <div class="provider-grid">
            <div
              v-for="p in PROVIDER_META"
              :key="p.id"
              class="provider-card"
              :class="{
                connected: connectedProviders.includes(p.id),
                connecting: connectingProvider === p.id,
              }"
              @click="!connectedProviders.includes(p.id) && !connectingProvider ? connectProvider(p.id) : undefined"
            >
              <Icon :icon="providerIcon(p.id)" width="22" height="22" />
              <div class="provider-name">{{ p.label }}</div>
              <div class="provider-status">
                <span v-if="connectedProviders.includes(p.id)" class="status-ok">CONNECTED</span>
                <span v-else-if="connectingProvider === p.id" class="status-busy">...</span>
                <span v-else class="status-off">+ ADD</span>
              </div>
            </div>
          </div>

          <div v-if="showClientIdForm" class="client-id-form">
            <label>CLIENT ID FOR {{ showClientIdForm.toUpperCase() }}</label>
            <div class="client-id-row">
              <input v-model="clientIdInput" class="bw-input" placeholder="OAuth Client ID" @keyup.enter="saveClientId" />
              <button class="bw-btn bw-btn-cancel" @click="showClientIdForm = null; clientIdInput = ''">CANCEL</button>
              <button class="bw-btn bw-btn-inverse" :disabled="!clientIdInput.trim()" @click="saveClientId">SAVE</button>
            </div>
          </div>

          <div v-if="providerError && !showClientIdForm" class="provider-error">{{ providerError }}</div>
        </div>
      </div>
    </div>

    <!-- ═══ Mega.nz modal ═══ -->
    <div v-if="showMegaModal" class="mega-modal-overlay" @click.self="closeMegaModal">
      <div class="mega-modal">
        <div class="mega-modal-header">
          <Icon icon="logos:mega" width="28" height="28" />
          <span>CONNECT MEGA.NZ</span>
        </div>
        <div class="mega-modal-body">
          <div class="form-row">
            <label>EMAIL</label>
            <input v-model="megaEmail" class="bw-input" placeholder="Mega.nz account email" autocomplete="email" @keyup.enter="verifyAndConnectMega" />
          </div>
          <div class="form-row">
            <label>PASSWORD</label>
            <input v-model="megaPassword" class="bw-input" type="password" placeholder="Mega.nz password" autocomplete="current-password" @keyup.enter="verifyAndConnectMega" />
          </div>
          <div class="form-row">
            <label>LABEL (OPTIONAL)</label>
            <input v-model="megaLabel" class="bw-input" placeholder="e.g. My Mega" />
          </div>
          <div class="form-row">
            <label>2FA CODE (OPTIONAL)</label>
            <input v-model="mega2FACode" class="bw-input" placeholder="Six-digit authenticator code" autocomplete="one-time-code" inputmode="numeric" maxlength="6" @keyup.enter="verifyAndConnectMega" />
          </div>
          <div v-if="megaVerifyError" class="mega-modal-error">{{ megaVerifyError }}</div>
          <div v-if="megaVerifying" class="mega-modal-verifying">
            <Icon icon="svg-spinners:blocks-wave" width="16" height="16" />
            VERIFYING...
          </div>
        </div>
        <div class="mega-modal-footer">
          <button class="bw-btn" :disabled="megaVerifying" @click="closeMegaModal">CANCEL</button>
          <button class="bw-btn bw-btn-inverse" :disabled="megaVerifying || !megaEmail.trim() || !megaPassword.trim()" @click="verifyAndConnectMega">
            {{ megaVerifying ? 'VERIFYING...' : 'VERIFY & CONNECT' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════
   PSYCHEDELIC GAUSSIAN SPOTS
   ═══════════════════════════════════════════════════════ */
@keyframes spot-drift {
  0% { transform: translate(0, 0) scale(1); opacity: 0; }
  15% { opacity: 0.5; }
  50% { transform: translate(var(--drift), calc(var(--drift) * -0.6)) scale(1.3); opacity: 0.35; }
  85% { opacity: 0.5; }
  100% { transform: translate(calc(var(--drift) * -0.5), var(--drift)) scale(0.9); opacity: 0; }
}

.gaussian-layer {
  position: absolute;
  inset: 0;
  z-index: 1;
  pointer-events: none;
  overflow: hidden;
}

.gaussian-spot {
  position: absolute;
  border-radius: 50%;
  background: radial-gradient(circle, hsla(var(--hue), 80%, 55%, 0.25) 0%, hsla(var(--hue), 70%, 45%, 0.08) 40%, transparent 70%);
  filter: blur(40px);
  animation: spot-drift linear infinite;
  mix-blend-mode: screen;
}

/* ═══════════════════════════════════════════════════════
   BIZARRE FLOATING FIGURES
   ═══════════════════════════════════════════════════════ */
@keyframes figure-float {
  0% { transform: translate(0, 0) rotate(0deg) scale(0.6); opacity: 0; }
  20% { opacity: 0.5; }
  50% { transform: translate(15px, -20px) rotate(180deg) scale(1.1); opacity: 0.35; }
  80% { opacity: 0.5; }
  100% { transform: translate(-10px, 15px) rotate(360deg) scale(0.7); opacity: 0; }
}

.figures-layer {
  position: absolute;
  inset: 0;
  z-index: 1;
  pointer-events: none;
  overflow: hidden;
}

.bizarre-figure {
  position: absolute;
  color: hsla(var(--hue), 70%, 65%, 0.5);
  animation: figure-float ease-in-out infinite;
  filter: blur(0.5px);
}

/* ═══════════════════════════════════════════════════════
   CRT + GLITCH EFFECTS
   ═══════════════════════════════════════════════════════ */
@keyframes scanlines {
  0% { transform: translateY(0); }
  100% { transform: translateY(4px); }
}

@keyframes flicker {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.97; }
}

@keyframes glitch-skew {
  0% { transform: skew(0deg); }
  20% { transform: skew(0.5deg); }
  40% { transform: skew(-0.8deg); }
  60% { transform: skew(0.3deg); }
  80% { transform: skew(-0.2deg); }
  100% { transform: skew(0deg); }
}

@keyframes glitch-shift {
  0% { transform: translate(0); filter: hue-rotate(0deg); }
  25% { transform: translate(-3px, 1px); filter: hue-rotate(90deg); }
  50% { transform: translate(2px, -1px); filter: hue-rotate(180deg); }
  75% { transform: translate(-1px, 2px); filter: hue-rotate(270deg); }
  100% { transform: translate(0); filter: hue-rotate(0deg); }
}

@keyframes glitch-slice {
  0% { transform: translateX(-100%); opacity: 0; }
  10% { opacity: 0.6; }
  90% { opacity: 0.6; }
  100% { transform: translateX(100%); opacity: 0; }
}

@keyframes static-noise {
  0% { background-position: 0 0; }
  100% { background-position: 256px 256px; }
}

@keyframes cursor-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

@keyframes hue-rotate {
  0% { filter: hue-rotate(0deg); }
  100% { filter: hue-rotate(360deg); }
}

@keyframes rgb-shift {
  0% { text-shadow: 2px 0 #ff0000, -2px 0 #00ffff; }
  25% { text-shadow: -2px 0 #ff0000, 2px 0 #00ffff; }
  50% { text-shadow: 1px 0 #00ff00, -1px 0 #ff00ff; }
  75% { text-shadow: -1px 0 #00ff00, 1px 0 #ff00ff; }
  100% { text-shadow: 2px 0 #ff0000, -2px 0 #00ffff; }
}

/* ═══════════════════════════════════════════════════════
   BOOT SCREEN
   ═══════════════════════════════════════════════════════ */
.boot-screen {
  position: fixed;
  inset: 0;
  z-index: 99999;
  background: #030308;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'SF Mono', 'Fira Code', 'JetBrains Mono', 'Courier New', monospace;
  overflow: hidden;
}

.boot-screen.glitch-active {
  animation: glitch-skew 0.3s ease-in-out, rgb-shift 0.2s ease-in-out;
}

.boot-screen.glitch-active .boot-terminal {
  animation: glitch-shift 0.3s ease-in-out;
}

.boot-screen.crt-flicker {
  animation: flicker 0.1s ease-in-out 3;
}

/* CRT scan lines */
.crt-scanlines {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0, 0, 0, 0.12) 2px,
    rgba(0, 0, 0, 0.12) 4px
  );
  pointer-events: none;
  z-index: 2;
  animation: scanlines 0.1s linear infinite;
}

/* CRT vignette */
.crt-vignette {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse at center, transparent 50%, rgba(0,0,0,0.7) 100%);
  pointer-events: none;
  z-index: 2;
}

/* Glitch horizontal slices */
.glitch-slice {
  position: absolute;
  left: 0;
  right: 0;
  background: rgba(0, 255, 65, 0.08);
  z-index: 3;
  pointer-events: none;
  animation: glitch-slice 2s ease-in-out infinite;
}

/* Static noise overlay */
.static-overlay {
  position: absolute;
  inset: 0;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
  background-size: 256px 256px;
  animation: static-noise 0.2s steps(4) infinite;
  pointer-events: none;
  z-index: 1;
  opacity: 0.03;
  mix-blend-mode: screen;
}

/* ═══════════════════════════════════════════════════════
   TERMINAL
   ═══════════════════════════════════════════════════════ */
.boot-terminal {
  position: relative;
  z-index: 10;
  width: 720px;
  max-width: 94vw;
  border: 1px solid #1a1a2e;
  border-radius: 8px;
  background: rgba(3, 3, 8, 0.92);
  box-shadow:
    0 0 80px rgba(0, 255, 65, 0.04),
    0 0 160px rgba(0, 255, 65, 0.02),
    inset 0 0 60px rgba(0, 200, 255, 0.01);
  overflow: hidden;
  backdrop-filter: blur(8px);
}

.boot-terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: rgba(5, 5, 15, 0.9);
  border-bottom: 1px solid #1a1a2e;
}

.boot-title {
  font-size: 11px;
  font-weight: 800;
  color: #00ff41;
  letter-spacing: 2px;
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.2), 0 0 20px rgba(0, 255, 65, 0.05);
}

.boot-version {
  font-size: 9px;
  color: #445;
  letter-spacing: 1px;
}

.boot-terminal-text {
  height: 320px;
  max-height: 50vh;
  overflow-y: auto;
  padding: 12px 16px;
  background: rgba(3, 3, 8, 0.6);
}

.boot-terminal-text::-webkit-scrollbar { width: 4px; }
.boot-terminal-text::-webkit-scrollbar-track { background: transparent; }
.boot-terminal-text::-webkit-scrollbar-thumb { background: #1a1a2e; border-radius: 2px; }

.boot-line {
  font-size: 11px;
  line-height: 1.5;
  color: #aab;
  font-weight: 500;
  white-space: pre-wrap;
  word-break: break-all;
}

.line-arrow {
  color: #00ff41;
  margin-right: 8px;
  opacity: 0.6;
}

.line-text { color: #aab; }
.error-line .line-text { color: #ff5f57; }
.warn-line .line-text { color: #febc2e; }
.panic-line .line-text { color: #ff0000; animation: rgb-shift 0.5s ease-in-out; }
.warn { color: #febc2e; }
.err { color: #ff5f57; }
.cyan { color: #00e5ff; }
.green { color: #00ff41; }

.phase-post .line-text { color: #667; }
.phase-kernel .line-text { color: #aab; }

.boot-cursor { margin-top: 2px; }
.cursor-blink {
  color: #00ff41;
  font-weight: 700;
  animation: cursor-blink 0.8s step-end infinite;
}

.boot-progress-track {
  position: relative;
  height: 2px;
  background: rgba(10, 10, 20, 0.8);
  margin: 0 16px 12px;
  border-radius: 1px;
  overflow: hidden;
}

.boot-progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #00ff41, #00e5ff, #00ff41);
  background-size: 200% 100%;
  animation: hue-rotate 2s linear infinite;
  transition: width 0.15s ease-out;
  border-radius: 1px;
  box-shadow: 0 0 12px rgba(0, 255, 65, 0.3), 0 0 24px rgba(0, 229, 255, 0.15);
}

.boot-progress-label {
  position: absolute;
  right: 0;
  top: -16px;
  font-size: 9px;
  color: #00ff41;
  font-weight: 700;
  letter-spacing: 1px;
  text-shadow: 0 0 6px rgba(0, 255, 65, 0.2);
}

.boot-skip-hint {
  text-align: center;
  padding: 0 16px 6px;
}

.skip-text {
  font-family: 'Courier New', monospace;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.3);
  letter-spacing: 2px;
  text-transform: uppercase;
  animation: skip-pulse 2s ease-in-out infinite;
}

.skip-active {
  color: rgba(0, 255, 65, 0.6);
  animation: none;
}

@keyframes skip-pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
}

.boot-hints {
  text-align: center;
  padding: 0 16px 14px;
  font-size: 9px;
  color: #445;
  letter-spacing: 2px;
  font-weight: 600;
}

.boot-error-hint {
  color: #ff5f57;
  animation: cursor-blink 1s step-end infinite;
}

.boot-emergency {
  text-align: center;
  padding: 0 16px 14px;
}

.emergency-btn {
  background: rgba(255, 95, 87, 0.08);
  border: 1px solid #ff5f57;
  border-radius: 4px;
  color: #ff5f57;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 6px 20px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.emergency-btn:hover {
  background: rgba(255, 95, 87, 0.2);
  box-shadow: 0 0 12px rgba(255, 95, 87, 0.15);
}

.login-hint {
  color: #00ff41;
  animation: cursor-blink 1.2s step-end infinite;
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.3);
}

/* ═══════════════════════════════════════════════════════
   LOGIN OVERLAY
   ═══════════════════════════════════════════════════════ */
.login-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'SF Mono', 'Fira Code', 'JetBrains Mono', 'Courier New', monospace;
  background: rgba(3, 3, 8, 0.97);
  outline: none;
}

.login-backdrop {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse at 20% 50%, rgba(0, 255, 65, 0.03) 0%, transparent 60%),
    radial-gradient(ellipse at 80% 50%, rgba(0, 229, 255, 0.02) 0%, transparent 60%),
    radial-gradient(ellipse at 50% 20%, rgba(138, 43, 226, 0.015) 0%, transparent 50%);
  pointer-events: none;
}

.login-container {
  position: relative;
  z-index: 1;
  width: 500px;
  max-width: 94vw;
}

.login-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.login-brand {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 4px;
}

.login-logo {
  filter: drop-shadow(0 0 8px rgba(0, 255, 65, 0.3));
}

.login-brand-title {
  font-size: 16px;
  font-weight: 800;
  color: #00ff41;
  letter-spacing: 3px;
  text-shadow: 0 0 10px rgba(0, 255, 65, 0.2);
}

.login-brand-sub {
  font-size: 9px;
  color: #556;
  letter-spacing: 1px;
}

.login-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-label {
  font-size: 9px;
  color: #556;
  letter-spacing: 2px;
  font-weight: 700;
}

.username-input-wrap {
  display: flex;
  gap: 8px;
  align-items: center;
}

.username-input {
  flex: 1;
  background: rgba(8, 8, 20, 0.8);
  border: 1px solid #1a1a2e;
  border-radius: 6px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  padding: 10px 14px;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
  text-transform: uppercase;
  letter-spacing: 2px;
}

.username-input:focus {
  border-color: #00ff41;
  box-shadow: 0 0 12px rgba(0, 255, 65, 0.1), 0 0 24px rgba(0, 255, 65, 0.03);
}

.username-input::placeholder {
  color: #334;
  letter-spacing: 1px;
}

.login-go-btn {
  background: transparent;
  border: 1px solid #00ff41;
  border-radius: 6px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  font-weight: 700;
  padding: 10px 20px;
  cursor: pointer;
  letter-spacing: 2px;
  transition: all 0.15s;
  white-space: nowrap;
}

.login-go-btn:hover:not(:disabled) {
  background: rgba(0, 255, 65, 0.1);
  box-shadow: 0 0 16px rgba(0, 255, 65, 0.15);
}

.login-go-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.login-divider {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
}

.login-divider::before,
.login-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: #1a1a2e;
}

.login-divider span {
  font-size: 8px;
  color: #445;
  letter-spacing: 1px;
  white-space: nowrap;
}

.provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
}

.provider-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px 8px;
  border: 1px solid #1a1a2e;
  border-radius: 8px;
  background: rgba(8, 8, 20, 0.6);
  cursor: pointer;
  transition: all 0.15s;
}

.provider-card:hover {
  border-color: #334;
  background: rgba(15, 15, 30, 0.8);
}

.provider-card.connected {
  border-color: #00ff41;
  background: rgba(0, 255, 65, 0.03);
  cursor: default;
}

.provider-card.connecting {
  border-color: #febc2e;
  background: rgba(254, 188, 46, 0.03);
  cursor: default;
}

.provider-name {
  font-size: 9px;
  font-weight: 700;
  color: #bbc;
  letter-spacing: 1px;
  text-align: center;
}

.provider-status { font-size: 8px; letter-spacing: 1px; }
.status-ok { color: #00ff41; font-weight: 700; }
.status-busy { color: #febc2e; }
.status-off { color: #556; }

.client-id-form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  border: 1px solid #febc2e;
  border-radius: 6px;
  background: rgba(8, 8, 20, 0.8);
}

.client-id-form label {
  font-size: 9px;
  color: #febc2e;
  letter-spacing: 1px;
  font-weight: 700;
}

.client-id-row { display: flex; gap: 6px; }

.bw-btn-cancel {
  background: transparent;
  border: 1px solid #334;
  border-radius: 4px;
  color: #889;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 6px 10px;
  cursor: pointer;
  letter-spacing: 1px;
}

.bw-btn-cancel:hover { border-color: #556; color: #dde; }

.provider-error {
  font-size: 10px;
  color: #ff5f57;
  padding: 4px 8px;
  border: 1px solid rgba(255, 95, 87, 0.2);
  border-radius: 4px;
  background: rgba(255, 95, 87, 0.05);
  text-align: center;
  margin-top: 6px;
}

.bw-input {
  flex: 1;
  background: rgba(10, 10, 25, 0.8);
  border: 1px solid #2a2a3e;
  border-radius: 4px;
  color: #dde;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  padding: 6px 10px;
  outline: none;
}

.bw-input:focus { border-color: #00ff41; }

.bw-btn {
  background: transparent;
  border: 1px solid #334;
  border-radius: 4px;
  color: #889;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 6px 14px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.bw-btn:hover { border-color: #556; color: #dde; }

.bw-btn-inverse {
  border-color: #00ff41;
  color: #00ff41;
}

.bw-btn-inverse:hover {
  background: rgba(0, 255, 65, 0.1);
}

.bw-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.boot-screen.ready { cursor: default; }

.boot-screen.dismissing {
  animation: glitch-skew 0.2s ease-in-out, rgb-shift 0.15s ease-in-out;
  opacity: 0;
  transition: opacity 0.3s ease-out;
}

/* ═══════════════════════════════════════════════════════
   MEGA MODAL
   ═══════════════════════════════════════════════════════ */
.mega-modal-overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.8);
  z-index: 9999;
}

.mega-modal {
  width: 380px;
  max-width: 92vw;
  background: rgba(8, 8, 20, 0.95);
  border: 1px solid #D9272E;
  border-radius: 12px;
  box-shadow: 0 0 60px rgba(217, 39, 46, 0.08);
  overflow: hidden;
}

.mega-modal-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 20px 12px;
  font-size: 13px;
  font-weight: 800;
  color: #dde;
  letter-spacing: 1px;
  border-bottom: 1px solid #1a1a2e;
}

.mega-modal-body {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mega-modal-error {
  font-size: 10px;
  color: #ff5f57;
  padding: 6px 8px;
  border: 1px solid rgba(255, 95, 87, 0.2);
  border-radius: 4px;
  background: rgba(255, 95, 87, 0.05);
  text-align: center;
}

.mega-modal-verifying {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 10px;
  color: #febc2e;
  letter-spacing: 1px;
}

.mega-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px 16px;
  border-top: 1px solid #1a1a2e;
}

@media (max-width: 640px) {
  .boot-terminal {
    border-radius: 0;
    max-width: 100vw;
    border-left: none;
    border-right: none;
  }

  .boot-terminal-text { height: 240px; }
  .boot-title { font-size: 9px; }
}
</style>
