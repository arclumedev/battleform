/**
 * WebSocket setup.
 * Registers the upgrade handler on the raw Node HTTP server,
 * wires game engine callbacks, and restores active matches on startup.
 */
import app from '@adonisjs/core/services/app'
import { spectatorService } from '../app/services/spectator_service.js'
import { gameEngine } from '../app/engine/game_engine.js'
import Match from '#models/match'
import { DateTime } from 'luxon'
import type { IncomingMessage } from 'node:http'
import type { Duplex } from 'node:stream'
import type { MapConfig } from '../app/engine/state.js'

app.ready(async () => {
  // Get the underlying Node HTTP server from AdonisJS
  const server = await app.container.make('server')
  const httpServer = server.getNodeServer()

  if (httpServer) {
    httpServer.on('upgrade', (request: IncomingMessage, socket: Duplex, head: Buffer) => {
      const pathname = new URL(request.url || '', `http://${request.headers.host}`).pathname
      const match = pathname.match(/^\/api\/matches\/([^/]+)\/spectate$/)

      if (!match) {
        socket.destroy()
        return
      }

      const matchId = match[1]
      console.log(`[ws] Spectator upgrade for match ${matchId.slice(0, 8)}`)
      spectatorService.handleUpgrade(request, socket, head, matchId)
    })
    console.log('[ws] WebSocket upgrade handler registered on HTTP server')
  } else {
    console.warn('[ws] Could not get Node HTTP server — WebSocket spectating disabled')
  }

  // Wire game engine callbacks
  gameEngine.setCallbacks(
    (matchId, diff) => {
      spectatorService.broadcastDiff(matchId, diff)
    },
    async (matchId, winnerSlot) => {
      spectatorService.broadcastFinish(matchId, winnerSlot)
      const dbMatch = await Match.find(matchId)
      if (dbMatch) {
        dbMatch.status = 'FINISHED'
        dbMatch.winnerSlot = winnerSlot
        dbMatch.finishedAt = DateTime.now()
        await dbMatch.save()
      }
    }
  )

  // Restore active matches into the engine on startup
  const activeMatches = await Match.query().where('status', 'ACTIVE').preload('players')

  for (const match of activeMatches) {
    const autopilotSlots = match.players
      .filter((p) => p.playerType === 'autopilot')
      .map((p) => p.slot)

    gameEngine.createMatch(
      match.id,
      match.mapConfig as unknown as MapConfig,
      autopilotSlots.length > 0 ? autopilotSlots : undefined
    )
    gameEngine.startMatch(match.id)
    console.log(
      `[ws] Restored active match ${match.id.slice(0, 8)} (${match.players.length} players, ${autopilotSlots.length} autopilots)`
    )
  }

  if (activeMatches.length > 0) {
    console.log(`[ws] Restored ${activeMatches.length} active match(es)`)
  }

  console.log('[ws] Game engine callbacks registered')
})
