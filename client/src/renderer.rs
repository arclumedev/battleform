use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::state::*;
use crate::EntityMap;

const HEX_SIZE: f32 = 12.0;
const HEX_WIDTH: f32 = HEX_SIZE * 1.732; // sqrt(3) * size
const HEX_HEIGHT: f32 = HEX_SIZE * 2.0;
const ISO_Y_SCALE: f32 = 0.5; // Vertical squish (isometric foreshortening)
const ISO_SHEAR: f32 = 0.45; // Horizontal shear (Civ-style angle)

/// Convert hex grid (col, row) to Civ-style isometric pixel coordinates.
/// Applies: 1) standard hex layout, 2) Y squish, 3) horizontal shear.
fn hex_to_pixel(col: i32, row: i32) -> Vec2 {
    let offset = if row % 2 != 0 { HEX_WIDTH * 0.5 } else { 0.0 };
    let flat_x = col as f32 * HEX_WIDTH + offset;
    let flat_y = -(row as f32 * HEX_HEIGHT * 0.75);

    // Isometric transform: squish Y, then shear X based on Y
    let iso_y = flat_y * ISO_Y_SCALE;
    let iso_x = flat_x + flat_y * ISO_SHEAR;
    Vec2::new(iso_x, iso_y)
}

/// Convert world pixel coordinates back to the nearest hex grid (col, row).
fn pixel_to_hex(world: Vec2) -> (i32, i32) {
    // Reverse: un-shear, un-squish, then standard hex lookup
    let iso_y = world.y;
    let flat_y = iso_y / ISO_Y_SCALE;
    let flat_x = world.x - flat_y * ISO_SHEAR;

    let row = (-flat_y / (HEX_HEIGHT * 0.75)).round() as i32;
    let offset = if row % 2 != 0 { HEX_WIDTH * 0.5 } else { 0.0 };
    let col = ((flat_x - offset) / HEX_WIDTH).round() as i32;
    (col, row)
}

/// Public wrapper for JS tooltip queries.
pub fn pixel_to_hex_pub(world_x: f32, world_y: f32) -> (i32, i32) {
    pixel_to_hex(Vec2::new(world_x, world_y))
}

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
    let center = hex_to_pixel(16, 16);
    commands.spawn((
        Camera2d,
        Transform::from_xyz(center.x, center.y, 999.0),
    ));
}

