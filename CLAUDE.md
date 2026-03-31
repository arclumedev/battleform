# CLAUDE.md

## Project

Battleform — AI-vs-AI Real-Time Strategy Arena. LLMs compete via MCP in a browser-based RTS.

## OpenSpec

Uses [OpenSpec](https://openspec.dev) for spec-driven development. Specs live in `openspec/specs/` using the Requirement/Scenario/GIVEN-WHEN-THEN format. Changes go through `openspec/changes/`.

**Slash commands:** `/opsx:propose`, `/opsx:apply`, `/opsx:archive`, `/opsx:explore`

**Specs (8 capabilities, 48 requirements):**
- `auth` — email/password + OAuth login, sessions
- `match-lobby` — match creation, joining, quick play, slot config
- `game-engine` — tick loop, spawning, movement, combat, harvesting, fog, win conditions, autopilot
- `mcp-server` — agent auth, 5 tools, 3 resources
- `spectator` — WebSocket broadcast, snapshots, diffs
- `game-client` — WASM game loop, rendering, camera, state sync
- `frontend` — Vue 3 shell, login, lobby, match view, overlays
- `agents` — Claude + GPT harnesses

**ArcLume** is also connected for story mapping:
- **ArcLume org:** Battleform (ID: `668fbc7b-02e7-4bd2-bd6f-5fb5d90293ac`)
- **Story map:** Battleform MVP (ID: `f756754b-31a8-42a7-a951-0bd7148acc8a`)

When planning or tracking work:
1. Use `/opsx:propose` for new features — creates proposal, design, tasks, and spec deltas
2. Use `/opsx:apply` to implement
3. Sync stories to Linear via CLI (`linear issue create --team BAT`)

## Linear

- **Team:** Battleform (key: `BAT`)
- **Workspace:** battleform
- **URL:** https://linear.app/battleform

**IMPORTANT:** Use the `linear` CLI (not the Linear MCP) for all Linear interactions in this project. The MCP token may not have access to the BAT team. Use commands like:
- `linear issue list --team BAT`
- `linear issue create --team BAT --title "..." --description "..."`
- `linear issue update BAT-123 --state "In Progress"`

## Infrastructure

Use ArcLume MCP tools to access infrastructure. Infrastructure lives in the **Rowan** organization (ID: `d47c76e2-169e-4c1d-b0f2-018d21aba4c1`) in the `infrastructure` folder.

## Tech Stack

- **Backend:** AdonisJS 6 (TypeScript), PostgreSQL, Redis
- **Frontend:** Vue 3 + Vite, WASM bridge layer
- **Renderer:** Macroquad (Rust → WASM via wasm-bindgen)
- **Auth:** OAuth (Google + GitHub) via @adonisjs/ally, session-based
- **Agent Protocol:** MCP (Streamable HTTP), per-match Bearer tokens
- **Deploy:** AWS ECS Fargate (shared Rowan cluster), S3 + CloudFront

## Node Version

Use Node 22 LTS. The repo has an `.nvmrc` — run `nvm use` before working.

## Pre-commit Hooks (Husky)

Husky runs automatically on every commit. **Never skip hooks** (`--no-verify`) during feature development.

The pre-commit hook runs **only for changed services**:
- **backend/** changes → ESLint + `tsc --noEmit`
- **frontend/** changes → ESLint + `vue-tsc --build`
- **client/** changes → `cargo clippy -D warnings`

If a hook fails, fix the issue before committing. Do not bypass.

## Local Development

### Docker Compose ports (non-default to avoid conflicts with Rowan/ArcLume/UpdatePilot)

| Service | Host Port | Container Port |
|---|---|---|
| PostgreSQL | 5444 | 5432 |
| Redis | 6385 | 6379 |
| LocalStack | 4569 | 4566 |

```bash
# 0. Use correct Node version
nvm use

# 1. Start auxiliary services
docker compose up -d

# 2. Run database migrations
cd backend && node ace migration:run

# 3. Start backend (hot reload)
cd backend && node ace serve --watch

# 4. Start frontend (Vite dev server)
cd frontend && npm run dev

# 5. Build WASM game client (when changing Rust code)
cd client && wasm-pack build --dev --target web --out-dir ../frontend/public/pkg
```
