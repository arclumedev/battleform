import type { MapConfig, TileType, Tile } from './state.js'

/**
 * Generate a map config for the given number of players.
 * Produces varied terrain with biomes, elevation, and symmetric start positions.
 */
export function generateMapConfig(playerCount: number): MapConfig {
  const width = 32
  const height = 32

  const startPositions = getStartPositions(playerCount, width, height)
  const resourceNodes = getResourceNodes(playerCount, width, height)
  const terrain = generateTerrain(width, height, startPositions)

  return {
    width,
    height,
    terrain,
    startPositions,
    resourceNodes,
  }
}

/**
 * Simple terrain generator using distance-from-center and pseudo-random noise.
 * Creates a naturalistic map with water edges, mountains in the center-ish,
 * and grass around start positions.
 */
function generateTerrain(
  width: number,
  height: number,
  startPositions: { x: number; y: number }[]
): Tile[][] {
  const cx = width / 2
  const cy = height / 2
  const maxDist = Math.sqrt(cx * cx + cy * cy)

  // Simple seeded pseudo-random (deterministic per position)
  const noise = (x: number, y: number): number => {
    const n = Math.sin(x * 127.1 + y * 311.7) * 43758.5453
    return n - Math.floor(n)
  }

  const terrain: Tile[][] = []

  for (let y = 0; y < height; y++) {
    const row: Tile[] = []
    for (let x = 0; x < width; x++) {
      const distFromCenter = Math.sqrt((x - cx) ** 2 + (y - cy) ** 2) / maxDist
      const distFromEdge = Math.min(x, y, width - 1 - x, height - 1 - y)
      const n = noise(x, y)
      const n2 = noise(x + 100, y + 100)

      // Ensure start positions and their surroundings are grass
      const nearStart = startPositions.some((s) => Math.abs(s.x - x) + Math.abs(s.y - y) <= 4)

      let type: TileType
      let elevation: number

      if (nearStart) {
        type = 'grass'
        elevation = 1
      } else if (distFromEdge <= 1) {
        // Map border — water
        type = 'water_sea'
        elevation = 0
      } else if (distFromEdge <= 3 && n < 0.4) {
        type = 'water_lake'
        elevation = 0
      } else if (distFromCenter < 0.2 && n < 0.3) {
        // Center region — some mountains
        type = 'mountain'
        elevation = 3
      } else if (distFromCenter < 0.35 && n < 0.25) {
        type = 'forest'
        elevation = 2
      } else if (n2 < 0.12) {
        type = 'desert'
        elevation = 1
      } else if (n2 < 0.18) {
        type = 'snow'
        elevation = 2
      } else if (n < 0.2) {
        type = 'forest'
        elevation = 1
      } else {
        type = 'grass'
        elevation = 1
      }

      row.push({ type, elevation })
    }
    terrain.push(row)
  }

  return terrain
}

function getStartPositions(
  playerCount: number,
  width: number,
  height: number
): { x: number; y: number }[] {
  const margin = 2

  switch (playerCount) {
    case 2:
      return [
        { x: margin, y: margin },
        { x: width - 1 - margin, y: height - 1 - margin },
      ]

    case 3:
      return [
        { x: margin, y: margin },
        { x: width - 1 - margin, y: margin },
        { x: Math.floor(width / 2), y: height - 1 - margin },
      ]

    case 4:
      return [
        { x: margin, y: margin },
        { x: width - 1 - margin, y: margin },
        { x: margin, y: height - 1 - margin },
        { x: width - 1 - margin, y: height - 1 - margin },
      ]

    case 5:
    case 6:
      return [
        { x: margin, y: margin },
        { x: width - 1 - margin, y: margin },
        { x: margin, y: height - 1 - margin },
        { x: width - 1 - margin, y: height - 1 - margin },
        { x: Math.floor(width / 2), y: margin },
        ...(playerCount >= 6 ? [{ x: Math.floor(width / 2), y: height - 1 - margin }] : []),
      ].slice(0, playerCount)

    case 7:
    case 8:
    default:
      return [
        { x: margin, y: margin }, // top-left
        { x: width - 1 - margin, y: margin }, // top-right
        { x: margin, y: height - 1 - margin }, // bottom-left
        { x: width - 1 - margin, y: height - 1 - margin }, // bottom-right
        { x: Math.floor(width / 2), y: margin }, // top-center
        { x: Math.floor(width / 2), y: height - 1 - margin }, // bottom-center
        { x: margin, y: Math.floor(height / 2) }, // left-center
        { x: width - 1 - margin, y: Math.floor(height / 2) }, // right-center
      ].slice(0, playerCount)
  }
}

function getResourceNodes(
  playerCount: number,
  width: number,
  height: number
): { x: number; y: number; energy: number }[] {
  const nodes: { x: number; y: number; energy: number }[] = []
  const cx = Math.floor(width / 2)
  const cy = Math.floor(height / 2)
  const energy = 500

  // Central resources — always present
  nodes.push(
    { x: cx - 2, y: cy - 2, energy },
    { x: cx + 2, y: cy + 2, energy },
    { x: cx - 2, y: cy + 2, energy },
    { x: cx + 2, y: cy - 2, energy }
  )

  // Per-player nearby resources (close to each start position)
  const starts = getStartPositions(playerCount, width, height)
  for (const pos of starts) {
    const dx = pos.x < cx ? 4 : -4
    const dy = pos.y < cy ? 4 : -4
    nodes.push({ x: pos.x + dx, y: pos.y + dy, energy })
  }

  // Extra resources for larger games
  if (playerCount >= 4) {
    nodes.push(
      { x: cx, y: 6, energy },
      { x: cx, y: height - 7, energy },
      { x: 6, y: cy, energy },
      { x: width - 7, y: cy, energy }
    )
  }

  return nodes
}

/** Predefined configs for convenience */
export const MAP_CONFIGS = {
  '1v1': () => generateMapConfig(2),
  '2v2': () => generateMapConfig(4),
  'ffa-4': () => generateMapConfig(4),
  'ffa-8': () => generateMapConfig(8),
}
