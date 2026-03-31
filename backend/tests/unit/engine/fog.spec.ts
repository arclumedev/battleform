import { test } from '@japa/runner'
import { computeFog } from '../../../app/engine/fog.js'
import { createTestState, addUnit } from './helpers.js'

test.group('Fog of War', () => {
  test('units reveal tiles within vision radius', ({ assert }) => {
    const state = createTestState()
    addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 8, y: 8 })

    computeFog(state)

    // Soldier has vision 3 — tile at (8,8) should be visible for player 0
    assert.equal(state.visibility[0][8][8], 'visible')
    // Tile within radius
    assert.equal(state.visibility[0][8][10], 'visible')
    // Tile outside radius (distance > 3)
    assert.equal(state.visibility[0][8][12], 'unseen')
  })

  test('scout has extended vision', ({ assert }) => {
    const state = createTestState()
    addUnit(state, { playerSlot: 0, unitType: 'scout', x: 8, y: 8 })

    computeFog(state)

    // Scout has vision 6
    assert.equal(state.visibility[0][8][13], 'visible')
    // Regular soldier vision range (3) wouldn't reach this far
    assert.equal(state.visibility[0][8][15], 'unseen')
  })

  test('previously visible tiles downgrade to previously_seen', ({ assert }) => {
    const state = createTestState()
    const unit = addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 8, y: 8 })

    computeFog(state)
    assert.equal(state.visibility[0][8][8], 'visible')

    // Move the unit away
    unit.x = 2
    unit.y = 2
    computeFog(state)

    // Old position should now be previously_seen
    assert.equal(state.visibility[0][8][8], 'previously_seen')
  })

  test('getVisibleState filters by fog', ({ assert }) => {
    const state = createTestState()
    addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 2, y: 2 })
    addUnit(state, { playerSlot: 1, unitType: 'soldier', x: 14, y: 14 })

    computeFog(state)

    const visible = state.getVisibleState(0)
    const units = visible.units as { playerSlot: number }[]

    // Player 0 should see their own unit but not player 1's (too far)
    const ownUnits = units.filter((u) => u.playerSlot === 0)
    const enemyUnits = units.filter((u) => u.playerSlot === 1)

    assert.lengthOf(ownUnits, 1)
    assert.lengthOf(enemyUnits, 0)
  })
})
