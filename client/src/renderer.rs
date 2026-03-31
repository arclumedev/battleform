use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::state::*;
use crate::EntityMap;

const TILE_SIZE: f32 = 20.0;

const PLAYER_COLORS: [Color; 8] = [
    Color::srgb(0.23, 0.51, 0.96),
    Color::srgb(0.94, 0.27, 0.27),
    Color::srgb(0.20, 0.83, 0.60),
    Color::srgb(0.95, 0.75, 0.18),
    Color::srgb(0.73, 0.33, 0.83),
    Color::srgb(0.98, 0.50, 0.20),
    Color::srgb(0.20, 0.80, 0.80),
    Color::srgb(0.80, 0.80, 0.80),
];

fn player_color(slot: u8) -> Color {
    PLAYER_COLORS
        .get(slot as usize)
        .copied()
        .unwrap_or(Color::WHITE)
}

#[derive(Component)]
pub struct UnitMarker(#[allow(dead_code)] pub String);

#[derive(Component)]
pub struct BuildingMarker(#[allow(dead_code)] pub String);

#[derive(Component)]
pub struct ResourceMarker(#[allow(dead_code)] pub String);

#[derive(Component)]
pub struct FogTile;

#[derive(Component)]
pub struct TerrainTile;

pub fn setup_camera(mut commands: Commands) {
    let center_x = 16.0 * TILE_SIZE;
    let center_y = -16.0 * TILE_SIZE;
    commands.spawn((
        Camera2d,
        Transform::from_xyz(center_x, center_y, 999.0),
    ));
}

pub fn sync_entities(
    mut commands: Commands,
    state: Res<GameStateView>,
    mut entity_map: ResMut<EntityMap>,
) {
    if state.map_width == 0 || state.map_height == 0 {
        return;
    }

    if !entity_map.terrain_spawned {
        spawn_terrain(&mut commands, &state, &mut entity_map);
    }

    sync_units(&mut commands, &state, &mut entity_map);
    sync_buildings(&mut commands, &state, &mut entity_map);
    sync_resources(&mut commands, &state, &mut entity_map);
    sync_fog(&mut commands, &state, &mut entity_map);
}

fn spawn_terrain(commands: &mut Commands, state: &GameStateView, entity_map: &mut EntityMap) {
    for y in 0..state.map_height {
        for x in 0..state.map_width {
            let color = state
                .terrain
                .get(y as usize)
                .and_then(|row| row.get(x as usize))
                .map(|tile| match tile {
                    TileType::Blocked => Color::srgb(0.3, 0.25, 0.2),
                    TileType::Open => Color::srgb(0.15, 0.2, 0.15),
                })
                .unwrap_or(Color::srgb(0.15, 0.2, 0.15));

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE - 1.0, TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32 * TILE_SIZE), 0.0),
                TerrainTile,
            ));
        }
    }
    entity_map.terrain_spawned = true;
}

fn sync_units(commands: &mut Commands, state: &GameStateView, entity_map: &mut EntityMap) {
    let mut seen = std::collections::HashSet::new();

    for unit in &state.units {
        seen.insert(unit.id.clone());
        let size = match unit.unit_type {
            UnitType::Worker => Vec2::splat(TILE_SIZE * 0.5),
            UnitType::Soldier => Vec2::splat(TILE_SIZE * 0.8),
            UnitType::Scout => Vec2::splat(TILE_SIZE * 0.6),
        };

        if let Some(&entity) = entity_map.units.get(&unit.id) {
            commands.entity(entity).insert((
                Transform::from_xyz(unit.x as f32 * TILE_SIZE, -(unit.y as f32 * TILE_SIZE), 2.0),
                Sprite {
                    color: player_color(unit.player_slot),
                    custom_size: Some(size),
                    ..default()
                },
            ));
        } else {
            let entity = commands
                .spawn((
                    Sprite {
                        color: player_color(unit.player_slot),
                        custom_size: Some(size),
                        ..default()
                    },
                    Transform::from_xyz(
                        unit.x as f32 * TILE_SIZE,
                        -(unit.y as f32 * TILE_SIZE),
                        2.0,
                    ),
                    UnitMarker(unit.id.clone()),
                ))
                .id();
            entity_map.units.insert(unit.id.clone(), entity);
        }
    }

    entity_map.units.retain(|id, &mut entity| {
        if seen.contains(id) {
            true
        } else {
            commands.entity(entity).despawn();
            false
        }
    });
}

