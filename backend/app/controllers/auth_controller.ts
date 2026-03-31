import type { HttpContext } from '@adonisjs/core/http'
import hash from '@adonisjs/core/services/hash'
import OAuthAccountService from '#services/oauth_account_service'
import User from '#models/user'
import AuthIdentity from '#models/auth_identity'
import env from '#start/env'
import { DateTime } from 'luxon'

const oauthService = new OAuthAccountService()

export default class AuthController {
  async register({ request, auth, response }: HttpContext) {
    const email = request.input('email')
    const password = request.input('password')
    const fullName = request.input('full_name')

    if (!email || !password) {
      return response.badRequest({ error: 'Email and password are required' })
    }

    if (password.length < 8) {
      return response.badRequest({ error: 'Password must be at least 8 characters' })
    }

    // Check if email already registered with password provider
    const existing = await AuthIdentity.query()
      .where('provider', 'email')
      .where('email', email)
      .first()

    if (existing) {
      return response.conflict({ error: 'Email already registered' })
    }

    // Check if email exists via OAuth — link to that account
    const oauthIdentity = await AuthIdentity.query()
      .where('email', email)
      .where('email_verified', true)
      .preload('user')
      .first()

    let user: User

    if (oauthIdentity) {
      user = oauthIdentity.user
    } else {
      user = await User.create({
        fullName: fullName || null,
        systemRole: 'CUSTOMER',
        isActive: true,
        lastLoginAt: DateTime.now(),
      })
    }

    const passwordHash = await hash.make(password)

    await AuthIdentity.create({
      userId: user.id,
      provider: 'email',
      providerSubject: email,
      email,
      emailVerified: false,
      passwordHash,
    })

    await auth.use('web').login(user)

    return response.created({
      id: user.id,
      fullName: user.fullName,
      avatarUrl: user.avatarUrl,
    })
  }

  async login({ request, auth, response }: HttpContext) {
    const email = request.input('email')
    const password = request.input('password')

    if (!email || !password) {
      return response.badRequest({ error: 'Email and password are required' })
    }

    const identity = await AuthIdentity.query()
      .where('provider', 'email')
      .where('email', email)
      .preload('user')
      .first()

    if (!identity || !identity.passwordHash) {
      return response.unauthorized({ error: 'Invalid email or password' })
    }

    const valid = await hash.verify(identity.passwordHash, password)
    if (!valid) {
      return response.unauthorized({ error: 'Invalid email or password' })
    }

    if (!identity.user.isActive) {
      return response.forbidden({ error: 'Account is disabled' })
    }

    identity.lastUsedAt = DateTime.now()
    await identity.save()

    identity.user.lastLoginAt = DateTime.now()
    await identity.user.save()

    await auth.use('web').login(identity.user)

    return response.ok({
      id: identity.user.id,
      fullName: identity.user.fullName,
      avatarUrl: identity.user.avatarUrl,
    })
  }

  async googleRedirect({ ally }: HttpContext) {
    return ally.use('google').redirect()
  }

  async googleCallback({ ally, auth, response }: HttpContext) {
    const google = ally.use('google')

    if (google.accessDenied()) {
      return response.redirect(`${env.get('FRONTEND_URL')}/login?error=access_denied`)
    }

    if (google.stateMisMatch()) {
      return response.redirect(`${env.get('FRONTEND_URL')}/login?error=state_mismatch`)
    }

    if (google.hasError()) {
      return response.redirect(`${env.get('FRONTEND_URL')}/login?error=unknown`)
    }

    const googleUser = await google.user()

    const user = await oauthService.findOrCreate({
      provider: 'google',
      providerId: googleUser.id,
      email: googleUser.email,
      emailVerified: googleUser.emailVerificationState === 'verified',
      name: googleUser.name,
      avatarUrl: googleUser.avatarUrl,
      profile: googleUser.original as Record<string, unknown>,
    })

    await auth.use('web').login(user)
    return response.redirect(`${env.get('FRONTEND_URL')}/lobby`)
  }

  async githubRedirect({ ally }: HttpContext) {
    return ally.use('github').redirect()
  }

  async githubCallback({ ally, auth, response }: HttpContext) {
    const github = ally.use('github')

    if (github.accessDenied()) {
      return response.redirect(`${env.get('FRONTEND_URL')}/login?error=access_denied`)
    }

    if (github.stateMisMatch()) {
      return response.redirect(`${env.get('FRONTEND_URL')}/login?error=state_mismatch`)
    }

    if (github.hasError()) {
      return response.redirect(`${env.get('FRONTEND_URL')}/login?error=unknown`)
    }

    const githubUser = await github.user()

    const user = await oauthService.findOrCreate({
      provider: 'github',
      providerId: githubUser.id,
      email: githubUser.email,
      emailVerified: githubUser.emailVerificationState === 'verified',
      name: githubUser.name,
      avatarUrl: githubUser.avatarUrl,
      profile: githubUser.original as Record<string, unknown>,
    })

    await auth.use('web').login(user)
    return response.redirect(`${env.get('FRONTEND_URL')}/lobby`)
  }

  async profile({ auth, response }: HttpContext) {
    await auth.use('web').authenticate()
    const user = auth.use('web').user!

    return response.ok({
      id: user.id,
      fullName: user.fullName,
      avatarUrl: user.avatarUrl,
      systemRole: user.systemRole,
    })
  }

  async logout({ auth, response }: HttpContext) {
    await auth.use('web').authenticate()
    await auth.use('web').logout()
    return response.ok({ message: 'Logged out' })
  }
}
