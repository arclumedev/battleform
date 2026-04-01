import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'auth_identities'

  async up() {
    this.schema.createTable(this.tableName, (table) => {
      table.uuid('id').primary().defaultTo(this.raw('gen_random_uuid()'))
      table.uuid('user_id').notNullable().references('id').inTable('users').onDelete('CASCADE')
      table.string('provider').notNullable()
      table.string('provider_subject').nullable()
      table.string('email').nullable()
      table.boolean('email_verified').defaultTo(false)
      table.string('password_hash').nullable()
      table.jsonb('provider_profile').nullable()
      table.timestamp('last_used_at').nullable()
      table.timestamp('created_at').notNullable()
      table.timestamp('updated_at').nullable()

      table.unique(['user_id', 'provider'])
      table.unique(['provider', 'provider_subject'])
    })
  }

  async down() {
    this.schema.dropTable(this.tableName)
  }
}
