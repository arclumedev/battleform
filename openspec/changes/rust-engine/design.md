## Context

Battleform's game engine is ~1400 lines of TypeScript across 8 files in `backend/app/engine/`. The logic is pure game rules with no inherent web/database dependencies. Porting to Rust allows the game to run as a standalone native binary and embeds the engine directly in the Bevy renderer process.

## Goals / Non-Goals

**Goals:**
- Port all game logic from TypeScript to Rust with identical behavior
- Run a complete match (with bot AI) in a single native binary, no network
- Share types between engine and renderer (no duplicate struct definitions)
- Support local MCP agent connections via stdio
- The Rust binary is the game — it connects to AdonisJS as a client for multiplayer, not the other way around

**Non-Goals:**
- Rewriting the AdonisJS backend in Rust (it stays as the multiplayer service layer)
- Changing game rules, balance, or mechanics (this is a 1:1 port)
- Full multiplayer netcode (initial scope is local play; multiplayer connects to existing backend)
- Replacing the browser WASM spectator mode (it still works via the JS bridge)

## Decisions

### Workspace layout

Following Bevy ecosystem conventions (bevy_game_template, bevy_new_2d): use a Cargo workspace with `resolver = "2"`, separate crates for types/simulation/client, and workspace-level dependency declarations and profiles.

Three Rust crates, using a `crates/` directory to keep the repo root clean:

```
battleform/
├── Cargo.toml                (workspace root)
├── rust-toolchain.toml
├── crates/
│   ├── bf_types/             (shared types — no Bevy dependency)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        (re-exports)
│   │       ├── state.rs      (GameStateView, StateDiff, enums)
│   │       ├── commands.rs   (Command enum, player config)
│   │       └── hex.rs        (hex math — pure functions)
│   ├── bf_engine/            (game simulation — no Bevy, no IO)
│   │   ├── Cargo.toml        (depends on bf_types)
│   │   └── src/
│   │       ├── lib.rs        (GameEngine struct, tick loop)
│   │       ├── commands.rs   (command execution + resolution)
│   │       ├── pathfinding.rs
│   │       ├── fog.rs
│   │       ├── maps.rs
│   │       ├── bot_ai.rs
│   │       └── mcp.rs        (stdio MCP server)
│   └── bf_client/            (Bevy app — renderer + integration)
│       ├── Cargo.toml        (depends on bf_types, optionally bf_engine)
│       └── src/
│           ├── main.rs       (native binary entry point)
│           ├── lib.rs        (WASM entry point + shared Bevy setup)
│           ├── game.rs       (root GamePlugin)
│           ├── camera.rs     (CameraPlugin — setup, orbit, zoom)
│           ├── map.rs        (MapPlugin — terrain spawning)
│           ├── units.rs      (UnitsPlugin — unit/building/resource sync)
│           ├── hud.rs        (HudPlugin — tooltip, overlays)
│           ├── input.rs      (InputPlugin — mouse/keyboard handling)
│           ├── local_match.rs (LocalMatchPlugin — embedded engine)
│           └── network.rs    (NetworkPlugin — WebSocket state receiver)
├── backend/                  (unchanged — multiplayer service)
└── frontend/                 (unchanged — browser shell)
```

**Why three crates instead of two:**

- `bf_types` has zero Bevy dependency — just `serde` + pure Rust. It can be used by the backend (via WASM or FFI), agents, tests, and tooling without pulling in Bevy's compile times. Changing a type recompiles only this tiny crate + dependents.
- `bf_engine` depends on `bf_types` but not Bevy. It's a headless simulation — pure game logic. Can be compiled to WASM independently for the backend to use, or run in tests without a GPU.
- `bf_client` depends on both and adds the Bevy rendering layer. This is what compiles to the WASM spectator or the native binary.

