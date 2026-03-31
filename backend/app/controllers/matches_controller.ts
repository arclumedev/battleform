import type { HttpContext } from '@adonisjs/core/http'
import { DateTime } from 'luxon'
import Match from '#models/match'
import { gameEngine } from '../engine/game_engine.js'
import type { MapConfig } from '../engine/state.js'
import { generateMapConfig } from '../engine/maps.js'
import MatchPlayer from '#models/match_player'
import crypto from 'node:crypto'

const DEFAULT_MAP_CONFIG = {
  width: 32,
  height: 32,
  terrain: 'mirrored',
  resourceNodes: [
    { x: 5, y: 5, energy: 500 },
    { x: 5, y: 26, energy: 500 },
    { x: 26, y: 5, energy: 500 },
    { x: 26, y: 26, energy: 500 },
    { x: 16, y: 8, energy: 500 },
    { x: 16, y: 23, energy: 500 },
    { x: 8, y: 16, energy: 500 },
    { x: 23, y: 16, energy: 500 },
  ],
  startPositions: [
    { x: 2, y: 2 },
    { x: 29, y: 29 },
  ],
}

function generateAgentToken(matchId: string, slot: number): string {
  const random = crypto.randomBytes(16).toString('hex')
  return `arena_${matchId.slice(0, 8)}_${slot}_${random}`
}

export default class MatchesController {
  async index({ request, response }: HttpContext) {
    const status = request.input('status')

    const query = Match.query().preload('players').orderBy('created_at', 'desc')

    if (status) {
      query.where('status', status)
    }

    const matches = await query.limit(50)

    return response.ok({
      matches: matches.map((m) => ({
        id: m.id,
        status: m.status,
        tickRate: m.tickRate,
        maxTicks: m.maxTicks,
        currentTick: m.currentTick,
        winnerSlot: m.winnerSlot,
        startedAt: m.startedAt,
        finishedAt: m.finishedAt,
        createdAt: m.createdAt,
        players: m.players.map((p) => ({
          slot: p.slot,
          displayName: p.displayName,
          modelId: p.modelId,
          isConnected: p.isConnected,
        })),
      })),
    })
  }

  async store({ auth, response }: HttpContext) {
    await auth.use('web').authenticate()
    const user = auth.use('web').user!

    const match = await Match.create({
      status: 'LOBBY',
      mapConfig: DEFAULT_MAP_CONFIG,
      maxPlayers: 2,
      tickRate: 10,
      maxTicks: 2000,
      currentTick: 0,
      createdBy: user.id,
    })

    return response.created({
      id: match.id,
      status: match.status,
      mapConfig: match.mapConfig,
      createdAt: match.createdAt,
    })
  }

  async show({ params, response }: HttpContext) {
    const match = await Match.query().where('id', params.id).preload('players').firstOrFail()

    return response.ok({
      id: match.id,
      status: match.status,
      mapConfig: match.mapConfig,
      tickRate: match.tickRate,
      maxTicks: match.maxTicks,
      currentTick: match.currentTick,
      winnerSlot: match.winnerSlot,
      startedAt: match.startedAt,
      finishedAt: match.finishedAt,
      createdAt: match.createdAt,
      players: match.players.map((p) => ({
        id: p.id,
        slot: p.slot,
        displayName: p.displayName,
        modelId: p.modelId,
        isConnected: p.isConnected,
        finalScore: p.finalScore,
        finalEnergy: p.finalEnergy,
        finalUnits: p.finalUnits,
      })),
    })
  }

  async join({ auth, params, request, response }: HttpContext) {
    await auth.use('web').authenticate()
    const user = auth.use('web').user!

    const match = await Match.query().where('id', params.id).preload('players').firstOrFail()

    if (match.status !== 'LOBBY') {
      return response.badRequest({ error: 'Match is not in lobby' })
    }

    if (match.players.length >= match.maxPlayers) {
      return response.badRequest({ error: 'Match is full' })
    }

    const alreadyJoined = match.players.find((p) => p.userId === user.id)
    if (alreadyJoined) {
      return response.badRequest({ error: 'Already joined this match' })
    }

    // Find next available agent slot
    const takenSlots = new Set(match.players.map((p) => p.slot))
    let slot = 0
    while (takenSlots.has(slot)) slot++
    const displayName = request.input('display_name', user.fullName || 'Anonymous')
    const modelId = request.input('model_id')
    const agentToken = generateAgentToken(match.id, slot)

    const player = await MatchPlayer.create({
      matchId: match.id,
      userId: user.id,
      slot,
      displayName,
      modelId,
      playerType: 'agent',
      agentToken,
      isConnected: false,
    })

    return response.created({
      id: player.id,
      slot: player.slot,
      displayName: player.displayName,
      agentToken: player.agentToken,
    })
  }

