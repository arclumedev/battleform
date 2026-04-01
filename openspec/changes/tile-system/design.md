## Context

Current terrain is a flat enum (`open`/`blocked`) with uniform dark green hex meshes. The game needs visual terrain variety and terrain-based gameplay. The Bevy game client currently hand-rolls hex meshes which is inefficient and inflexible.

## Goals / Non-Goals

**Goals:**
- Define 7+ tile types with distinct visual appearance
- Use `bevy_ecs_tilemap` for optimized hex tilemap rendering
- Add tile elevation for 3D-like visual depth
- Support terrain movement costs in the game engine
- Split client crate into `battleform-types` (components/resources) and `battleform-client` (systems) for faster iterative compilation

**Non-Goals:**
- Animated tile textures (deferred)
- Runtime terrain modification (deferred)
- Hand-drawn art assets (use procedural/colored hexes for now)

## Decisions

**Use `bevy_ecs_tilemap` 0.16.** It provides batched GPU tilemap rendering, native hex support (row/column offset), and per-tile ECS components. This replaces our manual hex mesh spawning with a production-grade solution.

**Split into workspace crates.** Following the compile-time optimization pattern: components/resources go in `battleform-types` and systems go in `battleform-client`. When iterating on rendering systems, only the systems crate recompiles (~5s vs ~30s). Layout:
```
client/
├── Cargo.toml (workspace)
├── types/
│   ├── Cargo.toml
│   └── src/lib.rs (components, resources, tile enums, state types)
└── app/
    ├── Cargo.toml
    └── src/ (lib.rs, renderer.rs, systems)
```

**Tile type as enum, not trait objects.** Bevy ECS works best with concrete types. Each tile gets a `TileType` component (enum) rather than a trait hierarchy. The enum maps to texture atlas indices. The backend and client share the same enum variants via serde.

**Procedural texture atlas.** Generate a 7-tile hex texture atlas at startup using Bevy's `Image` API — each tile type is a colored hexagon with subtle pattern (grass has dots, desert has stipple, water has waves, mountain has triangle, etc). This avoids shipping image assets and keeps the WASM bundle small. Can replace with hand-drawn art later.

**Elevation as visual offset.** Tiles have an integer elevation (0-3). Higher tiles render with a Y offset and a shadow underneath. Mountains are elevation 3, hills are 2, plains are 1, water is 0. This creates a 2.5D layered look without actual 3D geometry.

**Weighted pathfinding.** A* edge costs vary by destination tile type:
- Grass: 1.0 (default)
- Desert: 1.5 (slow)
- Forest: 1.5 (slow, but provides cover)
- Mountain: impassable (like blocked)
- Water: impassable (unless unit has water movement)
- Snow: 2.0 (very slow)

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| `bevy_ecs_tilemap` WASM compat | Confirmed: crate supports WASM, hex grids |
| Workspace split breaks wasm-pack | wasm-pack targets the `app` crate, types crate is a dependency |
| Procedural textures look bad | Keep them simple (solid color + subtle pattern). Replace with art later. |
| Tile elevation Z-fighting | Use distinct Z layers per elevation level (0.0, 0.5, 1.0, 1.5) |
