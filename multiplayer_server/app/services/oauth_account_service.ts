import User from '#models/user'
import AuthIdentity from '#models/auth_identity'
import { DateTime } from 'luxon'

interface OAuthUserData {
  provider: string
  providerId: string
  email: string | null
  emailVerified: boolean
  name: string | null
  avatarUrl: string | null
  profile: Record<string, unknown>
}

export default class OAuthAccountService {
  /**
   * Find or create a user from OAuth data.
   * 3-case flow ported from Arclume:
   * 1. Existing provider+subject → log them in
   * 2. Email match, different provider → link accounts
   * 3. Brand new user → create User + AuthIdentity
   */
  async findOrCreate(data: OAuthUserData): Promise<User> {
    // Case 1: Existing identity for this provider+subject
    const existingIdentity = await AuthIdentity.query()
      .where('provider', data.provider)
      .where('provider_subject', data.providerId)
      .preload('user')
      .first()

    if (existingIdentity) {
      existingIdentity.lastUsedAt = DateTime.now()
      existingIdentity.providerProfile = data.profile
      await existingIdentity.save()

      const user = existingIdentity.user
      user.lastLoginAt = DateTime.now()
      if (data.avatarUrl && !user.avatarUrl) {
        user.avatarUrl = data.avatarUrl
      }
      await user.save()

      return user
    }

    // Case 2: Email match — link to existing user
    if (data.email) {
      const emailIdentity = await AuthIdentity.query()
        .where('email', data.email)
        .where('email_verified', true)
        .preload('user')
        .first()

      if (emailIdentity) {
        await AuthIdentity.create({
          userId: emailIdentity.userId,
          provider: data.provider,
          providerSubject: data.providerId,
          email: data.email,
          emailVerified: data.emailVerified,
          providerProfile: data.profile,
          lastUsedAt: DateTime.now(),
        })

        const user = emailIdentity.user
        user.lastLoginAt = DateTime.now()
        if (data.avatarUrl && !user.avatarUrl) {
          user.avatarUrl = data.avatarUrl
        }
        await user.save()

        return user
      }
    }

    // Case 3: Brand new user
    const user = await User.create({
      fullName: data.name,
      avatarUrl: data.avatarUrl,
      systemRole: 'CUSTOMER',
      isActive: true,
      lastLoginAt: DateTime.now(),
    })

    await AuthIdentity.create({
      userId: user.id,
      provider: data.provider,
      providerSubject: data.providerId,
      email: data.email,
      emailVerified: data.emailVerified,
      providerProfile: data.profile,
      lastUsedAt: DateTime.now(),
    })

    return user
  }
}
