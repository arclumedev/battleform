import type { GameState, VisibilityState } from './state.js'
import { UNIT_STATS, BASE_STATS } from './state.js'

/**
 * Recompute fog of war for all players.
 * Marks tiles as 'visible' if within vision range of any owned unit/building.
 * Previously visible tiles become 'previously_seen'.
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
  const r2 = radius * radius
  for (let dy = -radius; dy <= radius; dy++) {
    for (let dx = -radius; dx <= radius; dx++) {
      if (dx * dx + dy * dy > r2) continue
      const x = cx + dx
      const y = cy + dy
      if (x >= 0 && x < mapWidth && y >= 0 && y < mapHeight) {
        vis[y][x] = 'visible'
      }
    }
  }
}
