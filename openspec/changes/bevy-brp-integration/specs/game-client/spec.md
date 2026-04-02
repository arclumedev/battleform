## MODIFIED Requirements

### Requirement: Game loop
The game client SHALL run its own frame loop independently of the server tick rate. When built with the `brp` feature, the game client SHALL additionally expose a BRP HTTP endpoint for external inspection and control.

#### Scenario: Continuous rendering with BRP
- **WHEN** the native game client starts with the `brp` feature enabled
- **THEN** render the current game state every frame
- **AND** accept BRP JSON-RPC requests on port 15702
- **AND** process BRP commands (queries, mutations, screenshots) within the ECS schedule
