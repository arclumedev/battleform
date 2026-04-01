## 1. Workspace setup and crate scaffolding

- [x] 1.1 Create workspace `Cargo.toml` at repo root (`resolver = "2"`, workspace deps, dev/release profiles)
- [x] 1.2 Create `crates/bf_types/Cargo.toml` — shared types crate, depends on `serde` only (no Bevy)
- [x] 1.3 Create `crates/bf_engine/Cargo.toml` — simulation crate, depends on `bf_types` + `rand`
- [x] 1.4 Rename `client/` → `game/crates/bf_game/`, rename `backend/` → `multiplayer_server/`, update Cargo.toml to use workspace deps
- [x] 1.5 Move canonical types from `game/crates/bf_game/src/state.rs` into `bf_types/src/state.rs`
- [x] 1.6 Move hex math from `game/crates/bf_game/src/renderer.rs` into `bf_types/src/hex.rs`
- [x] 1.7 `bf_game/src/state.rs` becomes `pub use bf_types::*;` re-export
- [x] 1.8 Verify both `cargo check -p bf_game` (native) and `wasm-pack build game/crates/bf_game` still work

## 2. Port hex utilities and pathfinding

- [x] 2.1 Port `hex.ts` → `bf_types/src/hex.rs` (neighbors, cube coords, distance, hexes_in_radius)
- [x] 2.2 Port `pathfinding.ts` → `bf_engine/src/pathfinding.rs` (A* with terrain movement costs)
- [x] 2.3 Write unit tests matching TypeScript behavior (known paths, blocked tiles, cost weighting)

## 3. Port game state and stat tables

- [x] 3.1 Port `state.ts` unit stats (cost, HP, speed, range, damage, vision) to `bf_types/src/state.rs`
- [x] 3.2 Port `GameState` struct with command queue, units, buildings, resources, players, terrain, visibility into `bf_engine`
- [x] 3.3 Port helper methods: `is_blocked()`, `get_movement_cost()`, `get_player_base()`
- [x] 3.4 Port `StateDiff` generation (snapshot before/after tick, diff the two)

## 4. Port command execution and tick resolution

- [x] 4.1 Define `Command` enum in `bf_types/src/commands.rs` (spawn_unit, move_unit, attack_target, harvest)
- [x] 4.2 Port `executeCommand()` → `bf_engine/src/commands.rs` — spawn deducts energy, move computes A*, attack sets target, harvest starts worker
- [x] 4.3 Port `resolveMovement()` — advance units along paths up to speed per tick
- [x] 4.4 Port `resolveCombat()` — simultaneous damage, range check via hex distance, unit death
- [x] 4.5 Port `resolveHarvesting()` — harvest energy from nodes, auto-return to base, deposit cargo
- [x] 4.6 Write integration tests: spawn → move → attack → harvest sequences

## 5. Port fog of war

- [x] 5.1 Port `fog.ts` → `bf_engine/src/fog.rs` — per-player visibility from units and buildings
- [x] 5.2 Downgrade visible → previously_seen when out of range
- [x] 5.3 Test visibility computation against known unit positions

## 6. Port map generation

- [x] 6.1 Port `maps.ts` → `bf_engine/src/maps.rs` — 32x32 hex maps with biome terrain
- [x] 6.2 Port start position calculation (symmetric for 2-8 players)
- [x] 6.3 Port resource node placement (central + per-player nearby)
- [ ] 6.4 Use `rand::SmallRng` with seed for deterministic generation
- [x] 6.5 Port `MAP_CONFIGS` presets (1v1, 2v2, ffa-4, ffa-8)

## 7. Port bot AI

- [x] 7.1 Port `bot_ai.ts` → `bf_engine/src/bot_ai.rs` — autopilot command generation
- [x] 7.2 Port spawn priority logic (workers → scout → soldiers → more workers)
- [x] 7.3 Port worker orders (harvest nearest resource)
- [x] 7.4 Port scout orders (patrol toward enemy base)
- [x] 7.5 Port soldier orders (attack nearby enemies or push toward enemy base)

## 8. Engine orchestration

- [x] 8.1 Create `bf_engine/src/lib.rs` — `GameEngine` struct with `new()`, `queue_command()`, `tick()`, `state()`, `full_snapshot()`
- [x] 8.2 Port tick loop order: autopilot → drain commands → execute → movement → combat → harvesting → fog → win check → generate diff
- [x] 8.3 Port win conditions: last base standing, max ticks scoring
- [x] 8.4 Write full match integration test: create match → tick to completion → verify winner

