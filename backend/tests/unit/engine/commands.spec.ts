import { test } from '@japa/runner'
import { executeCommands } from '../../../app/engine/commands.js'
import { createTestState, getEnergy, setEnergy } from './helpers.js'
import { UNIT_STATS } from '../../../app/engine/state.js'

test.group('Commands', () => {
  test('spawn_unit deducts energy and creates unit', ({ assert }) => {
    const state = createTestState()
    setEnergy(state, 0, 200)

    executeCommands(state, [
      { playerSlot: 0, toolName: 'spawn_unit', toolInput: { unit_type: 'soldier' } },
    ])

    assert.equal(getEnergy(state, 0), 200 - UNIT_STATS.soldier.cost)
    assert.equal(state.units.size, 1)

    const unit = [...state.units.values()][0]
    assert.equal(unit.unitType, 'soldier')
    assert.equal(unit.playerSlot, 0)
  })

  test('spawn_unit fails with insufficient energy', ({ assert }) => {
    const state = createTestState()
    setEnergy(state, 0, 10)

    executeCommands(state, [
      { playerSlot: 0, toolName: 'spawn_unit', toolInput: { unit_type: 'soldier' } },
    ])

    assert.equal(state.units.size, 0)
    assert.equal(getEnergy(state, 0), 10) // Unchanged
  })

  test('spawn_unit places unit at player base', ({ assert }) => {
    const state = createTestState()
    setEnergy(state, 0, 200)
    const base = state.getPlayerBase(0)!

    executeCommands(state, [
      { playerSlot: 0, toolName: 'spawn_unit', toolInput: { unit_type: 'worker' } },
    ])

    const unit = [...state.units.values()][0]
    assert.equal(unit.x, base.x)
    assert.equal(unit.y, base.y)
  })

  test('move_unit sets path and moving status', ({ assert }) => {
    const state = createTestState()
    setEnergy(state, 0, 200)

    // Spawn a unit first
    executeCommands(state, [
      { playerSlot: 0, toolName: 'spawn_unit', toolInput: { unit_type: 'scout' } },
    ])
    const unit = [...state.units.values()][0]

    // Move it
    executeCommands(state, [
      { playerSlot: 0, toolName: 'move_unit', toolInput: { unit_id: unit.id, x: 8, y: 8 } },
    ])

    assert.equal(unit.status, 'moving')
    assert.isAbove(unit.path.length, 0)
  })

  test('move_unit rejects units owned by other player', ({ assert }) => {
    const state = createTestState()
    setEnergy(state, 0, 200)
    setEnergy(state, 1, 200)

    executeCommands(state, [
      { playerSlot: 0, toolName: 'spawn_unit', toolInput: { unit_type: 'scout' } },
    ])
    const unit = [...state.units.values()][0]

    // Player 1 tries to move player 0's unit
    executeCommands(state, [
      { playerSlot: 1, toolName: 'move_unit', toolInput: { unit_id: unit.id, x: 8, y: 8 } },
    ])

    assert.equal(unit.status, 'idle') // Unchanged
  })
})
