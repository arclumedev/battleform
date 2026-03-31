import { WebSocketServer, WebSocket } from 'ws'
import { pack } from 'msgpackr'
import type { IncomingMessage } from 'node:http'
import type { Duplex } from 'node:stream'
import type { StateDiff } from '../engine/state.js'
import { gameEngine } from '../engine/game_engine.js'

/**
 * Manages WebSocket connections for match spectators.
 * Broadcasts MessagePack-encoded state diffs each tick.
 */
class SpectatorService {
  private wss: WebSocketServer
  private matchClients: Map<string, Set<WebSocket>> = new Map()

  constructor() {
    this.wss = new WebSocketServer({ noServer: true })

    this.wss.on('connection', (ws: WebSocket, matchId: string) => {
      if (!this.matchClients.has(matchId)) {
        this.matchClients.set(matchId, new Set())
      }
      this.matchClients.get(matchId)!.add(ws)

      // Send full state snapshot on connect
      const state = gameEngine.getState(matchId)
      if (state) {
        const snapshot = {
          type: 'snapshot',
          tick: state.tick,
          mapWidth: state.mapConfig.width,
          mapHeight: state.mapConfig.height,
          terrain: state.terrain,
          units: [...state.units.values()],
          buildings: [...state.buildings.values()],
          resources: [...state.resources.values()],
          players: state.players.map((p) => ({ slot: p.slot, energy: p.energy })),
          visibility: state.visibility[0] ?? [],
        }
        const packed = pack(snapshot)
        console.log(
          `[ws] Sending snapshot to spectator: ${packed.byteLength} bytes, ${state.buildings.size} buildings, ${state.units.size} units`
        )
        ws.send(packed)
      } else {
        console.log(`[ws] No engine state for match ${matchId.slice(0, 8)}`)
      }

      ws.on('close', () => {
        const clients = this.matchClients.get(matchId)
        if (clients) {
          clients.delete(ws)
          if (clients.size === 0) {
            this.matchClients.delete(matchId)
          }
        }
      })

      ws.on('error', () => {
        ws.close()
      })
    })
  }

  /**
   * Handle WebSocket upgrade from the HTTP server.
   */
  handleUpgrade(request: IncomingMessage, socket: Duplex, head: Buffer, matchId: string): void {
    this.wss.handleUpgrade(request, socket, head, (ws) => {
      this.wss.emit('connection', ws, matchId)
    })
  }

  /**
   * Broadcast a state diff to all spectators of a match.
   */
  broadcastDiff(matchId: string, diff: StateDiff): void {
    const clients = this.matchClients.get(matchId)
    if (!clients || clients.size === 0) return

    const message = pack({ type: 'diff', ...diff })

    for (const ws of clients) {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(message)
      }
    }
  }

  /**
   * Notify spectators that a match has ended.
   */
  broadcastFinish(matchId: string, winnerSlot: number | null): void {
    const clients = this.matchClients.get(matchId)
    if (!clients) return

    const message = pack({ type: 'finished', winnerSlot })

    for (const ws of clients) {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(message)
        ws.close()
      }
    }

    this.matchClients.delete(matchId)
  }

  /**
   * Get the count of connected spectators for a match.
   */
  getSpectatorCount(matchId: string): number {
    return this.matchClients.get(matchId)?.size ?? 0
  }
}

export const spectatorService = new SpectatorService()