## 9. Refactor bf_game into plugin architecture

- [x] 9.1 Create `bf_game/src/game.rs` — root `GamePlugin` composing sub-plugins, define `AppState` enum and `GameSystems` system sets
- [x] 9.2 Extract `bf_game/src/camera.rs` — `CameraPlugin` (setup, orbit, zoom)
- [x] 9.3 Extract `bf_game/src/map.rs` — `MapPlugin` (terrain spawning, fog sync)
- [x] 9.4 Extract `bf_game/src/units.rs` — `UnitsPlugin` (unit/building/resource entity sync)
- [x] 9.5 Extract `bf_game/src/hud.rs` — `HudPlugin` (tooltip, overlays, tick counter)
- [x] 9.6 Extract `bf_game/src/input.rs` — `InputPlugin` (mouse/keyboard handling)
- [x] 9.7 Update `lib.rs` (WASM) and `main.rs` (native) to use `GamePlugin`

## 10. Bevy local match integration

- [x] 10.1 Create `bf_game/src/local_match.rs` — `LocalMatchPlugin` with `FixedUpdate` tick system at 100ms
- [x] 10.2 Add `LocalEngine` resource wrapping `GameEngine`
- [x] 10.3 Implement `tick_engine` system in `GameSystems::Simulation` set
- [x] 10.4 Implement `sync_engine_to_view` system — copies engine state to renderer's `GameStateView`
- [x] 10.5 Feature-gate: `LocalMatchPlugin` only included with `native` feature

## 11. Native binary entry point

- [x] 11.1 Create `bf_game/src/main.rs` — boots Bevy app with `GamePlugin` + `LocalMatchPlugin`
- [ ] 11.2 Add match setup UI or CLI args (opponent: bot, map preset, player count)
- [x] 11.3 Gate `web-sys`, `wasm-bindgen` behind `#[cfg(target_arch = "wasm32")]`
- [x] 11.4 Replace `web_sys::console::log_1` calls with Bevy's `info!`/`warn!`/`error!` macros (work on both targets)
- [ ] 11.5 Verify `cargo run -p bf_game` launches a playable local match against the bot AI

## 12. Local MCP agent support

- [ ] 12.1 Create `bf_engine/src/mcp.rs` — stdio MCP server exposing game tools
- [ ] 12.2 Implement tools: `get_game_state`, `spawn_unit`, `move_unit`, `attack_target`, `harvest`
- [ ] 12.3 Native binary accepts `--agent <command>` flag to spawn an agent process with stdio MCP
- [ ] 12.4 Agent process connects via stdin/stdout, sends tool calls, receives results
- [ ] 12.5 Test with existing Claude/GPT agent harnesses from `agents/`

## 13. Multiplayer client (engine connects to multiplayer server)

- [ ] 13.1 Add `bf_game/src/network.rs` — `NetworkPlugin` for WebSocket state receiver (existing WASM bridge pattern, ported to native)
- [ ] 13.2 HTTP client: auth (`POST /api/auth/login`), lobby (`GET/POST /api/matches`), join (`POST /api/matches/{id}/join`)
- [ ] 13.3 WebSocket client: connect to `/api/matches/{id}/spectate`, receive state diffs, feed to `GameStateView`
- [ ] 13.4 Command submission: send player commands via `POST /api/mcp` (same endpoint agents use)
- [ ] 13.5 Host mode: local engine ticks, sends diffs upstream to multiplayer server for relay to other players
- [ ] 13.6 Mode selection in native binary: "Local Play" (embedded engine) vs "Online Play" (connect to multiplayer server)
- [ ] 13.7 Feature-gate: multiplayer networking behind `online` feature (requires `reqwest`, `tokio-tungstenite`)

## 14. Cleanup and verification

- [ ] 14.1 Run full match: native binary, bot AI opponent, verify game plays to completion
- [ ] 14.2 Run full match: native binary, MCP agent opponent, verify agent can play
- [ ] 14.3 Run full match: native binary, online mode, connect to AdonisJS multiplayer server
- [ ] 14.4 Verify WASM build still works (`wasm-pack build crates/bf_game --no-default-features --features wasm`)
- [ ] 14.5 Update `CLAUDE.md` with new workspace layout, build commands, and `crates/` structure
- [ ] 14.6 Update `openspec/specs/game-engine/spec.md` to note Rust implementation
- [ ] 14.7 Update `openspec/specs/game-client/spec.md` to add local/offline play, plugin architecture, and multiplayer client capabilities
