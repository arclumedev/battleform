## MODIFIED Requirements

### Requirement: Game loop

The game client SHALL run a Bevy ECS application with systems scheduled in the Update set.

#### Scenario: Continuous rendering

- **WHEN** the WASM module loads via `#[wasm_bindgen(start)]`
- **THEN** start a Bevy App with `DefaultPlugins` (minimal feature set)
- **AND** target the `#glcanvas` canvas element via `WindowPlugin`
- **AND** run Update systems each frame to drain state, reconcile entities, and render

### Requirement: State synchronization

The game client SHALL receive state updates via `#[wasm_bindgen]` typed functions instead of unsafe C ABI.

#### Scenario: Full snapshot

- **WHEN** JS calls `push_full_state(data: &[u8])` via wasm-bindgen
- **THEN** queue the bytes in a `Mutex<Vec<Vec<u8>>>` static
- **AND** a Bevy system drains the queue and replaces the `GameStateView` Resource

#### Scenario: State diff

- **WHEN** JS calls `push_state_diff(data: &[u8])` via wasm-bindgen
- **THEN** queue the bytes in a `Mutex<Vec<Vec<u8>>>` static
- **AND** a Bevy system drains the queue and applies the diff to the `GameStateView` Resource

## ADDED Requirements

### Requirement: Entity reconciliation

The game client SHALL maintain a mapping between game state IDs and Bevy entities.

#### Scenario: New entity

- **WHEN** a unit/building/resource ID appears in `GameStateView` but not in the `EntityMap`
- **THEN** spawn a Bevy entity with the appropriate components and sprite
- **AND** insert the mapping into `EntityMap`

#### Scenario: Updated entity

- **WHEN** an existing entity's position or state changes
- **THEN** update the entity's `Transform` and components to match

#### Scenario: Removed entity

- **WHEN** an ID exists in `EntityMap` but not in `GameStateView`
- **THEN** despawn the Bevy entity
- **AND** remove it from `EntityMap`

### Requirement: ECS rendering

The game client SHALL use Bevy systems and `SpriteBundle` components for all rendering instead of immediate-mode draw calls.

#### Scenario: Unit rendering

- **WHEN** units exist in the game state
- **THEN** render each as a colored sprite entity positioned by `Transform`
- **AND** color by player slot, shape/size by unit type

#### Scenario: Fog rendering

- **WHEN** visibility state changes for tiles
- **THEN** update fog overlay sprite alpha (0.9 unseen, 0.5 previously seen, 0.0 visible)

### Requirement: Build pipeline

The game client SHALL build via `wasm-pack` instead of manual `cargo build` + copy.

#### Scenario: Development build

- **WHEN** `wasm-pack build --target web --out-dir ../frontend/public/pkg` is run
- **THEN** produce `battleform_renderer.js` (wasm-bindgen glue) and `battleform_renderer_bg.wasm`
- **AND** the frontend can import typed functions directly
