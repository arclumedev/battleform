import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'match_snapshots'

  async up() {
    this.schema.createTable(this.tableName, (table) => {
      table.uuid('id').primary().defaultTo(this.raw('gen_random_uuid()'))
      table.uuid('match_id').notNullable().references('id').inTable('matches').onDelete('CASCADE')
      table.integer('tick').notNullable()
      table.jsonb('game_state').notNullable()
      table.timestamp('created_at').notNullable()

      table.unique(['match_id', 'tick'])
    })
  }

  async down() {
    this.schema.dropTable(this.tableName)
  }
}
