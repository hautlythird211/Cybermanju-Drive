<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ (e: 'complete'): void }>()

const bootLogs = ref<string[]>([])
const progress = ref(0)
const showGlitch = ref(false)
const crtFlicker = ref(false)
const ready = ref(false)
const dismissing = ref(false)

let timer: ReturnType<typeof setInterval> | null = null
let glitchTimer: ReturnType<typeof setInterval> | null = null

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

function handleLogin() {
  if (!ready.value || dismissing.value) return
  dismissing.value = true
  triggerGlitch()
  setTimeout(() => {
    emit('complete')
  }, 400)
}

onMounted(() => {
  glitchTimer = setInterval(() => {
    if (Math.random() < 0.05) triggerGlitch()
    crtFlicker.value = Math.random() < 0.003
  }, 1000)

  runBoot()
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
  if (glitchTimer) clearInterval(glitchTimer)
})
</script>

<template>
  <div class="boot-screen" :class="{ 'glitch-active': showGlitch, 'crt-flicker': crtFlicker, ready, dismissing }" @click="handleLogin">
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
        <span v-if="progress < 30">INITIALIZING HARDWARE...</span>
        <span v-else-if="progress < 60">LOADING KERNEL MODULES...</span>
        <span v-else-if="progress < 85">STARTING DAEMONS...</span>
        <span v-else-if="progress < 100">FINALIZING...</span>
        <span v-else class="login-hint">SYSTEM READY — CLICK ANYWHERE TO LOGIN</span>
      </div>
    </div>
    <div v-if="ready" class="login-overlay">
      <div class="login-prompt">
        <div class="login-lock-icon">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#00ff41" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
        </div>
        <div class="login-title">CYBERMANJU DRIVE</div>
        <div class="login-subtitle">ENCRYPTED FILE SYSTEM v4.2.0</div>
        <div class="login-field">
          <span class="login-label">username:</span>
          <span class="login-value-blank">_</span>
        </div>
        <div class="login-cursor-blink">&#9612;</div>
        <div class="login-btn" @click.stop="handleLogin">
          <span class="login-btn-text">[ LOGIN ]</span>
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

.login-hint {
  color: #00ff41;
  animation: cursor-blink 1.2s step-end infinite;
  text-shadow: 0 0 8px rgba(0, 255, 65, 0.3);
}

.login-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 20;
  cursor: pointer;
}

.login-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 32px 48px;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  background: rgba(5, 5, 5, 0.9);
  box-shadow: 0 0 60px rgba(0, 255, 65, 0.05);
}

.login-lock-icon {
  margin-bottom: 4px;
  opacity: 0.7;
}

.login-title {
  font-size: 14px;
  font-weight: 800;
  color: #00ff41;
  letter-spacing: 3px;
  text-shadow: 0 0 12px rgba(0, 255, 65, 0.15);
}

.login-subtitle {
  font-size: 8px;
  color: #444;
  letter-spacing: 2px;
  margin-bottom: 8px;
}

.login-field {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: #e0e0e0;
}

.login-label {
  color: #00ff41;
  opacity: 0.7;
}

.login-value-blank {
  color: #00ff41;
  animation: cursor-blink 0.8s step-end infinite;
}

.login-cursor-blink {
  font-size: 18px;
  color: #00ff41;
  font-weight: 700;
  animation: cursor-blink 0.6s step-end infinite;
  margin-top: -4px;
}

.login-btn {
  margin-top: 12px;
  padding: 8px 32px;
  border: 1px solid #00ff41;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
  background: rgba(0, 255, 65, 0.03);
}

.login-btn:hover {
  background: rgba(0, 255, 65, 0.12);
  box-shadow: 0 0 16px rgba(0, 255, 65, 0.15);
}

.login-btn-text {
  font-size: 11px;
  font-weight: 700;
  color: #00ff41;
  letter-spacing: 2px;
  text-shadow: 0 0 6px rgba(0, 255, 65, 0.2);
}

.boot-screen.ready {
  cursor: pointer;
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
