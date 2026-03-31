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
const showStats = ref(true)
const showLog = ref(true)

onMounted(() => {
  matchStore.clearCommands()
})

onUnmounted(() => {
  // cleanup handled by GameCanvas
})

function goBack() {
  router.push('/lobby')
}
</script>

<template>
  <div class="match-page">
    <header class="match-header">
      <button class="btn-back" @click="goBack">&larr; Lobby</button>
      <h1 class="match-title">Match {{ matchId }}</h1>
      <div class="header-controls">
        <button
          class="btn-toggle"
          :class="{ active: showStats }"
          @click="showStats = !showStats"
        >
          Stats
        </button>
        <button
          class="btn-toggle"
          :class="{ active: showLog }"
          @click="showLog = !showLog"
        >
          Log
        </button>
      </div>
    </header>

    <div class="match-layout">
      <div class="canvas-area">
        <GameCanvas :match-id="matchId" />
      </div>
      <aside v-if="showStats || showLog" class="sidebar">
        <StatsPanel v-if="showStats" :match-id="matchId" />
        <CommandLog v-if="showLog" />
      </aside>
    </div>
  </div>
</template>

<style scoped>
.match-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.match-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1rem;
  background: #111827;
  border-bottom: 1px solid #1f2937;
  flex-shrink: 0;
}
.btn-back {
  background: none;
  border: 1px solid #374151;
  border-radius: 6px;
  color: #9ca3af;
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  cursor: pointer;
  transition: all 0.15s;
}
.btn-back:hover { color: #f9fafb; border-color: #6b7280; }
.match-title {
  font-size: 1rem;
  font-weight: 600;
  flex: 1;
}
.header-controls {
  display: flex;
  gap: 0.5rem;
}
.btn-toggle {
  background: #1f2937;
  border: 1px solid #374151;
  border-radius: 6px;
  color: #6b7280;
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}
.btn-toggle.active { color: #60a5fa; border-color: #3b82f6; }
.btn-toggle:hover { color: #d1d5db; }

.match-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
}
.canvas-area {
  flex: 1;
  position: relative;
  background: #000;
}
.sidebar {
  width: 320px;
  display: flex;
  flex-direction: column;
  background: #111827;
  border-left: 1px solid #1f2937;
  overflow: hidden;
}
</style>
