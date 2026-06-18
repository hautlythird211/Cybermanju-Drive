<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useLogin } from '@/composables/useLogin'
import { useAppStore } from '@/stores/app'

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

/* --- Storage providers (integrated in login) --- */
const PROVIDER_META = [
  { id: 'google', label: 'Google (Drive + Photos)', icon: 'logos:google-icon', color: '#4285F4' },
  { id: 'github', label: 'GitHub', icon: 'logos:github-icon', color: '#333' },
  { id: 'gitlab', label: 'GitLab', icon: 'logos:gitlab', color: '#FC6D26' },
  { id: 'telegram', label: 'Telegram', icon: 'logos:telegram', color: '#0088CC' },
  { id: 'mega', label: 'Mega.nz', icon: 'logos:mega', color: '#D9272E' },
]
const connectedProviders = ref<string[]>([])
const connectingProvider = ref<string | null>(null)
const showClientIdForm = ref<string | null>(null)
const clientIdInput = ref('')

let timer: ReturnType<typeof setInterval> | null = null
let glitchTimer: ReturnType<typeof setInterval> | null = null
let bootTimeout: ReturnType<typeof setTimeout> | null = null

const BOOT_LINES: Array<{ msg: string; delay: number; pct: number }> = [
  { msg: '[BOOT] Cybermanju Drive Kernel v4.2.0-RELEASE (x86_64)', delay: 80, pct: 1 },
  { msg: '[BOOT] CPU: HYBRID PQC-NEON @ 2.8GHz, 8 cores / 16 threads', delay: 60, pct: 3 },
  { msg: '[BOOT] MEM: 32768MB POST-QUANTUM CRYPTO RAM (ECC)', delay: 70, pct: 5 },
  { msg: '[BIOS] CMOS checksum OK — system battery nominal', delay: 90, pct: 7 },
  { msg: '[BIOS] ACPI: IRQ routing table loaded', delay: 50, pct: 8 },
  { msg: '[BIOS] PCI: Enumeration complete — 47 devices on bus', delay: 65, pct: 10 },
  { msg: '[BIOS] SATA: 6 devices detected (SSD x2, NVMe x4)', delay: 55, pct: 12 },
  { msg: '[KERN] Initializing memory protection — NX, ASLR, SMEP', delay: 70, pct: 14 },
  { msg: '[KERN] CRYPTO: ChaCha20-Poly1305 hardware acceleration ENABLED', delay: 60, pct: 16 },
  { msg: '[KERN] CRYPTO: Kyber-1024 key encapsulation module loaded', delay: 75, pct: 18 },
  { msg: '[KERN] CRYPTO: Dilithium-5 signature verification online', delay: 65, pct: 20 },
  { msg: '[KERN] VFS: Mounting root filesystem (ext4, encrypted)', delay: 80, pct: 23 },
  { msg: '[KERN] VFS: /dev/sda1 — LUKS2 (Argon2id) unlocked', delay: 70, pct: 25 },
  { msg: '[KERN] VFS: /dev/sdb1 — XFS, journal replay OK', delay: 55, pct: 27 },
  { msg: '[KERN] NET: eth0 — 10 GbE link UP (MAC: 2A:4F:8E:0C:D1:73)', delay: 60, pct: 29 },
  { msg: '[KERN] NET: wlan0 — 802.11ax (6 GHz) scan complete', delay: 65, pct: 31 },
  { msg: '[KERN] NET: IPv6 stack ready — SLAAC configured', delay: 50, pct: 33 },
  { msg: '[KERN] USB: OHCI controller #1 at 0xFE800000 (irq 16)', delay: 55, pct: 35 },
  { msg: '[KERN] USB: 4 hubs, 12 devices enumerated', delay: 50, pct: 37 },
  { msg: '[KERN] ACPI: Thermal zone monitoring active', delay: 45, pct: 39 },
  { msg: '[KERN] DRM: efifb — 1920x1080 @ 60Hz (32 bpp)', delay: 60, pct: 41 },
  { msg: '[KERN] DRM: fbcon — font set to "Terminus" 8x16', delay: 55, pct: 43 },
  { msg: '[KERN] SND: HDA Intel PCH — Realtek ALC1220 detected', delay: 50, pct: 45 },
  { msg: '[KERN] SND: ALSA device list: hdaudioC0D0, hdaudioC0D2', delay: 50, pct: 47 },
  { msg: '[KERN] RNG: crng init done — entropy pool seeded', delay: 60, pct: 49 },
  { msg: '[KERN] RTC: system clock synced to hardware (UTC)', delay: 45, pct: 51 },
  { msg: '[INIT] Starting init daemon (PID 1): openrc-0.52', delay: 55, pct: 53 },
  { msg: '[INIT] Mounting pseudo-filesystems: proc, sysfs, tmpfs, devpts', delay: 60, pct: 55 },
  { msg: '[INIT] Activating swap: /dev/sda2 (32 GB, encrypted)', delay: 50, pct: 57 },
  { msg: '[INIT] Loading kernel modules: cryptodev, ipsec, wireguard', delay: 55, pct: 59 },
  { msg: '[INIT] Starting udev: device manager online', delay: 45, pct: 61 },
  { msg: '[INIT] Starting syslog-ng: logging daemon active', delay: 50, pct: 63 },
  { msg: '[INIT] Starting cronie: periodic scheduler loaded', delay: 45, pct: 65 },
  { msg: '[INIT] Starting sshd: OpenSSH_9.4 (port 2222)', delay: 55, pct: 67 },
  { msg: '[INIT] Starting nginx: HTTPS reverse proxy online', delay: 50, pct: 69 },
  { msg: '[INIT] Starting postgresql: database cluster ready', delay: 60, pct: 71 },
  { msg: '[INIT] Starting redis: cache layer initialized', delay: 45, pct: 73 },
  { msg: '[DAEMON] cybermanju-syncd — sync orchestrator starting...', delay: 55, pct: 75 },
  { msg: '[DAEMON] cybermanju-syncd — 6 backends registered', delay: 50, pct: 77 },
  { msg: '[DAEMON] cybermanju-cryptd — quantum-safe tunnel established', delay: 60, pct: 79 },
  { msg: '[DAEMON] cybermanju-watchd — file watcher active (inotify)', delay: 45, pct: 81 },
  { msg: '[DAEMON] cybermanju-indexd — full-text index rebuilt', delay: 55, pct: 83 },
  { msg: '[DAEMON] cybermanju-faced — facial recognition model loaded (453 tags)', delay: 65, pct: 85 },
  { msg: '[DAEMON] cybermanju-geod — geotag index initialized (41 markers)', delay: 50, pct: 87 },
  { msg: '[SHELL] Starting Cybermanju Drive Session Manager (SDM)', delay: 55, pct: 89 },
  { msg: '[SHELL] SDM: policykit authority acquired', delay: 45, pct: 91 },
  { msg: '[SHELL] SDM: D-Bus session bus listening', delay: 50, pct: 93 },
  { msg: '[SHELL] SDM: compositor starting — Wayland (wlroots)', delay: 60, pct: 95 },
  { msg: '[SHELL] SDM: desktop environment — cybermanju-shell', delay: 55, pct: 97 },
  { msg: '[SHELL] SDM: startup sequence complete.', delay: 80, pct: 99 },
  { msg: '[SHELL] Welcome to Cybermanju Drive. initializing UI...', delay: 120, pct: 100 },
]

