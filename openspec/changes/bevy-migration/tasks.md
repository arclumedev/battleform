## 1. Bevy scaffolding

- [ ] 1.1 Update `client/Cargo.toml`: remove macroquad, add bevy (0.16, minimal features) + wasm-bindgen
- [ ] 1.2 Install wasm-pack: `cargo install wasm-pack`
- [ ] 1.3 Create new `client/src/lib.rs` with Bevy App entry point (`#[wasm_bindgen(start)]`)
- [ ] 1.4 Configure `WindowPlugin` to target `#glcanvas` canvas
- [ ] 1.5 Verify `wasm-pack build --target web` produces output and shows a colored background in the browser

## 2. Bridge and state

- [ ] 2.1 Replace unsafe `extern "C"` functions with `#[wasm_bindgen]` typed `push_state_diff` and `push_full_state`
- [ ] 2.2 Keep `Mutex<Vec<Vec<u8>>>` statics for cross-boundary state
- [ ] 2.3 Add `Resource` derive to `GameStateView`
- [ ] 2.4 Create `drain_pending_updates` Bevy system that reads statics → updates `Res<GameStateView>`
- [ ] 2.5 Update `frontend/src/lib/bridge.ts` to import wasm-bindgen typed functions
- [ ] 2.6 Remove `mq_js_bundle.js` from `frontend/public/pkg/`
- [ ] 2.7 Update `GameCanvas.vue` to remove macroquad script loading

## 3. Entity reconciliation

- [ ] 3.1 Create `EntityMap` resource with `HashMap<String, Entity>` for units, buildings, resources
- [ ] 3.2 Create `sync_entities` system: spawn/update/despawn entities based on `GameStateView` vs `EntityMap`
- [ ] 3.3 Define ECS components: `Unit`, `Building`, `ResourceNode`, `FogTile`, `HealthBar`

## 4. Rendering systems

- [ ] 4.1 Create `setup_camera` startup system with `Camera2d`
- [ ] 4.2 Create terrain rendering system: spawn `SpriteBundle` entities for grid tiles
- [ ] 4.3 Create unit rendering system: colored sprites by type/player
- [ ] 4.4 Create building rendering system: larger colored sprites
- [ ] 4.5 Create resource rendering system: yellow diamond sprites
- [ ] 4.6 Create fog overlay system: alpha-blended sprites per tile
- [ ] 4.7 Create health bar system: bar sprites above damaged entities
- [ ] 4.8 Create combat flash system: transient white sprites at combat locations
- [ ] 4.9 Create HUD system: tick counter text

## 5. Camera controls

- [ ] 5.1 Create `camera_controls` system: pan with mouse drag, zoom with scroll
- [ ] 5.2 Clamp zoom between 0.3x and 3x via `OrthographicProjection::scale`

## 6. Cleanup and verification

- [ ] 6.1 Delete old macroquad `renderer.rs` and macroquad-specific code
- [ ] 6.2 Update `.vscode/tasks.json` build commands to use wasm-pack
- [ ] 6.3 Update `CLAUDE.md` build instructions
- [ ] 6.4 Update `rust-toolchain.toml` if Bevy requires specific Rust version
- [ ] 6.5 Run `cargo clippy --all-targets -- -D warnings` clean
- [ ] 6.6 Verify full flow: backend → WebSocket → JS bridge → WASM → Bevy renders game state
- [ ] 6.7 Update `openspec/specs/game-client/spec.md` with Bevy architecture
