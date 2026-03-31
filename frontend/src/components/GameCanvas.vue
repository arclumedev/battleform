<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { GameBridge } from '../lib/bridge'

const props = defineProps<{ matchId: string }>()

const bridge = new GameBridge()
const canvasReady = ref(false)
const wasmAvailable = ref(false)
const statusMessage = ref('Initializing...')

onMounted(async () => {
  canvasReady.value = true
  statusMessage.value = 'Loading WASM game client...'
  wasmAvailable.value = await bridge.init()

  if (wasmAvailable.value) {
    statusMessage.value = 'Connecting to match...'
    bridge.connectSpectator(props.matchId)
    statusMessage.value = ''
  } else {
    statusMessage.value = 'WASM game client not available. Spectating in data-only mode.'
  }
})

onUnmounted(() => {
  bridge.disconnect()
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
  width: 100%;
  height: 100%;
  position: relative;
}
.game-canvas {
  width: 100%;
  height: 100%;
  display: block;
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
