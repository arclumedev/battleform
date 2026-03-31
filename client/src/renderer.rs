use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::state::*;
use crate::EntityMap;

const HEX_SIZE: f32 = 12.0;
const HEX_WIDTH: f32 = HEX_SIZE * 1.732; // sqrt(3) * size
const HEX_HEIGHT: f32 = HEX_SIZE * 2.0;

/// Camera rotation for Civ-style tilt (~25 degrees, tilted left)
const CAM_ANGLE: f32 = -0.22;
/// Camera Y scale for isometric foreshortening
const CAM_Y_SCALE: f32 = 0.58;

/// Convert hex grid (col, row) to flat pixel coordinates.
/// The isometric perspective is handled entirely by the camera transform.
fn hex_to_pixel(col: i32, row: i32) -> Vec2 {
    let offset = if row % 2 != 0 { HEX_WIDTH * 0.5 } else { 0.0 };
    let px = col as f32 * HEX_WIDTH + offset;
    let py = -(row as f32 * HEX_HEIGHT * 0.75);
    Vec2::new(px, py)
}

/// Convert screen pixel coordinates back to hex grid, accounting for camera transform.
fn pixel_to_hex(screen: Vec2, cam_x: f32, cam_y: f32, cam_scale: f32, win_w: f32, win_h: f32) -> (i32, i32) {
    // Screen to world: offset from center, scale, un-squish Y, un-rotate, translate
    let cx = (screen.x - win_w * 0.5) * cam_scale;
    let cy = -(screen.y - win_h * 0.5) * cam_scale;

    // Reverse Y squish
    let uy = cy / CAM_Y_SCALE;
    let ux = cx;

    // Reverse rotation
    let cos_a = CAM_ANGLE.cos();
    let sin_a = CAM_ANGLE.sin();
    let flat_x = ux * cos_a + uy * sin_a + cam_x;
    let flat_y = -ux * sin_a + uy * cos_a + cam_y;

    let row = (-flat_y / (HEX_HEIGHT * 0.75)).round() as i32;
    let offset = if row % 2 != 0 { HEX_WIDTH * 0.5 } else { 0.0 };
    let col = ((flat_x - offset) / HEX_WIDTH).round() as i32;
    (col, row)
}