**Workspace Cargo.toml:**
```toml
[workspace]
resolver = "2"
members = ["crates/bf_types", "crates/bf_engine", "crates/bf_client"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rmp-serde = "1"
rand = { version = "0.9", features = ["small_rng"] }
bevy = { version = "0.16", default-features = false }

# Fast dev builds: optimize dependencies but not our code
[profile.dev]
opt-level = 1
[profile.dev.package."*"]
opt-level = 3

# WASM release profile
[profile.wasm-release]
inherits = "release"
opt-level = "s"
strip = "debuginfo"

[profile.release]
codegen-units = 1
lto = "thin"
```

### Engine API

The engine crate exposes a simple synchronous API. No async, no networking, no IO. Pure game logic.

```rust
pub struct GameEngine {
    state: GameState,
    config: MatchConfig,
    tick_count: u32,
    finished: bool,
}

impl GameEngine {
    pub fn new(config: MatchConfig) -> Self;
    pub fn queue_command(&mut self, player: u8, cmd: Command);
    pub fn tick(&mut self) -> TickResult;
    pub fn state(&self) -> &GameState;
    pub fn full_snapshot(&self) -> GameStateView;
    pub fn is_finished(&self) -> bool;
    pub fn winner(&self) -> Option<u8>;
}

pub struct TickResult {
    pub diff: StateDiff,
    pub events: Vec<GameEvent>,
}

pub struct MatchConfig {
    pub map_preset: MapPreset,
    pub players: Vec<PlayerConfig>,
    pub max_ticks: u32,      // default 2000
    pub tick_rate_ms: u32,   // default 100
}

pub struct PlayerConfig {
    pub slot: u8,
    pub kind: PlayerKind,    // Human, Bot, McpAgent
    pub name: String,
}
```

### Bevy plugin architecture

Following the Bevy convention of one plugin per domain, the client is organized as a root `GamePlugin` that composes sub-plugins. Each plugin owns its systems, components, and resources.

```rust
// game.rs — root plugin
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        app.add_plugins((
            camera::CameraPlugin,
            map::MapPlugin,
            units::UnitsPlugin,
            hud::HudPlugin,
            input::InputPlugin,
        ));

        // Mode-specific plugins (only one active at a time)
        #[cfg(feature = "native")]
        app.add_plugins(local_match::LocalMatchPlugin);

        #[cfg(feature = "wasm")]
        app.add_plugins(network::NetworkPlugin);
    }
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    PostGame,
}
```

**System scheduling:** `FixedUpdate` for simulation ticks (deterministic, 100ms). `Update` for rendering, input, and UI. System ordering via `SystemSet` enums:

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystems {
    Simulation,     // FixedUpdate: engine tick, state sync
    Input,          // Update: mouse/keyboard handling
    EntitySync,     // Update: spawn/update/despawn ECS entities
    Render,         // Update: camera, visual effects, HUD
}

// In GamePlugin::build:
app.configure_sets(Update, (
    GameSystems::Input,
    GameSystems::EntitySync,
    GameSystems::Render,
).chain());
```

### Bevy integration for local play

`LocalMatchPlugin` wraps the engine for offline matches. The engine ticks on `FixedUpdate` (100ms), decoupled from the render frame rate.

```rust
pub struct LocalMatchPlugin;

impl Plugin for LocalMatchPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LocalEngine::default())
           .insert_resource(Time::<Fixed>::from_seconds(0.1))  // 10 ticks/sec
           .add_systems(FixedUpdate, tick_engine.in_set(GameSystems::Simulation))
           .add_systems(Update, sync_engine_to_view
               .in_set(GameSystems::EntitySync)
               .run_if(in_state(AppState::InGame)));
    }
}

#[derive(Resource, Default)]
struct LocalEngine(Option<GameEngine>);

fn tick_engine(mut engine: ResMut<LocalEngine>) {
    if let Some(ref mut e) = engine.0 {
        e.tick();
    }
}

