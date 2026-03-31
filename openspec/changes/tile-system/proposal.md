## Why

The current terrain is uniform dark green hexagons with no visual distinction between tile types. A proper tile system with distinct terrain types (grass, desert, water, mountains) and textures gives the game visual identity and enables terrain-based gameplay mechanics (movement cost, impassable water, height advantage).

## What Changes

- **Tile types** — expand from `open`/`blocked` to `grass`, `desert`, `water_lake`, `water_sea`, `mountain`, `forest`, `snow`
- **Terrain textures** — each tile type gets a hex texture (procedurally generated colored hex with visual detail)
- **3D elevation** — tiles have a height value rendered as layered shadows / vertical offset to simulate elevation
- **bevy_ecs_tilemap** — replace hand-rolled hex mesh spawning with the optimized tilemap crate (batched GPU rendering, built-in hex support)
- **Backend tile generation** — map generator produces terrain type + elevation per hex

## Capabilities

### New Capabilities

_None — extends existing game-engine and game-client capabilities_

### Modified Capabilities

- `game-engine`: TileType enum expanded, map generator produces varied terrain, movement cost per terrain type
- `game-client`: tilemap rendering via bevy_ecs_tilemap with texture atlas, elevation rendering

## Impact

- `backend/app/engine/state.ts` — expand TileType enum
- `backend/app/engine/maps.ts` — terrain generation with biomes
- `backend/app/engine/commands.ts` — movement cost per terrain
- `backend/app/engine/pathfinding.ts` — weighted A* for terrain costs
- `client/Cargo.toml` — add bevy_ecs_tilemap dependency
- `client/src/tiles/` — new module for tile type definitions and texture generation
- `client/src/renderer.rs` — replace manual hex spawning with tilemap
- `client/src/state.rs` — expand TileType enum to match backend
- `client/assets/` — tile texture atlas (procedurally generated or hand-drawn)
