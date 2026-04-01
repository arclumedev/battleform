import '@adonisjs/core/types/http'

type ParamValue = string | number | bigint | boolean

export type ScannedRoutes = {
  ALL: {
    'auth.register': { paramsTuple?: []; params?: {} }
    'auth.login': { paramsTuple?: []; params?: {} }
    'auth.google_redirect': { paramsTuple?: []; params?: {} }
    'auth.google_callback': { paramsTuple?: []; params?: {} }
    'auth.github_redirect': { paramsTuple?: []; params?: {} }
    'auth.github_callback': { paramsTuple?: []; params?: {} }
    'auth.profile': { paramsTuple?: []; params?: {} }
    'auth.logout': { paramsTuple?: []; params?: {} }
    'matches.index': { paramsTuple?: []; params?: {} }
    'matches.store': { paramsTuple?: []; params?: {} }
    'matches.store_configured': { paramsTuple?: []; params?: {} }
    'matches.quick': { paramsTuple?: []; params?: {} }
    'matches.show': { paramsTuple: [ParamValue]; params: {'id': ParamValue} }
    'matches.join': { paramsTuple: [ParamValue]; params: {'id': ParamValue} }
    'matches.start': { paramsTuple: [ParamValue]; params: {'id': ParamValue} }
    'mcp': { paramsTuple?: []; params?: {} }
  }
  GET: {
    'auth.google_redirect': { paramsTuple?: []; params?: {} }
    'auth.google_callback': { paramsTuple?: []; params?: {} }
    'auth.github_redirect': { paramsTuple?: []; params?: {} }
    'auth.github_callback': { paramsTuple?: []; params?: {} }
    'auth.profile': { paramsTuple?: []; params?: {} }
    'matches.index': { paramsTuple?: []; params?: {} }
    'matches.show': { paramsTuple: [ParamValue]; params: {'id': ParamValue} }
  }
  HEAD: {
    'auth.google_redirect': { paramsTuple?: []; params?: {} }
    'auth.google_callback': { paramsTuple?: []; params?: {} }
    'auth.github_redirect': { paramsTuple?: []; params?: {} }
    'auth.github_callback': { paramsTuple?: []; params?: {} }
    'auth.profile': { paramsTuple?: []; params?: {} }
    'matches.index': { paramsTuple?: []; params?: {} }
    'matches.show': { paramsTuple: [ParamValue]; params: {'id': ParamValue} }
  }
  POST: {
    'auth.register': { paramsTuple?: []; params?: {} }
    'auth.login': { paramsTuple?: []; params?: {} }
    'auth.logout': { paramsTuple?: []; params?: {} }
    'matches.store': { paramsTuple?: []; params?: {} }
    'matches.store_configured': { paramsTuple?: []; params?: {} }
    'matches.quick': { paramsTuple?: []; params?: {} }
    'matches.join': { paramsTuple: [ParamValue]; params: {'id': ParamValue} }
    'matches.start': { paramsTuple: [ParamValue]; params: {'id': ParamValue} }
    'mcp': { paramsTuple?: []; params?: {} }
  }
}
declare module '@adonisjs/core/types/http' {
  export interface RoutesList extends ScannedRoutes {}
}