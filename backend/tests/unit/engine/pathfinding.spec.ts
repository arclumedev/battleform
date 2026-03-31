import { test } from '@japa/runner'
import { findPath } from '../../../app/engine/pathfinding.js'
import { createTestState } from './helpers.js'

test.group('Pathfinding', () => {
  test('finds direct path on open grid', ({ assert }) => {
    const state = createTestState()
    const path = findPath(state, { x: 0, y: 0 }, { x: 3, y: 0 })

    assert.isAbove(path.length, 0)
    assert.deepEqual(path[path.length - 1], { x: 3, y: 0 })
  })

  test('returns empty path when destination is blocked', ({ assert }) => {
    const state = createTestState()
    state.terrain[5][5] = { type: 'mountain', elevation: 3 }

    const path = findPath(state, { x: 4, y: 5 }, { x: 5, y: 5 })
    assert.lengthOf(path, 0)
  })

  test('returns empty path when start equals destination', ({ assert }) => {
    const state = createTestState()
    const path = findPath(state, { x: 3, y: 3 }, { x: 3, y: 3 })
    assert.lengthOf(path, 0)
  })

  test('navigates around obstacles', ({ assert }) => {
    const state = createTestState()
    // Create a wall from y=3 to y=7 at x=5
    for (let y = 3; y <= 7; y++) {
      state.terrain[y][5] = { type: 'mountain', elevation: 3 }
    }

    const path = findPath(state, { x: 4, y: 5 }, { x: 6, y: 5 })
    assert.isAbove(path.length, 2) // Must go around
    assert.deepEqual(path[path.length - 1], { x: 6, y: 5 })

    // Verify no step goes through the wall
    for (const step of path) {
      assert.isFalse(step.x === 5 && step.y >= 3 && step.y <= 7)
    }
  })

  test('returns empty path when destination is out of bounds', ({ assert }) => {
    const state = createTestState()
    const path = findPath(state, { x: 0, y: 0 }, { x: -1, y: 0 })
    assert.lengthOf(path, 0)
  })

  test('path excludes start position', ({ assert }) => {
    const state = createTestState()
    const path = findPath(state, { x: 2, y: 2 }, { x: 5, y: 2 })

    assert.isAbove(path.length, 0)
    assert.notDeepEqual(path[0], { x: 2, y: 2 })
  })
})
