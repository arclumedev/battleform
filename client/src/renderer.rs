use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::state::*;
use crate::EntityMap;

const HEX_SIZE: f32 = 1.0;
const HEX_WIDTH: f32 = HEX_SIZE * 1.732;
const HEX_HEIGHT: f32 = HEX_SIZE * 2.0;

/// Convert hex grid (col, row) to flat world coordinates.
fn hex_to_pixel(col: i32, row: i32) -> Vec2 {
    let offset = if row % 2 != 0 { HEX_WIDTH * 0.5 } else { 0.0 };
    let px = col as f32 * HEX_WIDTH + offset;
    let py = -(row as f32 * HEX_HEIGHT * 0.75);
    Vec2::new(px, py)
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
    PLAYER_COLORS.get(slot as usize).copied().unwrap_or(Color::WHITE)
}

fn tile_color(tile_type: &TileType) -> Color {
    match tile_type {
        TileType::Grass => Color::srgb(0.35, 0.55, 0.25),
        TileType::Desert => Color::srgb(0.72, 0.62, 0.38),
        TileType::Forest => Color::srgb(0.18, 0.40, 0.15),
        TileType::Mountain => Color::srgb(0.50, 0.48, 0.45),
        TileType::WaterLake => Color::srgb(0.25, 0.45, 0.65),
        TileType::WaterSea => Color::srgb(0.15, 0.32, 0.55),
        TileType::Snow => Color::srgb(0.82, 0.84, 0.88),
    }
}

fn tile_height(tile_type: &TileType, elevation: u8) -> f32 {
    let base = match tile_type {
        TileType::WaterLake | TileType::WaterSea => 0.05,
        TileType::Grass | TileType::Desert => 0.15,
        TileType::Forest => 0.25,
        TileType::Snow => 0.3,
        TileType::Mountain => 0.6,
    };
    base + elevation as f32 * 0.1
}

// --- Components ---

#[derive(Component)]
pub struct UnitMarker(#[allow(dead_code)] pub String);

#[derive(Component)]
pub struct BuildingMarker(#[allow(dead_code)] pub String);

#[derive(Component)]
pub struct ResourceMarker(#[allow(dead_code)] pub String);

#[derive(Component)]
pub struct TerrainTile;

// --- Setup ---

pub fn setup_camera(mut commands: Commands) {
    let center = hex_to_pixel(16, 16);

    // 3D orthographic camera looking down at an angle (isometric-ish)
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scale: 12.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(center.x + 15.0, 25.0, center.y - 15.0)
            .looking_at(Vec3::new(center.x, 0.0, center.y), Vec3::Y),
    ));

    // Directional light (sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.3, 0.0)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: false,
    });
}

// --- Entity Sync ---

