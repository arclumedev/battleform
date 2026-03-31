import crypto from 'node:crypto'
import { findPath } from './pathfinding.js'
import {
  type GameState,
  type GameCommand,
  type Unit,
  type CombatEvent,
  UNIT_STATS,
  HARVEST_AMOUNT,
} from './state.js'

export function executeCommands(state: GameState, commands: GameCommand[]): void {
  for (const cmd of commands) {
    switch (cmd.toolName) {
      case 'spawn_unit':
        executeSpawnUnit(state, cmd)
        break
      case 'move_unit':
        executeMoveUnit(state, cmd)
        break
      case 'attack_target':
        executeAttackTarget(state, cmd)
        break
      case 'harvest':
        executeHarvest(state, cmd)
        break
    }
  }
}

function executeSpawnUnit(state: GameState, cmd: GameCommand): void {
  const unitType = cmd.toolInput.unit_type as string
  const stats = UNIT_STATS[unitType as keyof typeof UNIT_STATS]
  if (!stats) return

  const player = state.players[cmd.playerSlot]
  if (player.energy < stats.cost) return

  const base = state.getPlayerBase(cmd.playerSlot)
  if (!base) return

  player.energy -= stats.cost

  const unit: Unit = {
    id: crypto.randomUUID(),
    playerSlot: cmd.playerSlot,
    unitType: unitType as Unit['unitType'],
    x: base.x,
    y: base.y,
    hp: stats.hp,
    maxHp: stats.hp,
    status: 'idle',
    path: [],
    targetId: null,
    cargo: 0,
  }

  state.units.set(unit.id, unit)
}

function executeMoveUnit(state: GameState, cmd: GameCommand): void {
  const unitId = cmd.toolInput.unit_id as string
  const x = cmd.toolInput.x as number
  const y = cmd.toolInput.y as number

  const unit = state.units.get(unitId)
  if (!unit || unit.playerSlot !== cmd.playerSlot) return

  const path = findPath(state, { x: unit.x, y: unit.y }, { x, y })
  if (path.length === 0) return

  unit.path = path
  unit.status = 'moving'
  unit.targetId = null
}

function executeAttackTarget(state: GameState, cmd: GameCommand): void {
  const unitId = cmd.toolInput.unit_id as string
  const targetId = cmd.toolInput.target_id as string

  const unit = state.units.get(unitId)
  if (!unit || unit.playerSlot !== cmd.playerSlot) return

  // Target can be a unit or building
  const targetUnit = state.units.get(targetId)
  const targetBuilding = state.buildings.get(targetId)
  if (!targetUnit && !targetBuilding) return

  unit.targetId = targetId
  unit.status = 'attacking'
}

function executeHarvest(state: GameState, cmd: GameCommand): void {
  const unitId = cmd.toolInput.unit_id as string
  const resourceId = cmd.toolInput.resource_id as string

  const unit = state.units.get(unitId)
  if (!unit || unit.playerSlot !== cmd.playerSlot) return
  if (unit.unitType !== 'worker') return

  const resource = state.resources.get(resourceId)
  if (!resource || resource.remaining <= 0) return

  // Move to resource if not adjacent
  const dist = Math.abs(unit.x - resource.x) + Math.abs(unit.y - resource.y)
  if (dist > 1) {
    const path = findPath(state, { x: unit.x, y: unit.y }, { x: resource.x, y: resource.y })
    if (path.length > 0) {
      unit.path = path
      unit.status = 'moving'
      unit.targetId = resourceId
    }
    return
  }

  unit.status = 'harvesting'
  unit.targetId = resourceId
}

// --- Per-tick resolution ---

export function resolveMovement(state: GameState): void {
  for (const unit of state.units.values()) {
    if (unit.status !== 'moving' || unit.path.length === 0) continue

    const stats = UNIT_STATS[unit.unitType]
    // Move up to speed tiles per tick
    const steps = Math.min(stats.speed, unit.path.length)

    for (let i = 0; i < steps; i++) {
      const next = unit.path[0]
      if (state.isBlocked(next.x, next.y)) {
        unit.path = []
        unit.status = 'idle'
        break
      }
      unit.x = next.x
      unit.y = next.y
      unit.path.shift()
    }

    if (unit.path.length === 0) {
      // Arrived — check if we have a target to act on
      if (unit.targetId) {
        const resource = state.resources.get(unit.targetId)
        if (resource && unit.unitType === 'worker') {
          unit.status = 'harvesting'
        } else {
          unit.status = 'idle'
        }
      } else {
        unit.status = 'idle'
      }
    }
  }
}

