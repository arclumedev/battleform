import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.alterTable('matches', (table) => {
      table.integer('max_players').notNullable().defaultTo(2)
    })

    this.schema.alterTable('match_players', (table) => {
      table.string('player_type').notNullable().defaultTo('agent')
      table.string('agent_token', 64).nullable().alter()
    })
  }

  async down() {
    this.schema.alterTable('matches', (table) => {
      table.dropColumn('max_players')
    })

    this.schema.alterTable('match_players', (table) => {
      table.dropColumn('player_type')
      table.string('agent_token', 64).notNullable().alter()
    })
  }
}
