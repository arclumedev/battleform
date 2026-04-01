import type { GameState, GameCommand } from './state.js'
import { UNIT_STATS } from './state.js'
import { hexDistance } from './hex.js'

/**
 * Simple bot AI that generates commands each tick.
 *
 * Strategy:
 * - Early game: spawn workers, harvest energy
 * - Mid game: spawn scouts to find enemy, soldiers to attack
 * - Late game: all-in attack on enemy base
 */
export function generateBotCommands(state: GameState, botSlot: number): GameCommand[] {
  const commands: GameCommand[] = []
  const player = state.players[botSlot]
  const base = state.getPlayerBase(botSlot)
  if (!base) return commands

  const myUnits = [...state.units.values()].filter((u) => u.playerSlot === botSlot)
  const workers = myUnits.filter((u) => u.unitType === 'worker')
  const soldiers = myUnits.filter((u) => u.unitType === 'soldier')
  const scouts = myUnits.filter((u) => u.unitType === 'scout')
  const idleWorkers = workers.filter((u) => u.status === 'idle')
  const idleSoldiers = soldiers.filter((u) => u.status === 'idle')
  const idleScouts = scouts.filter((u) => u.status === 'idle')

  // Find nearest resource node with energy
  const resources = [...state.resources.values()].filter((r) => r.remaining > 0)
  const nearestResource = resources.sort((a, b) => {
    const distA = hexDistance(a, base)
    const distB = hexDistance(b, base)
    return distA - distB
  })[0]

  // Find enemy buildings
  const enemyBuildings = [...state.buildings.values()].filter((b) => b.playerSlot !== botSlot)

  // Find visible enemy units
  const enemyUnits = [...state.units.values()].filter((u) => u.playerSlot !== botSlot)

  // --- Spawning logic ---

  // Phase 1: Need at least 2 workers for economy
  if (workers.length < 2 && player.energy >= UNIT_STATS.worker.cost) {
    commands.push({
      playerSlot: botSlot,
      toolName: 'spawn_unit',
      toolInput: { unit_type: 'worker' },
    })
  }
  // Phase 2: Scout to find enemy
  else if (scouts.length < 1 && player.energy >= UNIT_STATS.scout.cost) {
    commands.push({
      playerSlot: botSlot,
      toolName: 'spawn_unit',
      toolInput: { unit_type: 'scout' },
    })
  }
  // Phase 3: Build army
  else if (soldiers.length < 4 && player.energy >= UNIT_STATS.soldier.cost) {
    commands.push({
      playerSlot: botSlot,
      toolName: 'spawn_unit',
      toolInput: { unit_type: 'soldier' },
    })
  }
  // Phase 4: More workers if we have army
  else if (workers.length < 4 && soldiers.length >= 2 && player.energy >= UNIT_STATS.worker.cost) {
    commands.push({
      playerSlot: botSlot,
      toolName: 'spawn_unit',
      toolInput: { unit_type: 'worker' },
    })
  }
  // Phase 5: Keep building soldiers
  else if (player.energy >= UNIT_STATS.soldier.cost) {
    commands.push({
      playerSlot: botSlot,
      toolName: 'spawn_unit',
      toolInput: { unit_type: 'soldier' },
    })
  }

  // --- Worker orders: harvest ---

  for (const worker of idleWorkers) {
    if (nearestResource) {
      commands.push({
        playerSlot: botSlot,
        toolName: 'harvest',
        toolInput: { unit_id: worker.id, resource_id: nearestResource.id },
      })
    }
  }

  // --- Scout orders: explore toward nearest enemy ---

  // Find nearest enemy start position
  const enemyStartPositions = state.mapConfig.startPositions
    .filter((_, i) => i !== botSlot)
    .sort((a, b) => {
      const distA = hexDistance(a, base)
      const distB = hexDistance(b, base)
      return distA - distB
    })
  const enemyStartPos = enemyStartPositions[0] ?? { x: 16, y: 16 }

  // Find nearest enemy base (for soldier targeting too)
  const nearestEnemyBase = enemyBuildings
    .filter((b) => b.buildingType === 'base')
    .sort((a, b) => {
      const distA = hexDistance(a, base)
      const distB = hexDistance(b, base)
      return distA - distB
    })[0]

  for (const scout of idleScouts) {
    // If we see enemies, scout around them; otherwise head toward enemy start
    if (nearestEnemyBase) {
      const enemyBase = nearestEnemyBase
      // Patrol around enemy base
      const offsetX = Math.floor(Math.random() * 6) - 3
      const offsetY = Math.floor(Math.random() * 6) - 3
      const tx = Math.max(0, Math.min(state.mapConfig.width - 1, enemyBase.x + offsetX))
      const ty = Math.max(0, Math.min(state.mapConfig.height - 1, enemyBase.y + offsetY))
      commands.push({
        playerSlot: botSlot,
        toolName: 'move_unit',
        toolInput: { unit_id: scout.id, x: tx, y: ty },
      })
    } else {
      // Head toward enemy start position
      commands.push({
        playerSlot: botSlot,
        toolName: 'move_unit',
        toolInput: { unit_id: scout.id, x: enemyStartPos.x, y: enemyStartPos.y },
      })
    }
  }

  // --- Soldier orders: attack ---

  for (const soldier of idleSoldiers) {
    // Priority: attack nearby enemies, then enemy base
    const nearbyEnemy = enemyUnits.sort((a, b) => {
      const distA = hexDistance(a, soldier)
      const distB = hexDistance(b, soldier)
      return distA - distB
    })[0]

    if (nearbyEnemy) {
      const dist = hexDistance(nearbyEnemy, soldier)
      if (dist < 10) {
        commands.push({
          playerSlot: botSlot,
          toolName: 'attack_target',
          toolInput: { unit_id: soldier.id, target_id: nearbyEnemy.id },
        })
        continue
      }
    }

    if (nearestEnemyBase) {
      commands.push({
        playerSlot: botSlot,
        toolName: 'attack_target',
        toolInput: { unit_id: soldier.id, target_id: nearestEnemyBase.id },
      })
    } else {
      // No enemy base visible — move toward enemy start
      commands.push({
        playerSlot: botSlot,
        toolName: 'move_unit',
        toolInput: { unit_id: soldier.id, x: enemyStartPos.x, y: enemyStartPos.y },
      })
    }
  }

  return commands
}
