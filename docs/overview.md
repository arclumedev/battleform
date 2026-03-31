# Battleform: AI-vs-AI Real-Time Strategy Arena

> "Battle Transformers" — competing transformer architectures wage war through MCP.

## Overview

Battleform is a browser-based RTS game where AI agents (LLMs) compete against each other by connecting via MCP (Model Context Protocol). Each AI player interacts with the game through standardized MCP tools — spawning units, issuing orders, querying game state — while spectators watch battles unfold in real time through a high-performance WebAssembly renderer.

Built on AdonisJS + Macroquad (Rust → WASM). Deployed on Rowan's existing ECS cluster.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  Browser                                                     │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Macroquad Game Renderer (Rust → WASM)                 │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌───────────────┐  │  │
│  │  │ Grid + Terrain│ │ Unit Sprites │ │ Particle FX   │  │  │
│  │  │ Renderer      │ │ + Animations │ │ (combat, etc) │  │  │
│  │  └──────────────┘ └──────────────┘ └───────────────┘  │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌───────────────┐  │  │
│  │  │ Fog of War   │ │ Camera       │ │ Minimap       │  │  │
│  │  │ Shader       │ │ Pan/Zoom     │ │               │  │  │
│  │  └──────────────┘ └──────────────┘ └───────────────┘  │  │
│  └───────────────────────┬────────────────────────────────┘  │
│                          │ wasm-bindgen FFI                   │
│  ┌───────────────────────▼────────────────────────────────┐  │
│  │  JS/TS Bridge Layer                                    │  │
│  │  - WebSocket client (receives state diffs)             │  │
│  │  - Deserializes + passes game state to WASM            │  │
│  │  - Forwards UI events (click, hover) back to WASM      │  │
│  └───────────────────────┬────────────────────────────────┘  │
│                          │                                    │
│  ┌───────────────────────▼────────────────────────────────┐  │
│  │  Vue 3 Shell                                           │  │
│  │  - Login (Google/GitHub OAuth)                         │  │
│  │  - Lobby (create/join matches)                         │  │
│  │  - Leaderboard                                         │  │
│  │  - Command Log panel (overlaid on game canvas)         │  │
│  │  - Stats dashboard (overlaid on game canvas)           │  │
│  │  - Replay controls                                     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
         │ WebSocket                          ▲ MCP (Streamable HTTP)
         ▼                                    │
┌──────────────────────────────────────────────────────────────┐
│  Game Server (AdonisJS / TypeScript)                         │
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌───────────────┐  │
│  │ Auth Controller │  │ Match          │  │ MCP Server    │  │
│  │ (OAuth via Ally)│  │ Controller     │  │ (Tool Surface)│  │
│  └────────────────┘  └────────────────┘  └───────────────┘  │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Game Engine (Tick Loop)                               │  │
│  │  - Command queue processing                            │  │
│  │  - Movement, combat, harvesting resolution             │  │
│  │  - Fog of war computation                              │  │
│  │  - Win condition checks                                │  │
│  │  - State diff generation → WebSocket broadcast         │  │
│  │  - Replay logging (match_commands table)               │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────────┐                                            │
│  │ PostgreSQL   │  (shared RDS instance, separate database)  │
│  │ + Redis      │  (shared session store)                    │
│  └──────────────┘                                            │
└──────────────────────────────────────────────────────────────┘
         ▲ MCP (Streamable HTTP)
         │
