## Why

The Bevy game client currently compiles exclusively to WASM via `wasm-pack`, requiring a full rebuild + browser refresh on every code change (~30s+). Bevy supports dynamic linking for native desktop builds, bringing recompile times down to ~2-5 seconds. Adding a native desktop runner for local development will dramatically speed up the edit-compile-test loop for renderer and game client work.

## What Changes

- Add a native desktop binary target (`client/src/main.rs`) that runs the same Bevy app in a native window instead of a browser canvas
- Conditionally compile the WASM bridge layer (`wasm_bindgen` entry points, `web-sys` logging, JS-push state ingestion) only for the `wasm32` target
- For native builds, replace the JS-push state ingestion with a built-in WebSocket client that connects directly to the backend spectator endpoint
- Add `bevy/dynamic_linking` as a dev-dependency feature for fast native recompiles
- Replace `web_sys::console::log` calls with Bevy's `info!`/`warn!`/`error!` macros behind `#[cfg]` gates (or unconditionally, since `bevy::log` works on both targets)
- Add a `cargo watch` / `cargo run` dev script for auto-rebuild on save
- The existing WASM build path (`wasm-pack build`) remains unchanged for production and integration testing

## Capabilities

### New Capabilities

- `native-dev-runner`: Native desktop development runner for the Bevy game client, enabling fast recompiles via dynamic linking and direct WebSocket connection to the backend

### Modified Capabilities

- `game-client`: Add requirement that the game client SHALL support both WASM (production) and native desktop (development) targets using shared rendering and state code

## Impact

- **`client/` crate**: New binary target, conditional compilation for WASM vs native, new WebSocket client dependency for native builds
- **Dependencies**: Add `tungstenite` or `bevy_tokio_tasks` + `tokio-tungstenite` for native WebSocket, feature-gate `wasm-bindgen` and `web-sys`
- **Dev workflow**: New `cargo run` / `cargo watch` commands documented in CLAUDE.md and README
- **No backend changes**: Native runner connects to the same spectator WebSocket endpoint
- **No frontend changes**: The Vue frontend and WASM build path are unaffected
