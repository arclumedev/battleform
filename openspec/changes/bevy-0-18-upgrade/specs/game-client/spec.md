## MODIFIED Requirements

### Requirement: Game loop
The game client SHALL run its own frame loop independently of the server tick rate, built on Bevy 0.18.

#### Scenario: Continuous rendering
- **WHEN** the WASM module is loaded
- **THEN** render the current game state every frame
- **AND** poll for pending state updates from JS

### Requirement: Camera controls
The game client SHALL support pan and zoom via mouse and keyboard input using Bevy 0.18's opt-in input features and message-based event system.

#### Scenario: Pan
- **WHEN** the user holds left mouse button and drags
- **THEN** translate the camera by the drag delta

#### Scenario: Zoom
- **WHEN** the user scrolls the mouse wheel
- **THEN** scale the camera zoom (clamped between min and max bounds)
