import { DateTime } from 'luxon'
import { BaseModel, column, belongsTo } from '@adonisjs/lucid/orm'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import Match from './match.js'

export default class MatchCommand extends BaseModel {
  @column({ isPrimary: true })
  declare id: string

  @column()
  declare matchId: string

  @column()
  declare playerSlot: number

  @column()
  declare tick: number

  @column()
  declare toolName: string

  @column()
  declare toolInput: Record<string, unknown>

  @column()
  declare toolOutput: Record<string, unknown> | null

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @belongsTo(() => Match)
  declare match: BelongsTo<typeof Match>
}
