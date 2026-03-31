<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '../lib/api'

const props = defineProps<{ matchId: string }>()

interface MatchStats {
  status: string
  tick: number
  players: { id: string; name: string; score: number; units: number }[]
  elapsed: string
}

const stats = ref<MatchStats | null>(null)
const error = ref<string | null>(null)
let pollTimer: ReturnType<typeof setInterval> | null = null

async function fetchStats() {
  try {
    const res = await api.get(`/matches/${props.matchId}/stats`)
    stats.value = res.data as MatchStats
    error.value = null
  } catch {
    error.value = 'Failed to load stats'
  }
}

onMounted(() => {
  fetchStats()
  pollTimer = setInterval(fetchStats, 2000)
})

function cleanup() {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
}

defineExpose({ cleanup })

import { onUnmounted } from 'vue'
onUnmounted(cleanup)
</script>

<template>
  <div class="stats-panel">
    <div class="stats-header">
      <h3>Match Stats</h3>
      <span v-if="stats" class="stats-tick">Tick {{ stats.tick }}</span>
    </div>

    <div v-if="error" class="stats-error">{{ error }}</div>

    <div v-else-if="!stats" class="stats-loading">Loading stats...</div>

    <template v-else>
      <div class="stats-meta">
        <div class="meta-item">
          <span class="meta-label">Status</span>
          <span class="meta-value" :class="stats.status">{{ stats.status }}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Elapsed</span>
          <span class="meta-value">{{ stats.elapsed }}</span>
        </div>
      </div>

      <div class="player-list">
        <div
          v-for="player in stats.players"
          :key="player.id"
          class="player-row"
        >
          <span class="player-name">{{ player.name }}</span>
          <div class="player-stats">
            <span class="stat-badge score">{{ player.score }} pts</span>
            <span class="stat-badge units">{{ player.units }} units</span>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.stats-panel {
  display: flex;
  flex-direction: column;
  padding: 0;
}
.stats-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #1f2937;
}
.stats-header h3 {
  font-size: 0.8125rem;
  font-weight: 600;
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.stats-tick {
  font-size: 0.75rem;
  color: #4b5563;
  font-family: 'SF Mono', 'Fira Code', monospace;
}
.stats-error {
  color: #f87171;
  font-size: 0.8125rem;
  padding: 1rem;
  text-align: center;
}
.stats-loading {
  color: #4b5563;
  font-size: 0.8125rem;
  padding: 2rem 1rem;
  text-align: center;
}
.stats-meta {
  display: flex;
  gap: 1rem;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #1f2937;
}
.meta-item {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}
.meta-label {
  font-size: 0.6875rem;
  color: #4b5563;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.meta-value {
  font-size: 0.875rem;
  font-weight: 600;
}
.meta-value.waiting { color: #60a5fa; }
.meta-value.running { color: #4ade80; }
.meta-value.finished { color: #a78bfa; }
.player-list {
  padding: 0.5rem;
}
.player-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem;
  border-radius: 6px;
}
.player-row:hover { background: #1f2937; }
.player-name {
  font-size: 0.875rem;
  font-weight: 500;
}
.player-stats {
  display: flex;
  gap: 0.5rem;
}
.stat-badge {
  font-size: 0.75rem;
  padding: 0.125rem 0.5rem;
  border-radius: 4px;
  font-weight: 500;
}
.stat-badge.score { background: #1e3a5f; color: #60a5fa; }
.stat-badge.units { background: #1a3b2a; color: #4ade80; }
</style>
