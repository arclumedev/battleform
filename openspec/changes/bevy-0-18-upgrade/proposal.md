## Why

The game client uses Bevy 0.16, which is two major versions behind the current stable (0.18). Upgrading now keeps us on a supported version, avoids accumulating migration debt, and is a prerequisite for the native dev runner change (which benefits from 0.18's improved dynamic linking and input handling). The migration is small — only 3–4 breaking changes affect our codebase.

## What Changes

- Bump `bevy` dependency from `0.16` to `0.18`
- Rename `EventReader<MouseWheel>` → `MessageReader<MouseWheel>` (0.17 event→message rename)
- Add `"mouse"` and `"keyboard"` feature flags (0.18 input features are now opt-in)
- Add `getrandom` WASM configuration for `wasm32-unknown-unknown` target (0.17 requirement)
- Update `bevy::render::camera::ScalingMode` import path if it moved to `bevy::camera`
- Verify all existing rendering, input, and state sync code compiles cleanly

## Capabilities

### New Capabilities

_(none — this is a dependency upgrade, not a new capability)_

### Modified Capabilities

- `game-client`: Update Bevy version constraint from 0.16 to 0.18; adapt to renamed event/message APIs

## Impact

- **`client/Cargo.toml`**: Version bump, new feature flags, new `getrandom` dependency
- **`client/.cargo/config.toml`**: New rustflags for WASM `getrandom` backend
- **`client/src/renderer.rs`**: `EventReader` → `MessageReader` rename (1 line)
- **`client/src/renderer.rs`**: Possibly update `ScalingMode` import path
- **`client/src/lib.rs`**: No changes expected
- **`client/src/state.rs`**: No changes expected
- **No backend or frontend changes**
