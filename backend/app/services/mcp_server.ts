import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js'
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js'
import { z } from 'zod'
import { gameEngine } from '../engine/game_engine.js'
import { UNIT_STATS, BASE_STATS, HARVEST_AMOUNT, STARTING_ENERGY } from '../engine/state.js'
import type MatchPlayer from '#models/match_player'
import type { IncomingMessage, ServerResponse } from 'node:http'

const GAME_RULES = `# Battleform Game Rules

## Resources
Energy — harvested by workers from nodes. Used to spawn units.
Starting energy: ${STARTING_ENERGY}

## Units
| Unit | Cost | HP | Speed | Range | Damage | Vision |
|---|---|---|---|---|---|---|
| Worker | ${UNIT_STATS.worker.cost} | ${UNIT_STATS.worker.hp} | ${UNIT_STATS.worker.speed} | ${UNIT_STATS.worker.range} | ${UNIT_STATS.worker.damage} | ${UNIT_STATS.worker.vision} |
| Soldier | ${UNIT_STATS.soldier.cost} | ${UNIT_STATS.soldier.hp} | ${UNIT_STATS.soldier.speed} | ${UNIT_STATS.soldier.range} | ${UNIT_STATS.soldier.damage} | ${UNIT_STATS.soldier.vision} |
| Scout | ${UNIT_STATS.scout.cost} | ${UNIT_STATS.scout.hp} | ${UNIT_STATS.scout.speed} | ${UNIT_STATS.scout.range} | ${UNIT_STATS.scout.damage} | ${UNIT_STATS.scout.vision} |

## Buildings
| Building | HP | Vision | Special |
|---|---|---|---|
| Base | ${BASE_STATS.hp} | ${BASE_STATS.vision} | Pre-placed. Spawns units. |

## Harvesting
Workers collect ${HARVEST_AMOUNT} energy per harvest, then must return to base to deposit.
Resource nodes have finite energy.

## Win Conditions
1. Destroy opponent's Base — instant win
2. Highest score after max ticks — score = base HP + (unit count × 10) + energy
`

