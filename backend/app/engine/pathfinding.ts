import type { GameState, Position } from './state.js'

interface AStarNode {
  x: number
  y: number
  g: number
  h: number
  f: number
  parent: AStarNode | null
}

function heuristic(a: Position, b: Position): number {
  return Math.abs(a.x - b.x) + Math.abs(a.y - b.y)
}

const NEIGHBORS = [
  { dx: 0, dy: -1 },
  { dx: 0, dy: 1 },
  { dx: -1, dy: 0 },
  { dx: 1, dy: 0 },
]

/**
 * A* pathfinding on the game grid.
 * Returns array of positions from start (exclusive) to end (inclusive).
 * Returns empty array if no path found.
 */
export function findPath(state: GameState, start: Position, end: Position): Position[] {
  if (state.isBlocked(end.x, end.y)) return []
  if (start.x === end.x && start.y === end.y) return []

  const openSet: AStarNode[] = []
  const closedSet = new Set<string>()

  const key = (x: number, y: number) => `${x},${y}`

  openSet.push({
    x: start.x,
    y: start.y,
    g: 0,
    h: heuristic(start, end),
    f: heuristic(start, end),
    parent: null,
  })

  while (openSet.length > 0) {
    // Find node with lowest f
    let lowestIdx = 0
    for (let i = 1; i < openSet.length; i++) {
      if (openSet[i].f < openSet[lowestIdx].f) {
        lowestIdx = i
      }
    }

    const current = openSet[lowestIdx]

    if (current.x === end.x && current.y === end.y) {
      // Reconstruct path (exclude start)
      const path: Position[] = []
      let node: AStarNode | null = current
      while (node && !(node.x === start.x && node.y === start.y)) {
        path.unshift({ x: node.x, y: node.y })
        node = node.parent
      }
      return path
    }

    openSet.splice(lowestIdx, 1)
    closedSet.add(key(current.x, current.y))

    for (const { dx, dy } of NEIGHBORS) {
      const nx = current.x + dx
      const ny = current.y + dy
      const nKey = key(nx, ny)

      if (closedSet.has(nKey)) continue
      if (state.isBlocked(nx, ny)) continue

      const g = current.g + 1
      const existing = openSet.find((n) => n.x === nx && n.y === ny)

      if (existing) {
        if (g < existing.g) {
          existing.g = g
          existing.f = g + existing.h
          existing.parent = current
        }
      } else {
        const h = heuristic({ x: nx, y: ny }, end)
        openSet.push({ x: nx, y: ny, g, h, f: g + h, parent: current })
      }
    }
  }

  return [] // No path found
}