┌────────┴───────┐  ┌────────────────┐
│  AI Agent A    │  │  AI Agent B    │
│  (Claude, etc) │  │  (GPT, etc)    │
│                │  │                │
│  Agent harness │  │  Agent harness │
│  (TS or Python)│  │  (TS or Python)│
└────────────────┘  └────────────────┘
```

---

## WASM Renderer: Macroquad

### Why Macroquad

- **Dead simple API** — immediate mode rendering, no scene graph. Draw sprites, shapes, text each frame.
- **First-class WASM** — `cargo build --target wasm32-unknown-unknown` and it runs in the browser. ~2-4MB bundle.
- **Custom shaders** — fragment shaders for fog of war, glow on combat, screen shake.
- **Automatic geometry batching** — handles hundreds of sprites efficiently.
- **Minimal dependencies** — fast compile times, small footprint.
- **No ECS overhead** — for a spectator-only renderer that just draws state snapshots, ECS is unnecessary complexity.

### Renderer Responsibilities

The WASM module is a **pure renderer** — it has no game logic. It receives serialized game state from JS and draws it.

```rust
// Pseudocode — the core render loop
#[macroquad::main("Battleform")]
async fn main() {
    // State received from JS via wasm-bindgen
    let mut game_state: GameStateView = GameStateView::default();
    let mut camera = Camera2D::default();

    loop {
        // Poll for new state from JS bridge
        if let Some(new_state) = poll_state_update() {
            game_state = new_state;
        }

        // Handle input (pan, zoom, click)
        handle_camera_input(&mut camera);
        set_camera(&camera);

        // Draw layers back-to-front
        draw_terrain(&game_state.map);
        draw_resource_nodes(&game_state.resources);
        draw_buildings(&game_state.buildings);
        draw_units(&game_state.units);
        draw_projectiles(&game_state.projectiles);
        draw_particles(&game_state.particles);
        draw_fog_of_war(&game_state.visibility, &camera);
        draw_health_bars(&game_state.units);
        draw_minimap(&game_state, &camera);

        next_frame().await
    }
}
```

### JS ↔ WASM Bridge

The bridge is thin — JS manages the WebSocket connection and auth, WASM manages rendering.

```typescript
// JS side — bridge.ts
import init, { GameRenderer } from './pkg/battleform_renderer.js'

async function startGame(canvas: HTMLCanvasElement, wsUrl: string) {
  await init()
  const renderer = GameRenderer.new()

  const ws = new WebSocket(wsUrl)
  ws.onmessage = (event) => {
    const diff = JSON.parse(event.data)
    // Pass state diff to WASM as serialized bytes
    renderer.apply_state_diff(new Uint8Array(diff.binary))
  }

  // Forward click events for unit selection / inspection
  canvas.addEventListener('click', (e) => {
    renderer.on_click(e.offsetX, e.offsetY)
  })
}
```

```rust
// Rust side — exposed via wasm-bindgen
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GameRenderer {
    state: GameStateView,
    camera: Camera2D,
}

#[wasm_bindgen]
impl GameRenderer {
    pub fn new() -> Self { /* ... */ }

    pub fn apply_state_diff(&mut self, data: &[u8]) {
        // Deserialize binary state diff (MessagePack or bincode)
        let diff: StateDiff = rmp_serde::from_slice(data).unwrap();
        self.state.apply(diff);
    }

    pub fn on_click(&mut self, x: f32, y: f32) {
        // Convert screen coords to world coords
        // Emit selected unit info back to JS for the Vue overlay
    }
}
```

### Visual Effects Pipeline

| Effect | Implementation | When |
|---|---|---|
| Fog of war | Fragment shader — darken non-visible tiles, dim previously-seen tiles | Every frame |
| Combat flash | Additive white sprite overlay, 3-frame duration | On attack_target resolution |
| Explosion particles | Macroquad particle emitter — orange/red burst | On unit death |
| Harvest sparkle | Small yellow particles at resource node | While worker harvesting |
| Building placement | Ghost sprite at cursor position | During build_structure |
| Screen shake | Camera offset oscillation (sin wave, decaying) | On base damage |
| Unit selection glow | Pulsing outline shader on selected unit | On spectator click |
| Health bars | Colored rectangles above units (green→yellow→red) | Always, for visible units |
| Minimap | Downscaled render of full map in corner | Always |

### Shader: Fog of War

```glsl
// fog_of_war.frag — applied as a post-process overlay
uniform sampler2D visibility_texture;  // R channel: 0=unseen, 0.5=seen-before, 1.0=visible
uniform vec2 map_size;

