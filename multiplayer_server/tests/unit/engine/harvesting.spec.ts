import { test } from '@japa/runner'
import { resolveHarvesting } from '../../../app/engine/commands.js'
import { createTestState, addUnit, getEnergy } from './helpers.js'

test.group('Harvesting', () => {
  test('workers harvest from resource nodes', ({ assert }) => {
    const state = createTestState()
    const resource = [...state.resources.values()][0]
    const worker = addUnit(state, {
      playerSlot: 0,
      unitType: 'worker',
      x: resource.x,
      y: resource.y - 1,
    })

    worker.status = 'harvesting'
    worker.targetId = resource.id

    const initialRemaining = resource.remaining
    resolveHarvesting(state)

    assert.equal(resource.remaining, initialRemaining - 25)
    assert.equal(worker.cargo, 25)
    assert.equal(worker.status, 'returning')
  })

  test('workers deposit energy at base', ({ assert }) => {
    const state = createTestState()
    const base = state.getPlayerBase(0)!
    const worker = addUnit(state, { playerSlot: 0, unitType: 'worker', x: base.x, y: base.y - 1 })

    worker.status = 'returning'
    worker.cargo = 25
    worker.path = [] // Already at base

    const initialEnergy = getEnergy(state, 0)
    resolveHarvesting(state)

    assert.equal(getEnergy(state, 0), initialEnergy + 25)
    assert.equal(worker.cargo, 0)
    assert.equal(worker.status, 'idle')
  })

  test('depleted resource nodes stop yielding', ({ assert }) => {
    const state = createTestState()
    const resource = [...state.resources.values()][0]
    resource.remaining = 0

    const worker = addUnit(state, {
      playerSlot: 0,
      unitType: 'worker',
      x: resource.x,
      y: resource.y - 1,
    })
    worker.status = 'harvesting'
    worker.targetId = resource.id

    resolveHarvesting(state)

    assert.equal(worker.cargo, 0)
    assert.equal(worker.status, 'idle')
  })
})