function createMcpServer(): McpServer {
  const server = new McpServer(
    { name: 'battleform', version: '0.1.0' },
    { capabilities: { tools: {}, resources: {} } }
  )

  // --- Tools ---

  server.tool(
    'get_game_state',
    'Get the current visible game state for your player (units, buildings, resources, energy)',
    {},
    async (_args, extra) => {
      const player = (extra as any).mcpPlayer as MatchPlayer | undefined
      if (!player) {
        return { content: [{ type: 'text', text: JSON.stringify({ error: 'No player context' }) }] }
      }

      const state = gameEngine.getState(player.matchId)
      if (!state) {
        return { content: [{ type: 'text', text: JSON.stringify({ error: 'Match not found' }) }] }
      }

      const visible = state.getVisibleState(player.slot)
      return { content: [{ type: 'text', text: JSON.stringify(visible) }] }
    }
  )

  server.tool(
    'spawn_unit',
    'Spawn a unit from your Base (costs energy)',
    { unit_type: z.enum(['worker', 'soldier', 'scout']).describe('Type of unit to spawn') },
    async (args, extra) => {
      const player = (extra as any).mcpPlayer as MatchPlayer | undefined
      if (!player) {
        return { content: [{ type: 'text', text: JSON.stringify({ error: 'No player context' }) }] }
      }

      const state = gameEngine.getState(player.matchId)
      if (!state) {
        return { content: [{ type: 'text', text: JSON.stringify({ error: 'Match not found' }) }] }
      }

      const stats = UNIT_STATS[args.unit_type]
      if (state.players[player.slot].energy < stats.cost) {
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                error: 'Insufficient energy',
                cost: stats.cost,
                energy: state.players[player.slot].energy,
              }),
            },
          ],
        }
      }

      gameEngine.queueCommand(player.matchId, {
        playerSlot: player.slot,
        toolName: 'spawn_unit',
        toolInput: { unit_type: args.unit_type },
      })

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({ status: 'queued', unit_type: args.unit_type, cost: stats.cost }),
          },
        ],
      }
    }
  )

  server.tool(
    'move_unit',
    'Order a unit to move to target coordinates',
    {
      unit_id: z.string().describe('ID of the unit to move'),
      x: z.number().int().min(0).describe('Target x coordinate'),
      y: z.number().int().min(0).describe('Target y coordinate'),
    },
    async (args, extra) => {
      const player = (extra as any).mcpPlayer as MatchPlayer | undefined
      if (!player) {
        return { content: [{ type: 'text', text: JSON.stringify({ error: 'No player context' }) }] }
      }

      gameEngine.queueCommand(player.matchId, {
        playerSlot: player.slot,
        toolName: 'move_unit',
        toolInput: { unit_id: args.unit_id, x: args.x, y: args.y },
      })

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              status: 'queued',
              unit_id: args.unit_id,
              target: { x: args.x, y: args.y },
            }),
          },
        ],
      }
    }
  )

  server.tool(
    'attack_target',
    'Order a unit to attack a target unit or building',
    {
      unit_id: z.string().describe('ID of the attacking unit'),
      target_id: z.string().describe('ID of the target (unit or building)'),
    },
    async (args, extra) => {
      const player = (extra as any).mcpPlayer as MatchPlayer | undefined
      if (!player) {
        return { content: [{ type: 'text', text: JSON.stringify({ error: 'No player context' }) }] }
      }

      gameEngine.queueCommand(player.matchId, {
        playerSlot: player.slot,
        toolName: 'attack_target',
        toolInput: { unit_id: args.unit_id, target_id: args.target_id },
      })

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              status: 'queued',
              unit_id: args.unit_id,
              target_id: args.target_id,
            }),
          },
        ],
      }
    }
  )

  server.tool(
    'harvest',
    'Order a worker to harvest energy from a resource node',
    {
      unit_id: z.string().describe('ID of the worker'),
      resource_id: z.string().describe('ID of the resource node'),
    },
    async (args, extra) => {
      const player = (extra as any).mcpPlayer as MatchPlayer | undefined
      if (!player) {
        return { content: [{ type: 'text', text: JSON.stringify({ error: 'No player context' }) }] }
      }

      gameEngine.queueCommand(player.matchId, {
        playerSlot: player.slot,
        toolName: 'harvest',
        toolInput: { unit_id: args.unit_id, resource_id: args.resource_id },
      })

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              status: 'queued',
              unit_id: args.unit_id,
              resource_id: args.resource_id,
            }),
          },
        ],
      }
    }
  )

  // --- Resources ---

  server.resource('game://rules', 'game://rules', async () => ({
    contents: [{ uri: 'game://rules', mimeType: 'text/markdown', text: GAME_RULES }],
  }))

  server.resource('game://map/topology', 'game://map/topology', async (_uri, extra) => {
    const player = (extra as any).mcpPlayer as MatchPlayer | undefined
    if (!player) {
      return {
        contents: [
          {
            uri: 'game://map/topology',
            mimeType: 'application/json',
            text: '{"error":"No player context"}',
          },
        ],
      }
    }

    const state = gameEngine.getState(player.matchId)
    if (!state) {
      return {
        contents: [
          {
            uri: 'game://map/topology',
            mimeType: 'application/json',
            text: '{"error":"Match not found"}',
          },
        ],
      }
    }

    return {
      contents: [
        {
          uri: 'game://map/topology',
          mimeType: 'application/json',
          text: JSON.stringify({
            width: state.mapConfig.width,
            height: state.mapConfig.height,
            startPositions: state.mapConfig.startPositions,
          }),
        },
      ],
    }
  })

  server.resource('game://match/status', 'game://match/status', async (_uri, extra) => {
    const player = (extra as any).mcpPlayer as MatchPlayer | undefined
    if (!player) {
      return {
        contents: [
          {
            uri: 'game://match/status',
            mimeType: 'application/json',
            text: '{"error":"No player context"}',
          },
        ],
      }
    }

    const state = gameEngine.getState(player.matchId)
    if (!state) {
      return {
        contents: [
          {
            uri: 'game://match/status',
            mimeType: 'application/json',
            text: '{"error":"Match not found"}',
          },
        ],
      }
    }

    return {
      contents: [
        {
          uri: 'game://match/status',
          mimeType: 'application/json',
          text: JSON.stringify({
            phase: state.phase,
            tick: state.tick,
            maxTicks: state.mapConfig.width * 100,
            winnerSlot: state.winnerSlot,
            players: state.players.map((p) => ({ slot: p.slot, energy: p.energy })),
          }),
        },
      ],
    }
  })

  return server
}

// Per-session transports keyed by session ID
const transports = new Map<string, StreamableHTTPServerTransport>()

/**
 * Handle an MCP request. Creates a transport per session, connects to the shared MCP server.
 */
export async function handleMcpRequest(
  req: IncomingMessage,
  res: ServerResponse,
  player: MatchPlayer
): Promise<void> {
  const sessionId = req.headers['mcp-session-id'] as string | undefined

  if (sessionId && transports.has(sessionId)) {
    const transport = transports.get(sessionId)!
    // Inject player context into the transport
    ;(transport as any)._mcpPlayer = player
    await transport.handleRequest(req, res)
    return
  }

  // New session — create transport and connect to server
  const transport = new StreamableHTTPServerTransport({
    sessionIdGenerator: () => crypto.randomUUID(),
  })

  ;(transport as any)._mcpPlayer = player

  transport.onclose = () => {
    if (transport.sessionId) {
      transports.delete(transport.sessionId)
    }
  }

  const server = createMcpServer()

  // Inject player into tool/resource extra context
  const originalConnect = server.connect.bind(server)
  await originalConnect(transport)

  if (transport.sessionId) {
    transports.set(transport.sessionId, transport)
  }

  await transport.handleRequest(req, res)
}
