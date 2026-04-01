<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { useMatchStore } from '../stores/match'

const auth = useAuthStore()
const matchStore = useMatchStore()
const router = useRouter()

onMounted(async () => {
  await auth.fetchProfile()
  if (!auth.user) {
    router.push('/login')
    return
  }
  await matchStore.fetchMatches()
})

async function handleQuickPlay(maxPlayers: number) {
  try {
    const match = await matchStore.quickPlay(maxPlayers)
    router.push(`/match/${match.id}`)
  } catch {
    // error shown in store
  }
}

async function handleJoin(matchId: string) {
  try {
    await matchStore.joinMatch(matchId)
    router.push(`/match/${matchId}`)
  } catch {
    // error shown in store
  }
}

async function handleLogout() {
  await auth.logout()
  router.push('/login')
}
</script>

<template>
  <div class="lobby-page">
    <header class="lobby-header">
      <h1 class="logo-text">BATTLEFORM</h1>
      <div class="user-info" v-if="auth.user">
        <span class="username">{{ auth.user.display_name }}</span>
        <button class="btn-ghost" @click="handleLogout">Logout</button>
      </div>
    </header>

    <main class="lobby-content">
      <section class="quick-play">
        <h2>Quick Play vs Autopilot</h2>
        <p class="section-desc">Jump into a match against AI opponents instantly.</p>
        <div class="quick-buttons">
          <button class="btn-quick" @click="handleQuickPlay(2)">
            <span class="player-count">1v1</span>
            <span class="player-label">Duel</span>
          </button>
          <button class="btn-quick" @click="handleQuickPlay(4)">
            <span class="player-count">1v3</span>
            <span class="player-label">Skirmish</span>
          </button>
          <button class="btn-quick" @click="handleQuickPlay(8)">
            <span class="player-count">1v7</span>
            <span class="player-label">Battle Royale</span>
          </button>
        </div>
      </section>

      <section class="custom-match">
        <h2>Custom Match</h2>
        <p class="section-desc">Browse or create custom matches.</p>

        <p v-if="matchStore.error" class="error">{{ matchStore.error }}</p>

        <div v-if="matchStore.loading" class="loading">Loading matches...</div>

        <div v-else-if="matchStore.matches.length === 0" class="empty-state">
          No matches available. Start a quick play game above!
        </div>

        <div v-else class="match-list">
          <div
            v-for="match in matchStore.matches"
            :key="match.id"
            class="match-row"
          >
            <div class="match-info">
              <span class="match-name">{{ match.name }}</span>
              <span class="match-status" :class="match.status">{{ match.status }}</span>
            </div>
            <div class="match-meta">
              <span class="player-slots">
                {{ match.player_count }}/{{ match.max_players }} players
              </span>
              <button
                v-if="match.status === 'waiting'"
                class="btn-join"
                @click="handleJoin(match.id)"
              >
                Join
              </button>
              <button
                v-else-if="match.status === 'running'"
                class="btn-spectate"
                @click="router.push(`/match/${match.id}`)"
              >
                Spectate
              </button>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.lobby-page {
  min-height: 100vh;
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
}
.lobby-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 2.5rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid #1f2937;
}
.logo-text {
  font-size: 1.25rem;
  font-weight: 800;
  letter-spacing: 0.15em;
  background: linear-gradient(135deg, #60a5fa, #a78bfa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}
.user-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}
.username {
  color: #9ca3af;
  font-size: 0.875rem;
}
.btn-ghost {
  background: none;
  border: 1px solid #374151;
  border-radius: 6px;
  color: #9ca3af;
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  cursor: pointer;
  transition: all 0.15s;
}
.btn-ghost:hover { color: #f9fafb; border-color: #6b7280; }

section {
  margin-bottom: 2.5rem;
}
section h2 {
  font-size: 1.125rem;
  font-weight: 700;
  margin-bottom: 0.25rem;
}
.section-desc {
  color: #6b7280;
  font-size: 0.875rem;
  margin-bottom: 1.25rem;
}

.quick-buttons {
  display: flex;
  gap: 1rem;
}
.btn-quick {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  background: #111827;
  border: 1px solid #1f2937;
  border-radius: 12px;
  padding: 1.5rem 1rem;
  cursor: pointer;
  transition: all 0.15s;
}
.btn-quick:hover { border-color: #3b82f6; background: #0f172a; }
.player-count {
  font-size: 1.5rem;
  font-weight: 800;
  background: linear-gradient(135deg, #60a5fa, #a78bfa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}
.player-label {
  color: #6b7280;
  font-size: 0.8125rem;
}

.error {
  color: #f87171;
  font-size: 0.875rem;
  margin-bottom: 1rem;
}
.loading, .empty-state {
  color: #6b7280;
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
  background: #111827;
  border: 1px solid #1f2937;
  border-radius: 12px;
}

.match-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.match-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #111827;
  border: 1px solid #1f2937;
  border-radius: 8px;
  padding: 0.875rem 1rem;
}
.match-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.match-name {
  font-weight: 600;
  font-size: 0.9375rem;
}
.match-status {
  font-size: 0.75rem;
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  font-weight: 500;
}
.match-status.waiting { background: #1e3a5f; color: #60a5fa; }
.match-status.running { background: #1a3b2a; color: #4ade80; }
.match-status.finished { background: #2d2235; color: #a78bfa; }
.match-meta {
  display: flex;
  align-items: center;
  gap: 1rem;
}
.player-slots {
  color: #6b7280;
  font-size: 0.8125rem;
}
.btn-join, .btn-spectate {
  border: none;
  border-radius: 6px;
  padding: 0.375rem 0.875rem;
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-join { background: #3b82f6; color: #fff; }
.btn-join:hover { opacity: 0.85; }
.btn-spectate { background: #1f2937; color: #d1d5db; }
.btn-spectate:hover { background: #374151; }
</style>
