## Context

The game client (`client/`) currently uses Macroquad for rendering. It owns its game loop via `#[macroquad::main]`, receives state from JS through unsafe `extern "C"` functions with raw pointer passing, and renders using immediate-mode draw calls. The JS bridge loads `mq_js_bundle.js` at runtime and manually writes bytes into WASM linear memory.

The backend game engine and state serialization format (MessagePack `GameStateView`/`StateDiff`) are stable and unchanged by this migration.

## Goals / Non-Goals

**Goals:**
- Replace Macroquad with Bevy 0.16 for ECS-based rendering
- Eliminate unsafe C ABI bridge with `wasm-bindgen` typed functions
- Maintain all existing visual behavior (grid, units, buildings, health bars, fog, combat flash)
- Simplify the JS↔WASM bridge to typed function imports
- Use `wasm-pack` for a single-command build pipeline
- Keep the `GameStateView`/`StateDiff` serialization format identical

**Non-Goals:**
- Sprite art or texture atlas (keep colored primitives for now)
- Post-MVP visual effects (particles, shaders, screen shake) — deferred, but Bevy makes them easier
- ECS on the backend — game engine stays TypeScript
- Bevy UI widgets — Vue handles all UI overlays

## Decisions

**Bevy 0.16 with minimal features.** Disable unused defaults (`bevy_audio`, `bevy_gltf`, `bevy_scene`, `bevy_ui`) to minimize WASM binary size. Enable only: `bevy_asset`, `bevy_render`, `bevy_core_pipeline`, `bevy_sprite`, `bevy_text`, `bevy_winit`, `bevy_color`, `png`.

**Entity reconciliation via HashMap.** The client receives external state from the backend and must keep ECS entities in sync. Use a `Resource` containing `HashMap<String, Entity>` for units, buildings, and resources. Each frame: spawn new, update existing, despawn removed. No fancy diffing — simple ID-based lookup.

**`#[wasm_bindgen(start)]` entry point.** Bevy's App starts when the WASM module loads. The `WindowPlugin` targets `canvas: Some("#glcanvas")` to render into the existing DOM canvas element.

**Global Mutex statics for cross-boundary state.** Keep `PENDING_DIFFS` and `PENDING_SNAPSHOTS` as `Mutex<Vec<Vec<u8>>>` statics. The `#[wasm_bindgen]` functions push data in, a Bevy system drains them each frame. This pattern is the same as macroquad but with a safe API boundary.

**wasm-pack replaces manual build.** Single command: `wasm-pack build --target web --out-dir ../frontend/public/pkg`. Outputs `.js` glue + `.wasm` binary. Eliminates `mq_js_bundle.js`.

**Incremental migration order.** Execute in steps that keep the client functional at each stage: scaffolding → bridge → state → terrain → units → fog → health bars → combat → camera → cleanup.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|---|---|---|
| WASM binary size increase | ~5-8 MB release vs ~2-4 MB macroquad | Aggressive feature gating, `wasm-opt -O3`, lazy loading |
| Bevy compile times | 30-60s incremental in dev | Use `dynamic_linking` feature in dev, pre-warm CI cache |
| Bevy WASM maturity | Some features don't work in WASM | We only need 2D sprites, text, camera — all well-supported |
| Entity reconciliation complexity | New pattern to maintain | Keep it simple: HashMap lookup, no fancy diffing |
| Bevy version churn | Breaking changes between minors | Pin to 0.16, update deliberately |
| Rust 1.85+ requirement | Bevy 0.16 needs newer Rust | Already on 1.94.1 via rustup |