fn sync_engine_to_view(
    engine: Res<LocalEngine>,
    mut view: ResMut<GameStateView>,
) {
    if let Some(ref e) = engine.0 {
        *view = e.full_snapshot();
    }
}
```

### Dual-mode client

The client supports two modes via feature flags, following the Bevy template pattern (bevy_game_template uses features to split native/mobile):

**Native mode** (`cargo run`):
- `main.rs` boots a Bevy app with `GamePlugin` + `LocalMatchPlugin`
- Engine runs in-process, state goes directly to renderer
- No WebSocket, no JS bridge, no web dependencies
- Bot AI and local MCP agents work out of the box
- Uses `dynamic_linking` in dev for fast iteration (~2s incremental vs ~30s)

**WASM mode** (`wasm-pack build --no-default-features --features wasm`):
- `lib.rs` boots the Bevy app as today (wasm_bindgen entry point)
- State arrives via the JS bridge from the backend's WebSocket
- Engine does NOT run in WASM (backend is authoritative for multiplayer)
- Same renderer code (shared `GamePlugin`), different state source

```toml
# crates/bf_client/Cargo.toml
[features]
default = ["native"]
native = [
    "bf_engine",
    "bevy/default",
    "bevy/dynamic_linking",  # fast dev builds
]
wasm = [
    "wasm-bindgen",
    "web-sys",
    "bevy/webgl2",
]
dev = ["bevy/bevy_dev_tools"]

[dependencies]
bf_types = { path = "../bf_types" }
bf_engine = { path = "../bf_engine", optional = true }
bevy = { workspace = true, features = [
    "bevy_asset",
    "bevy_render",
    "bevy_core_pipeline",
    "bevy_pbr",
    "bevy_text",
    "bevy_winit",
    "bevy_color",
    "png",
] }
serde = { workspace = true }
serde_json = { workspace = true }
rmp-serde = { workspace = true }
wasm-bindgen = { version = "0.2", optional = true }
web-sys = { version = "0.3", features = ["console"], optional = true }

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
```

Build commands:
```bash
# Native development (fast iteration)
cargo run -p bf_client

# Native release
cargo build --release -p bf_client

# WASM build
wasm-pack build crates/bf_client --no-default-features --features wasm \
    --target web --out-dir ../../frontend/public/pkg

# WASM optimized
wasm-pack build crates/bf_client --no-default-features --features wasm \
    --target web --out-dir ../../frontend/public/pkg --release
wasm-opt -Oz -o frontend/public/pkg/bf_client_bg.wasm \
    frontend/public/pkg/bf_client_bg.wasm
```

### Multiplayer: engine connects to backend

The Rust binary is the game. AdonisJS is a service it optionally connects to. The dependency flows one direction:

```
Engine (Rust binary) ──connects to──▶ Backend (AdonisJS)
                                         ├── Auth (login, sessions)
                                         ├── Matchmaking (lobby, slots)
                                         └── State relay (WebSocket hub)
