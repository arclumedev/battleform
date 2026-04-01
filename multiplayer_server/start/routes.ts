import router from '@adonisjs/core/services/router'

router.get('/', () => {
  return { name: 'battleform', status: 'ok' }
})

router
  .group(() => {
    // Auth
    router
      .group(() => {
        // Email + Password
        router.post('/register', '#controllers/auth_controller.register')
        router.post('/login', '#controllers/auth_controller.login')

        // OAuth
        router.get('/google/redirect', '#controllers/auth_controller.googleRedirect')
        router.get('/google/callback', '#controllers/auth_controller.googleCallback')
        router.get('/github/redirect', '#controllers/auth_controller.githubRedirect')
        router.get('/github/callback', '#controllers/auth_controller.githubCallback')

        // Session
        router.get('/profile', '#controllers/auth_controller.profile')
        router.post('/logout', '#controllers/auth_controller.logout')
      })
      .prefix('/auth')

    // Matches
    router
      .group(() => {
        router.get('/', '#controllers/matches_controller.index')
        router.post('/', '#controllers/matches_controller.store')
        router.post('/configured', '#controllers/matches_controller.storeConfigured')
        router.post('/quick', '#controllers/matches_controller.quick')
        router.get('/:id', '#controllers/matches_controller.show')
        router.post('/:id/join', '#controllers/matches_controller.join')
        router.post('/:id/start', '#controllers/matches_controller.start')
      })
      .prefix('/matches')

    // MCP (agent auth via Bearer token)
    router.post('/mcp', '#controllers/mcp_controller.handle')
  })
  .prefix('/api')