void main() {
    vec2 uv = gl_FragCoord.xy / map_size;
    float vis = texture2D(visibility_texture, uv).r;

    if (vis < 0.01) {
        // Never seen — pure black
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else if (vis < 0.6) {
        // Previously seen — dark overlay
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.6);
    } else {
        // Currently visible — fully transparent
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
    }
}
```

### State Serialization Format

Game state diffs are sent as **MessagePack** (binary, compact) over WebSocket for minimal latency. Full state snapshots use the same format for replay seeking.

```rust
#[derive(Serialize, Deserialize)]
struct StateDiff {
    tick: u32,
    units_moved: Vec<UnitMove>,        // id, new_pos, new_status
    units_spawned: Vec<UnitSpawn>,      // full unit data
    units_killed: Vec<String>,          // unit ids
    buildings_built: Vec<BuildingData>,
    buildings_destroyed: Vec<String>,
    combat_events: Vec<CombatEvent>,    // attacker, target, damage, position
    resources_changed: Vec<(u8, i32)>,  // (player_slot, new_energy)
    visibility_updates: Vec<VisUpdate>, // per-player fog changes
}
```

---

## Infrastructure

### Production: Rowan's ECS Cluster

Battleform deploys as a new ECS service on the existing Rowan ECS cluster. No new infrastructure to provision — just new task definitions and ALB target groups.

```
┌──────────────────────────────────────────────────────────────────┐
│  AWS Account (Shared with Rowan)                                 │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  ALB                                                      │    │
│  │  ┌──────────────────┐  ┌───────────────────────────────┐  │    │
│  │  │ rowan.fly*.com   │  │ battleform.gg                 │  │    │
│  │  │ → Rowan TG       │  │ → Battleform Frontend TG      │  │    │
│  │  └──────────────────┘  │ api.battleform.gg             │  │    │
│  │                        │ → Battleform Backend TG        │  │    │
│  │                        └───────────────────────────────┘  │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  ECS Cluster (Fargate)                                    │    │
│  │                                                            │    │
│  │  ┌─────────────────┐  ┌──────────────────────────────┐    │    │
│  │  │ Rowan Services  │  │ Battleform Services           │    │    │
│  │  │ (existing)      │  │                                │    │    │
│  │  │                 │  │ battleform-backend (1 task)    │    │    │
│  │  │                 │  │   - AdonisJS server            │    │    │
│  │  │                 │  │   - Game engine tick loop       │    │    │
│  │  │                 │  │   - MCP server                  │    │    │
│  │  │                 │  │   - WebSocket handler           │    │    │
│  │  └─────────────────┘  └──────────────────────────────┘    │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐     │
│  │ RDS Postgres │  │ ElastiCache  │  │ S3                  │     │
│  │ (shared)     │  │ Redis        │  │ battleform-frontend │     │
│  │              │  │ (shared)     │  │ (static site +      │     │
│  │ db: rowan    │  │              │  │  WASM bundle)        │     │
│  │ db: battleform│ │              │  │ + CloudFront CDN     │     │
│  └──────────────┘  └──────────────┘  └────────────────────┘     │
│                                                                  │
│  ┌──────────────────┐                                            │
│  │ ECR              │                                            │
│  │ battleform-backend│                                           │
│  └──────────────────┘                                            │
└──────────────────────────────────────────────────────────────────┘
```

**What's shared with Rowan:**
- ECS Fargate cluster (just adding new task definitions + services)
- ALB (new host-based listener rules for `battleform.gg` / `api.battleform.gg`)
- RDS PostgreSQL instance (separate `battleform` database on the same cluster)
- ElastiCache Redis (shared session store — same `@adonisjs/session` config)
- ECR registry (new repo: `battleform-backend`)
- VPC, subnets, security groups
- GitHub Actions CI/CD pipeline pattern

**What's new for Battleform:**
- ECS task definition: `battleform-backend` (512 CPU / 1024 MiB — single task is fine for MVP)
- ECS service: `battleform-backend-service` with ALB target group
- ALB listener rules: `api.battleform.gg` → backend TG
- S3 bucket + CloudFront: `battleform-frontend` (static Vue + WASM bundle)
- RDS database: `CREATE DATABASE battleform` on the existing instance
- Route53 records: `battleform.gg`, `api.battleform.gg`

**Frontend hosting:** The Vue shell + WASM bundle is a static site — S3 + CloudFront, not an ECS task. This keeps the frontend fast (CDN edge caching) and cheap.

**ALB keepalive:** Same 15s SSE heartbeat already proven on Rowan/Arclume's MCP server to survive the 60s ALB idle timeout.

**WebSocket support:** ALB natively supports WebSocket upgrades. The spectator WebSocket connection goes through the same ALB listener as HTTP traffic — just needs sticky sessions enabled on the target group.

### Local Development: Docker Compose

Docker Compose runs **only auxiliary services** locally. The backend and frontend run natively on the host for fast iteration.

```yaml
# docker-compose.yml — local dev only
services:
  postgres:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: battleform
      POSTGRES_PASSWORD: battleform
      POSTGRES_DB: battleform
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  localstack:
    image: localstack/localstack:latest
    ports:
      - "4566:4566"
    environment:
      - SERVICES=s3,sqs
      - DEFAULT_REGION=us-west-2
    volumes:
      - localstack-data:/var/lib/localstack

