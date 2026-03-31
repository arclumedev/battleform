import type { GameState, Position } from './state.js'
import { hexNeighbors, hexDistance } from './hex.js'

interface AStarNode {
  x: number
  y: number
  g: number
  h: number
  f: number
  parent: AStarNode | null
}

/**
 * A* pathfinding on a hex grid (odd-r offset coordinates).
 * Returns array of positions from start (exclusive) to end (inclusive).
 * Returns empty array if no path found.
 */
export function findPath(state: GameState, start: Position, end: Position): Position[] {
  if (state.isBlocked(end.x, end.y)) return []
  if (start.x === end.x && start.y === end.y) return []

  const openSet: AStarNode[] = []
  const closedSet = new Set<string>()

  const key = (x: number, y: number) => `${x},${y}`
  const h0 = hexDistance(start, end)

  openSet.push({
    x: start.x,
    y: start.y,
    g: 0,
    h: h0,
    f: h0,
    parent: null,
  })

  while (openSet.length > 0) {
    let lowestIdx = 0
    for (let i = 1; i < openSet.length; i++) {
      if (openSet[i].f < openSet[lowestIdx].f) {
        lowestIdx = i
      }
    }

    const current = openSet[lowestIdx]

    if (current.x === end.x && current.y === end.y) {
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

    for (const neighbor of hexNeighbors({ x: current.x, y: current.y })) {
      const nKey = key(neighbor.x, neighbor.y)

      if (closedSet.has(nKey)) continue
      if (state.isBlocked(neighbor.x, neighbor.y)) continue

      const g = current.g + 1
      const existing = openSet.find((n) => n.x === neighbor.x && n.y === neighbor.y)

      if (existing) {
        if (g < existing.g) {
          existing.g = g
          existing.f = g + existing.h
          existing.parent = current
        }
      } else {
        const h = hexDistance(neighbor, end)
        openSet.push({ x: neighbor.x, y: neighbor.y, g, h, f: g + h, parent: current })
      }
    }
  }

  return []
}
