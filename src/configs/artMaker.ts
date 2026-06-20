export interface LayerSettings {
  enabled: boolean
  opacity: number
  speed: number
}

export interface ArtMakerSettings {
  globalSpeed: number
  globalHueShift: number
  globalSaturation: number
  globalBrightness: number
  opacity: number
  palette: string
  mouseMode: string
  layers: Record<string, LayerSettings>
}

export interface PalettePreset {
  name: string
  label: string
  hueShift: number
  saturation: number
  brightness: number
  accentColor: string
}

const DEFAULT_LAYER = (enabled = true): LayerSettings => ({
  enabled,
  opacity: 1,
  speed: 1,
})

export const DEFAULT_SETTINGS: ArtMakerSettings = {
  globalSpeed: 1,
  globalHueShift: 0,
  globalSaturation: 50,
  globalBrightness: 50,
  opacity: 1,
  palette: 'cyberpunk',
  mouseMode: 'warp',
  layers: {
    noiseField: DEFAULT_LAYER(true),
    checkerboard: DEFAULT_LAYER(true),
    flowerOfLife: DEFAULT_LAYER(true),
    opArtRings: DEFAULT_LAYER(true),
    networkGrid: DEFAULT_LAYER(true),
    vortex: DEFAULT_LAYER(true),
    lissajous: DEFAULT_LAYER(true),
    mandala: DEFAULT_LAYER(true),
    spiralParticles: DEFAULT_LAYER(true),
    core: DEFAULT_LAYER(true),
    warpField: DEFAULT_LAYER(true),
    disintegration: DEFAULT_LAYER(true),
    screenTear: DEFAULT_LAYER(true),
    crtOverlay: DEFAULT_LAYER(true),
    matrixRain: DEFAULT_LAYER(false),
    fractalTree: DEFAULT_LAYER(true),
    plasmaWave: DEFAULT_LAYER(true),
    stardust: DEFAULT_LAYER(true),
  },
}

export const PALETTE_PRESETS: PalettePreset[] = [
  { name: 'cyberpunk', label: 'Cyberpunk', hueShift: 0, saturation: 50, brightness: 50, accentColor: '#00ff41' },
  { name: 'matrix', label: 'Matrix', hueShift: 120, saturation: 60, brightness: 35, accentColor: '#00ff41' },
  { name: 'sunset', label: 'Sunset', hueShift: 340, saturation: 55, brightness: 55, accentColor: '#ff6b9d' },
  { name: 'aurora', label: 'Aurora', hueShift: 180, saturation: 45, brightness: 45, accentColor: '#5af0ff' },
  { name: 'synthwave', label: 'Synthwave', hueShift: 290, saturation: 60, brightness: 50, accentColor: '#b388ff' },
  { name: 'monochrome', label: 'Monochrome', hueShift: 0, saturation: 5, brightness: 45, accentColor: '#a0a0b0' },
  { name: 'rainbow', label: 'Rainbow', hueShift: 0, saturation: 65, brightness: 55, accentColor: '#ffd700' },
  { name: 'fire', label: 'Fire', hueShift: 15, saturation: 70, brightness: 55, accentColor: '#ff453a' },
  { name: 'ocean', label: 'Ocean', hueShift: 210, saturation: 50, brightness: 40, accentColor: '#007aff' },
  { name: 'neon', label: 'Neon', hueShift: 300, saturation: 70, brightness: 60, accentColor: '#ff6b9d' },
]

