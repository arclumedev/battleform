import { test } from '@japa/runner'
import { generateMapConfig } from '../../../app/engine/maps.js'

test.group('Map Generation', () => {
  for (const count of [2, 3, 4, 5, 6, 7, 8]) {
    test(`generates ${count} start positions for ${count} players`, ({ assert }) => {
      const config = generateMapConfig(count)

      assert.lengthOf(config.startPositions, count)
    })
  }

  test('all start positions within map bounds', ({ assert }) => {
    for (const count of [2, 4, 8]) {
      const config = generateMapConfig(count)

      for (const pos of config.startPositions) {
        assert.isAtLeast(pos.x, 0)
        assert.isAtLeast(pos.y, 0)
        assert.isBelow(pos.x, config.width)
        assert.isBelow(pos.y, config.height)
      }
    }
  })

  test('start positions are unique', ({ assert }) => {
    for (const count of [2, 4, 8]) {
      const config = generateMapConfig(count)
      const keys = config.startPositions.map((p) => `${p.x},${p.y}`)
      const unique = new Set(keys)

      assert.equal(unique.size, keys.length, `Duplicate start positions for ${count} players`)
    }
  })

  test('resource nodes placed for each player count', ({ assert }) => {
    for (const count of [2, 4, 8]) {
      const config = generateMapConfig(count)

      assert.isAbove(config.resourceNodes.length, 0)

      for (const node of config.resourceNodes) {
        assert.isAtLeast(node.x, 0)
        assert.isAtLeast(node.y, 0)
        assert.isBelow(node.x, config.width)
        assert.isBelow(node.y, config.height)
        assert.isAbove(node.energy, 0)
      }
    }
  })
})
