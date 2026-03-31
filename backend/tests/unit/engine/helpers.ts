import { GameState, type MapConfig, type Unit } from '../../../app/engine/state.js'

/**
 * Create a minimal 2-player map config for testing.
 */
export function testMapConfig(playerCount = 2): MapConfig {
  return {
    width: 16,
    height: 16,
    terrain: 'open',
    startPositions: Array.from({ length: playerCount }, (_, i) => {
      if (i === 0) return { x: 1, y: 1 }
      if (i === 1) return { x: 14, y: 14 }
      if (i === 2) return { x: 1, y: 14 }
      return { x: 14, y: 1 }
    }).slice(0, playerCount),
    resourceNodes: [
      { x: 4, y: 4, energy: 500 },
      { x: 12, y: 12, energy: 500 },
    ],
  }
}

/**
 * Create a fresh game state for testing.
 */
export function createTestState(playerCount = 2): GameState {
  return new GameState(testMapConfig(playerCount))
}

/**
 * Add a unit to the game state and return it.
 */
export function addUnit(
  state: GameState,
  opts: {
    playerSlot: number
    unitType: Unit['unitType']
    x: number
    y: number
    hp?: number
  }
): Unit {
  const { playerSlot, unitType, x, y, hp } = opts
  const stats = { worker: 30, soldier: 80, scout: 40 }
  const maxHp = hp ?? stats[unitType]

  const unit: Unit = {
    id: crypto.randomUUID(),
    playerSlot,
    unitType,
    x,
    y,
    hp: maxHp,
    maxHp,
    status: 'idle',
    path: [],
    targetId: null,
    cargo: 0,
  }

  state.units.set(unit.id, unit)
  return unit
}

/**
 * Get a player's energy.
 */
export function getEnergy(state: GameState, slot: number): number {
  return state.players[slot].energy
}

/**
 * Set a player's energy.
 */
export function setEnergy(state: GameState, slot: number, energy: number): void {
  state.players[slot].energy = energy
}
