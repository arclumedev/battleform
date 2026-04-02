## Context

Bevy 0.18 includes `bevy_remote`, which implements the Bevy Remote Protocol (BRP) — an HTTP-based JSON-RPC interface to the ECS world. The `bevy_brp_mcp` project (github.com/natepiano/bevy_brp) wraps this in an MCP server, letting Claude Code interact with a running Bevy app: query entities, mutate components, simulate input, take screenshots, and more.

Our native game client (`bf_game` with `native` feature) already runs standalone with a local bot-vs-bot match. Adding BRP to this gives us a fully inspectable, testable game client without manual browser work.

## Goals / Non-Goals

**Goals:**
- Expose the ECS world of the native game client over HTTP via BRP
- Enable Claude Code to query, inspect, and mutate game state in a running native build
- Support screenshot capture for visual verification
- Support keyboard/mouse input simulation for automated interaction testing
- Keep BRP completely out of WASM/production builds

**Non-Goals:**
- Adding BRP to the WASM client (HTTP server can't run in browser WASM)
- Building a custom test framework — BRP + MCP is the test interface
- Production monitoring or remote admin — BRP is a dev-only tool
- Automated CI integration (for now) — this is for local dev workflow

## Decisions

### 1. Feature-gate BRP behind a `brp` feature, enabled by `hot` and optionally by `native`

**Decision:** Add a `brp` feature that pulls in `bevy_remote` and `bevy_brp_extras`. The `hot` feature (used for fast iteration) enables `brp` by default. The `native` feature does not enable it by default but can be combined: `--features native,brp`.

**Alternatives considered:**
- **Always enable for native**: Adds unnecessary compile time when you just want to run the game without inspection.
- **Separate binary**: Overkill; a feature flag is simpler and keeps one binary target.

**Rationale:** Developers doing active iteration (`hot`) almost always want inspection. Developers running `native` for a quick playtest may not.

### 2. Use `bevy_brp_extras` for screenshots and input simulation

**Decision:** Add `bevy_brp_extras` as a dependency alongside `bevy_remote`. This gives us screenshot capture, keyboard/mouse simulation, and diagnostics out of the box.

**Alternatives considered:**
- **Only `bevy_remote` (no extras)**: We'd get entity queries but no screenshots or input sim. Those are the most valuable testing tools.
- **Custom BRP methods**: Unnecessary work when `bevy_brp_extras` already provides what we need.

**Rationale:** The extras plugin is lightweight (one `add_plugins` call) and provides the high-value tools: screenshots for visual verification, input for automated interaction.

### 3. Configure `bevy_brp_mcp` as a project-level MCP server

**Decision:** Register `bevy_brp_mcp` in `.claude/settings.json` so it's available in all Claude Code sessions for this project. The MCP server connects to the BRP HTTP endpoint on the running Bevy app.

**Rationale:** Project-level config means any developer with Claude Code gets BRP access automatically. No global config pollution.

### 4. Default BRP port 15702

**Decision:** Use the default `bevy_brp_extras` port (15702). It can be overridden via `BRP_EXTRAS_PORT` env var.

**Rationale:** No conflicts with our other services (backend 3333, frontend 5173, postgres 5444, redis 6385).

## Risks / Trade-offs

**[Risk] BRP exposes full ECS write access** → Mitigated by feature-gating: BRP is never compiled into WASM or production builds. It's a dev-only tool.

**[Risk] `bevy_brp_mcp` is a third-party crate** → It's MIT/Apache licensed, actively maintained, and pinned to Bevy 0.18.1 (our version). If it becomes unmaintained, `bevy_remote` alone still works for basic JSON-RPC queries.

**[Trade-off] Additional compile time for `brp` feature** → Only affects builds with `brp` enabled. The `wasm` build path is untouched. The `hot` feature already has longer first-build times due to dynamic linking; BRP adds marginal overhead.

**[Trade-off] MCP server is a separate process** → `bevy_brp_mcp` runs as its own process and connects to the Bevy app over HTTP. This is the standard MCP architecture and works well — no in-process complexity.
