## Why

Macroquad's WASM integration model is fragile — it requires `mq_js_bundle.js` for GL context setup, raw pointer passing into linear memory via unsafe C ABI, and global Mutex statics for cross-boundary communication. It has no scene graph, no asset pipeline, and a minimal plugin ecosystem. As the game client grows beyond colored primitives (sprites, particles, animations, tilemaps), every feature requires hand-rolled state management.

Bevy provides an ECS, a real asset pipeline, `wasm-bindgen` integration (eliminating the unsafe bridge), and a large plugin ecosystem. Migrating now — before we invest in post-MVP visuals — avoids a more expensive rewrite later.

## What Changes

- **Full rewrite of `client/` Rust source** — replace Macroquad with Bevy ECS. Entry point becomes a Bevy App with systems instead of a `#[macroquad::main]` loop.
- **Safe WASM bridge** — replace unsafe `extern "C"` pointer functions with `#[wasm_bindgen]` typed functions. Eliminate `mq_js_bundle.js`.
- **Entity reconciliation system** — new ECS pattern to sync external game state (from WebSocket) with Bevy entities each frame.
- **Build pipeline change** — switch from `cargo build` + manual copy to `wasm-pack build --target web` which handles wasm-bindgen and output in one step.
- **Frontend bridge simplification** — `bridge.ts` imports typed wasm-bindgen functions instead of raw memory writes.
- **GameCanvas.vue update** — Bevy targets `#glcanvas` via `WindowPlugin`, removing the macroquad script loader.

## Capabilities

### New Capabilities

_None — this is a rewrite of the existing game-client capability, not a new one._

### Modified Capabilities

- `game-client`: Rendering architecture changes from immediate-mode to ECS. State sync changes from unsafe C ABI to wasm-bindgen. All existing requirements (game loop, state sync, rendering, camera, fog) are preserved with the same behavior but different implementation.

## Impact

- **`client/`** — full rewrite (Cargo.toml, lib.rs, renderer.rs, state.rs)
- **`frontend/src/lib/bridge.ts`** — simplified, type-safe imports
- **`frontend/src/components/GameCanvas.vue`** — minor update (remove mq_js_bundle loading)
- **`frontend/public/pkg/`** — new wasm-pack output replaces macroquad artifacts
- **`.vscode/tasks.json`** — build commands change to wasm-pack
- **`CLAUDE.md`** — update build instructions
- **Backend and game engine are unaffected** — state serialization format unchanged
