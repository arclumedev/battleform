<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { api } from '../lib/api'

const props = defineProps<{ matchId: string }>()

interface Player {
  slot: number
  displayName: string
  modelId: string | null
  isConnected: boolean
  playerType: string
}

interface MatchData {
  id: string
  status: string
  currentTick: number
  maxTicks: number
  winnerSlot: number | null
  players: Player[]
}

const PLAYER_COLORS = ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6', '#f97316', '#06b6d4', '#9ca3af']

const match = ref<MatchData | null>(null)
const error = ref<string | null>(null)
let pollTimer: ReturnType<typeof setInterval> | null = null

async function fetchStats() {
  try {
    const res = await api.get(`/matches/${props.matchId}`)
    match.value = res.data as MatchData
    error.value = null
  } catch {
    error.value = 'Failed to load stats'
  }
}

onMounted(() => {
  fetchStats()
  pollTimer = setInterval(fetchStats, 2000)
})

onUnmounted(() => {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
})
</script>

<template>
  <div class="stats-panel">
    <div class="stats-header">
      <h3>MATCH STATS</h3>
    </div>

    <div v-if="error" class="stats-error">{{ error }}</div>

    <div v-else-if="!match" class="stats-loading">Loading...</div>

    <template v-else>
      <div class="stats-meta">
        <div class="meta-row">
          <span class="meta-label">Status</span>
          <span class="meta-value">{{ match.status }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">Tick</span>
          <span class="meta-value">{{ match.currentTick }} / {{ match.maxTicks }}</span>
        </div>
      </div>

      <div class="player-list">
        <div
          v-for="player in match.players"
          :key="player.slot"
          class="player-card"
        >
          <div class="player-name" :style="{ color: PLAYER_COLORS[player.slot] }">
            P{{ player.slot + 1 }}: {{ player.displayName }}
          </div>
          <div class="player-meta">
            <span v-if="player.modelId" class="model">{{ player.modelId }}</span>
            <span :class="['connection', player.isConnected ? 'connected' : 'disconnected']">
              {{ player.isConnected ? 'Connected' : 'Disconnected' }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="match.status === 'FINISHED'" class="match-result">
        <span v-if="match.winnerSlot !== null">
          Winner: P{{ match.winnerSlot + 1 }}
        </span>
        <span v-else>Draw</span>
      </div>
    </template>
  </div>
</template>

<style scoped>
.stats-panel {
  border-bottom: 1px solid #1f2937;
}
.stats-header {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #1f2937;
}
.stats-header h3 {
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
  letter-spacing: 0.05em;
  margin: 0;
}
.stats-error {
  color: #f87171;
  font-size: 0.8125rem;
  padding: 1rem;
  text-align: center;
}
.stats-loading {
  color: #6b7280;
  padding: 1rem;
  text-align: center;
}
.stats-meta {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #1f2937;
}
.meta-row {
  display: flex;
  justify-content: space-between;
  padding: 0.2rem 0;
  font-size: 0.85rem;
}
.meta-label { color: #6b7280; }
.meta-value { color: #f9fafb; font-weight: 500; }
.player-list {
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}
.player-card {
  padding: 0.5rem 0.75rem;
  background: #111827;
  border-radius: 6px;
}
.player-name {
  font-weight: 600;
  font-size: 0.85rem;
}
.player-meta {
  display: flex;
  gap: 0.5rem;
  font-size: 0.75rem;
  margin-top: 0.125rem;
}
.model { color: #6b7280; }
.connection { font-weight: 500; }
.connected { color: #34d399; }
.disconnected { color: #ef4444; }
.match-result {
  padding: 0.75rem 1rem;
  text-align: center;
  font-weight: 600;
  font-size: 1rem;
  color: #fbbf24;
  border-top: 1px solid #1f2937;
}
</style>
