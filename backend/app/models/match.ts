import { DateTime } from 'luxon'
import { BaseModel, column, hasMany, belongsTo } from '@adonisjs/lucid/orm'
import type { HasMany, BelongsTo } from '@adonisjs/lucid/types/relations'
import User from './user.js'
import MatchPlayer from './match_player.js'
import MatchCommand from './match_command.js'
import MatchSnapshot from './match_snapshot.js'

export default class Match extends BaseModel {
  @column({ isPrimary: true })
  declare id: string

  @column()
  declare status: 'LOBBY' | 'ACTIVE' | 'FINISHED' | 'CANCELLED'

  @column()
  declare mapConfig: Record<string, unknown>

  @column()
  declare tickRate: number

  @column()
  declare maxTicks: number

  @column()
  declare currentTick: number

  @column()
  declare maxPlayers: number

  @column()
  declare winnerSlot: number | null

  @column.dateTime()
  declare startedAt: DateTime | null

  @column.dateTime()
  declare finishedAt: DateTime | null

  @column()
  declare createdBy: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime | null

  @belongsTo(() => User, { foreignKey: 'createdBy' })
  declare creator: BelongsTo<typeof User>

  @hasMany(() => MatchPlayer)
  declare players: HasMany<typeof MatchPlayer>

  @hasMany(() => MatchCommand)
  declare commands: HasMany<typeof MatchCommand>

  @hasMany(() => MatchSnapshot)
  declare snapshots: HasMany<typeof MatchSnapshot>
}
