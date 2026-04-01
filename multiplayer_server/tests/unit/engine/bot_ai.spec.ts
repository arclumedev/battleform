import { test } from '@japa/runner'
import { generateBotCommands } from '../../../app/engine/bot_ai.js'
import { createTestState, setEnergy } from './helpers.js'

test.group('Bot AI', () => {
  test('spawns workers in early game', ({ assert }) => {
    const state = createTestState()
    setEnergy(state, 0, 200)

    const commands = generateBotCommands(state, 0)

    const spawnCmd = commands.find((c) => c.toolName === 'spawn_unit')
    assert.exists(spawnCmd)
    assert.equal(spawnCmd!.toolInput.unit_type, 'worker')
  })

  test('targets nearest enemy start position for scouting', ({ assert }) => {
    const state = createTestState()
    setEnergy(state, 0, 500)

    // Spawn some units to get past early phases
    for (let i = 0; i < 2; i++) {
      const unit = {
        id: crypto.randomUUID(),
        playerSlot: 0,
        unitType: 'worker' as const,
        x: 1,
        y: 1,
        hp: 30,
        maxHp: 30,
        status: 'harvesting' as const,
        path: [],
        targetId: null,
        cargo: 0,
      }
      state.units.set(unit.id, unit)
    }

    // Add an idle scout
    const scout = {
      id: crypto.randomUUID(),
      playerSlot: 0,
      unitType: 'scout' as const,
      x: 5,
      y: 5,
      hp: 40,
      maxHp: 40,
      status: 'idle' as const,
      path: [],
      targetId: null,
      cargo: 0,
    }
    state.units.set(scout.id, scout)

    const commands = generateBotCommands(state, 0)
    const moveCmd = commands.find(
      (c) => c.toolName === 'move_unit' && c.toolInput.unit_id === scout.id
    )

    assert.exists(moveCmd)
    // Should target toward enemy side of the map (x > scout's position)
    assert.isAbove(moveCmd!.toolInput.x as number, scout.x)
    assert.isAbove(moveCmd!.toolInput.y as number, scout.y)
  })

  test('works correctly for non-zero bot slot', ({ assert }) => {
    const state = createTestState()
    setEnergy(state, 1, 200)

    const commands = generateBotCommands(state, 1)

    assert.isAbove(commands.length, 0)
    // All commands should be for slot 1
    for (const cmd of commands) {
      assert.equal(cmd.playerSlot, 1)
    }
  })
})
