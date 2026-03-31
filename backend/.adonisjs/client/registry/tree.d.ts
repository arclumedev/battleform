/* eslint-disable prettier/prettier */
import type { routes } from './index.ts'

export interface ApiDefinition {
  auth: {
    register: typeof routes['auth.register']
    login: typeof routes['auth.login']
    googleRedirect: typeof routes['auth.google_redirect']
    googleCallback: typeof routes['auth.google_callback']
    githubRedirect: typeof routes['auth.github_redirect']
    githubCallback: typeof routes['auth.github_callback']
    profile: typeof routes['auth.profile']
    logout: typeof routes['auth.logout']
  }
  matches: {
    index: typeof routes['matches.index']
    store: typeof routes['matches.store']
    storeConfigured: typeof routes['matches.store_configured']
    quick: typeof routes['matches.quick']
    show: typeof routes['matches.show']
    join: typeof routes['matches.join']
    start: typeof routes['matches.start']
  }
  mcp: typeof routes['mcp']
}
