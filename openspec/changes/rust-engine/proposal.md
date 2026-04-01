## Why

The game engine currently lives entirely in the AdonisJS backend (`backend/app/engine/`, ~1400 lines of TypeScript). This means the game cannot run without a running Node.js server, a PostgreSQL database, and a Redis instance. Every game — even a local 1v1 against the built-in bot — requires a full backend deployment.

This blocks:
- **Standalone desktop builds** — users can't download and play offline
- **Low-latency local play** — even local matches round-trip through HTTP/WebSocket
- **Distribution** — shipping a single native binary is far simpler than asking users to run Docker + Node
- **Development iteration** — testing game logic changes requires restarting the backend

The game engine (tick loop, commands, combat, harvesting, pathfinding, fog, map generation, bot AI) has no inherent dependency on Node.js, PostgreSQL, or HTTP. It's pure game logic that should run anywhere.

## Directory Renames

To reflect the new architecture — where the Rust binary IS the game, and AdonisJS is just a multiplayer service — two top-level directories are renamed:

- **`client/` → `game/`** — this is the game, not a "client" of some server. The `crates/` layout lives under `game/` (i.e. `game/crates/bf_types/`, `game/crates/bf_engine/`, `game/crates/bf_game/`). The main Bevy crate is `bf_game` (not `bf_game`).
- **`backend/` → `multiplayer_server/`** — it's a multiplayer relay service, not "the backend" of the game. The game works without it.

## What Changes

- **New `engine/` Rust crate** — port all game logic from `multiplayer_server/app/engine/*.ts` to a `battleform-engine` Rust library crate. This becomes the single source of truth for game rules.
- **Embed engine in the Bevy game** — for local/offline play, the engine runs in-process alongside the renderer. No network, no server. The Bevy app spawns a match, ticks the engine, and feeds state directly to the renderer.
- **Native binary entry point** — `game/crates/bf_game/src/main.rs` boots a Bevy app with the embedded engine for standalone play. The WASM target remains for browser spectating.
- **Engine is the primary application, multiplayer_server is a service** — the Rust binary is the game. For multiplayer, the engine connects to AdonisJS as a client for auth, matchmaking, and state relay. AdonisJS never imports or calls the Rust engine — it's infrastructure the game talks to, not the other way around. This keeps the game fully functional without the multiplayer server and makes it swappable.
- **Multiplayer model** — for online play, the native binary connects to the multiplayer server via WebSocket. Two modes: (1) **host mode** — one player's engine is authoritative, multiplayer server relays commands/state to other players; (2) **dedicated server mode** — multiplayer server runs its own TS engine instance (later replaceable with WASM-compiled Rust engine) for competitive fairness. Either way, the game initiates the connection to the multiplayer server, not the reverse.
- **MCP agents work locally** — agents connect via stdio MCP transport to the local engine (no HTTP server needed). For multiplayer, agents still connect via HTTP through AdonisJS.
- **Shared types crate** — `battleform-types` holds all shared types (GameState, StateDiff, units, buildings, commands, tile types) used by both the engine and the game.

## Capabilities

### New Capabilities

- **Offline play** — launch the native binary, play against bot AI with no internet connection
- **Local MCP agents** — AI agents connect to the local engine via stdio transport
- **Embedded engine** — game logic runs in the same process as the renderer, zero network latency

### Modified Capabilities

- `game-engine`: moves from TypeScript to Rust. Same rules, same tick rate, same behavior. The spec requirements and scenarios are unchanged — this is a port, not a redesign.
- `game-client`: gains an embedded engine mode alongside the existing WebSocket spectator mode. When running natively, the client ticks the engine directly. When running as WASM in a browser, it still receives state via the JS bridge.
- `mcp-server`: gains a local stdio transport. Agents can connect to the native binary directly. The HTTP transport through AdonisJS remains for multiplayer.
- `match-lobby`: split into local (in-engine, no persistence) and online (AdonisJS with Postgres). Local matches skip auth and matchmaking.
- `spectator`: in local mode, the renderer reads state directly from the engine (no WebSocket). In multiplayer, WebSocket spectating remains.

## Impact

### New files

- `game/Cargo.toml` (workspace root) — workspace with `resolver = "2"`, shared deps, dev/release profiles
- `game/crates/bf_types/` — shared types crate (no Bevy dep): state.rs, hex.rs, commands.rs
- `game/crates/bf_engine/` — game simulation crate: lib.rs, commands.rs, pathfinding.rs, fog.rs, maps.rs, bot_ai.rs, mcp.rs
- `game/crates/bf_game/src/main.rs` — native binary entry point with embedded engine
- `game/crates/bf_game/src/game.rs` — root `GamePlugin` composing per-domain sub-plugins
- `game/crates/bf_game/src/camera.rs` — `CameraPlugin`
- `game/crates/bf_game/src/map.rs` — `MapPlugin`
- `game/crates/bf_game/src/units.rs` — `UnitsPlugin`
- `game/crates/bf_game/src/hud.rs` — `HudPlugin`
- `game/crates/bf_game/src/input.rs` — `InputPlugin`
- `game/crates/bf_game/src/local_match.rs` — `LocalMatchPlugin` (embedded engine, FixedUpdate ticking)
- `game/crates/bf_game/src/network.rs` — `NetworkPlugin` (WebSocket state receiver + multiplayer)

### Modified files

- `client/` → `game/crates/bf_game/` — renamed, Cargo.toml updated with workspace deps and feature flags (`native`/`wasm`)
- `backend/` → `multiplayer_server/` — renamed to reflect its role as a multiplayer service, not "the backend"
- `game/crates/bf_game/src/lib.rs` — gate WASM-specific code, shared Bevy setup via `GamePlugin`
- `game/crates/bf_game/src/state.rs` — re-exports from `bf_types` instead of defining types locally
- `game/crates/bf_game/src/renderer.rs` — split into per-domain plugin modules (camera, map, units, hud)

### Unchanged files

- `multiplayer_server/` (formerly `backend/`) — stays as the multiplayer service layer. Engine never calls into it at the code level — it connects via HTTP/WebSocket at runtime.
- `frontend/` — unchanged, still the browser shell for online play
- `openspec/specs/game-engine/spec.md` — requirements unchanged (same game rules, different implementation language)