export const ART_MAKER_PRESETS: Record<string, Partial<ArtMakerSettings>> = {
  default: {},
  psychedelic: {
    globalSpeed: 1.5,
    globalHueShift: 0,
    globalSaturation: 70,
    globalBrightness: 60,
    palette: 'rainbow',
    mouseMode: 'disintegrate',
    layers: {
      screenTear: { enabled: true, opacity: 1, speed: 2 },
      matrixRain: { enabled: false, opacity: 0.5, speed: 1 },
    },
  },
  darkAmbient: {
    globalSpeed: 0.5,
    globalSaturation: 30,
    globalBrightness: 30,
    palette: 'monochrome',
    mouseMode: 'ripple',
    layers: {
      screenTear: { enabled: false, opacity: 1, speed: 1 },
      stardust: { enabled: true, opacity: 0.8, speed: 0.5 },
      core: { enabled: true, opacity: 0.6, speed: 0.7 },
      fractalTree: { enabled: false, opacity: 1, speed: 1 },
    },
  },
  matrixMode: {
    globalSpeed: 0.8,
    globalHueShift: 120,
    globalSaturation: 55,
    globalBrightness: 35,
    palette: 'matrix',
    mouseMode: 'warp',
    layers: {
      matrixRain: { enabled: true, opacity: 1, speed: 1 },
      noiseField: { enabled: false, opacity: 1, speed: 1 },
      checkerboard: { enabled: false, opacity: 1, speed: 1 },
      flowerOfLife: { enabled: false, opacity: 1, speed: 1 },
      opArtRings: { enabled: false, opacity: 1, speed: 1 },
      networkGrid: { enabled: false, opacity: 1, speed: 1 },
      vortex: { enabled: false, opacity: 1, speed: 1 },
      lissajous: { enabled: false, opacity: 1, speed: 1 },
      mandala: { enabled: false, opacity: 1, speed: 1 },
      spiralParticles: { enabled: false, opacity: 1, speed: 1 },
      core: { enabled: true, opacity: 0.3, speed: 1 },
      warpField: { enabled: true, opacity: 0.5, speed: 1 },
      disintegration: { enabled: false, opacity: 1, speed: 1 },
      screenTear: { enabled: false, opacity: 1, speed: 1 },
      crtOverlay: { enabled: true, opacity: 0.5, speed: 1 },
      fractalTree: { enabled: false, opacity: 1, speed: 1 },
      plasmaWave: { enabled: false, opacity: 1, speed: 1 },
      stardust: { enabled: false, opacity: 1, speed: 1 },
    },
  },
  glitchCore: {
    globalSpeed: 2,
    globalHueShift: 0,
    globalSaturation: 60,
    globalBrightness: 55,
    palette: 'neon',
    mouseMode: 'disintegrate',
    layers: {
      screenTear: { enabled: true, opacity: 1.5, speed: 3 },
      crtOverlay: { enabled: true, opacity: 1, speed: 1 },
      noiseField: { enabled: true, opacity: 0.5, speed: 1.5 },
      checkerboard: { enabled: true, opacity: 1, speed: 2 },
      matrixRain: { enabled: false, opacity: 1, speed: 1 },
      core: { enabled: true, opacity: 1, speed: 1.5 },
    },
  },
  heavenly: {
    globalSpeed: 0.6,
    globalSaturation: 40,
    globalBrightness: 55,
    palette: 'aurora',
    mouseMode: 'wind',
    layers: {
      stardust: { enabled: true, opacity: 1, speed: 0.5 },
      fractalTree: { enabled: true, opacity: 0.6, speed: 0.5 },
      flowerOfLife: { enabled: true, opacity: 1, speed: 0.7 },
      core: { enabled: true, opacity: 0.8, speed: 0.5 },
      screenTear: { enabled: false, opacity: 1, speed: 1 },
      disintegration: { enabled: true, opacity: 0.5, speed: 1 },
    },
  },
}

export const LAYER_LABELS: Record<string, string> = {
  noiseField: 'Alive Noise Field',
  checkerboard: 'Checkerboard Warp',
  flowerOfLife: 'Flower of Life',
  opArtRings: 'Op-Art Rings',
  networkGrid: 'Network Grid',
  vortex: 'Vortex',
  lissajous: 'Lissajous Curves',
  mandala: 'Sacred Mandala',
  spiralParticles: 'Spiral Particles',
  core: 'Pulsating Core',
  warpField: 'Mouse Warp Field',
  disintegration: 'Disintegration',
  screenTear: 'Screen Tear / Glitch',
  crtOverlay: 'CRT Overlay',
  matrixRain: 'Matrix Rain',
  fractalTree: 'Fractal Tree',
  plasmaWave: 'Plasma Wave',
  stardust: 'Stardust / Nebula',
}

export const MOUSE_MODES = [
  { id: 'warp', label: 'Warp Field' },
  { id: 'disintegrate', label: 'Disintegrate' },
  { id: 'ripple', label: 'Ripple' },
  { id: 'wind', label: 'Wind' },
  { id: 'paint', label: 'Paint Trail' },
  { id: 'none', label: 'None' },
]

export function applyPreset(settings: ArtMakerSettings, presetName: string): ArtMakerSettings {
  const preset = ART_MAKER_PRESETS[presetName]
  if (!preset) return { ...settings }
  return {
    ...settings,
    ...preset,
    layers: {
      ...settings.layers,
      ...Object.fromEntries(
        Object.entries(preset.layers || {}).map(([key, val]) => [
          key,
          { ...settings.layers[key], ...val } as LayerSettings,
        ])
      ),
    },
  }
}

export function randomizeSettings(base: ArtMakerSettings): ArtMakerSettings {
  const speed = 0.3 + Math.random() * 2.5
  const hueShift = Math.random() * 360
  const saturation = 20 + Math.random() * 50
  const brightness = 20 + Math.random() * 40
  const palette = PALETTE_PRESETS[(Math.random() * PALETTE_PRESETS.length) | 0].name
  const mouseMode = MOUSE_MODES[(Math.random() * MOUSE_MODES.length) | 0].id

  const layers: Record<string, LayerSettings> = {}
  for (const key of Object.keys(base.layers)) {
    layers[key] = {
      enabled: Math.random() > 0.4,
      opacity: 0.3 + Math.random() * 0.7,
      speed: 0.3 + Math.random() * 2.2,
    }
  }

  return { ...base, globalSpeed: speed, globalHueShift: hueShift, globalSaturation: saturation, globalBrightness: brightness, palette, mouseMode, layers }
}