function addBootLine(line: string) {
  bootLogs.value.push(line)
  const el = document.querySelector('.boot-terminal-text')
  if (el) el.scrollTop = el.scrollHeight
}

function triggerGlitch() {
  showGlitch.value = true
  crtFlicker.value = true
  setTimeout(() => {
    showGlitch.value = false
  }, 150 + Math.random() * 300)
  setTimeout(() => {
    crtFlicker.value = false
  }, 30 + Math.random() * 60)
}

async function runBoot() {
  for (const entry of BOOT_LINES) {
    await new Promise(r => setTimeout(r, entry.delay + Math.random() * 40))
    addBootLine(entry.msg)
    progress.value = entry.pct

    if (Math.random() < 0.15) triggerGlitch()

    if (entry.msg.includes('RNG') || entry.msg.includes('dilithium')) {
      addBootLine('  \x1b[33mWARN\x1b[0m: entropy level marginal — using hybrid seed')
    }
    if (entry.msg.includes('NVMe')) {
      addBootLine('  \x1b[31mERR\x1b[0m: NVMe nvme2: link training retry (3/3), OK')
    }
    if (entry.msg.includes('eth0')) {
      addBootLine('  \x1b[33mWARN\x1b[0m: interface rx buffer adjusted (4096 -> 8192)')
    }

    if (Math.random() < 0.08) {
      await new Promise(r => setTimeout(r, 200 + Math.random() * 400))
      triggerGlitch()
      addBootLine(`  \x1b[31mPANIC\x1b[0m: ... recovering via watchdog ... OK`)
    }
  }

  await new Promise(r => setTimeout(r, 500))
  triggerGlitch()

  await new Promise(r => setTimeout(r, 600))
  addBootLine('[LOGIN] Cybermanju Drive — system ready. awaiting authentication...')
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
  if (connectingProvider.value) return
  connectingProvider.value = pid
  try {
    const { oauth } = await import('@/wasm')
    const data = await import('@/wasm/data')
    oauth.loadClientIdsFromEnv()
    const clientId = oauth.getProviderClientId(pid)
    if (!clientId) {
      showClientIdForm.value = pid
      return
    }
    const existingToken = await oauth.loadTokenFromStorage(pid)
    let token = existingToken ? await oauth.getValidToken(existingToken) : null
    if (token) {
      oauth.saveTokenToStorage(token)
    } else {
      token = await oauth.authenticateWithPopup(pid)
      oauth.saveTokenToStorage(token)
    }
    await data.upsertOAuthAccount(pid, token)
    await store.fetchAccounts()
    if (!connectedProviders.value.includes(pid)) {
      connectedProviders.value.push(pid)
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    if (msg.includes('Client ID not configured')) {
      showClientIdForm.value = pid
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
  const { oauth } = await import('@/wasm')
  oauth.setProviderClientId(showClientIdForm.value, clientIdInput.value.trim())
  const pid = showClientIdForm.value
  showClientIdForm.value = null
  clientIdInput.value = ''
  await connectProvider(pid)
}

onMounted(() => {
  usernameInput.value = localStorage.getItem('cybermanju_username') || ''

  glitchTimer = setInterval(() => {
    if (Math.random() < 0.05) triggerGlitch()
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
  if (timer) clearInterval(timer)
  if (glitchTimer) clearInterval(glitchTimer)
  if (bootTimeout) clearTimeout(bootTimeout)
})
</script>

<template>
  <div class="boot-screen" :class="{ 'glitch-active': showGlitch, 'crt-flicker': crtFlicker, ready, dismissing }">
    <div class="crt-scanlines"></div>
    <div class="crt-vignette"></div>
    <div class="glitch-slice" v-for="n in 5" :key="n" :style="{ top: `${10 + Math.random() * 80}%`, height: `${2 + Math.random() * 6}px`, animationDelay: `${Math.random() * 2}s` }"></div>
    <div class="static-overlay" :style="{ opacity: Math.random() * 0.04 }"></div>

    <div class="boot-terminal">
      <div class="boot-terminal-header">
        <span class="boot-title">CYBERMANJU DRIVE — INIT SEQUENCE</span>
        <span class="boot-version">v4.2.0</span>
      </div>
      <div class="boot-terminal-text">
        <div v-for="(line, i) in bootLogs" :key="i" class="boot-line" :class="{ 'error-line': line.includes('ERR'), 'warn-line': line.includes('WARN'), 'panic-line': line.includes('PANIC') }">
          <span class="line-arrow">></span>
          <span class="line-text" v-html='line.replace(/\x1b\[33m/g, "<span class=\"warn\">").replace(/\x1b\[31m/g, "<span class=\"err\">").replace(/\x1b\[0m/g, "</span>")'></span>
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
      <div class="boot-hints">
        <span v-if="bootError" class="boot-error-hint">BOOT ERROR — CHECK LOGS</span>
        <span v-else-if="progress < 30">INITIALIZING HARDWARE...</span>
        <span v-else-if="progress < 60">LOADING KERNEL MODULES...</span>
        <span v-else-if="progress < 85">STARTING DAEMONS...</span>
        <span v-else-if="progress < 100">FINALIZING...</span>
        <span v-else class="login-hint">SYSTEM READY — ENTER YOUR NAME</span>
      </div>
      <div v-if="bootError" class="boot-emergency">
        <button class="emergency-btn" @click.stop="submitLogin">[ FORCE BOOT ]</button>
      </div>
    </div>
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
              <input v-model="clientIdInput" class="bw-input" placeholder="OAuth Client ID" />
              <button class="bw-btn bw-btn-inverse" :disabled="!clientIdInput.trim()" @click="saveClientId">SAVE</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
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

@keyframes screen-tear {
  0% { clip-path: inset(0); }
  50% { clip-path: inset(25% 0 50% 0); }
  51% { clip-path: inset(60% 0 10% 0); }
  100% { clip-path: inset(0); }
}

.boot-screen {
  position: fixed;
  inset: 0;
  z-index: 99999;
  background: #050505;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Courier New', 'Fira Code', monospace;
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
  background: radial-gradient(ellipse at center, transparent 60%, rgba(0,0,0,0.6) 100%);
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

.boot-terminal {
  position: relative;
  z-index: 10;
  width: 720px;
  max-width: 94vw;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  background: rgba(5, 5, 5, 0.95);
  box-shadow: 0 0 80px rgba(0, 255, 65, 0.04), 0 0 160px rgba(0, 255, 65, 0.02);
  overflow: hidden;
}

.boot-terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: #0a0a0a;
  border-bottom: 1px solid #1a1a1a;
}

.boot-title {
  font-size: 11px;
  font-weight: 800;
  color: #00ff41;
  letter-spacing: 2px;
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.15);
}

.boot-version {
  font-size: 9px;
  color: #444;
  letter-spacing: 1px;
}

.boot-terminal-text {
  height: 320px;
  max-height: 50vh;
  overflow-y: auto;
  padding: 12px 16px;
  background: #050505;
}

.boot-terminal-text::-webkit-scrollbar {
  width: 4px;
}
.boot-terminal-text::-webkit-scrollbar-track {
  background: transparent;
}
.boot-terminal-text::-webkit-scrollbar-thumb {
  background: #1a1a1a;
  border-radius: 2px;
}

.boot-line {
  font-size: 11px;
  line-height: 1.5;
  color: #ccc;
  font-weight: 500;
  white-space: pre-wrap;
  word-break: break-all;
}

.line-arrow {
  color: #00ff41;
  margin-right: 8px;
  opacity: 0.6;
}

.line-text {
  color: #ccc;
}

.error-line .line-text {
  color: #ff5f57;
}

.warn-line .line-text {
  color: #febc2e;
}

.panic-line .line-text {
  color: #ff0000;
  animation: rgb-shift 0.5s ease-in-out;
}

.warn {
  color: #febc2e;
}

.err {
  color: #ff5f57;
}

.boot-cursor {
  margin-top: 2px;
}

.cursor-blink {
  color: #00ff41;
  font-weight: 700;
  animation: cursor-blink 0.8s step-end infinite;
}

.boot-progress-track {
  position: relative;
  height: 2px;
  background: #0a0a0a;
  margin: 0 16px 12px;
  border-radius: 1px;
  overflow: hidden;
}

.boot-progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #00ff41, #00ff88, #00ff41);
  background-size: 200% 100%;
  animation: hue-rotate 2s linear infinite;
  transition: width 0.15s ease-out;
  border-radius: 1px;
  box-shadow: 0 0 12px rgba(0, 255, 65, 0.3);
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

.boot-hints {
  text-align: center;
  padding: 0 16px 14px;
  font-size: 9px;
  color: #444;
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

.login-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Courier New', 'Fira Code', monospace;
  background: #050505;
  outline: none;
}

.login-backdrop {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse at 20% 50%, rgba(0, 255, 65, 0.03) 0%, transparent 60%),
    radial-gradient(ellipse at 80% 50%, rgba(0, 255, 65, 0.02) 0%, transparent 60%);
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
  color: #555;
  letter-spacing: 1px;
}

.login-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-label {
  font-size: 9px;
  color: #555;
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
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  padding: 10px 14px;
  outline: none;
  transition: border-color 0.15s;
  text-transform: uppercase;
  letter-spacing: 2px;
}

.username-input:focus {
  border-color: #00ff41;
  box-shadow: 0 0 12px rgba(0, 255, 65, 0.1);
}

.username-input::placeholder {
  color: #333;
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
  background: #1a1a1a;
}

.login-divider span {
  font-size: 8px;
  color: #444;
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
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  background: #0d0d0d;
  cursor: pointer;
  transition: all 0.15s;
}

.provider-card:hover {
  border-color: #333;
  background: #111;
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
  color: #ccc;
  letter-spacing: 1px;
  text-align: center;
}

.provider-status {
  font-size: 8px;
  letter-spacing: 1px;
}

.status-ok {
  color: #00ff41;
  font-weight: 700;
}

.status-busy {
  color: #febc2e;
}

.status-off {
  color: #555;
}

.client-id-form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  border: 1px solid #febc2e;
  border-radius: 6px;
  background: #0d0d0d;
}

.client-id-form label {
  font-size: 9px;
  color: #febc2e;
  letter-spacing: 1px;
  font-weight: 700;
}

.client-id-row {
  display: flex;
  gap: 6px;
}

.bw-input {
  flex: 1;
  background: #111;
  border: 1px solid #2a2a2a;
  border-radius: 4px;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  padding: 6px 10px;
  outline: none;
}

.bw-input:focus {
  border-color: #00ff41;
}

.bw-btn {
  background: transparent;
  border: 1px solid #333;
  border-radius: 4px;
  color: #888;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 6px 14px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.bw-btn:hover {
  border-color: #555;
  color: #e0e0e0;
}

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

.boot-screen.ready {
  cursor: default;
}

.boot-screen.dismissing {
  animation: glitch-skew 0.2s ease-in-out, rgb-shift 0.15s ease-in-out;
  opacity: 0;
  transition: opacity 0.3s ease-out;
}

@media (max-width: 640px) {
  .boot-terminal {
    border-radius: 0;
    max-width: 100vw;
    border-left: none;
    border-right: none;
  }

  .boot-terminal-text {
    height: 240px;
  }

  .boot-title {
    font-size: 9px;
  }
}
</style>
