import { DateTime } from 'luxon'
import { BaseModel, column, belongsTo } from '@adonisjs/lucid/orm'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import User from './user.js'
import Match from './match.js'

export default class MatchPlayer extends BaseModel {
  @column({ isPrimary: true })
  declare id: string

  @column()
  declare matchId: string

  @column()
  declare userId: string | null

  @column()
  declare slot: number

  @column()
  declare displayName: string

  @column()
  declare modelId: string | null

  @column()
  declare playerType: 'agent' | 'autopilot'

  @column()
  declare agentToken: string | null

  @column()
  declare isConnected: boolean

  @column()
  declare finalScore: number | null

  @column()
  declare finalEnergy: number | null

  @column()
  declare finalUnits: number | null

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime | null

  @belongsTo(() => Match)
  declare match: BelongsTo<typeof Match>

  @belongsTo(() => User)
  declare user: BelongsTo<typeof User>
}
