const g = window as unknown as Record<string, unknown>
function getPushDiff(): ((data: Uint8Array) => void) | null { return (g.__bf_pushDiff as ((data: Uint8Array) => void)) ?? null }
function getPushSnapshot(): ((data: Uint8Array) => void) | null { return (g.__bf_pushSnapshot as ((data: Uint8Array) => void)) ?? null }
function isBevyStarted(): boolean { return g.__bf_bevyStarted === true }

export class GameBridge {
  private ws: WebSocket | null = null
  private initialized = false
  onStateUpdate?: (data: ArrayBuffer) => void

  async init(): Promise<boolean> {
    if (isBevyStarted() && getPushDiff() && getPushSnapshot()) {
      console.log('[bridge] Bevy already running, reusing existing instance')
      this.initialized = true
      return true
    }
    try {
      const wasm = (await Function('return import("/pkg/battleform_renderer.js")')()) as {
        default: () => Promise<void>; push_state_diff: (data: Uint8Array) => void
        push_full_state: (data: Uint8Array) => void; start: () => void
      }
      await wasm.default()
      g.__bf_pushDiff = wasm.push_state_diff
      g.__bf_pushSnapshot = wasm.push_full_state
      if (!isBevyStarted()) { console.log('[bridge] Starting Bevy app'); wasm.start(); g.__bf_bevyStarted = true }
      this.initialized = true
      return true
    } catch (e) { console.warn('WASM game client not available:', e); this.initialized = false; return false }
  }

  connectSpectator(matchId: string): void {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const wsBase = import.meta.env.VITE_WS_URL || `${protocol}//${window.location.host}`
    this.ws = new WebSocket(`${wsBase}/api/matches/${matchId}/spectate`)
    this.ws.binaryType = 'arraybuffer'
    let isFirstMessage = true
    this.ws.onopen = () => console.log('[bridge] WebSocket connected')
    this.ws.onmessage = (event) => {
      const data = new Uint8Array(event.data as ArrayBuffer)
      const pushSnapshot = getPushSnapshot(); const pushDiff = getPushDiff()
      if (this.initialized) {
        if (isFirstMessage && pushSnapshot) { console.log(`[bridge] Pushing snapshot to WASM: ${data.byteLength} bytes`); pushSnapshot(data); isFirstMessage = false }
        else if (pushDiff) { pushDiff(data) }
      } else { console.log(`[bridge] Received ${data.byteLength} bytes but not initialized`) }
      if (this.onStateUpdate) this.onStateUpdate(event.data as ArrayBuffer)
    }
    this.ws.onclose = () => console.log('[bridge] WebSocket closed')
  }

  disconnect(): void { if (this.ws) { this.ws.close(); this.ws = null } }
  isReady(): boolean { return this.initialized }
}
