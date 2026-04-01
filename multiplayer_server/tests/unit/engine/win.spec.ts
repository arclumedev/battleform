import { test } from '@japa/runner'
import { GameEngine } from '../../../app/engine/game_engine.js'
import { createTestState } from './helpers.js'

test.group('Win Conditions', () => {
  test('last base standing wins in 2-player', ({ assert }) => {
    const engine = new GameEngine()
    const state = createTestState()

    // Destroy player 1's base
    const base1 = state.getPlayerBase(1)!
    state.buildings.delete(base1.id)

    // Use private method via type assertion
    ;(engine as any).checkWinConditions(state)

    assert.equal(state.phase, 'finished')
    assert.equal(state.winnerSlot, 0)
  })

  test('last base standing wins in 4-player', ({ assert }) => {
    const engine = new GameEngine()
    const state = createTestState(4)

    // Destroy bases for players 0, 1, 2 — player 3 should win
    for (let i = 0; i < 3; i++) {
      const base = state.getPlayerBase(i)!
      state.buildings.delete(base.id)
    }

    ;(engine as any).checkWinConditions(state)

    assert.equal(state.phase, 'finished')
    assert.equal(state.winnerSlot, 3)
  })

  test('draw when all bases destroyed simultaneously', ({ assert }) => {
    const engine = new GameEngine()
    const state = createTestState()

    // Destroy both bases
    for (const player of state.players) {
      state.buildings.delete(player.baseId)
    }

    ;(engine as any).checkWinConditions(state)

    assert.equal(state.phase, 'finished')
    assert.isNull(state.winnerSlot)
  })

  test('max ticks triggers score comparison', ({ assert }) => {
    const engine = new GameEngine()
    const state = createTestState()

    // Set tick to max
    state.tick = state.mapConfig.width * 100

    // Give player 0 more energy for a higher score
    state.players[0].energy = 1000
    state.players[1].energy = 100
    ;(engine as any).checkWinConditions(state)

    assert.equal(state.phase, 'finished')
    assert.equal(state.winnerSlot, 0)
  })

  test('tie when scores are equal at max ticks', ({ assert }) => {
    const engine = new GameEngine()
    const state = createTestState()

    state.tick = state.mapConfig.width * 100
    state.players[0].energy = 500
    state.players[1].energy = 500
    ;(engine as any).checkWinConditions(state)

    assert.equal(state.phase, 'finished')
    assert.isNull(state.winnerSlot)
  })
})
