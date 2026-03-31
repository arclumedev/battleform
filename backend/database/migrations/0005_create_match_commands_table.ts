import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'match_commands'

  async up() {
    this.schema.createTable(this.tableName, (table) => {
      table.uuid('id').primary().defaultTo(this.raw('gen_random_uuid()'))
      table.uuid('match_id').notNullable().references('id').inTable('matches').onDelete('CASCADE')
      table.integer('player_slot').notNullable()
      table.integer('tick').notNullable()
      table.string('tool_name').notNullable()
      table.jsonb('tool_input').notNullable()
      table.jsonb('tool_output').nullable()
      table.timestamp('created_at').notNullable()

      table.index(['match_id', 'tick'])
    })
  }

  async down() {
    this.schema.dropTable(this.tableName)
  }
}