```

The backend never imports, calls, or wraps the Rust engine. It's infrastructure — like a game server browser or relay service.

**Two multiplayer modes:**

**Host mode (peer-relayed):**
One player's engine is authoritative. Their native binary runs the engine and sends state diffs to the backend. The backend relays diffs to other connected players (spectators and opponents). Opponents send commands to the backend, which relays them to the host's engine. Low infrastructure cost, but the host has a latency advantage.

```
Host engine ──state diffs──▶ Backend ──relay──▶ Other players
Other players ──commands──▶ Backend ──relay──▶ Host engine
```

**Dedicated server mode (competitive):**
The backend runs its own engine instance (the existing TS engine, or later a WASM-compiled Rust engine) for fairness. All players send commands to the backend, which ticks its own authoritative engine and broadcasts diffs. This is the current architecture — it stays as-is for ranked/competitive play.

```
All players ──commands──▶ Backend (authoritative engine) ──diffs──▶ All players
```

The native binary doesn't care which mode is active — it sends commands to the backend and receives diffs, same as the WASM browser client does today. The only difference is that in host mode, one client also sends diffs upstream instead of just receiving them.

**Connection flow for multiplayer:**
1. Player launches native binary, chooses "Online Play"
2. Binary calls `POST /api/auth/login` to authenticate
3. Binary calls `GET /api/matches` or `POST /api/matches` for lobby
4. Binary opens WebSocket to `/api/matches/{id}/spectate` for state
5. Binary sends commands via `POST /api/mcp` (same as AI agents do today)
6. When match ends, binary shows results locally — no dependency on backend for the UI

This means the backend API surface doesn't change at all. The native binary is just another client, using the same endpoints as the Vue frontend and MCP agents.

### MCP stdio transport

For local play, AI agents connect to the engine via stdio MCP:

```
agent process ←→ stdio ←→ native binary (engine + MCP server)
```

The engine crate includes an MCP server that exposes the same 5 tools as the backend (`get_game_state`, `spawn_unit`, `move_unit`, `attack_target`, `harvest`). The native binary spawns the agent as a child process and pipes MCP messages over stdin/stdout.

This uses the `rmcp` Rust MCP SDK (or a lightweight custom implementation) rather than the Node.js `@modelcontextprotocol/sdk`.

### Porting strategy: line-by-line

The TypeScript engine is clean, stateless, and functional. Each file maps 1:1 to a Rust module:

| TypeScript file | Lines | Rust module | Notes |
|---|---|---|---|
| `state.ts` | 340 | `bf_types/src/state.rs` | Types + stat tables. Canonical source, no Bevy dep. |
| `hex.ts` | 76 | `bf_types/src/hex.rs` | Hex math. Pure functions, shared by engine + client. |
| `commands.ts` | 293 | `bf_engine/src/commands.rs` | Command execution + tick resolution. Heaviest file. |
| `maps.ts` | 200 | `bf_engine/src/maps.rs` | Map generation. Pseudo-random → use `rand` crate. |
| `bot_ai.ts` | 180 | `bf_engine/src/bot_ai.rs` | Autopilot AI. Straightforward port. |
| `game_engine.ts` | 179 | `bf_engine/src/lib.rs` | Match lifecycle + tick loop. Timer becomes Bevy FixedUpdate. |
| `pathfinding.ts` | 85 | `bf_engine/src/pathfinding.rs` | A* with terrain costs. Standard algorithm. |
| `fog.ts` | 56 | `bf_engine/src/fog.rs` | Visibility. Per-player fog computation. |

Total: ~1409 lines of TypeScript → estimated ~1200-1600 lines of Rust (Rust is slightly more verbose for types, slightly less for logic).

### State serialization compatibility

The engine's `GameStateView` and `StateDiff` are serialized with MessagePack (and JSON fallback) for the WebSocket protocol. The Rust types already have `#[derive(Serialize, Deserialize)]` with `#[serde(rename_all = "camelCase")]` to match the backend's JSON field names. This compatibility is maintained — the same serialization format works whether state comes from the Rust engine or the TypeScript backend.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Behavior drift between Rust engine and TypeScript backend | Write property-based tests that run identical command sequences through both engines and compare state. Eventually deprecate the TS version. |
| Rust engine bugs in combat/pathfinding | Port tests alongside logic. The TypeScript engine's behavior is the reference. |
| MCP stdio complexity | Start simple: single agent, synchronous tool calls. Async multi-agent is a follow-up. |
| WASM bundle size with engine included | Engine is NOT included in WASM builds (feature-gated). WASM clients still use the backend. |
| `rand` crate in map generation | Use `rand` with `SmallRng` and a seed for deterministic maps. Same seed = same map across platforms. |
| Workspace refactor breaks wasm-pack | wasm-pack targets `client/` crate specifically. Engine is a dependency, not the build target. |
