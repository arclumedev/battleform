## ADDED Requirements

### Requirement: BRP HTTP endpoint
The native game client SHALL expose a Bevy Remote Protocol HTTP endpoint when built with the `brp` feature.

#### Scenario: BRP endpoint available
- **WHEN** the native game client starts with the `brp` feature enabled
- **THEN** a JSON-RPC HTTP endpoint is available on port 15702

#### Scenario: BRP disabled by default
- **WHEN** the native game client starts with only the `native` feature
- **THEN** no BRP HTTP endpoint is exposed

### Requirement: Entity inspection
The BRP endpoint SHALL support querying ECS entities and their components.

#### Scenario: Query all entities with a component
- **WHEN** a `world_query` request is sent for entities with the `TerrainTile` component
- **THEN** the response contains all terrain tile entities with their component data

#### Scenario: Get components for a specific entity
- **WHEN** a `world_get_components` request is sent with an entity ID
- **THEN** the response contains all components attached to that entity

### Requirement: State mutation
The BRP endpoint SHALL support spawning, despawning, and mutating entities and resources.

#### Scenario: Mutate a resource
- **WHEN** a `world_mutate_resources` request modifies the `BevyGameState` resource
- **THEN** the game state is updated and the renderer reflects the change on the next frame

#### Scenario: Spawn an entity
- **WHEN** a `world_spawn_entity` request creates a new entity with `Transform` and `Mesh3d` components
- **THEN** the entity appears in the scene

### Requirement: Screenshot capture
The BRP endpoint SHALL support capturing screenshots of the current frame.

#### Scenario: Take screenshot
- **WHEN** a `brp_extras_screenshot` request is sent
- **THEN** a PNG screenshot of the current viewport is returned

### Requirement: Input simulation
The BRP endpoint SHALL support simulating keyboard and mouse input.

#### Scenario: Simulate keyboard input
- **WHEN** a `brp_extras_send_keys` request sends a WASD key press
- **THEN** the camera moves as if the key was physically pressed

#### Scenario: Simulate mouse scroll
- **WHEN** a `brp_extras_scroll_mouse` request sends a scroll event
- **THEN** the camera zoom changes

### Requirement: MCP server configuration
The project SHALL include an MCP server configuration for `bevy_brp_mcp` so Claude Code can interact with the running game.

#### Scenario: MCP tools available
- **WHEN** a developer opens Claude Code in the Battleform project with a BRP-enabled game client running
- **THEN** Claude Code has access to BRP tools (world_query, world_spawn_entity, brp_extras_screenshot, etc.)
