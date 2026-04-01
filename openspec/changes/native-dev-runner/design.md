## Context

The Battleform game client is a Bevy 0.16 app compiled to WASM via `wasm-pack`. Game state is pushed from JavaScript (received over WebSocket in the Vue frontend) into the WASM module via `wasm_bindgen` exported functions (`push_full_state`, `push_state_diff`). The renderer, state types, and camera controls live in `client/src/` and are target-agnostic except for `web_sys::console` logging and the `wasm_bindgen` entry points in `lib.rs`.

The WASM compile cycle is slow (~30s+), requiring a full `wasm-pack build` and browser refresh. Bevy's `dynamic_linking` feature enables ~2-5s native recompiles, but requires a native binary target.

## Goals / Non-Goals

**Goals:**
- Sub-5-second recompile cycle for game client changes during development
- Native desktop window running the same Bevy renderer, state, and camera code
- Native runner connects directly to the backend spectator WebSocket (no JS bridge)
- Shared code between WASM and native targets (renderer.rs, state.rs) — write once, run both ways
- Simple `cargo run` or `cargo watch` workflow for developers

**Non-Goals:**
- Native builds for production deployment (WASM remains the production target)
- Feature parity with the full Vue frontend (no lobby, auth, or UI chrome in the native runner)
- Hot-reload without recompile (Bevy doesn't support true hot-reload of Rust code)
- Mobile or other non-desktop native targets

## Decisions

### 1. Conditional compilation via `cfg(target_arch)` rather than separate crates

**Decision:** Use `#[cfg(target_arch = "wasm32")]` and `#[cfg(not(target_arch = "wasm32"))]` to gate WASM-specific code (wasm_bindgen exports, web-sys logging, JS-push ingestion) vs native-specific code (WebSocket client, native entry point) within the same crate.

**Alternatives considered:**
- **Separate workspace crates** (`client-wasm`, `client-native`, `client-core`): Cleaner separation but adds workspace complexity, duplicate Cargo configs, and makes it harder to keep in sync. Overkill for the current codebase size.
- **Feature flags**: More flexible but easy to misconfigure. Target arch is the natural split since WASM vs native is inherently a target distinction.

**Rationale:** The shared code (renderer.rs, state.rs) is already target-agnostic. Only `lib.rs` has WASM-specific code. A `main.rs` for native + cfg gates on `lib.rs` is minimal and keeps everything in one crate.

### 2. `tungstenite` (blocking) for the native WebSocket client

**Decision:** Use `tungstenite` (the synchronous WebSocket library) running on a dedicated thread, pushing received messages into a Bevy `Event` channel.

**Alternatives considered:**
- **`tokio-tungstenite` + `bevy_tokio_tasks`**: Async runtime adds complexity. Bevy's main loop is synchronous; bridging async into Bevy systems requires extra plumbing.
- **`ewebsock`**: Lightweight, works on both native and WASM, but less mature and doesn't support all WebSocket features.
- **`reqwest` polling**: Not WebSocket, would require API changes.

**Rationale:** A simple blocking WebSocket on a background thread is the simplest approach. The thread writes to a `Mutex<Vec<Vec<u8>>>` — the same pattern already used for WASM's `PENDING_SNAPSHOTS`/`PENDING_DIFFS`. This means `drain_pending_updates` works identically on both targets.

### 3. Shared ingestion path via static Mutexes

**Decision:** Keep the existing `PENDING_DIFFS` and `PENDING_SNAPSHOTS` static Mutexes. On WASM, JS pushes data via `wasm_bindgen` exports. On native, the WebSocket thread pushes data into the same Mutexes.

**Rationale:** This is the lowest-friction path. The `drain_pending_updates` Bevy system doesn't change at all. Only the producer side differs by target.

### 4. Use `bevy::log` macros unconditionally

**Decision:** Replace all `web_sys::console::log_1(...)` calls with `info!(...)`, `warn!(...)`, `error!(...)` from `bevy::log`. These macros work on both WASM (via `console_log`) and native (via `tracing`/stdout).

**Rationale:** Eliminates the need for `#[cfg]` on every log call. Bevy already configures the correct log backend per target.

### 5. Dynamic linking via Cargo profile, not default feature

**Decision:** Add a `dev-native` Cargo profile that enables `bevy/dynamic_linking`. The `cargo run` command uses this profile. WASM builds are unaffected.

**Alternatives considered:**
- **Default feature**: Risk of accidentally enabling dynamic linking in WASM builds.
- **CLI flag** (`--features dynamic_linking`): Easy to forget; baking it into a profile is more ergonomic.

**Rationale:** A dedicated profile keeps dev-native concerns isolated and is easy to invoke: `cargo run --profile dev-native`.

### 6. Connection config via CLI args or environment variable

**Decision:** The native runner accepts the backend WebSocket URL via `--url` CLI arg or `BATTLEFORM_WS_URL` env var, defaulting to `ws://localhost:3333/spectator/ws`.

**Rationale:** Simple, no config files needed. Environment variable works well with `.env` files already used by the backend.

## Risks / Trade-offs

**[Risk] Native and WASM rendering diverge** → Mitigated by sharing `renderer.rs` and `state.rs` with zero `#[cfg]` gates. Only the entry point and data ingestion layer differ. CI runs `cargo clippy` for both targets.

**[Risk] Bevy version differences between native and WASM** → Not a real risk since both use the same `Cargo.toml` and Bevy version. The `webgl2` feature is additive and doesn't affect native builds.

**[Risk] WebSocket reconnection / error handling in native runner** → Accept basic reconnection (retry on disconnect with backoff). This is a dev tool, not production infra. Print clear error messages.

**[Trade-off] Dynamic linking adds ~1s to first build** → Acceptable; subsequent rebuilds are 2-5s vs 30s+. Only applies to dev-native profile.

**[Trade-off] Native runner has no UI chrome** → By design. It's a renderer viewport, not a full frontend. Developers use the Vue frontend for lobby/auth flows and the native runner for renderer iteration.
