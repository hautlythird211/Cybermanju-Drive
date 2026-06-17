<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ (e: 'complete', user: string): void }>()

const bootLogs = ref<string[]>([])
const progress = ref(0)
const showGlitch = ref(false)
const crtFlicker = ref(false)
const ready = ref(false)
const dismissing = ref(false)
const bootError = ref(false)
const usernameInput = ref('')
const usernameSubmitted = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)

/* --- LoginScreen integration --- */
const users = ref([
  { name: 'admin', avatar: 'AD', lastLogin: 'Just now', session: 'cybermanju-shell' },
  { name: 'operator', avatar: 'OP', lastLogin: 'Yesterday', session: 'cybermanju-shell' },
  { name: 'guest', avatar: 'GT', lastLogin: 'Never', session: 'cybermanju-shell' },
])
const selectedUserIdx = ref(0)
const password = ref('')
const showPassword = ref(false)
const showUserList = ref(true)
const unlocking = ref(false)
const showSessionOptions = ref(false)
const showAccessibility = ref(false)
const loginError = ref('')

const sessions = [
  { id: 'cybermanju-shell', label: 'Cybermanju Shell', desc: 'Post-quantum encrypted desktop' },
  { id: 'kde', label: 'KDE Plasma', desc: 'Full-featured desktop environment' },
  { id: 'gnome', label: 'GNOME', desc: 'Modern user-friendly desktop' },
  { id: 'i3', label: 'i3 WM', desc: 'Minimal tiling window manager' },
]

const accessibilityOptions = [
  { id: 'screen-reader', label: 'Screen Reader', desc: 'Orca screen reader' },
  { id: 'large-text', label: 'Large Text', desc: 'Increased font sizes' },
  { id: 'high-contrast', label: 'High Contrast', desc: 'Enhanced contrast theme' },
  { id: 'sticky-keys', label: 'Sticky Keys', desc: 'One-handed modifier keys' },
]

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

function forceComplete(user?: string) {
  if (dismissing.value) return
  dismissing.value = true
  triggerGlitch()
  const name = user || users.value[selectedUserIdx.value].name
  localStorage.setItem('cybermanju_username', name)
  setTimeout(() => emit('complete', name), 400)
}

function handleLogin() {
  if (!ready.value || dismissing.value) return
  forceComplete()
}

function submitUsername() {
  if (!ready.value || dismissing.value || usernameSubmitted.value) return
  const name = usernameInput.value.trim()
  if (name) {
    usernameSubmitted.value = true
    localStorage.setItem('cybermanju_username', name)
    dismissBoot()
  } else if (users.value.length > 0) {
    forceComplete()
  }
}

function handleUsernameKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') submitUsername()
}

function dismissBoot() {
  if (dismissing.value) return
  dismissing.value = true
  triggerGlitch()
  const name = usernameInput.value.trim() || users.value[selectedUserIdx.value].name
  localStorage.setItem('cybermanju_username', name)
  setTimeout(() => emit('complete', name), 400)
}

/* --- LoginScreen functions --- */
function selectUser(idx: number) {
  selectedUserIdx.value = idx
  showUserList.value = false
  password.value = ''
  loginError.value = ''
}

function showAllUsers() {
  showUserList.value = true
  password.value = ''
  loginError.value = ''
  unlocking.value = false
}

function doLogin() {
  if (!password.value.trim()) {
    loginError.value = 'Password required'
    return
  }
  unlocking.value = true
  loginError.value = ''
  setTimeout(() => {
    forceComplete(users.value[selectedUserIdx.value].name)
  }, 1800 + Math.random() * 600)
}

function handleKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (showSessionOptions.value) { showSessionOptions.value = false; return }
    if (showAccessibility.value) { showAccessibility.value = false; return }
    if (!showUserList.value) { showAllUsers(); return }
  }
  if (e.key === 'Enter' && !showUserList.value) {
    doLogin()
  }
  if (showUserList.value) {
    if (e.key === 'ArrowUp') { e.preventDefault(); selectedUserIdx.value = (selectedUserIdx.value - 1 + users.value.length) % users.value.length }
    if (e.key === 'ArrowDown') { e.preventDefault(); selectedUserIdx.value = (selectedUserIdx.value + 1) % users.value.length }
    if (e.key === 'ArrowRight' || e.key === 'Enter') { selectUser(selectedUserIdx.value) }
  }
}