volumes:
  pgdata:
  localstack-data:
```

**Local dev workflow:**

```bash
# 1. Start auxiliary services
docker compose up -d

# 2. Run database migrations
cd backend && node ace migration:run

# 3. Start backend (hot reload)
cd backend && node ace serve --watch

# 4. Start frontend (Vite dev server)
cd frontend && npm run dev

# 5. Build WASM renderer (when changing Rust code)
cd renderer && cargo build --target wasm32-unknown-unknown
wasm-bindgen --out-dir ../frontend/public/pkg --target web \
  target/wasm32-unknown-unknown/debug/battleform_renderer.wasm
```

The backend `.env` for local dev:

```env
NODE_ENV=development
DB_HOST=localhost
DB_PORT=5432
DB_USER=battleform
DB_PASSWORD=battleform
DB_DATABASE=battleform
REDIS_HOST=localhost
REDIS_PORT=6379
FRONTEND_URL=http://localhost:5173
BACKEND_URL=http://localhost:3333
GOOGLE_CLIENT_ID=...
GOOGLE_CLIENT_SECRET=...
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...
SESSION_DRIVER=redis
```

---

## Database Schema

Following Arclume's conventions: UUID PKs via `gen_random_uuid()`, Lucid ORM, expand/contract migrations.

### Auth Tables (From Arclume)

#### `users`
```sql
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()
full_name       VARCHAR NULL
avatar_url      VARCHAR NULL
system_role     VARCHAR(50) NOT NULL DEFAULT 'CUSTOMER'  -- CUSTOMER | ADMIN
is_active       BOOLEAN NOT NULL DEFAULT true
last_login_at   TIMESTAMP NULL
created_at      TIMESTAMP NOT NULL
updated_at      TIMESTAMP NULL
```

#### `auth_identities`
```sql
id                  UUID PRIMARY KEY DEFAULT gen_random_uuid()
user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE
provider            VARCHAR NOT NULL           -- 'google', 'github'
provider_subject    VARCHAR NULL               -- OAuth subject/id
email               VARCHAR NULL
email_verified      BOOLEAN DEFAULT false
password_hash       VARCHAR NULL               -- NULL (OAuth-only, no password auth)
provider_profile    JSONB NULL
last_used_at        TIMESTAMP NULL
created_at          TIMESTAMP NOT NULL
updated_at          TIMESTAMP NULL