fn sync_buildings(commands: &mut Commands, state: &GameStateView, entity_map: &mut EntityMap) {
    let mut seen = std::collections::HashSet::new();

    for building in &state.buildings {
        seen.insert(building.id.clone());
        let pos = Transform::from_xyz(
            building.x as f32 * TILE_SIZE,
            -(building.y as f32 * TILE_SIZE),
            1.0,
        );
        let sprite = Sprite {
            color: player_color(building.player_slot),
            custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)),
            ..default()
        };

        if let Some(&entity) = entity_map.buildings.get(&building.id) {
            commands.entity(entity).insert((pos, sprite));
        } else {
            let entity = commands
                .spawn((sprite, pos, BuildingMarker(building.id.clone())))
                .id();
            entity_map.buildings.insert(building.id.clone(), entity);
        }
    }

    entity_map.buildings.retain(|id, &mut entity| {
        if seen.contains(id) {
            true
        } else {
            commands.entity(entity).despawn();
            false
        }
    });
}

fn sync_resources(commands: &mut Commands, state: &GameStateView, entity_map: &mut EntityMap) {
    let mut seen = std::collections::HashSet::new();

    for resource in &state.resources {
        seen.insert(resource.id.clone());
        let brightness = (resource.remaining as f32 / 500.0).clamp(0.3, 1.0);
        let color = Color::srgb(brightness, brightness * 0.9, 0.1);
        let pos = Transform::from_xyz(
            resource.x as f32 * TILE_SIZE,
            -(resource.y as f32 * TILE_SIZE),
            1.5,
        );
        let sprite = Sprite {
            color,
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.6)),
            ..default()
        };

        if let Some(&entity) = entity_map.resources.get(&resource.id) {
            commands.entity(entity).insert((pos, sprite));
        } else {
            let entity = commands
                .spawn((sprite, pos, ResourceMarker(resource.id.clone())))
                .id();
            entity_map.resources.insert(resource.id.clone(), entity);
        }
    }

    entity_map.resources.retain(|id, &mut entity| {
        if seen.contains(id) {
            true
        } else {
            commands.entity(entity).despawn();
            false
        }
    });
}

fn sync_fog(commands: &mut Commands, state: &GameStateView, entity_map: &mut EntityMap) {
    let rows = state.map_height as usize;
    let cols = state.map_width as usize;

    let needs_rebuild =
        entity_map.fog_tiles.len() != rows || (rows > 0 && entity_map.fog_tiles[0].len() != cols);

    if needs_rebuild {
        for row in entity_map.fog_tiles.drain(..) {
            for entity in row {
                commands.entity(entity).despawn();
            }
        }

        for y in 0..rows {
            let mut row_entities = Vec::with_capacity(cols);
            for x in 0..cols {
                let alpha = fog_alpha(&state.visibility, x, y);
                let entity = commands
                    .spawn((
                        Sprite {
                            color: Color::srgba(0.0, 0.0, 0.0, alpha),
                            custom_size: Some(Vec2::new(TILE_SIZE - 1.0, TILE_SIZE - 1.0)),
                            ..default()
                        },
                        Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32 * TILE_SIZE), 5.0),
                        FogTile,
                    ))
                    .id();
                row_entities.push(entity);
            }
            entity_map.fog_tiles.push(row_entities);
        }
    } else {
        for (y, row) in entity_map.fog_tiles.iter().enumerate() {
            for (x, &entity) in row.iter().enumerate() {
                let alpha = fog_alpha(&state.visibility, x, y);
                commands.entity(entity).insert(Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, alpha),
                    custom_size: Some(Vec2::new(TILE_SIZE - 1.0, TILE_SIZE - 1.0)),
                    ..default()
                });
            }
        }
    }
}

fn fog_alpha(visibility: &[Vec<VisibilityState>], x: usize, y: usize) -> f32 {
    visibility
        .get(y)
        .and_then(|row| row.get(x))
        .map(|vis| match vis {
            VisibilityState::Unseen => 0.9,
            VisibilityState::PreviouslySeen => 0.5,
            VisibilityState::Visible => 0.0,
        })
        .unwrap_or(0.9)
}

pub fn camera_controls(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Query<&Window>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut transform, mut projection)) = camera.single_mut() else {
        return;
    };

    for event in scroll_events.read() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            let factor = if event.y > 0.0 { 0.9 } else { 1.1 };
            ortho.scale = (ortho.scale * factor).clamp(0.3, 3.0);
        }
    }

    if mouse_button.pressed(MouseButton::Right) {
        if let Some(cursor) = window.cursor_position() {
            let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
            let delta = (cursor - center) * 0.02;
            transform.translation.x += delta.x;
            transform.translation.y -= delta.y;
        }
    }
}
