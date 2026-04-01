## Why

Square grids look generic and create diagonal movement artifacts. Hexagonal grids are the standard for strategy games — they provide uniform distance to all neighbors (no diagonal shortcut), look more natural, and give the game a distinctive visual identity.

## What Changes

- **Rendering** — tiles drawn as hexagons instead of squares, using flat-top hex layout with offset coordinates
- **Coordinate system** — offset hex coordinates (odd-r) replace square grid (x,y). Hex-to-pixel and pixel-to-hex conversion added.
- **Pathfinding** — A* updated from 4 neighbors to 6 neighbors per hex
- **Movement/range** — all distance calculations use hex distance instead of Manhattan distance
- **Fog of war** — vision computed using hex radius instead of square radius
- **Map generation** — start positions and resource placement adapted for hex grid

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `game-engine`: pathfinding uses 6 hex neighbors, distance calculations use hex distance, fog uses hex radius
- `game-client`: tiles rendered as hexagons with hex-to-pixel positioning

## Impact

- `backend/app/engine/pathfinding.ts` — 6 neighbors, hex distance heuristic
- `backend/app/engine/fog.ts` — hex radius vision
- `backend/app/engine/state.ts` — hex distance utility, coordinate helpers
- `backend/app/engine/commands.ts` — range checks use hex distance
- `backend/app/engine/maps.ts` — hex-aware start positions and resources
- `client/src/renderer.rs` — hex tile rendering, hex-to-pixel positioning
- `client/src/state.rs` — no change (coordinates are still x,y integers)
- Backend tests updated for hex neighbors/distance