/// Public wrapper for JS tooltip queries — reads camera state from global.
pub fn pixel_to_hex_pub(screen_x: f32, screen_y: f32) -> (i32, i32) {
    if let Ok(cam) = crate::CAMERA_STATE.lock() {
        pixel_to_hex(
            Vec2::new(screen_x, screen_y),
            cam.x,
            cam.y,
            cam.scale,
            cam.win_w,
            cam.win_h,
        )
    } else {
        (0, 0)
    }
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
        Transform::from_xyz(center.x, center.y, 999.0)
            .with_rotation(Quat::from_rotation_z(-CAM_ANGLE))
            .with_scale(Vec3::new(1.0, 1.0 / CAM_Y_SCALE, 1.0)),
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

    sync_units(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
    sync_buildings(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
    sync_resources(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
    sync_fog(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
}

fn tile_color(tile_type: &TileType) -> Color {
    match tile_type {
        TileType::Grass => Color::srgb(0.22, 0.38, 0.18),
        TileType::Desert => Color::srgb(0.55, 0.45, 0.28),
        TileType::Forest => Color::srgb(0.12, 0.28, 0.10),
        TileType::Mountain => Color::srgb(0.35, 0.32, 0.30),
        TileType::WaterLake => Color::srgb(0.15, 0.30, 0.50),
        TileType::WaterSea => Color::srgb(0.10, 0.22, 0.42),
        TileType::Snow => Color::srgb(0.70, 0.72, 0.75),
    }
}

fn spawn_terrain(
    commands: &mut Commands,
    state: &GameStateView,
    entity_map: &mut EntityMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let hex_mesh = meshes.add(RegularPolygon::new(HEX_SIZE - 0.5, 6));

    // Pre-create materials for each tile type
    let tile_mats: std::collections::HashMap<String, Handle<ColorMaterial>> = [
        TileType::Grass,
        TileType::Desert,
        TileType::Forest,
        TileType::Mountain,
        TileType::WaterLake,
        TileType::WaterSea,
        TileType::Snow,
    ]
    .into_iter()
    .map(|t| {
        let key = format!("{:?}", t);
        let mat = materials.add(ColorMaterial::from_color(tile_color(&t)));
        (key, mat)
    })
    .collect();

    let default_mat = materials.add(ColorMaterial::from_color(tile_color(&TileType::Grass)));

    for y in 0..state.map_height {
        for x in 0..state.map_width {
            let tile = state
                .terrain
                .get(y as usize)
                .and_then(|row| row.get(x as usize));

            let tile_type = tile.map(|t| &t.tile_type).unwrap_or(&TileType::Grass);
            let elevation = tile.map(|t| t.elevation).unwrap_or(1);

            let pos = hex_to_pixel(x as i32, y as i32);
            let key = format!("{:?}", tile_type);
            let mat = tile_mats.get(&key).cloned().unwrap_or(default_mat.clone());

            // Elevation: shift Y up slightly for higher tiles
            let elev_offset = elevation as f32 * 1.5;

            commands.spawn((
                Mesh2d(hex_mesh.clone()),
                MeshMaterial2d(mat),
                Transform::from_xyz(pos.x, pos.y + elev_offset, elevation as f32 * 0.1),
                TerrainTile,
            ));
        }
    }
    entity_map.terrain_spawned = true;
}

fn sync_units(
    commands: &mut Commands,
    state: &GameStateView,
    entity_map: &mut EntityMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let mut seen = std::collections::HashSet::new();

    for unit in &state.units {
        seen.insert(unit.id.clone());
        let radius = match unit.unit_type {
            UnitType::Worker => HEX_SIZE * 0.35,
            UnitType::Soldier => HEX_SIZE * 0.55,
            UnitType::Scout => HEX_SIZE * 0.4,
        };
        let p = hex_to_pixel(unit.x, unit.y);
        let mat = materials.add(ColorMaterial::from_color(player_color(unit.player_slot)));
        let mesh = meshes.add(Circle::new(radius));

        if let Some(&entity) = entity_map.units.get(&unit.id) {
            commands.entity(entity).insert((
                Transform::from_xyz(p.x, p.y, 2.0),
                Mesh2d(mesh),
                MeshMaterial2d(mat),
            ));
        } else {
            let entity = commands
                .spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(mat),
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

fn sync_buildings(
    commands: &mut Commands,
    state: &GameStateView,
    entity_map: &mut EntityMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let mut seen = std::collections::HashSet::new();

    for building in &state.buildings {
        seen.insert(building.id.clone());
        let p = hex_to_pixel(building.x, building.y);
        let pos = Transform::from_xyz(p.x, p.y, 1.0);
        let mat = materials.add(ColorMaterial::from_color(player_color(building.player_slot)));
        let mesh = meshes.add(RegularPolygon::new(HEX_SIZE * 0.75, 6));

        if let Some(&entity) = entity_map.buildings.get(&building.id) {
            commands.entity(entity).insert((pos, Mesh2d(mesh), MeshMaterial2d(mat)));
        } else {
            let entity = commands
                .spawn((Mesh2d(mesh), MeshMaterial2d(mat), pos, BuildingMarker(building.id.clone())))
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

fn sync_resources(
    commands: &mut Commands,
    state: &GameStateView,
    entity_map: &mut EntityMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let mut seen = std::collections::HashSet::new();

    for resource in &state.resources {
        seen.insert(resource.id.clone());
        let brightness = (resource.remaining as f32 / 500.0).clamp(0.3, 1.0);
        let color = Color::srgb(brightness, brightness * 0.9, 0.1);
        let p = hex_to_pixel(resource.x, resource.y);
        let pos = Transform::from_xyz(p.x, p.y, 1.5);
        let mat = materials.add(ColorMaterial::from_color(color));
        let mesh = meshes.add(RegularPolygon::new(HEX_SIZE * 0.4, 6));

        if let Some(&entity) = entity_map.resources.get(&resource.id) {
            commands.entity(entity).insert((pos, Mesh2d(mesh), MeshMaterial2d(mat)));
        } else {
            let entity = commands
                .spawn((Mesh2d(mesh), MeshMaterial2d(mat), pos, ResourceMarker(resource.id.clone())))
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
    keys: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Query<&Window>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut transform, mut projection)) = camera.single_mut() else {
        return;
    };

    // Keyboard pan (WASD + arrow keys)
    let speed = 200.0 * time.delta_secs();
    let mut move_dir = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        move_dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        move_dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        move_dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        move_dir.x += 1.0;
    }

    if move_dir != Vec2::ZERO {
        let move_dir = move_dir.normalize() * speed;
        transform.translation.x += move_dir.x;
        transform.translation.y += move_dir.y;
    }

    // Scroll zoom
    for event in scroll_events.read() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            let factor = if event.y > 0.0 { 0.9 } else { 1.1 };
            ortho.scale = (ortho.scale * factor).clamp(0.3, 3.0);
        }
    }

    // Right-click drag pan
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
