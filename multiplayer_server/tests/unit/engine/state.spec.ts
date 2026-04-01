import { test } from '@japa/runner'
import { createTestState, addUnit } from './helpers.js'

test.group('GameState', () => {
  test('initializes correct number of players and bases', ({ assert }) => {
    const state = createTestState(4)

    assert.lengthOf(state.players, 4)
    assert.equal(state.buildings.size, 4)

    for (let i = 0; i < 4; i++) {
      const base = state.getPlayerBase(i)
      assert.exists(base)
      assert.equal(base!.buildingType, 'base')
      assert.equal(base!.playerSlot, i)
    }
  })

  test('initializes resource nodes from config', ({ assert }) => {
    const state = createTestState()
    assert.equal(state.resources.size, 2) // testMapConfig has 2 nodes
  })

  test('state diff detects moved units', ({ assert }) => {
    const state = createTestState()
    const unit = addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 3, y: 3 })

    state.snapshotForDiff()
    unit.x = 4
    unit.y = 3

    const diff = state.computeDiff()
    assert.lengthOf(diff.unitsMoved, 1)
    assert.equal(diff.unitsMoved[0].id, unit.id)
    assert.equal(diff.unitsMoved[0].x, 4)
  })

  test('state diff detects spawned units', ({ assert }) => {
    const state = createTestState()

    state.snapshotForDiff()
    addUnit(state, { playerSlot: 0, unitType: 'worker', x: 2, y: 2 })

    const diff = state.computeDiff()
    assert.lengthOf(diff.unitsSpawned, 1)
    assert.equal(diff.unitsSpawned[0].unitType, 'worker')
  })

  test('state diff detects killed units', ({ assert }) => {
    const state = createTestState()
    const unit = addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 3, y: 3 })

    state.snapshotForDiff()
    state.units.delete(unit.id)

    const diff = state.computeDiff()
    assert.lengthOf(diff.unitsKilled, 1)
    assert.include(diff.unitsKilled, unit.id)
  })

  test('state diff detects energy changes', ({ assert }) => {
    const state = createTestState()

    state.snapshotForDiff()
    state.players[0].energy += 50

    const diff = state.computeDiff()
    assert.lengthOf(diff.resourcesChanged, 1)
    assert.deepEqual(diff.resourcesChanged[0], [0, state.players[0].energy])
  })
})
