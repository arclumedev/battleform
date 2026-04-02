## Why

We currently have no way to programmatically inspect or test the running Bevy game client. Verifying rendering, entity state, and camera behavior requires manual browser testing. The Bevy Remote Protocol (BRP) — built into Bevy 0.18 — exposes the ECS world over HTTP. Combined with `bevy_brp_mcp`, this gives Claude Code (and other MCP-aware tools) direct access to query entities, mutate game state, take screenshots, simulate input, and verify rendering — all from the development workflow. This turns the native game client into a testable, inspectable system.

## What Changes

- Add `bevy_remote` plugin to the native game client, exposing the ECS world over HTTP (default port 15702)
- Add `bevy_brp_extras` plugin for screenshot capture, keyboard/mouse simulation, and diagnostics
- Configure `bevy_brp_mcp` as an MCP server in the Claude Code project settings, giving Claude direct access to the running game
- Feature-gate BRP behind a `brp` feature flag — disabled for WASM builds (BRP uses HTTP which isn't available in browser WASM), enabled for native dev builds
- Add integration test workflows that use BRP to verify game state (terrain spawned, entities positioned correctly, camera setup)

## Capabilities

### New Capabilities

- `bevy-brp`: Bevy Remote Protocol integration for live ECS inspection, state mutation, input simulation, and screenshot capture on native builds

### Modified Capabilities

- `game-client`: Add requirement that the native game client SHALL expose a BRP endpoint for development tooling when the `brp` feature is enabled

## Impact

- **`game/crates/bf_game/Cargo.toml`**: Add `bevy_remote`, `bevy_brp_extras` dependencies; new `brp` feature flag
- **`game/crates/bf_game/src/main.rs`**: Add `RemotePlugin` and `BrpExtrasPlugin` behind `#[cfg(feature = "brp")]`
- **`.claude/settings.json`** or **`claude_desktop_config.json`**: Register `bevy_brp_mcp` as an MCP server
- **No WASM changes**: BRP is native-only
- **No backend changes**: BRP runs inside the Bevy process, no server involvement
