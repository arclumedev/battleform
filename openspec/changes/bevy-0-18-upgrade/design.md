## Context

The `client/` crate is a Bevy 0.16 WASM game renderer. It uses a narrow slice of the Bevy API: 3D rendering (meshes, materials, lights, camera), input (keyboard, mouse, scroll), and the ECS (systems, resources, components). The codebase is ~500 lines across 3 files. The upgrade path from 0.16 → 0.18 crosses two major versions (0.17, 0.18), but the breaking changes that intersect our API usage are minimal.

## Goals / Non-Goals

**Goals:**
- Upgrade to Bevy 0.18 with zero rendering or behavioral regressions
- Maintain WASM build compatibility (`wasm-pack build --target web`)
- Keep all existing tests passing

**Non-Goals:**
- Adopting new 0.17/0.18 features (e.g., new UI system, asset processor) — that's future work
- Skipping 0.17 changes — we jump straight to 0.18 but must apply both sets of migrations

## Decisions

### 1. Jump directly to 0.18 (skip 0.17 as intermediate)

**Decision:** Upgrade from 0.16 → 0.18 in one step, applying both migration guides simultaneously.

**Alternatives considered:**
- **Two-step upgrade (0.16 → 0.17 → 0.18)**: Safer in large codebases, but our API surface is small and the changes don't conflict. Two steps would double the compile/test cycles for no practical benefit.

**Rationale:** Only ~4 breaking changes affect us total. Applying them all at once is simpler and faster.

### 2. Use `.cargo/config.toml` for getrandom WASM config

**Decision:** Add a `.cargo/config.toml` with `rustflags` for the `wasm32-unknown-unknown` target rather than requiring a `RUSTFLAGS` environment variable.

**Alternatives considered:**
- **Environment variable**: Works but is easy to forget, especially for new contributors.
- **Build script**: Overkill for a single cfg flag.

**Rationale:** Checked into the repo, works automatically for everyone.

### 3. Add input feature flags explicitly

**Decision:** Add `"mouse"` and `"keyboard"` to the Bevy features list. Do not add other new opt-in features we don't use.

**Rationale:** 0.18 made input sources opt-in. We use both mouse and keyboard. Adding only what we need keeps compile times down.

## Risks / Trade-offs

**[Risk] `ScalingMode` import path changed** → Mitigation: Try compilation first; if `bevy::render::camera::ScalingMode` fails, switch to `bevy::camera::ScalingMode`. Both are easy to verify.

**[Risk] Subtle runtime behavior changes in event/message system** → Mitigation: The rename from `EventReader` to `MessageReader` is mechanical. The underlying semantics (buffered, read-once) are preserved. Manual testing of scroll zoom confirms behavior.

**[Risk] `getrandom` WASM config breaks native builds** → Not a real risk: the `.cargo/config.toml` rustflags are scoped to `[target.wasm32-unknown-unknown]` only.

**[Trade-off] Pinning to 0.18 rather than 0.18.x** → We use `"0.18"` in Cargo.toml which allows patch updates. This is fine; Bevy follows semver within patch versions.
