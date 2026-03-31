<script setup lang="ts">
import { onMounted, onUnmounted, ref, nextTick } from 'vue'
import { GameBridge } from '../lib/bridge'

const props = defineProps<{ matchId: string }>()

const bridge = new GameBridge()
const wasmAvailable = ref(false)
const statusMessage = ref('Initializing...')

onMounted(async () => {
  statusMessage.value = 'Loading WASM game client...'
  wasmAvailable.value = await bridge.init()

  if (wasmAvailable.value) {
    statusMessage.value = 'Connecting to match...'
    bridge.connectSpectator(props.matchId)
    statusMessage.value = ''

    // Focus the WASM canvas so it captures keyboard input immediately
    await nextTick()
    const canvas = document.getElementById('glcanvas') as HTMLCanvasElement | null
    if (canvas) {
      canvas.tabIndex = 0
      canvas.focus()

      // Re-focus canvas when clicked anywhere on the page
      // (in case user clicks a HUD button and focus shifts)
      document.addEventListener('click', refocusCanvas)
    }
  } else {
    statusMessage.value = 'WASM game client not available.'
  }
})

function refocusCanvas() {
  const canvas = document.getElementById('glcanvas') as HTMLCanvasElement | null
  // Only refocus if the click target isn't an interactive HUD element
  const active = document.activeElement
  if (canvas && (!active || !active.closest('.hud-sidebar, .hud-top'))) {
    canvas.focus()
  }
}

onUnmounted(() => {
  bridge.disconnect()
  document.removeEventListener('click', refocusCanvas)
})
</script>

<template>
  <div class="game-canvas-wrapper">
    <canvas
      v-show="wasmAvailable"
      id="glcanvas"
      class="game-canvas"
    />
    <div v-if="statusMessage" class="status-overlay">
      <p>{{ statusMessage }}</p>
    </div>
  </div>
</template>

<style scoped>
.game-canvas-wrapper {
  position: absolute;
  inset: 0;
}
.game-canvas {
  width: 100% !important;
  height: 100% !important;
  display: block;
  outline: none;
}
.status-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(10, 14, 23, 0.85);
  color: #6b7280;
  font-size: 0.875rem;
}
</style>