pub fn sync_entities(
    mut commands: Commands,
    state: Res<GameStateView>,
    mut entity_map: ResMut<EntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if state.map_width == 0 || state.map_height == 0 {
        return;
    }

    if !entity_map.terrain_spawned {
        spawn_terrain(
            &mut commands,
            &state,
            &mut entity_map,
            &mut meshes,
            &mut materials,
        );
    }

    sync_units(&mut commands, &state, &mut entity_map);
    sync_buildings(&mut commands, &state, &mut entity_map);
    sync_resources(&mut commands, &state, &mut entity_map);
    sync_fog(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
}

fn spawn_terrain(
    commands: &mut Commands,
    state: &GameStateView,
    entity_map: &mut EntityMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let hex_mesh = meshes.add(RegularPolygon::new(HEX_SIZE - 0.5, 6));

    let open_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.15, 0.2, 0.15)));
    let blocked_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.3, 0.25, 0.2)));

    for y in 0..state.map_height {
        for x in 0..state.map_width {
            let is_blocked = state
                .terrain
                .get(y as usize)
                .and_then(|row| row.get(x as usize))
                .map(|tile| *tile == TileType::Blocked)
                .unwrap_or(false);

            let pos = hex_to_pixel(x as i32, y as i32);
            let mat = if is_blocked {
                blocked_mat.clone()
            } else {
                open_mat.clone()
            };

            commands.spawn((
                Mesh2d(hex_mesh.clone()),
                MeshMaterial2d(mat),
                Transform::from_xyz(pos.x, pos.y, 0.0)
                    .with_scale(Vec3::new(1.0, ISO_Y_SCALE, 1.0)),
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
        let s = match unit.unit_type {
            UnitType::Worker => HEX_SIZE * 0.8,
            UnitType::Soldier => HEX_SIZE * 1.3,
            UnitType::Scout => HEX_SIZE,
        };
        let size = Vec2::new(s, s * ISO_Y_SCALE);
        let p = hex_to_pixel(unit.x, unit.y);

        if let Some(&entity) = entity_map.units.get(&unit.id) {
            commands.entity(entity).insert((
                Transform::from_xyz(p.x, p.y, 2.0),
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
                    Transform::from_xyz(p.x, p.y, 2.0),
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
        let p = hex_to_pixel(building.x, building.y);
        let pos = Transform::from_xyz(p.x, p.y, 1.0);
        let sprite = Sprite {
            color: player_color(building.player_slot),
            custom_size: Some(Vec2::new(HEX_SIZE * 1.6, HEX_SIZE * 1.6 * ISO_Y_SCALE)),
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
        let p = hex_to_pixel(resource.x, resource.y);
        let pos = Transform::from_xyz(p.x, p.y, 1.5);
        let sprite = Sprite {
            color,
            custom_size: Some(Vec2::new(HEX_SIZE * 0.8, HEX_SIZE * 0.8 * ISO_Y_SCALE)),
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

fn sync_fog(
    commands: &mut Commands,
    state: &GameStateView,
    entity_map: &mut EntityMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
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

        let hex_mesh = meshes.add(RegularPolygon::new(HEX_SIZE - 0.3, 6));

        for y in 0..rows {
            let mut row_entities = Vec::with_capacity(cols);
            for x in 0..cols {
                let alpha = fog_alpha(&state.visibility, x, y);
                let p = hex_to_pixel(x as i32, y as i32);
                let mat = materials.add(ColorMaterial::from_color(Color::srgba(0.0, 0.0, 0.0, alpha)));
                let entity = commands
                    .spawn((
                        Mesh2d(hex_mesh.clone()),
                        MeshMaterial2d(mat),
                        Transform::from_xyz(p.x, p.y, 5.0)
                            .with_scale(Vec3::new(1.0, ISO_Y_SCALE, 1.0)),
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
                let mat = materials.add(ColorMaterial::from_color(Color::srgba(0.0, 0.0, 0.0, alpha)));
                commands.entity(entity).insert(MeshMaterial2d(mat));
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

/// Each frame, push the camera state into the global so JS can use it for tooltips.
pub fn export_camera_state(
    camera_q: Query<(&Transform, &Projection), With<Camera2d>>,
    windows: Query<&Window>,
) {
    let Ok((transform, projection)) = camera_q.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = if let Projection::Orthographic(ref ortho) = *projection {
        ortho.scale
    } else {
        1.0
    };
    if let Ok(mut cam) = crate::CAMERA_STATE.lock() {
        *cam = crate::CameraSnapshot {
            x: transform.translation.x,
            y: transform.translation.y,
            scale,
            win_w: window.width(),
            win_h: window.height(),
        };
    }
}

/// Build tooltip lines for a given hex. Used by the exported wasm-bindgen function.
pub fn tile_info_at(state: &GameStateView, col: i32, row: i32) -> Vec<String> {
    let mut lines = Vec::new();

    for unit in &state.units {
        if unit.x == col && unit.y == row {
            let type_name = match unit.unit_type {
                UnitType::Worker => "Worker",
                UnitType::Soldier => "Soldier",
                UnitType::Scout => "Scout",
            };
            let status = match unit.status {
                UnitStatus::Idle => "Idle",
                UnitStatus::Moving => "Moving",
                UnitStatus::Attacking => "Attacking",
                UnitStatus::Harvesting => "Harvesting",
                UnitStatus::Returning => "Returning",
                UnitStatus::Dead => "Dead",
            };
            lines.push(format!(
                "P{} {} - HP {}/{} [{}]",
                unit.player_slot + 1,
                type_name,
                unit.hp,
                unit.max_hp,
                status,
            ));
        }
    }

    for building in &state.buildings {
        if building.x == col && building.y == row {
            let type_name = match building.building_type {
                BuildingType::Base => "Base",
                BuildingType::Turret => "Turret",
                BuildingType::Wall => "Wall",
            };
            let mut info = format!(
                "P{} {} - HP {}/{}",
                building.player_slot + 1,
                type_name,
                building.hp,
                building.max_hp,
            );
            if let Some(energy) = building.energy {
                info.push_str(&format!(" [Energy: {}]", energy));
            }
            lines.push(info);
        }
    }

    for resource in &state.resources {
        if resource.x == col && resource.y == row {
            lines.push(format!("Resource: {} remaining", resource.remaining));
        }
    }

    lines
}
