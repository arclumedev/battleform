import type { GameState, VisibilityState } from './state.js'
import { UNIT_STATS, BASE_STATS } from './state.js'
import { hexesInRadius } from './hex.js'

/**
 * Recompute fog of war for all players using hex radius.
 */
export function computeFog(state: GameState): void {
  for (let slot = 0; slot < state.players.length; slot++) {
    const vis = state.visibility[slot]

    // Downgrade current 'visible' to 'previously_seen'
    for (let y = 0; y < state.mapConfig.height; y++) {
      for (let x = 0; x < state.mapConfig.width; x++) {
        if (vis[y][x] === 'visible') {
          vis[y][x] = 'previously_seen'
        }
      }
    }

    // Mark tiles visible from units
    for (const unit of state.units.values()) {
      if (unit.playerSlot !== slot) continue
      const vision = UNIT_STATS[unit.unitType].vision
      markVisible(vis, unit.x, unit.y, vision, state.mapConfig.width, state.mapConfig.height)
    }

    // Mark tiles visible from buildings
    for (const building of state.buildings.values()) {
      if (building.playerSlot !== slot) continue
      markVisible(
        vis,
        building.x,
        building.y,
        BASE_STATS.vision,
        state.mapConfig.width,
        state.mapConfig.height
      )
    }
  }
}

function markVisible(
  vis: VisibilityState[][],
  cx: number,
  cy: number,
  radius: number,
  mapWidth: number,
  mapHeight: number
): void {
  for (const pos of hexesInRadius({ x: cx, y: cy }, radius)) {
    if (pos.x >= 0 && pos.x < mapWidth && pos.y >= 0 && pos.y < mapHeight) {
      vis[pos.y][pos.x] = 'visible'
    }
  }
}
