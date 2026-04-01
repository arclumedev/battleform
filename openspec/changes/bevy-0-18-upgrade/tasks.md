## 1. Cargo Configuration

- [ ] 1.1 Bump `bevy` version from `"0.16"` to `"0.18"` in `client/Cargo.toml`
- [ ] 1.2 Add `"mouse"` and `"keyboard"` to Bevy feature flags
- [ ] 1.3 Add `getrandom = { version = "0.3", features = ["wasm_js"] }` dependency
- [ ] 1.4 Create `client/.cargo/config.toml` with `[target.wasm32-unknown-unknown] rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]`

## 2. API Migration

- [ ] 2.1 Rename `EventReader<MouseWheel>` to `MessageReader<MouseWheel>` in `renderer.rs` camera_controls
- [ ] 2.2 Update `use bevy::render::camera::ScalingMode` import path if needed (try compile, fix if broken)
- [ ] 2.3 Run `cargo check --target wasm32-unknown-unknown` and fix any remaining compile errors

## 3. Validation

- [ ] 3.1 Run `cargo test` — all state/serde tests pass
- [ ] 3.2 Run `cargo clippy --target wasm32-unknown-unknown` — no warnings
- [ ] 3.3 Run `wasm-pack build --dev --target web --out-dir ../frontend/public/pkg` — WASM builds successfully
- [ ] 3.4 Manual test: load WASM in browser, verify terrain renders, camera controls work (pan, zoom, keyboard)
