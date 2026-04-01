import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'match_players'

  async up() {
    this.schema.createTable(this.tableName, (table) => {
      table.uuid('id').primary().defaultTo(this.raw('gen_random_uuid()'))
      table.uuid('match_id').notNullable().references('id').inTable('matches').onDelete('CASCADE')
      table.uuid('user_id').nullable().references('id').inTable('users')
      table.integer('slot').notNullable()
      table.string('display_name').notNullable()
      table.string('model_id').nullable()
      table.string('agent_token', 64).notNullable().unique()
      table.boolean('is_connected').notNullable().defaultTo(false)
      table.integer('final_score').nullable()
      table.integer('final_energy').nullable()
      table.integer('final_units').nullable()
      table.timestamp('created_at').notNullable()
      table.timestamp('updated_at').nullable()

      table.unique(['match_id', 'slot'])
    })
  }

  async down() {
    this.schema.dropTable(this.tableName)
  }
}