onMounted(() => {
  usernameInput.value = localStorage.getItem('cybermanju_username') || ''
  const saved = localStorage.getItem('cybermanju_username')
  if (saved) {
    const idx = users.value.findIndex(u => u.name === saved)
    if (idx >= 0) {
      selectedUserIdx.value = idx
      showUserList.value = false
    }
  }

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
        <span v-else class="login-hint">SYSTEM READY — SELECT A USER OR ENTER A NICKNAME</span>
      </div>
      <div v-if="bootError" class="boot-emergency">
        <button class="emergency-btn" @click.stop="() => forceComplete()">[ FORCE BOOT ]</button>
      </div>
    </div>
    <div v-if="ready && !usernameSubmitted" class="login-overlay" @keydown="handleKey" tabindex="0" autofocus>
      <div class="login-backdrop"></div>
      <div class="login-container">
        <!-- User List -->
        <div v-if="showUserList" class="user-list-panel">
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
          <div class="user-list-title">Select a user</div>
          <div
            v-for="(user, i) in users"
            :key="user.name"
            class="user-card"
            :class="{ 'user-selected': selectedUserIdx === i }"
            @click="selectUser(i)"
          >
            <div class="user-avatar">{{ user.avatar }}</div>
            <div class="user-info">
              <div class="user-name">{{ user.name }}</div>
              <div class="user-meta">{{ user.lastLogin }}</div>
            </div>
            <div class="user-session">{{ user.session }}</div>
          </div>
          <div class="nickname-section">
            <div class="nickname-label">OR ENTER A NICKNAME</div>
            <div class="nickname-input-wrap">
              <input
                v-model="usernameInput"
                class="nickname-input"
                placeholder="YOUR NAME"
                maxlength="32"
                @keydown="handleUsernameKeydown"
              />
              <button class="nickname-submit" @click="submitUsername">[ GO ]</button>
            </div>
          </div>
          <div class="login-actions">
            <button class="login-action-btn" @click.stop="showSessionOptions = true">Session Type</button>
            <button class="login-action-btn" @click.stop="showAccessibility = true">Accessibility</button>
            <button class="login-action-btn power-btn">Power Off</button>
          </div>
        </div>

        <!-- Password Prompt -->
        <div v-else class="password-panel">
          <div class="pwd-avatar-large" @click="showAllUsers">
            <div class="avatar-circle">{{ users[selectedUserIdx].avatar }}</div>
            <div class="avatar-name">{{ users[selectedUserIdx].name }}</div>
            <div class="avatar-hint">Click to switch user</div>
          </div>
          <div class="pwd-field">
            <div class="pwd-label">Password</div>
            <div class="pwd-input-wrap">
              <input
                v-model="password"
                :type="showPassword ? 'text' : 'password'"
                class="pwd-input"
                placeholder="Enter password"
                autocomplete="current-password"
                spellcheck="false"
                @keyup.enter="doLogin"
              />
              <button class="pwd-toggle" @click.stop="showPassword = !showPassword">
                {{ showPassword ? 'HIDE' : 'SHOW' }}
              </button>
            </div>
            <div v-if="loginError" class="pwd-error">{{ loginError }}</div>
          </div>
          <div v-if="unlocking" class="unlocking-bar">
            <div class="unlocking-text">Unlocking session...</div>
            <div class="unlocking-track">
              <div class="unlocking-fill"></div>
            </div>
          </div>
          <div v-else class="pwd-actions">
            <button class="pwd-submit" @click="doLogin">[ UNLOCK ]</button>
            <button class="pwd-cancel" @click="showAllUsers">[ CANCEL ]</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Session Options Modal -->
    <div v-if="showSessionOptions" class="modal-overlay" @click.self="showSessionOptions = false">
      <div class="modal-panel">
        <div class="modal-title">Desktop Session</div>
        <div v-for="s in sessions" :key="s.id" class="modal-option" :class="{ 'modal-option-active': s.id === users[selectedUserIdx].session }" @click="users[selectedUserIdx].session = s.id">
          <div class="modal-option-label">{{ s.label }}</div>
          <div class="modal-option-desc">{{ s.desc }}</div>
        </div>
        <button class="modal-close" @click="showSessionOptions = false">[ CLOSE ]</button>
      </div>
    </div>

    <!-- Accessibility Modal -->
    <div v-if="showAccessibility" class="modal-overlay" @click.self="showAccessibility = false">
      <div class="modal-panel">
        <div class="modal-title">Accessibility</div>
        <div v-for="opt in accessibilityOptions" :key="opt.id" class="modal-option">
          <div class="modal-option-label">{{ opt.label }}</div>
          <div class="modal-option-desc">{{ opt.desc }}</div>
        </div>
        <button class="modal-close" @click="showAccessibility = false">[ CLOSE ]</button>
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
  width: 480px;
  max-width: 92vw;
}

.user-list-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.login-brand {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 8px;
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

.user-list-title {
  font-size: 11px;
  color: #666;
  letter-spacing: 2px;
  margin-bottom: 4px;
  padding: 0 4px;
}

.user-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 16px;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.user-card:hover {
  border-color: #2a2a2a;
  background: rgba(255, 255, 255, 0.02);
}

.user-selected {
  border-color: rgba(0, 255, 65, 0.3);
  background: rgba(0, 255, 65, 0.04);
}

.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: linear-gradient(135deg, #0a0a0a, #1a1a1a);
  border: 1px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 800;
  color: #00ff41;
  text-shadow: 0 0 6px rgba(0, 255, 65, 0.3);
}

.user-info {
  flex: 1;
}

.user-name {
  font-size: 13px;
  font-weight: 700;
  color: #ddd;
}

.user-meta {
  font-size: 9px;
  color: #555;
  margin-top: 2px;
}

.user-session {
  font-size: 8px;
  color: #444;
  letter-spacing: 1px;
}

.login-actions {
  display: flex;
  gap: 8px;
  justify-content: center;
  margin-top: 8px;
}

.login-action-btn {
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #666;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  font-weight: 600;
  padding: 8px 16px;
  cursor: pointer;
  letter-spacing: 1px;
  transition: all 0.15s;
}

.login-action-btn:hover {
  border-color: #333;
  color: #aaa;
}

.power-btn:hover {
  border-color: rgba(255, 95, 87, 0.3);
  color: #ff5f57;
}

.nickname-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px 0;
  border-top: 1px solid #1a1a1a;
  margin-top: 8px;
}

.nickname-label {
  font-size: 8px;
  color: #444;
  letter-spacing: 1px;
}

.nickname-input-wrap {
  display: flex;
  gap: 6px;
  align-items: center;
}

.nickname-input {
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  padding: 8px 12px;
  width: 200px;
  outline: none;
  transition: border-color 0.15s;
}

.nickname-input:focus {
  border-color: #00ff41;
}

.nickname-input::placeholder {
  color: #333;
}

.nickname-submit {
  background: transparent;
  border: 1px solid #00ff41;
  border-radius: 6px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 700;
  padding: 8px 14px;
  cursor: pointer;
  transition: all 0.15s;
}

.nickname-submit:hover {
  background: #00ff41;
  color: #0a0a0a;
}

.password-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.pwd-avatar-large {
  text-align: center;
  cursor: pointer;
}

.avatar-circle {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background: linear-gradient(135deg, #0a0a0a, #1a1a1a);
  border: 2px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  font-weight: 800;
  color: #00ff41;
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.3);
  box-shadow: 0 0 20px rgba(0, 255, 65, 0.05);
  margin: 0 auto;
}

.avatar-name {
  font-size: 14px;
  font-weight: 800;
  color: #ddd;
  margin-top: 10px;
  letter-spacing: 2px;
}

.avatar-hint {
  font-size: 9px;
  color: #444;
  margin-top: 4px;
}

.pwd-field {
  width: 100%;
}

.pwd-label {
  font-size: 9px;
  color: #555;
  letter-spacing: 2px;
  margin-bottom: 6px;
  text-transform: uppercase;
}

.pwd-input-wrap {
  display: flex;
  gap: 8px;
}

.pwd-input {
  flex: 1;
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  padding: 10px 12px;
  outline: none;
  transition: border-color 0.15s;
}

.pwd-input:focus {
  border-color: rgba(0, 255, 65, 0.3);
}

.pwd-toggle {
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 9px;
  font-weight: 600;
  padding: 0 12px;
  cursor: pointer;
  letter-spacing: 1px;
}

.pwd-toggle:hover {
  border-color: #333;
  color: #888;
}

.pwd-error {
  color: #ff5f57;
  font-size: 10px;
  margin-top: 6px;
  font-weight: 600;
}

.unlocking-bar {
  width: 100%;
}

.unlocking-text {
  font-size: 9px;
  color: #00ff41;
  letter-spacing: 2px;
  margin-bottom: 8px;
  text-align: center;
}

.unlocking-track {
  width: 100%;
  height: 3px;
  background: #111;
  border-radius: 2px;
  overflow: hidden;
}

.unlocking-fill {
  height: 100%;
  width: 100%;
  background: linear-gradient(90deg, #00ff41, #00ff88, #00ff41);
  background-size: 200% 100%;
  animation: unlock-progress 1.8s ease-out forwards;
  border-radius: 2px;
  box-shadow: 0 0 10px rgba(0, 255, 65, 0.3);
}

@keyframes unlock-progress {
  0% { width: 0%; }
  30% { width: 30%; }
  60% { width: 65%; }
  85% { width: 88%; }
  100% { width: 100%; }
}

.pwd-actions {
  display: flex;
  gap: 12px;
}

.pwd-submit {
  background: transparent;
  border: 1px solid rgba(0, 255, 65, 0.3);
  border-radius: 6px;
  color: #00ff41;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  font-weight: 700;
  padding: 10px 28px;
  cursor: pointer;
  letter-spacing: 2px;
  transition: all 0.15s;
}

.pwd-submit:hover {
  background: rgba(0, 255, 65, 0.08);
  border-color: #00ff41;
  box-shadow: 0 0 16px rgba(0, 255, 65, 0.15);
}

.pwd-cancel {
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  font-weight: 600;
  padding: 10px 28px;
  cursor: pointer;
  letter-spacing: 2px;
  transition: all 0.15s;
}

.pwd-cancel:hover {
  border-color: #333;
  color: #888;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100000;
}

.modal-panel {
  width: 400px;
  max-width: 90vw;
  background: #0a0a0a;
  border: 1px solid #1a1a1a;
  border-radius: 10px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.modal-title {
  font-size: 12px;
  font-weight: 800;
  color: #ddd;
  letter-spacing: 2px;
  margin-bottom: 12px;
}

.modal-option {
  padding: 10px 14px;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.modal-option:hover {
  border-color: #2a2a2a;
  background: rgba(255, 255, 255, 0.02);
}

.modal-option-active {
  border-color: rgba(0, 255, 65, 0.3);
  background: rgba(0, 255, 65, 0.04);
}

.modal-option-label {
  font-size: 11px;
  font-weight: 700;
  color: #ccc;
}

.modal-option-desc {
  font-size: 9px;
  color: #555;
  margin-top: 2px;
}

.modal-close {
  margin-top: 12px;
  background: transparent;
  border: 1px solid #1a1a1a;
  border-radius: 6px;
  color: #555;
  font-family: 'Courier New', monospace;
  font-size: 10px;
  font-weight: 600;
  padding: 8px 20px;
  cursor: pointer;
  letter-spacing: 1px;
  align-self: flex-end;
}

.modal-close:hover {
  border-color: #333;
  color: #888;
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