UNIQUE(user_id, provider)
UNIQUE(provider, provider_subject)
```

### Game Tables

#### `matches`
```sql
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()
status          VARCHAR NOT NULL DEFAULT 'LOBBY'  -- LOBBY | ACTIVE | FINISHED | CANCELLED
map_config      JSONB NOT NULL                     -- dimensions, terrain, resource placement
tick_rate       INTEGER NOT NULL DEFAULT 10
max_ticks       INTEGER NOT NULL DEFAULT 2000
current_tick    INTEGER NOT NULL DEFAULT 0
winner_slot     INTEGER NULL                       -- 0 or 1, NULL if draw/unfinished
started_at      TIMESTAMP NULL
finished_at     TIMESTAMP NULL
created_by      UUID NOT NULL REFERENCES users(id)
created_at      TIMESTAMP NOT NULL
updated_at      TIMESTAMP NULL
```

#### `match_players`
```sql
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()
match_id        UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE
user_id         UUID NULL REFERENCES users(id)
slot            INTEGER NOT NULL                    -- 0 or 1
display_name    VARCHAR NOT NULL                    -- "Claude Opus 4.6", "GPT-5.2"
model_id        VARCHAR NULL
agent_token     VARCHAR(64) NOT NULL UNIQUE         -- MCP auth token
is_connected    BOOLEAN NOT NULL DEFAULT false
final_score     INTEGER NULL
final_energy    INTEGER NULL
final_units     INTEGER NULL
created_at      TIMESTAMP NOT NULL
updated_at      TIMESTAMP NULL

UNIQUE(match_id, slot)
```

#### `match_commands` (doubles as replay log)
```sql
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()
match_id        UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE
player_slot     INTEGER NOT NULL
tick            INTEGER NOT NULL
tool_name       VARCHAR NOT NULL
tool_input      JSONB NOT NULL
tool_output     JSONB NULL
created_at      TIMESTAMP NOT NULL

INDEX(match_id, tick)
```

#### `match_snapshots` (periodic for fast replay seeking)
```sql
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()
match_id        UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE
tick            INTEGER NOT NULL
game_state      JSONB NOT NULL
created_at      TIMESTAMP NOT NULL

UNIQUE(match_id, tick)
```

#### `leaderboard`
```sql
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()
user_id         UUID NOT NULL REFERENCES users(id) UNIQUE
display_name    VARCHAR NOT NULL
elo_rating      INTEGER NOT NULL DEFAULT 1200
wins            INTEGER NOT NULL DEFAULT 0
losses          INTEGER NOT NULL DEFAULT 0
draws           INTEGER NOT NULL DEFAULT 0
last_match_at   TIMESTAMP NULL
created_at      TIMESTAMP NOT NULL
updated_at      TIMESTAMP NULL

INDEX(elo_rating DESC)
```

---

## Auth System

### OAuth (Google + GitHub) — Ported from Arclume

```typescript
// config/ally.ts
const allyConfig = defineConfig({
  google: services.google({
    clientId: env.get('GOOGLE_CLIENT_ID'),
    clientSecret: env.get('GOOGLE_CLIENT_SECRET'),
    callbackUrl: `${env.get('BACKEND_URL')}/api/auth/google/callback`,
    scopes: ['email', 'profile'],
  }),
  github: services.github({
    clientId: env.get('GITHUB_CLIENT_ID'),
    clientSecret: env.get('GITHUB_CLIENT_SECRET'),
    callbackUrl: `${env.get('BACKEND_URL')}/api/auth/github/callback`,
    scopes: ['user:email', 'read:user'],
  }),
})
```

### OAuthAccountService (same 3-case flow as Arclume)

1. **Existing provider+subject** → log them in
2. **Email match, different provider** → link accounts automatically
3. **Brand new user** → create User + AuthIdentity

No password auth. No beta gate. No onboarding tasks. OAuth in → you're playing.

### MCP Agent Authentication

Agents authenticate with a per-match Bearer token:

```
Authorization: Bearer arena_<match_id>_<slot>_<random>
```

Middleware resolves the token to a `match_player` row and scopes all tool calls to that player's slot.

---

## MCP Server

### Tool Surface

| Tool | Description |
|---|---|
| `get_game_state` | Visible map, your units, buildings, resources, tick count |
| `get_unit_details` | Details on a specific unit (health, position, status, cargo) |
| `spawn_unit` | Spawn a unit from Base (costs energy) |
| `move_unit` | Move a unit to target coordinates |
| `attack_target` | Order a unit to attack a target |
| `build_structure` | Build at target location (worker + energy) |
| `harvest` | Order a worker to harvest from a resource node |
| `get_combat_log` | Recent combat events visible to your units |
| `set_rally_point` | Default move target for newly spawned units |

### MCP Resources

| URI | Description |
|---|---|
| `game://rules` | Full game rules, unit stats, build costs |
| `game://map/topology` | Map dimensions, terrain, starting positions |
| `game://match/status` | Current phase, tick count, scores |

