## MODIFIED Requirements

### Requirement: Game loop
The game client SHALL run its own frame loop independently of the server tick rate. The game client SHALL support both WASM (production) and native desktop (development) targets using the same rendering and state code.

#### Scenario: Continuous rendering
- **WHEN** the application starts on either WASM or native target
- **THEN** render the current game state every frame
- **AND** poll for pending state updates from the target-appropriate source (JS bridge on WASM, WebSocket thread on native)

### Requirement: State synchronization
The game client SHALL receive and apply state updates from the data ingestion layer, which is target-dependent (JS-push on WASM, WebSocket on native).

#### Scenario: Full snapshot
- **GIVEN** a MessagePack-encoded full state arrives from the data source
- **WHEN** pushed to the shared pending state queue
- **THEN** deserialize and replace the current game state

#### Scenario: State diff
- **GIVEN** a MessagePack-encoded state diff arrives
- **WHEN** pushed to the shared pending state queue
- **THEN** apply the diff to the current game state (moved, spawned, killed units; energy changes; visibility updates)
