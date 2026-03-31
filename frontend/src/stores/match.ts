import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '../lib/api'

export interface Match {
  id: string
  name: string
  status: string
  player_count: number
  max_players: number
  created_at: string
}

export interface CommandEntry {
  tick: number
  player: string
  action: string
  timestamp: string
}

export const useMatchStore = defineStore('match', () => {
  const matches = ref<Match[]>([])
  const currentMatch = ref<Match | null>(null)
  const commandLog = ref<CommandEntry[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchMatches() {
    loading.value = true
    error.value = null
    try {
      const res = await api.get('/matches')
      matches.value = res.data as Match[]
    } catch {
      error.value = 'Failed to load matches'
    } finally {
      loading.value = false
    }
  }

  async function createMatch(name: string, maxPlayers: number) {
    loading.value = true
    error.value = null
    try {
      const res = await api.post('/matches', { name, max_players: maxPlayers })
      const match = res.data as Match
      matches.value.unshift(match)
      return match
    } catch {
      error.value = 'Failed to create match'
      throw new Error("Request failed")
    } finally {
      loading.value = false
    }
  }

  async function quickPlay(maxPlayers: number) {
    loading.value = true
    error.value = null
    try {
      const res = await api.post('/matches/quick', { max_players: maxPlayers })
      const match = res.data as Match
      currentMatch.value = match
      return match
    } catch {
      error.value = 'Failed to start quick play'
      throw new Error("Request failed")
    } finally {
      loading.value = false
    }
  }

  async function joinMatch(matchId: string) {
    loading.value = true
    error.value = null
    try {
      const res = await api.post(`/matches/${matchId}/join`)
      currentMatch.value = res.data as Match
      return currentMatch.value
    } catch {
      error.value = 'Failed to join match'
      throw new Error("Request failed")
    } finally {
      loading.value = false
    }
  }

  async function startMatch(matchId: string) {
    loading.value = true
    error.value = null
    try {
      const res = await api.post(`/matches/${matchId}/start`)
      currentMatch.value = res.data as Match
      return currentMatch.value
    } catch {
      error.value = 'Failed to start match'
      throw new Error("Request failed")
    } finally {
      loading.value = false
    }
  }

  function addCommand(entry: CommandEntry) {
    commandLog.value.push(entry)
    if (commandLog.value.length > 200) {
      commandLog.value = commandLog.value.slice(-100)
    }
  }

  function clearCommands() {
    commandLog.value = []
  }

  return {
    matches, currentMatch, commandLog, loading, error,
    fetchMatches, createMatch, quickPlay, joinMatch, startMatch,
    addCommand, clearCommands,
  }
})
