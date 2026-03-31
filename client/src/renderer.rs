use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::state::*;
use crate::EntityMap;

const HEX_SIZE: f32 = 12.0; // Radius of circumscribed circle
const HEX_WIDTH: f32 = HEX_SIZE * 1.732; // sqrt(3) * size
const HEX_HEIGHT: f32 = HEX_SIZE * 2.0;

/// Convert hex grid (col, row) in odd-r offset to pixel coordinates.
fn hex_to_pixel(col: i32, row: i32) -> Vec2 {
    let offset = if row % 2 != 0 { HEX_WIDTH * 0.5 } else { 0.0 };
    let px = col as f32 * HEX_WIDTH + offset;
    let py = -(row as f32 * HEX_HEIGHT * 0.75);
    Vec2::new(px, py)
}

/// Convert world pixel coordinates back to the nearest hex grid (col, row).
fn pixel_to_hex(world: Vec2) -> (i32, i32) {
    // Approximate row from y, then refine col accounting for odd-r offset
    let row = (-world.y / (HEX_HEIGHT * 0.75)).round() as i32;
    let offset = if row % 2 != 0 { HEX_WIDTH * 0.5 } else { 0.0 };
    let col = ((world.x - offset) / HEX_WIDTH).round() as i32;
    (col, row)
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

#[derive(Component)]
pub struct TooltipText;

#[derive(Component)]
pub struct TooltipBg;

pub fn setup_camera(mut commands: Commands) {
    let center = hex_to_pixel(16, 16);
    commands.spawn((
        Camera2d,
        Transform::from_xyz(center.x, center.y, 999.0),
    ));

    // Tooltip background (dark semi-transparent sprite)
    commands.spawn((
        Sprite {
            color: Color::srgba(0.05, 0.05, 0.1, 0.85),
            custom_size: Some(Vec2::ZERO),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 9.0),
        Visibility::Hidden,
        TooltipBg,
    ));

    // Tooltip text
    commands.spawn((
        Text2d::new(""),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Transform::from_xyz(0.0, 0.0, 10.0),
        Visibility::Hidden,
        TooltipText,
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
                Transform::from_xyz(pos.x, pos.y, 0.0),
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
            UnitType::Worker => Vec2::splat(HEX_SIZE * 0.8),
            UnitType::Soldier => Vec2::splat(HEX_SIZE * 1.3),
            UnitType::Scout => Vec2::splat(HEX_SIZE),
        };
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
            custom_size: Some(Vec2::splat(HEX_SIZE * 1.6)),
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
            custom_size: Some(Vec2::splat(HEX_SIZE * 0.8)),
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
                        Transform::from_xyz(p.x, p.y, 5.0),
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

#[allow(clippy::type_complexity)]
pub fn update_tooltip(
    state: Res<GameStateView>,
    windows: Query<&Window>,
    camera_q: Query<(&Transform, &Projection), With<Camera2d>>,
    mut text_q: Query<
        (&mut Text2d, &mut Transform, &mut Visibility),
        (With<TooltipText>, Without<Camera2d>, Without<TooltipBg>),
    >,
    mut bg_q: Query<
        (&mut Sprite, &mut Transform, &mut Visibility),
        (With<TooltipBg>, Without<Camera2d>, Without<TooltipText>),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((cam_transform, projection)) = camera_q.single() else {
        return;
    };
    let Ok((mut text, mut text_tf, mut text_vis)) = text_q.single_mut() else {
        return;
    };
    let Ok((mut bg_sprite, mut bg_tf, mut bg_vis)) = bg_q.single_mut() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        *text_vis = Visibility::Hidden;
        *bg_vis = Visibility::Hidden;
        return;
    };

    // Convert screen position to world coordinates
    let scale = if let Projection::Orthographic(ref ortho) = *projection {
        ortho.scale
    } else {
        1.0
    };

    let screen_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    let screen_offset = cursor_pos - screen_center;
    let world_pos = Vec2::new(
        cam_transform.translation.x + screen_offset.x * scale,
        cam_transform.translation.y - screen_offset.y * scale,
    );

    let (col, row) = pixel_to_hex(world_pos);

    // Out of bounds — hide tooltip
    if col < 0
        || row < 0
        || col >= state.map_width as i32
        || row >= state.map_height as i32
        || state.map_width == 0
    {
        *text_vis = Visibility::Hidden;
        *bg_vis = Visibility::Hidden;
        return;
    }

    // Build tooltip content
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("Hex ({}, {})", col, row));

    // Terrain
    if let Some(tile) = state
        .terrain
        .get(row as usize)
        .and_then(|r| r.get(col as usize))
    {
        lines.push(match tile {
            TileType::Open => "Terrain: Open".into(),
            TileType::Blocked => "Terrain: Blocked".into(),
        });
    }

    // Visibility
    if let Some(vis) = state
        .visibility
        .get(row as usize)
        .and_then(|r| r.get(col as usize))
    {
        lines.push(match vis {
            VisibilityState::Visible => "Visibility: Visible".into(),
            VisibilityState::PreviouslySeen => "Visibility: Fog".into(),
            VisibilityState::Unseen => "Visibility: Unseen".into(),
        });
    }

    // Units on this tile
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

    // Buildings on this tile
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

    // Resources on this tile
    for resource in &state.resources {
        if resource.x == col && resource.y == row {
            lines.push(format!("Resource: {} remaining", resource.remaining));
        }
    }

    let content = lines.join("\n");
    text.0 = content.clone();

    // Position tooltip offset from cursor in world space
    let tooltip_offset = Vec2::new(15.0, -15.0) * scale;
    let tooltip_world = Vec2::new(
        world_pos.x + tooltip_offset.x,
        world_pos.y + tooltip_offset.y,
    );

    // Scale text inversely to camera zoom so it stays readable
    let text_scale = scale.max(0.5);
    text_tf.translation = Vec3::new(tooltip_world.x, tooltip_world.y, 10.0);
    text_tf.scale = Vec3::splat(text_scale);
    *text_vis = Visibility::Visible;

    // Size the background to fit the text
    let line_count = content.lines().count() as f32;
    let max_chars = content.lines().map(|l| l.len()).max().unwrap_or(0) as f32;
    let bg_width = max_chars * 6.5 + 12.0;
    let bg_height = line_count * 14.0 + 8.0;

    bg_sprite.custom_size = Some(Vec2::new(bg_width, bg_height));
    bg_tf.translation = Vec3::new(
        tooltip_world.x + bg_width * text_scale * 0.5 - 4.0 * text_scale,
        tooltip_world.y - bg_height * text_scale * 0.5 + 6.0 * text_scale,
        9.0,
    );
    bg_tf.scale = Vec3::splat(text_scale);
    *bg_vis = Visibility::Visible;
}