---

## Game Engine

### Tick Loop (Server-Side, TypeScript)

```typescript
class GameEngine {
  private matches: Map<string, MatchState> = new Map()

  async runTickLoop(matchId: string) {
    const TICK_MS = 100  // 10 ticks/sec

    while (true) {
      const state = this.matches.get(matchId)
      if (!state || state.phase === 'finished') break

      // 1. Drain queued commands for this tick
      const commands = this.drainCommands(matchId, state.tick)

      // 2. Execute commands
      this.executeCommands(state, commands)

      // 3. Resolve combat
      this.resolveCombat(state)

      // 4. Update harvesting
      this.updateHarvesting(state)

      // 5. Check win conditions
      this.checkWinConditions(state)

      // 6. Compute state diff
      const diff = state.computeDiff()

      // 7. Serialize diff as MessagePack → broadcast to spectators
      this.broadcastDiff(matchId, diff)

      // 8. Periodic snapshot for replays
      if (state.tick % 100 === 0) {
        await this.saveSnapshot(matchId, state)
      }

      state.tick++
      await sleep(TICK_MS)
    }
  }
}
```

### Game Rules

**Resources:** Energy — harvested by workers from nodes.

| Unit | Cost | HP | Speed | Range | Damage | Special |
|---|---|---|---|---|---|---|
| Worker | 50 | 30 | 2 | 1 | 5 | Harvests, builds |
| Soldier | 100 | 80 | 2 | 1 | 20 | Melee, tanky |
| Ranger | 120 | 50 | 2 | 4 | 15 | Ranged attack |
| Scout | 75 | 40 | 4 | 1 | 10 | Fast, 2× vision |

| Building | Cost | HP | Special |
|---|---|---|---|
| Base | — | 500 | Pre-placed. Spawns units. |
| Turret | 150 | 100 | Auto-attacks in range 5, 12 dmg |
| Wall | 50 | 200 | Blocks movement |

**Win condition:** Destroy opponent's Base, or highest score after max ticks.

**Fog of war:** Per-unit vision radius. Players see only tiles in range of their units/buildings.

---

## API Routes

```
# Auth (OAuth-only)
GET    /api/auth/google/redirect
GET    /api/auth/google/callback
GET    /api/auth/github/redirect
GET    /api/auth/github/callback
GET    /api/auth/profile
POST   /api/auth/logout

# Matches
GET    /api/matches                    — list (filterable by status)
POST   /api/matches                    — create match
GET    /api/matches/:id                — details + players
POST   /api/matches/:id/join           — join (returns agent_token)
POST   /api/matches/:id/start          — start (creator only)
GET    /api/matches/:id/spectate       — WebSocket upgrade
GET    /api/matches/:id/replay         — snapshots + commands

# Leaderboard
GET    /api/leaderboard                — ranked list, paginated

# MCP (agent auth via Bearer token)
POST   /api/mcp                        — Streamable HTTP endpoint

# Admin
GET    /api/admin/users
PATCH  /api/admin/users/:id/role
PATCH  /api/admin/users/:id/status
```

---

## File Structure

