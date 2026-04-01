## ADDED Requirements

### Requirement: Native desktop window
The native dev runner SHALL launch a Bevy application in a native desktop window rendering the same game state as the WASM client.

#### Scenario: Launch native runner
- **WHEN** developer runs `cargo run --profile dev-native`
- **THEN** a desktop window opens displaying the Bevy renderer with the default clear color and camera setup

### Requirement: WebSocket connection to backend
The native dev runner SHALL connect to the backend spectator WebSocket endpoint to receive game state updates.

#### Scenario: Connect to default endpoint
- **WHEN** the native runner starts without explicit URL configuration
- **THEN** it connects to `ws://localhost:3333/spectator/ws`

#### Scenario: Connect to custom endpoint
- **WHEN** the native runner starts with `--url ws://custom:3333/spectator/ws` or `BATTLEFORM_WS_URL` set
- **THEN** it connects to the specified WebSocket URL

#### Scenario: Receive full snapshot
- **WHEN** a MessagePack-encoded full state arrives over the WebSocket
- **THEN** the runner deserializes and replaces the current game state

#### Scenario: Receive state diff
- **WHEN** a MessagePack-encoded state diff arrives over the WebSocket
- **THEN** the runner applies the diff to the current game state

### Requirement: Shared rendering code
The native dev runner SHALL use the same renderer, state types, and camera controls as the WASM client.

#### Scenario: Identical rendering
- **WHEN** the native runner and WASM client receive the same game state snapshot
- **THEN** both render the same terrain, units, buildings, and resources using shared code in `renderer.rs` and `state.rs`

### Requirement: Dynamic linking for fast recompiles
The native dev runner SHALL support Bevy's dynamic linking feature to reduce recompile times during development.

#### Scenario: Fast rebuild cycle
- **WHEN** a developer modifies `renderer.rs` and rebuilds with the dev-native profile
- **THEN** the incremental recompile completes in under 10 seconds

### Requirement: Cross-platform logging
The native dev runner SHALL use Bevy's built-in logging macros that work on both WASM and native targets.

#### Scenario: Log output on native
- **WHEN** the native runner logs a message via `info!()`, `warn!()`, or `error!()`
- **THEN** the message appears in the terminal's stdout/stderr

### Requirement: Reconnection on disconnect
The native dev runner SHALL attempt to reconnect when the WebSocket connection drops.

#### Scenario: Backend restarts
- **WHEN** the WebSocket connection is lost
- **THEN** the runner retries connection with exponential backoff (1s, 2s, 4s, up to 30s)
- **AND** displays a log message indicating reconnection attempts
