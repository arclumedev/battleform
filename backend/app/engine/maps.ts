import type { MapConfig } from './state.js'

/**
 * Generate a map config for the given number of players.
 * Start positions are placed symmetrically.
 * Resource nodes are scaled proportionally.
 */
export function generateMapConfig(playerCount: number): MapConfig {
  const width = 32
  const height = 32

  const startPositions = getStartPositions(playerCount, width, height)
  const resourceNodes = getResourceNodes(playerCount, width, height)

  return {
    width,
    height,
    terrain: 'open',
    startPositions,
    resourceNodes,
  }
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