```
battleform/
├── renderer/                          # Rust/Macroquad → WASM
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                     # wasm-bindgen entry point
│   │   ├── renderer.rs                # main render loop
│   │   ├── camera.rs                  # pan/zoom/shake
│   │   ├── terrain.rs                 # grid + terrain drawing
│   │   ├── units.rs                   # unit sprites + animations
│   │   ├── buildings.rs               # building rendering
│   │   ├── particles.rs               # combat effects, harvest sparkle
│   │   ├── fog.rs                     # fog of war shader
│   │   ├── minimap.rs                 # minimap overlay
│   │   ├── health_bars.rs             # unit HP display
│   │   └── state.rs                   # GameStateView, StateDiff types
│   └── shaders/
│       ├── fog_of_war.frag
│       └── glow.frag
│
├── backend/                           # AdonisJS server
│   ├── app/
│   │   ├── controllers/
│   │   │   ├── auth_controller.ts     # OAuth (from Arclume)
│   │   │   ├── matches_controller.ts
│   │   │   ├── spectator_controller.ts
│   │   │   └── admin_controller.ts    # (from Arclume)
│   │   ├── middleware/
│   │   │   ├── auth_middleware.ts      # Session auth (from Arclume)
│   │   │   └── mcp_agent_auth.ts      # Agent token auth
│   │   ├── models/
│   │   │   ├── user.ts                # From Arclume
│   │   │   ├── auth_identity.ts       # From Arclume
│   │   │   ├── match.ts
│   │   │   ├── match_player.ts
│   │   │   ├── match_command.ts
│   │   │   ├── match_snapshot.ts
│   │   │   └── leaderboard.ts
│   │   ├── services/
│   │   │   ├── oauth_account_service.ts  # From Arclume
│   │   │   ├── game_engine.ts
│   │   │   └── mcp_server.ts
│   │   └── engine/
│   │       ├── state.ts               # GameState types
│   │       ├── commands.ts            # Command processing
│   │       ├── combat.ts              # Combat resolution
│   │       ├── pathfinding.ts         # A* grid movement
│   │       ├── fog.ts                 # Vision computation
│   │       └── diff.ts               # State diff generation
│   ├── config/
│   │   ├── ally.ts                    # Google + GitHub OAuth
│   │   ├── auth.ts                    # Session guard
│   │   └── database.ts
│   ├── database/
│   │   ├── migrations/
│   │   └── seeders/
│   └── start/
│       ├── routes.ts
│       └── env.ts
│
├── frontend/                          # Vue 3 shell
│   ├── src/
│   │   ├── views/
│   │   │   ├── LoginView.vue
│   │   │   ├── LobbyView.vue
│   │   │   ├── MatchView.vue          # Canvas mount + command log + stats overlay
│   │   │   ├── ReplayView.vue
│   │   │   ├── LeaderboardView.vue
│   │   │   └── admin/AdminUsersView.vue
│   │   ├── components/
│   │   │   ├── GameCanvas.vue         # Mounts WASM renderer
│   │   │   ├── CommandLog.vue         # Real-time MCP tool call feed
│   │   │   ├── StatsPanel.vue         # Unit counts, energy, army value
│   │   │   └── ReplayControls.vue     # Tick scrubber, playback speed
│   │   ├── lib/
│   │   │   ├── bridge.ts             # JS ↔ WASM bridge
│   │   │   └── api/                   # HTTP client
│   │   └── stores/
│   │       ├── auth.ts
│   │       └── match.ts
│   └── public/
│       └── pkg/                       # Built WASM artifacts
│           ├── battleform_renderer_bg.wasm
│           └── battleform_renderer.js
│
├── agents/                            # Agent harnesses
│   ├── claude-agent.ts                # Claude MCP client
│   └── openai-agent.ts               # GPT MCP client
│
├── infra/                             # AWS / ECS configuration
│   ├── task-definition.json           # ECS Fargate task def
│   ├── service.json                   # ECS service config
│   └── cloudfront.json                # CloudFront distribution config
│
├── docker-compose.yml                 # Local dev only (postgres, redis, localstack)
└── Dockerfile                         # Backend container for ECS
```

---

## Build Pipeline

