import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'matches'

  async up() {
    this.schema.createTable(this.tableName, (table) => {
      table.uuid('id').primary().defaultTo(this.raw('gen_random_uuid()'))
      table.string('status').notNullable().defaultTo('LOBBY')
      table.jsonb('map_config').notNullable()
      table.integer('tick_rate').notNullable().defaultTo(10)
      table.integer('max_ticks').notNullable().defaultTo(2000)
      table.integer('current_tick').notNullable().defaultTo(0)
      table.integer('winner_slot').nullable()
      table.timestamp('started_at').nullable()
      table.timestamp('finished_at').nullable()
      table.uuid('created_by').notNullable().references('id').inTable('users')
      table.timestamp('created_at').notNullable()
      table.timestamp('updated_at').nullable()
    })
  }

  async down() {
    this.schema.dropTable(this.tableName)
  }
}
