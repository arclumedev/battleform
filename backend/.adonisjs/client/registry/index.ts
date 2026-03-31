/* eslint-disable prettier/prettier */
import type { AdonisEndpoint } from '@tuyau/core/types'
import type { Registry } from './schema.d.ts'
import type { ApiDefinition } from './tree.d.ts'

const placeholder: any = {}

const routes = {
  'auth.register': {
    methods: ["POST"],
    pattern: '/api/auth/register',
    tokens: [{"old":"/api/auth/register","type":0,"val":"api","end":""},{"old":"/api/auth/register","type":0,"val":"auth","end":""},{"old":"/api/auth/register","type":0,"val":"register","end":""}],
    types: placeholder as Registry['auth.register']['types'],
  },
  'auth.login': {
    methods: ["POST"],
    pattern: '/api/auth/login',
    tokens: [{"old":"/api/auth/login","type":0,"val":"api","end":""},{"old":"/api/auth/login","type":0,"val":"auth","end":""},{"old":"/api/auth/login","type":0,"val":"login","end":""}],
    types: placeholder as Registry['auth.login']['types'],
  },
  'auth.google_redirect': {
    methods: ["GET","HEAD"],
    pattern: '/api/auth/google/redirect',
    tokens: [{"old":"/api/auth/google/redirect","type":0,"val":"api","end":""},{"old":"/api/auth/google/redirect","type":0,"val":"auth","end":""},{"old":"/api/auth/google/redirect","type":0,"val":"google","end":""},{"old":"/api/auth/google/redirect","type":0,"val":"redirect","end":""}],
    types: placeholder as Registry['auth.google_redirect']['types'],
  },
  'auth.google_callback': {
    methods: ["GET","HEAD"],
    pattern: '/api/auth/google/callback',
    tokens: [{"old":"/api/auth/google/callback","type":0,"val":"api","end":""},{"old":"/api/auth/google/callback","type":0,"val":"auth","end":""},{"old":"/api/auth/google/callback","type":0,"val":"google","end":""},{"old":"/api/auth/google/callback","type":0,"val":"callback","end":""}],
    types: placeholder as Registry['auth.google_callback']['types'],
  },
  'auth.github_redirect': {
    methods: ["GET","HEAD"],
    pattern: '/api/auth/github/redirect',
    tokens: [{"old":"/api/auth/github/redirect","type":0,"val":"api","end":""},{"old":"/api/auth/github/redirect","type":0,"val":"auth","end":""},{"old":"/api/auth/github/redirect","type":0,"val":"github","end":""},{"old":"/api/auth/github/redirect","type":0,"val":"redirect","end":""}],
    types: placeholder as Registry['auth.github_redirect']['types'],
  },
  'auth.github_callback': {
    methods: ["GET","HEAD"],
    pattern: '/api/auth/github/callback',
    tokens: [{"old":"/api/auth/github/callback","type":0,"val":"api","end":""},{"old":"/api/auth/github/callback","type":0,"val":"auth","end":""},{"old":"/api/auth/github/callback","type":0,"val":"github","end":""},{"old":"/api/auth/github/callback","type":0,"val":"callback","end":""}],
    types: placeholder as Registry['auth.github_callback']['types'],
  },
  'auth.profile': {
    methods: ["GET","HEAD"],
    pattern: '/api/auth/profile',
    tokens: [{"old":"/api/auth/profile","type":0,"val":"api","end":""},{"old":"/api/auth/profile","type":0,"val":"auth","end":""},{"old":"/api/auth/profile","type":0,"val":"profile","end":""}],
    types: placeholder as Registry['auth.profile']['types'],
  },
  'auth.logout': {
    methods: ["POST"],
    pattern: '/api/auth/logout',
    tokens: [{"old":"/api/auth/logout","type":0,"val":"api","end":""},{"old":"/api/auth/logout","type":0,"val":"auth","end":""},{"old":"/api/auth/logout","type":0,"val":"logout","end":""}],
    types: placeholder as Registry['auth.logout']['types'],
  },
  'matches.index': {
    methods: ["GET","HEAD"],
    pattern: '/api/matches',
    tokens: [{"old":"/api/matches","type":0,"val":"api","end":""},{"old":"/api/matches","type":0,"val":"matches","end":""}],
    types: placeholder as Registry['matches.index']['types'],
  },
  'matches.store': {
    methods: ["POST"],
    pattern: '/api/matches',
    tokens: [{"old":"/api/matches","type":0,"val":"api","end":""},{"old":"/api/matches","type":0,"val":"matches","end":""}],
    types: placeholder as Registry['matches.store']['types'],
  },
  'matches.store_configured': {
    methods: ["POST"],
    pattern: '/api/matches/configured',
    tokens: [{"old":"/api/matches/configured","type":0,"val":"api","end":""},{"old":"/api/matches/configured","type":0,"val":"matches","end":""},{"old":"/api/matches/configured","type":0,"val":"configured","end":""}],
    types: placeholder as Registry['matches.store_configured']['types'],
  },
  'matches.quick': {
    methods: ["POST"],
    pattern: '/api/matches/quick',
    tokens: [{"old":"/api/matches/quick","type":0,"val":"api","end":""},{"old":"/api/matches/quick","type":0,"val":"matches","end":""},{"old":"/api/matches/quick","type":0,"val":"quick","end":""}],
    types: placeholder as Registry['matches.quick']['types'],
  },
  'matches.show': {
    methods: ["GET","HEAD"],
    pattern: '/api/matches/:id',
    tokens: [{"old":"/api/matches/:id","type":0,"val":"api","end":""},{"old":"/api/matches/:id","type":0,"val":"matches","end":""},{"old":"/api/matches/:id","type":1,"val":"id","end":""}],
    types: placeholder as Registry['matches.show']['types'],
  },
  'matches.join': {
    methods: ["POST"],
    pattern: '/api/matches/:id/join',
    tokens: [{"old":"/api/matches/:id/join","type":0,"val":"api","end":""},{"old":"/api/matches/:id/join","type":0,"val":"matches","end":""},{"old":"/api/matches/:id/join","type":1,"val":"id","end":""},{"old":"/api/matches/:id/join","type":0,"val":"join","end":""}],
    types: placeholder as Registry['matches.join']['types'],
  },
  'matches.start': {
    methods: ["POST"],
    pattern: '/api/matches/:id/start',
    tokens: [{"old":"/api/matches/:id/start","type":0,"val":"api","end":""},{"old":"/api/matches/:id/start","type":0,"val":"matches","end":""},{"old":"/api/matches/:id/start","type":1,"val":"id","end":""},{"old":"/api/matches/:id/start","type":0,"val":"start","end":""}],
    types: placeholder as Registry['matches.start']['types'],
  },
  'mcp': {
    methods: ["POST"],
    pattern: '/api/mcp',
    tokens: [{"old":"/api/mcp","type":0,"val":"api","end":""},{"old":"/api/mcp","type":0,"val":"mcp","end":""}],
    types: placeholder as Registry['mcp']['types'],
  },
} as const satisfies Record<string, AdonisEndpoint>

export { routes }

export const registry = {
  routes,
  $tree: {} as ApiDefinition,
}

declare module '@tuyau/core/types' {
  export interface UserRegistry {
    routes: typeof routes
    $tree: ApiDefinition
  }
}