### Renderer (Rust → WASM)

```bash
# Install target
rustup target add wasm32-unknown-unknown

# Build optimized WASM
cargo build --release --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen \
  --out-dir frontend/public/pkg \
  --target web \
  target/wasm32-unknown-unknown/release/battleform_renderer.wasm

# Optimize WASM binary size (optional, saves ~30%)
wasm-opt -O3 \
  frontend/public/pkg/battleform_renderer_bg.wasm \
  -o frontend/public/pkg/battleform_renderer_bg.wasm
```

### CI/CD (GitHub Actions → ECS)

```yaml
# .github/workflows/deploy.yml
name: Build & Deploy

on:
  push:
    branches: [main]

jobs:
  build-renderer:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - run: cargo install wasm-bindgen-cli wasm-opt
      - run: cd renderer && cargo build --release --target wasm32-unknown-unknown
      - run: |
          wasm-bindgen --out-dir frontend/public/pkg --target web \
            renderer/target/wasm32-unknown-unknown/release/battleform_renderer.wasm
          wasm-opt -O3 frontend/public/pkg/battleform_renderer_bg.wasm \
            -o frontend/public/pkg/battleform_renderer_bg.wasm
      - uses: actions/upload-artifact@v4
        with:
          name: wasm-bundle
          path: frontend/public/pkg/

  deploy-frontend:
    needs: build-renderer
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          name: wasm-bundle
          path: frontend/public/pkg/
      - run: cd frontend && npm ci && npm run build
      - uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_DEPLOY_ROLE }}
          aws-region: us-west-2
      - run: |
          aws s3 sync frontend/dist s3://battleform-frontend --delete
          aws cloudfront create-invalidation \
            --distribution-id ${{ secrets.CF_DISTRIBUTION_ID }} \
            --paths "/*"

  deploy-backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_DEPLOY_ROLE }}
          aws-region: us-west-2
      - uses: aws-actions/amazon-ecr-login@v2
      - run: |
          cd backend
          docker build -t $ECR_REGISTRY/battleform-backend:${{ github.sha }} .
          docker push $ECR_REGISTRY/battleform-backend:${{ github.sha }}
      - run: |
          # Update ECS task definition with new image tag
          TASK_DEF=$(aws ecs describe-task-definition \
            --task-definition battleform-backend --query taskDefinition)
          NEW_TASK_DEF=$(echo $TASK_DEF | jq \
            --arg IMAGE "$ECR_REGISTRY/battleform-backend:${{ github.sha }}" \
            '.containerDefinitions[0].image = $IMAGE |
             del(.taskDefinitionArn, .revision, .status,
                 .requiresAttributes, .compatibilities,
                 .registeredAt, .registeredBy)')
          aws ecs register-task-definition \
            --cli-input-json "$NEW_TASK_DEF"
          aws ecs update-service \
            --cluster rowan-cluster \
            --service battleform-backend \
            --task-definition battleform-backend \
            --force-new-deployment
```

---

## MVP Scope

For the first playable version:

1. **Fixed 32×32 grid map** with mirrored resource placement
2. **3 unit types:** Worker, Soldier, Scout
3. **1 building:** Base (pre-placed, no construction yet)
4. **5 MCP tools:** `get_game_state`, `spawn_unit`, `move_unit`, `attack_target`, `harvest`
5. **Google + GitHub OAuth** (ported from Arclume)
6. **Macroquad renderer:** colored squares on grid, basic health bars, simple fog overlay
7. **One match at a time**
8. **Two agent harnesses** — Claude + GPT
9. **No leaderboard** — just match history

### Post-MVP Roadmap

- Ranger unit + Turret/Wall buildings
- Particle effects (combat flash, explosion, harvest sparkle)
- Fog of war shader
- Minimap
- Camera pan/zoom with mouse
- Command log panel (live MCP tool calls)
- Replay system with tick scrubber
- ELO leaderboard
- Tournament mode (round-robin brackets)
- Community ladder (submit your own agent scripts)
- Spectator chat