pub fn sync_entities(
    mut commands: Commands,
    state: Res<GameStateView>,
    mut entity_map: ResMut<EntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if state.map_width == 0 || state.map_height == 0 {
        return;
    }

    if !entity_map.terrain_spawned {
        web_sys::console::log_1(
            &format!(
                "[wasm] Spawning 3D terrain: {}x{}, {} rows",
                state.map_width, state.map_height, state.terrain.len()
            )
            .into(),
        );
        spawn_terrain(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
    }

    sync_units(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
    sync_buildings(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
    sync_resources(&mut commands, &state, &mut entity_map, &mut meshes, &mut materials);
}

fn spawn_terrain(
    commands: &mut Commands,
    state: &GameStateView,
    entity_map: &mut EntityMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Pre-create materials for each tile type
    let tile_mats: std::collections::HashMap<String, Handle<StandardMaterial>> = [
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
        let mat = materials.add(StandardMaterial {
            base_color: tile_color(&t),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        });
        (key, mat)
    })
    .collect();

    let default_mat = materials.add(StandardMaterial {
        base_color: tile_color(&TileType::Grass),
        ..default()
    });

    for y in 0..state.map_height {
        for x in 0..state.map_width {
            let tile = state
                .terrain
                .get(y as usize)
                .and_then(|row| row.get(x as usize));

            let tile_type = tile.map(|t| &t.tile_type).unwrap_or(&TileType::Grass);
            let elevation = tile.map(|t| t.elevation).unwrap_or(1);

            let pos = hex_to_pixel(x as i32, y as i32);
            let height = tile_height(tile_type, elevation);
            let key = format!("{:?}", tile_type);
            let mat = tile_mats.get(&key).cloned().unwrap_or(default_mat.clone());

            // 3D hex column: a cylinder with 6 sides
            let hex_mesh = meshes.add(Extrusion::new(RegularPolygon::new(HEX_SIZE - 0.02, 6), height));

            commands.spawn((
                Mesh3d(hex_mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(pos.x, height * 0.5, pos.y),
                TerrainTile,
            ));
        }
    }
    entity_map.terrain_spawned = true;
    web_sys::console::log_1(&"[wasm] 3D terrain spawned!".into());
}

fn sync_units(
    commands: &mut Commands,
    state: &GameStateView,
    entity_map: &mut EntityMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut seen = std::collections::HashSet::new();

    for unit in &state.units {
        seen.insert(unit.id.clone());

        let radius = match unit.unit_type {
            UnitType::Worker => HEX_SIZE * 0.25,
            UnitType::Soldier => HEX_SIZE * 0.35,
            UnitType::Scout => HEX_SIZE * 0.28,
        };
        let unit_height = match unit.unit_type {
            UnitType::Worker => 0.3,
            UnitType::Soldier => 0.5,
            UnitType::Scout => 0.4,
        };

        let p = hex_to_pixel(unit.x, unit.y);
        // Place unit on top of the terrain
        let terrain_h = state
            .terrain
            .get(unit.y as usize)
            .and_then(|row| row.get(unit.x as usize))
            .map(|t| tile_height(&t.tile_type, t.elevation))
            .unwrap_or(0.15);

        let y_pos = terrain_h + unit_height * 0.5;

        if let Some(&entity) = entity_map.units.get(&unit.id) {
            commands
                .entity(entity)
                .insert(Transform::from_xyz(p.x, y_pos, p.y));
        } else {
            let mesh = meshes.add(Capsule3d::new(radius, unit_height));
            let mat = materials.add(StandardMaterial {
                base_color: player_color(unit.player_slot),
                perceptual_roughness: 0.5,
                metallic: 0.2,
                ..default()
            });

            let entity = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(p.x, y_pos, p.y),
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
    materials: &mut Assets<StandardMaterial>,
) {
    let mut seen = std::collections::HashSet::new();

    for building in &state.buildings {
        seen.insert(building.id.clone());

        let p = hex_to_pixel(building.x, building.y);
        let terrain_h = state
            .terrain
            .get(building.y as usize)
            .and_then(|row| row.get(building.x as usize))
            .map(|t| tile_height(&t.tile_type, t.elevation))
            .unwrap_or(0.15);

        let building_h = 1.2;
        let building_w = HEX_SIZE * 0.6;
        let y_pos = terrain_h + building_h * 0.5;

        if let Some(&entity) = entity_map.buildings.get(&building.id) {
            commands
                .entity(entity)
                .insert(Transform::from_xyz(p.x, y_pos, p.y));
        } else {
            let mesh = meshes.add(Cuboid::new(building_w, building_h, building_w));
            let mat = materials.add(StandardMaterial {
                base_color: player_color(building.player_slot),
                perceptual_roughness: 0.6,
                metallic: 0.1,
                ..default()
            });

            let entity = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(p.x, y_pos, p.y),
                    BuildingMarker(building.id.clone()),
                ))
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
    materials: &mut Assets<StandardMaterial>,
) {
    let mut seen = std::collections::HashSet::new();

    for resource in &state.resources {
        seen.insert(resource.id.clone());

        let p = hex_to_pixel(resource.x, resource.y);
        let terrain_h = state
            .terrain
            .get(resource.y as usize)
            .and_then(|row| row.get(resource.x as usize))
            .map(|t| tile_height(&t.tile_type, t.elevation))
            .unwrap_or(0.15);

        let brightness = (resource.remaining as f32 / 500.0).clamp(0.3, 1.0);

        if let Some(&entity) = entity_map.resources.get(&resource.id) {
            commands
                .entity(entity)
                .insert(Transform::from_xyz(p.x, terrain_h + 0.2, p.y));
        } else {
            let mesh = meshes.add(Sphere::new(HEX_SIZE * 0.2));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(brightness, brightness * 0.9, 0.1),
                emissive: LinearRgba::new(brightness * 0.3, brightness * 0.25, 0.0, 1.0),
                perceptual_roughness: 0.3,
                metallic: 0.5,
                ..default()
            });

            let entity = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(p.x, terrain_h + 0.2, p.y),
                    ResourceMarker(resource.id.clone()),
                ))
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

// --- Camera Controls ---

pub fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Query<&Window>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
    time: Res<Time>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut transform, mut projection)) = camera.single_mut() else {
        return;
    };

    let cam_scale = if let Projection::Orthographic(ref ortho) = *projection {
        ortho.scale
    } else {
        1.0
    };

    // Keyboard pan (WASD + arrow keys) — move along the ground plane
    let speed = 15.0 * cam_scale * time.delta_secs();
    let forward = Vec3::new(transform.forward().x, 0.0, transform.forward().z).normalize_or_zero();
    let right = Vec3::new(transform.right().x, 0.0, transform.right().z).normalize_or_zero();

    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        move_dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        move_dir -= forward;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        move_dir -= right;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        move_dir += right;
    }

    if move_dir != Vec3::ZERO {
        transform.translation += move_dir.normalize() * speed;
    }

    // Scroll zoom
    for event in scroll_events.read() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            let factor = if event.y > 0.0 { 0.9 } else { 1.1 };
            ortho.scale = (ortho.scale * factor).clamp(3.0, 30.0);
        }
    }

    // Mouse drag pan
    let dragging = mouse_button.pressed(MouseButton::Left)
        || mouse_button.pressed(MouseButton::Middle)
        || mouse_button.pressed(MouseButton::Right);

    if let Some(cursor) = window.cursor_position() {
        if dragging {
            if let Some(last) = *last_cursor {
                let delta = cursor - last;
                let pan_speed = cam_scale * 0.05;
                transform.translation -= right * delta.x * pan_speed;
                transform.translation += forward * delta.y * pan_speed;
            }
        }
        *last_cursor = Some(cursor);
    }

    if !dragging {
        *last_cursor = None;
    }
}
