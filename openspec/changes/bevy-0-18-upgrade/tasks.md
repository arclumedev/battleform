## 1. Cargo Configuration

- [ ] 1.1 Bump `bevy` version from `"0.16"` to `"0.18"` in `game/Cargo.toml` (workspace root)
- [ ] 1.2 Add `"mouse"` and `"keyboard"` to Bevy feature flags in `game/crates/bf_game/Cargo.toml`
- [ ] 1.3 Verify `getrandom` WASM config already in place (bf_game already has it)
- [ ] 1.4 Verify `game/.cargo/config.toml` exists with getrandom rustflags for wasm32 target

## 2. API Migration

- [ ] 2.1 Rename `EventReader<MouseWheel>` to `MessageReader<MouseWheel>` in `game/crates/bf_game/src/camera.rs`
- [ ] 2.2 Update `use bevy::render::camera::ScalingMode` import path if needed (try compile, fix if broken)
- [ ] 2.3 Run `cargo check -p bf_game --no-default-features --features wasm --target wasm32-unknown-unknown` and fix any remaining compile errors
- [ ] 2.4 Run `cargo check -p bf_game --no-default-features --features native` and fix any native compile errors

## 3. Validation

- [ ] 3.1 Run `cargo test` — all tests pass
- [ ] 3.2 Run `cargo clippy --target wasm32-unknown-unknown -p bf_game --no-default-features --features wasm` — no warnings
- [ ] 3.3 Run `wasm-pack build` for WASM — builds successfully
- [ ] 3.4 Manual test: load WASM in browser, verify terrain renders, camera controls work (pan, zoom, keyboard)
