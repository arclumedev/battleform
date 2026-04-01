<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import GameCanvas from '../components/GameCanvas.vue'
import StatsPanel from '../components/StatsPanel.vue'
import CommandLog from '../components/CommandLog.vue'
import { useMatchStore } from '../stores/match'

const route = useRoute()
const router = useRouter()
const matchStore = useMatchStore()
const matchId = ref(route.params.id as string)
const showHud = ref(true)

onMounted(() => {
  matchStore.clearCommands()
})

onUnmounted(() => {
  // cleanup handled by GameCanvas
})

function goBack() {
  router.push('/lobby')
}

function toggleHud() {
  showHud.value = !showHud.value
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    goBack()
  }
  if (e.key === 'Tab') {
    e.preventDefault()
    toggleHud()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="match-page">
    <!-- WASM canvas fills entire viewport -->
    <GameCanvas :match-id="matchId" />

    <!-- Floating HUD overlays -->
    <div v-if="showHud" class="hud-overlay">
      <!-- Top bar -->
      <div class="hud-top">
        <button class="hud-btn" @click="goBack">Esc: Lobby</button>
        <span class="hud-match-id">{{ matchId.slice(0, 8) }}</span>
        <button class="hud-btn" @click="toggleHud">Tab: Hide HUD</button>
      </div>

      <!-- Side panels -->
      <div class="hud-sidebar">
        <StatsPanel :match-id="matchId" />
        <CommandLog />
      </div>
    </div>
  </div>
</template>

<style scoped>
.match-page {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: #000;
}

/* HUD floats over the full-screen WASM canvas */
.hud-overlay {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 10;
}

.hud-top {
  pointer-events: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: rgba(17, 24, 39, 0.75);
  backdrop-filter: blur(4px);
}

.hud-match-id {
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 0.75rem;
  color: #6b7280;
}

.hud-btn {
  pointer-events: auto;
  background: rgba(31, 41, 55, 0.8);
  border: 1px solid rgba(55, 65, 81, 0.6);
  border-radius: 4px;
  color: #9ca3af;
  padding: 0.25rem 0.625rem;
  font-size: 0.6875rem;
  cursor: pointer;
  transition: all 0.15s;
}
.hud-btn:hover {
  color: #f9fafb;
  border-color: #6b7280;
}

.hud-sidebar {
  pointer-events: auto;
  position: absolute;
  top: 2.5rem;
  right: 0;
  bottom: 0;
  width: 280px;
  display: flex;
  flex-direction: column;
  background: rgba(17, 24, 39, 0.8);
  backdrop-filter: blur(4px);
  border-left: 1px solid rgba(31, 41, 55, 0.6);
  overflow-y: auto;
}
</style>
