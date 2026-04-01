import { GameState, type MapConfig, type GameCommand, type StateDiff } from './state.js'
import { executeCommands, resolveMovement, resolveCombat, resolveHarvesting } from './commands.js'
import { computeFog } from './fog.js'
import { generateBotCommands } from './bot_ai.js'

const TICK_MS = 100 // 10 ticks/sec

export type DiffCallback = (matchId: string, diff: StateDiff) => void
export type FinishCallback = (matchId: string, winnerSlot: number | null) => void

export class GameEngine {
  private matches: Map<string, GameState> = new Map()
  private timers: Map<string, ReturnType<typeof setTimeout>> = new Map()
  private autopilotSlots: Map<string, number[]> = new Map() // matchId -> autopilot slot numbers
  private onDiff: DiffCallback | null = null
  private onFinish: FinishCallback | null = null

  setCallbacks(onDiff: DiffCallback, onFinish: FinishCallback) {
    this.onDiff = onDiff
    this.onFinish = onFinish
  }

  createMatch(matchId: string, mapConfig: MapConfig, autopilotSlots?: number[]): GameState {
    const state = new GameState(mapConfig)
    this.matches.set(matchId, state)
    if (autopilotSlots && autopilotSlots.length > 0) {
      this.autopilotSlots.set(matchId, autopilotSlots)
    }
    computeFog(state)
    return state
  }

  startMatch(matchId: string): void {
    const state = this.matches.get(matchId)
    if (!state) return
    this.scheduleTick(matchId)
  }

  stopMatch(matchId: string): void {
    const timer = this.timers.get(matchId)
    if (timer) {
      clearTimeout(timer)
      this.timers.delete(matchId)
    }
  }

  getState(matchId: string): GameState | undefined {
    return this.matches.get(matchId)
  }

  queueCommand(matchId: string, command: GameCommand): void {
    const state = this.matches.get(matchId)
    if (!state || state.phase !== 'active') return
    state.commandQueue.push(command)
  }

  private scheduleTick(matchId: string): void {
    const timer = setTimeout(() => this.tick(matchId), TICK_MS)
    this.timers.set(matchId, timer)
  }

  private tick(matchId: string): void {
    const state = this.matches.get(matchId)
    if (!state || state.phase !== 'active') return

    // Snapshot for diff
    state.snapshotForDiff()

    // 0. Generate autopilot commands
    const autoSlots = this.autopilotSlots.get(matchId)
    if (autoSlots && state.tick % 5 === 0) {
      for (const slot of autoSlots) {
        // Skip eliminated autopilots
        const base = state.getPlayerBase(slot)
        if (!base) continue
        const botCommands = generateBotCommands(state, slot)
        state.commandQueue.push(...botCommands)
      }
    }

    // 1. Drain and execute commands
    const commands = state.commandQueue.splice(0)
    executeCommands(state, commands)

    // 2. Resolve movement
    resolveMovement(state)

    // 3. Resolve combat
    const combatEvents = resolveCombat(state)

    // 4. Resolve harvesting
    resolveHarvesting(state)

    // 5. Compute fog of war
    computeFog(state)

    // 6. Check win conditions
    this.checkWinConditions(state)

    // 7. Compute diff
    const diff = state.computeDiff()
    diff.combatEvents = combatEvents

    // 8. Broadcast diff
    if (this.onDiff) {
      this.onDiff(matchId, diff)
    }

    // 9. Advance tick
    state.tick++

    // 10. Continue or stop
    if (state.phase === 'active') {
      this.scheduleTick(matchId)
    } else {
      this.timers.delete(matchId)
      if (this.onFinish) {
        this.onFinish(matchId, state.winnerSlot)
      }
    }
  }

  private checkWinConditions(state: GameState): void {
    // Find players with surviving bases
    const alive = state.players.filter((p) => state.buildings.has(p.baseId))

    // Last base standing wins
    if (alive.length === 1) {
      state.phase = 'finished'
      state.winnerSlot = alive[0].slot
      return
    }

    // All bases destroyed (simultaneous) — draw
    if (alive.length === 0) {
      state.phase = 'finished'
      state.winnerSlot = null
      return
    }

    // Check max ticks
    if (state.tick >= state.mapConfig.width * 100) {
      // Score: base HP + unit count × 10 + energy
      let bestSlot: number | null = null
      let bestScore = -1
      let tied = false

      for (const player of alive) {
        const base = state.buildings.get(player.baseId)
        const baseHp = base ? base.hp : 0
        let unitCount = 0
        for (const unit of state.units.values()) {
          if (unit.playerSlot === player.slot) unitCount++
        }
        const score = baseHp + unitCount * 10 + player.energy

        if (score > bestScore) {
          bestScore = score
          bestSlot = player.slot
          tied = false
        } else if (score === bestScore) {
          tied = true
        }
      }

      state.phase = 'finished'
      state.winnerSlot = tied ? null : bestSlot
    }
  }

  removeMatch(matchId: string): void {
    this.stopMatch(matchId)
    this.matches.delete(matchId)
    this.autopilotSlots.delete(matchId)
  }
}

// Singleton engine instance
export const gameEngine = new GameEngine()
