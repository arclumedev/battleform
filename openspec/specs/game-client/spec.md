# Game Client Specification

## Purpose

Browser-side WASM application (Rust) that renders the game in real time. Owns its own game loop, receives state updates from the backend via WebSocket, handles camera input, and draws all visual elements.

## Requirements

### Requirement: Game loop

The game client SHALL run its own frame loop independently of the server tick rate.

#### Scenario: Continuous rendering

- GIVEN the WASM module is loaded
- WHEN the game loop runs
- THEN render the current game state every frame
- AND poll for pending state updates from JS

### Requirement: State synchronization

The game client SHALL receive and apply state updates pushed from JavaScript.

#### Scenario: Full snapshot

- GIVEN a MessagePack-encoded full state arrives from the WebSocket
- WHEN pushed to the WASM module
- THEN deserialize and replace the current game state

#### Scenario: State diff

- GIVEN a MessagePack-encoded state diff arrives
- WHEN pushed to the WASM module
- THEN apply the diff to the current game state (moved, spawned, killed units; energy changes; visibility updates)

### Requirement: Terrain rendering

The game client SHALL render the map grid with terrain tiles.

#### Scenario: Open grid

- GIVEN a 32x32 map
- WHEN rendering
- THEN draw each tile as a colored rectangle with 1px gap between tiles
- AND use distinct colors for open vs blocked terrain

### Requirement: Unit rendering

The game client SHALL render units with distinct shapes per type and colors per player.

#### Scenario: Unit display

- GIVEN units on the map
- WHEN rendering
- THEN draw workers as circles, soldiers as squares, scouts as triangles
- AND color by player (blue for slot 0, red for slot 1, etc.)

### Requirement: Building rendering

The game client SHALL render buildings as larger colored squares.

#### Scenario: Base display

- GIVEN a base building
- WHEN rendering
- THEN draw a filled square in the player's color with an inner accent

### Requirement: Health bars

The game client SHALL display health bars above damaged entities.

#### Scenario: Damaged unit

- GIVEN a unit with HP < max HP
- WHEN rendering
- THEN draw a colored bar above the unit (green > 60%, yellow > 30%, red otherwise)

### Requirement: Fog of war

The game client SHALL overlay fog on non-visible tiles.

#### Scenario: Fog display

- GIVEN per-tile visibility state
- WHEN rendering
- THEN draw black overlay at 90% alpha for unseen tiles
- AND 50% alpha for previously seen tiles
- AND no overlay for visible tiles

### Requirement: Camera controls

The game client SHALL support pan and zoom via mouse input.

#### Scenario: Pan

- GIVEN the user holds left mouse button and drags
- WHEN the mouse moves
- THEN translate the camera by the drag delta

#### Scenario: Zoom

- GIVEN the user scrolls the mouse wheel
- WHEN the scroll event fires
- THEN scale the camera zoom (clamped between 0.3x and 3x)

### Requirement: Resource rendering

The game client SHALL render resource nodes as yellow diamonds.

#### Scenario: Resource display

- GIVEN resource nodes on the map
- WHEN rendering
- THEN draw a yellow diamond at each node's position
- AND brightness proportional to remaining energy

### Requirement: Combat effects

The game client SHALL display visual feedback for combat events.

#### Scenario: Combat flash

- GIVEN combat events in the current state
- WHEN rendering
- THEN draw a white flash circle at each combat location
