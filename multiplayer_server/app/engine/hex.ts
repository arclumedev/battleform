import type { Position } from './state.js'

/**
 * Hex grid utilities using offset coordinates (odd-r: odd rows shifted right).
 *
 * The grid uses (col, row) stored as (x, y) in Position.
 * Odd rows have their columns shifted right by half a hex width.
 */

/**
 * Get the 6 hex neighbors of a position (odd-r offset coordinates).
 */
export function hexNeighbors(pos: Position): Position[] {
  const { x, y } = pos
  const isOddRow = y % 2 !== 0

  if (isOddRow) {
    return [
      { x: x + 1, y: y - 1 }, // top-right
      { x: x, y: y - 1 }, // top-left
      { x: x + 1, y: y }, // right
      { x: x - 1, y: y }, // left
      { x: x + 1, y: y + 1 }, // bottom-right
      { x: x, y: y + 1 }, // bottom-left
    ]
  } else {
    return [
      { x: x, y: y - 1 }, // top-right
      { x: x - 1, y: y - 1 }, // top-left
      { x: x + 1, y: y }, // right
      { x: x - 1, y: y }, // left
      { x: x, y: y + 1 }, // bottom-right
      { x: x - 1, y: y + 1 }, // bottom-left
    ]
  }
}

/**
 * Convert offset (col, row) to cube coordinates (q, r, s).
 */
function offsetToCube(col: number, row: number): { q: number; r: number; s: number } {
  const q = col - Math.floor(row / 2)
  const r = row
  const s = -q - r
  return { q, r, s }
}

/**
 * Hex distance between two positions (offset coordinates).
 */
export function hexDistance(a: Position, b: Position): number {
  const ac = offsetToCube(a.x, a.y)
  const bc = offsetToCube(b.x, b.y)
  return Math.max(Math.abs(ac.q - bc.q), Math.abs(ac.r - bc.r), Math.abs(ac.s - bc.s))
}

/**
 * Get all hex positions within a given radius of a center position.
 */
export function hexesInRadius(center: Position, radius: number): Position[] {
  const results: Position[] = []
  const cc = offsetToCube(center.x, center.y)

  for (let dq = -radius; dq <= radius; dq++) {
    for (let dr = Math.max(-radius, -dq - radius); dr <= Math.min(radius, -dq + radius); dr++) {
      const q = cc.q + dq
      const r = cc.r + dr
      // Convert cube back to offset
      const col = q + Math.floor(r / 2)
      const row = r
      results.push({ x: col, y: row })
    }
  }

  return results
}
