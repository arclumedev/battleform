import { test } from '@japa/runner'
import { resolveCombat } from '../../../app/engine/commands.js'
import { createTestState, addUnit } from './helpers.js'

test.group('Combat', () => {
  test('units deal correct damage per type', ({ assert }) => {
    const state = createTestState()
    const soldier = addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 5, y: 5 })
    const target = addUnit(state, { playerSlot: 1, unitType: 'soldier', x: 5, y: 6, hp: 80 })

    soldier.status = 'attacking'
    soldier.targetId = target.id

    resolveCombat(state)

    const updatedTarget = state.units.get(target.id)
    assert.exists(updatedTarget)
    assert.equal(updatedTarget!.hp, 60) // 80 - 20 (soldier damage)
  })

  test('simultaneous damage resolution', ({ assert }) => {
    const state = createTestState()
    const a = addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 5, y: 5, hp: 80 })
    const b = addUnit(state, { playerSlot: 1, unitType: 'soldier', x: 5, y: 6, hp: 80 })

    a.status = 'attacking'
    a.targetId = b.id
    b.status = 'attacking'
    b.targetId = a.id

    resolveCombat(state)

    // Both should take damage — simultaneous, no first-strike
    const updatedA = state.units.get(a.id)
    const updatedB = state.units.get(b.id)
    assert.exists(updatedA)
    assert.exists(updatedB)
    assert.equal(updatedA!.hp, 60)
    assert.equal(updatedB!.hp, 60)
  })

  test('units die at 0 HP and are removed', ({ assert }) => {
    const state = createTestState()
    const soldier = addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 5, y: 5 })
    const target = addUnit(state, { playerSlot: 1, unitType: 'worker', x: 5, y: 6, hp: 10 })

    soldier.status = 'attacking'
    soldier.targetId = target.id

    resolveCombat(state)

    assert.isFalse(state.units.has(target.id))
  })

  test('soldiers attack buildings', ({ assert }) => {
    const state = createTestState()
    const soldier = addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 2, y: 14 })
    const enemyBase = state.getPlayerBase(1)!

    // Move soldier adjacent to enemy base
    soldier.x = enemyBase.x
    soldier.y = enemyBase.y - 1
    soldier.status = 'attacking'
    soldier.targetId = enemyBase.id

    const initialHp = enemyBase.hp
    resolveCombat(state)

    assert.equal(enemyBase.hp, initialHp - 20)
  })

  test('dead target clears attacker status', ({ assert }) => {
    const state = createTestState()
    const soldier = addUnit(state, { playerSlot: 0, unitType: 'soldier', x: 5, y: 5 })
    const target = addUnit(state, { playerSlot: 1, unitType: 'worker', x: 5, y: 6, hp: 5 })

    soldier.status = 'attacking'
    soldier.targetId = target.id

    resolveCombat(state)

    // Target killed
    assert.isFalse(state.units.has(target.id))

    // Next combat pass — attacker should reset since target is gone
    resolveCombat(state)
    assert.equal(soldier.status, 'idle')
    assert.isNull(soldier.targetId)
  })
})