export function resolveCombat(state: GameState): CombatEvent[] {
  const events: CombatEvent[] = []
  const damageQueue: { targetId: string; damage: number; isUnit: boolean }[] = []

  for (const unit of state.units.values()) {
    if (unit.status !== 'attacking' || !unit.targetId) continue

    const stats = UNIT_STATS[unit.unitType]
    const targetUnit = state.units.get(unit.targetId)
    const targetBuilding = state.buildings.get(unit.targetId)

    if (targetUnit) {
      const dist = Math.abs(unit.x - targetUnit.x) + Math.abs(unit.y - targetUnit.y)

      if (dist <= stats.range) {
        // In range — attack
        damageQueue.push({ targetId: unit.targetId, damage: stats.damage, isUnit: true })
        events.push({
          attackerId: unit.id,
          targetId: unit.targetId,
          damage: stats.damage,
          x: targetUnit.x,
          y: targetUnit.y,
        })
      } else {
        // Move toward target
        const path = findPath(state, { x: unit.x, y: unit.y }, { x: targetUnit.x, y: targetUnit.y })
        if (path.length > 0) {
          unit.path = path
          unit.status = 'moving'
        } else {
          unit.status = 'idle'
          unit.targetId = null
        }
      }
    } else if (targetBuilding) {
      const dist = Math.abs(unit.x - targetBuilding.x) + Math.abs(unit.y - targetBuilding.y)

      if (dist <= stats.range) {
        damageQueue.push({ targetId: unit.targetId, damage: stats.damage, isUnit: false })
        events.push({
          attackerId: unit.id,
          targetId: unit.targetId,
          damage: stats.damage,
          x: targetBuilding.x,
          y: targetBuilding.y,
        })
      } else {
        const path = findPath(
          state,
          { x: unit.x, y: unit.y },
          { x: targetBuilding.x, y: targetBuilding.y }
        )
        if (path.length > 0) {
          unit.path = path
          unit.status = 'moving'
        } else {
          unit.status = 'idle'
          unit.targetId = null
        }
      }
    } else {
      // Target gone
      unit.status = 'idle'
      unit.targetId = null
    }
  }

  // Apply damage simultaneously
  for (const { targetId, damage, isUnit } of damageQueue) {
    if (isUnit) {
      const target = state.units.get(targetId)
      if (target) {
        target.hp -= damage
        if (target.hp <= 0) {
          state.units.delete(targetId)
        }
      }
    } else {
      const target = state.buildings.get(targetId)
      if (target) {
        target.hp -= damage
        if (target.hp <= 0) {
          state.buildings.delete(targetId)
        }
      }
    }
  }

  return events
}

export function resolveHarvesting(state: GameState): void {
  for (const unit of state.units.values()) {
    if (unit.unitType !== 'worker') continue

    if (unit.status === 'harvesting' && unit.targetId) {
      const resource = state.resources.get(unit.targetId)
      if (resource && resource.remaining > 0) {
        const amount = Math.min(HARVEST_AMOUNT, resource.remaining)
        resource.remaining -= amount
        unit.cargo += amount
        // After harvesting, return to base
        unit.status = 'returning'
        const base = state.getPlayerBase(unit.playerSlot)
        if (base) {
          const path = findPath(state, { x: unit.x, y: unit.y }, { x: base.x, y: base.y })
          unit.path = path
        }
      } else {
        unit.status = 'idle'
        unit.targetId = null
      }
    }

    if (unit.status === 'returning') {
      if (unit.path.length === 0) {
        // At base — deposit
        const base = state.getPlayerBase(unit.playerSlot)
        if (base) {
          const dist = Math.abs(unit.x - base.x) + Math.abs(unit.y - base.y)
          if (dist <= 1) {
            state.players[unit.playerSlot].energy += unit.cargo
            unit.cargo = 0
            unit.status = 'idle'
            unit.targetId = null
          }
        }
      }
    }
  }
}
