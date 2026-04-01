// @ts-nocheck
import { test } from '@japa/runner'
import db from '@adonisjs/lucid/services/db'

test.group('Auth API', (group) => {
  group.each.setup(async () => {
    await db.from('auth_identities').delete()
    await db.from('match_players').delete()
    await db.from('match_commands').delete()
    await db.from('match_snapshots').delete()
    await db.from('matches').delete()
    await db.from('users').delete()
  })

  test('POST /api/auth/register creates user', async ({ client, assert }) => {
    const res = await client
      .post('/api/auth/register')
      .json({ email: 'test@example.com', password: 'password123', full_name: 'Test User' })

    res.assertStatus(201)
    assert.exists(res.body().id)
    assert.equal(res.body().fullName, 'Test User')
  })

  test('POST /api/auth/register rejects duplicate email', async ({ client }) => {
    await client
      .post('/api/auth/register')
      .json({ email: 'dup@example.com', password: 'password123' })

    const res = await client
      .post('/api/auth/register')
      .json({ email: 'dup@example.com', password: 'password456' })

    res.assertStatus(409)
  })

  test('POST /api/auth/register rejects short password', async ({ client }) => {
    const res = await client
      .post('/api/auth/register')
      .json({ email: 'test@example.com', password: 'short' })

    res.assertStatus(400)
  })

  test('POST /api/auth/login succeeds with valid credentials', async ({ client, assert }) => {
    await client
      .post('/api/auth/register')
      .json({ email: 'login@example.com', password: 'password123' })

    const res = await client
      .post('/api/auth/login')
      .json({ email: 'login@example.com', password: 'password123' })

    res.assertStatus(200)
    assert.exists(res.body().id)
  })

  test('POST /api/auth/login fails with wrong password', async ({ client }) => {
    await client
      .post('/api/auth/register')
      .json({ email: 'wrong@example.com', password: 'password123' })

    const res = await client
      .post('/api/auth/login')
      .json({ email: 'wrong@example.com', password: 'wrongpassword' })

    res.assertStatus(401)
  })

  test('GET /api/auth/profile returns 401 when not authenticated', async ({ client }) => {
    const res = await client.get('/api/auth/profile')
    res.assertStatus(401)
  })

  test('GET /api/auth/profile returns user when authenticated', async ({ client, assert }) => {
    // Register user
    await client
      .post('/api/auth/register')
      .json({ email: 'profile@example.com', password: 'password123', full_name: 'Profile User' })

    const UserModule = await import('#models/user')
    const user = await UserModule.default.query().firstOrFail()

    // Use loginAs helper from @japa/plugin-adonisjs
    const res = await client.get('/api/auth/profile').loginAs(user)

    res.assertStatus(200)
    assert.equal(res.body().fullName, 'Profile User')
  })
})
