## 1. Backend: expand tile types and terrain generation

- [ ] 1.1 Expand `TileType` enum: `Grass`, `Desert`, `Forest`, `Mountain`, `WaterLake`, `WaterSea`, `Snow`
- [ ] 1.2 Add `elevation: u8` field to terrain data (0-3)
- [ ] 1.3 Update map generator to produce varied biomes (Perlin noise or zone-based)
- [ ] 1.4 Update pathfinding with terrain movement costs (weighted A*)
- [ ] 1.5 Update state serialization to include tile type + elevation
- [ ] 1.6 Update tests for new tile types

## 2. Client: workspace split for faster compilation

- [ ] 2.1 Convert `client/` to a Cargo workspace with `types` and `app` crates
- [ ] 2.2 Move components, resources, state types, tile enums to `types` crate
- [ ] 2.3 Move systems, renderer, lib.rs to `app` crate (depends on `types`)
- [ ] 2.4 Update wasm-pack build to target `app` crate
- [ ] 2.5 Verify WASM build and incremental compile times

## 3. Client: tile type definitions and texture atlas

- [ ] 3.1 Define `TileKind` enum in types crate matching backend `TileType`
- [ ] 3.2 Create procedural texture atlas generator (7 hex tiles, each 64x64)
- [ ] 3.3 Map `TileKind` to texture atlas indices
- [ ] 3.4 Define elevation visual properties (Y offset, shadow color)

## 4. Client: bevy_ecs_tilemap integration

- [ ] 4.1 Add `bevy_ecs_tilemap` 0.16 dependency
- [ ] 4.2 Replace manual terrain hex spawning with `TilemapBundle`
- [ ] 4.3 Configure hex grid layout (odd-r offset, matching backend)
- [ ] 4.4 Apply texture atlas to tilemap
- [ ] 4.5 Add elevation rendering (tile Y offset + shadow layer)
- [ ] 4.6 Update fog of war to work with tilemap (per-tile tint)
- [ ] 4.7 Update camera to work with tilemap coordinate system

## 5. Cleanup and verification

- [ ] 5.1 Remove old manual hex mesh terrain spawning code
- [ ] 5.2 Update isometric camera transform for tilemap coordinates
- [ ] 5.3 Verify full pipeline: backend generates varied terrain → client renders textured hex tilemap
- [ ] 5.4 Update OpenSpec specs for new tile types
