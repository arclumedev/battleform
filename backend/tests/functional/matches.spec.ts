// @ts-nocheck
import { test } from '@japa/runner'
import db from '@adonisjs/lucid/services/db'
import User from '#models/user'

test.group('Matches API', (group) => {
  let user1: User
  let user2: User

  group.each.setup(async () => {
    await db.from('match_players').delete()
    await db.from('match_commands').delete()
    await db.from('match_snapshots').delete()
    await db.from('matches').delete()
    await db.from('auth_identities').delete()
    await db.from('users').delete()

    user1 = await User.create({ fullName: 'Player 1', systemRole: 'CUSTOMER', isActive: true })
    user2 = await User.create({ fullName: 'Player 2', systemRole: 'CUSTOMER', isActive: true })
  })

  test('POST /api/matches creates lobby match', async ({ client, assert }) => {
    const res = await client.post('/api/matches').loginAs(user1)

    res.assertStatus(201)
    assert.equal(res.body().status, 'LOBBY')
    assert.exists(res.body().id)
  })

  test('GET /api/matches lists matches', async ({ client, assert }) => {
    await client.post('/api/matches').loginAs(user1)
    await client.post('/api/matches').loginAs(user1)

    const res = await client.get('/api/matches')

    res.assertStatus(200)
    assert.lengthOf(res.body().matches, 2)
  })

  test('GET /api/matches filters by status', async ({ client, assert }) => {
    await client.post('/api/matches').loginAs(user1)

    const res = await client.get('/api/matches?status=ACTIVE')

    res.assertStatus(200)
    assert.lengthOf(res.body().matches, 0)
  })

  test('POST /api/matches/:id/join assigns slot and returns token', async ({ client, assert }) => {
    const matchRes = await client.post('/api/matches').loginAs(user1)
    const matchId = matchRes.body().id

    // User1 joins first
    await client
      .post(`/api/matches/${matchId}/join`)
      .json({ display_name: 'Player 1' })
      .loginAs(user1)

    // User2 joins second
    const joinRes = await client
      .post(`/api/matches/${matchId}/join`)
      .json({ display_name: 'Player 2' })
      .loginAs(user2)

    joinRes.assertStatus(201)
    assert.exists(joinRes.body().agentToken)
    assert.equal(joinRes.body().slot, 1)
  })

  test('POST /api/matches/:id/start requires all slots filled', async ({ client }) => {
    const matchRes = await client.post('/api/matches').loginAs(user1)
    const matchId = matchRes.body().id

    const res = await client.post(`/api/matches/${matchId}/start`).loginAs(user1)

    res.assertStatus(400)
  })

  test('POST /api/matches/quick creates active match with autopilots', async ({
    client,
    assert,
  }) => {
    const res = await client
      .post('/api/matches/quick')
      .json({ player_count: 4, display_name: 'Me' })
      .loginAs(user1)

    res.assertStatus(201)
    assert.equal(res.body().status, 'ACTIVE')
    assert.equal(res.body().playerCount, 4)
    assert.equal(res.body().autopilotCount, 3)
    assert.exists(res.body().agentToken)
  })
})