  async start({ auth, params, response }: HttpContext) {
    await auth.use('web').authenticate()
    const user = auth.use('web').user!

    const match = await Match.query().where('id', params.id).preload('players').firstOrFail()

    if (match.createdBy !== user.id) {
      return response.forbidden({ error: 'Only the creator can start the match' })
    }

    if (match.status !== 'LOBBY') {
      return response.badRequest({ error: 'Match is not in lobby' })
    }

    if (match.players.length < match.maxPlayers) {
      return response.badRequest({
        error: `Need ${match.maxPlayers} players to start (have ${match.players.length})`,
      })
    }

    match.status = 'ACTIVE'
    match.startedAt = DateTime.now()
    await match.save()

    // Detect autopilot slots
    const autopilotSlots = match.players
      .filter((p) => p.playerType === 'autopilot')
      .map((p) => p.slot)

    gameEngine.createMatch(
      match.id,
      match.mapConfig as unknown as MapConfig,
      autopilotSlots.length > 0 ? autopilotSlots : undefined
    )
    gameEngine.startMatch(match.id)

    return response.ok({
      id: match.id,
      status: match.status,
      startedAt: match.startedAt,
    })
  }

  /**
   * POST /api/matches/quick
   * Quick play vs autopilot — creates match, auto-joins, starts immediately.
   */
  async quick({ auth, request, response }: HttpContext) {
    await auth.use('web').authenticate()
    const user = auth.use('web').user!

    const displayName = request.input('display_name', user.fullName || 'Player')
    const playerCount = Math.min(8, Math.max(2, request.input('player_count', 2)))
    const autopilotCount = Math.min(
      playerCount - 1,
      Math.max(1, request.input('autopilot_count', playerCount - 1))
    )

    const mapConfig = generateMapConfig(playerCount)

    const match = await Match.create({
      status: 'ACTIVE',
      mapConfig: mapConfig as unknown as Record<string, unknown>,
      maxPlayers: playerCount,
      tickRate: 10,
      maxTicks: 2000,
      currentTick: 0,
      createdBy: user.id,
      startedAt: DateTime.now(),
    })

    // Human in slot 0
    const humanToken = generateAgentToken(match.id, 0)
    await MatchPlayer.create({
      matchId: match.id,
      userId: user.id,
      slot: 0,
      displayName,
      playerType: 'agent',
      agentToken: humanToken,
      isConnected: true,
    })

    // Autopilots fill remaining slots
    const autopilotSlots: number[] = []
    for (let i = 1; i <= autopilotCount; i++) {
      autopilotSlots.push(i)
      await MatchPlayer.create({
        matchId: match.id,
        userId: null,
        slot: i,
        displayName: `Autopilot ${i}`,
        modelId: 'built-in',
        playerType: 'autopilot',
        agentToken: null,
        isConnected: true,
      })
    }

    gameEngine.createMatch(match.id, mapConfig as unknown as MapConfig, autopilotSlots)
    gameEngine.startMatch(match.id)

    return response.created({
      id: match.id,
      status: match.status,
      playerCount,
      autopilotCount,
      agentToken: humanToken,
      startedAt: match.startedAt,
    })
  }

  /**
   * POST /api/matches (updated) — create with slot configuration
   */
  async storeConfigured({ auth, request, response }: HttpContext) {
    await auth.use('web').authenticate()
    const user = auth.use('web').user!

    const slots = request.input('slots') as { type: 'agent' | 'autopilot' }[] | undefined
    const playerCount = slots ? slots.length : request.input('max_players', 2)
    const mapConfig = generateMapConfig(playerCount)

    const match = await Match.create({
      status: 'LOBBY',
      mapConfig: mapConfig as unknown as Record<string, unknown>,
      maxPlayers: playerCount,
      tickRate: 10,
      maxTicks: 2000,
      currentTick: 0,
      createdBy: user.id,
    })

    // Pre-create autopilot slots if specified
    if (slots) {
      for (const [i, slot] of slots.entries()) {
        if (slot.type === 'autopilot') {
          await MatchPlayer.create({
            matchId: match.id,
            userId: null,
            slot: i,
            displayName: `Autopilot ${i}`,
            modelId: 'built-in',
            playerType: 'autopilot',
            agentToken: null,
            isConnected: true,
          })
        }
      }
    }

    return response.created({
      id: match.id,
      status: match.status,
      maxPlayers: match.maxPlayers,
      mapConfig: match.mapConfig,
      createdAt: match.createdAt,
    })
  }
}
