import type { HttpContext } from '@adonisjs/core/http'
import MatchPlayer from '#models/match_player'
import { handleMcpRequest } from '#services/mcp_server'

export default class McpController {
  async handle({ request, response }: HttpContext) {
    // Authenticate via Bearer token
    const authHeader = request.header('authorization')
    if (!authHeader?.startsWith('Bearer ')) {
      return response.unauthorized({ error: 'Missing or invalid Bearer token' })
    }

    const token = authHeader.slice(7)
    const player = await MatchPlayer.query().where('agent_token', token).preload('match').first()

    if (!player) {
      return response.unauthorized({ error: 'Invalid agent token' })
    }

    if (player.match.status !== 'ACTIVE') {
      return response.badRequest({ error: 'Match is not active' })
    }

    if (!player.isConnected) {
      player.isConnected = true
      await player.save()
    }

    // Hand off to MCP server using raw Node.js objects
    const nodeReq = request.request
    const nodeRes = response.response

    await handleMcpRequest(nodeReq, nodeRes, player)
  }
}
