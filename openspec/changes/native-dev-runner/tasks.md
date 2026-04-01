## 1. Cargo Configuration

- [ ] 1.1 Add `[[bin]]` target for `battleform-dev` in `client/Cargo.toml` pointing to `src/main.rs`
- [ ] 1.2 Feature-gate `wasm-bindgen` and `web-sys` dependencies behind `target_arch = "wasm32"`
- [ ] 1.3 Add native-only dependencies: `tungstenite`, `clap` (for CLI args), `url`
- [ ] 1.4 Add `[profile.dev-native]` with `bevy/dynamic_linking` feature enabled
- [ ] 1.5 Verify `cargo check --target wasm32-unknown-unknown` still works (WASM path unbroken)

## 2. Conditional Compilation in lib.rs

- [ ] 2.1 Gate `wasm_bindgen` imports and `#[wasm_bindgen]` exports with `#[cfg(target_arch = "wasm32")]`
- [ ] 2.2 Keep `PENDING_DIFFS` and `PENDING_SNAPSHOTS` static Mutexes accessible from both targets
- [ ] 2.3 Extract shared Bevy app builder (plugins, resources, systems) into a `pub fn build_app() -> App` used by both WASM `start()` and native `main()`

## 3. Replace web-sys Logging

- [ ] 3.1 Replace all `web_sys::console::log_1(...)` calls in `renderer.rs` with `info!()` / `warn!()` / `error!()` from `bevy::log`
- [ ] 3.2 Replace all `web_sys::console` calls in `lib.rs` (`drain_pending_updates`) with Bevy log macros
- [ ] 3.3 Remove `web-sys` import from `renderer.rs` (no longer needed)

## 4. Native WebSocket Client

- [ ] 4.1 Create `client/src/native_ws.rs` with a blocking WebSocket client that connects to the backend spectator endpoint
- [ ] 4.2 Implement message receive loop: deserialize MessagePack/JSON and push into `PENDING_SNAPSHOTS` / `PENDING_DIFFS`
- [ ] 4.3 Add reconnection with exponential backoff (1s → 2s → 4s → max 30s) on disconnect
- [ ] 4.4 Support `--url` CLI arg and `BATTLEFORM_WS_URL` env var, defaulting to `ws://localhost:3333/spectator/ws`

## 5. Native Entry Point

- [ ] 5.1 Create `client/src/main.rs` that parses CLI args, spawns the WebSocket thread, and runs the shared Bevy app
- [ ] 5.2 Configure native `WindowPlugin` (no canvas, reasonable default size like 1280x720, title "Battleform Dev")
- [ ] 5.3 Conditionally compile `main.rs` only for non-WASM targets

## 6. Dev Workflow & Documentation

- [ ] 6.1 Add `dev` script to run native build: `cargo run --profile dev-native` (or shell alias)
- [ ] 6.2 Add `cargo-watch` command example: `cargo watch -w src -x 'run --profile dev-native'`
- [ ] 6.3 Update `CLAUDE.md` Local Development section with native runner commands
- [ ] 6.4 Verify full WASM build path still works: `wasm-pack build --dev --target web --out-dir ../frontend/public/pkg`

## 7. Validation

- [ ] 7.1 Run `cargo clippy` for native target — no warnings
- [ ] 7.2 Run `cargo clippy --target wasm32-unknown-unknown` — no warnings
- [ ] 7.3 Run existing `cargo test` — all state/serde tests pass
- [ ] 7.4 Manual test: start backend, run native dev runner, verify terrain + entities render in desktop window
