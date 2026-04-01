use bevy::prelude::*;

use bf_types::hex::{hex_to_pixel, HEX_SIZE};
use bf_types::UnitType;
use crate::game::GameSystems;
use crate::render_utils::{player_color, tile_height};
use crate::{BevyGameState, EntityMap};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (sync_units, sync_buildings, sync_resources)
                .in_set(GameSystems::EntitySync),
        );
    }
}

#[derive(Component)]
pub struct UnitMarker(#[allow(dead_code)] pub String);

#[derive(Component)]
pub struct BuildingMarker(#[allow(dead_code)] pub String);

#[derive(Component)]
pub struct ResourceMarker(#[allow(dead_code)] pub String);

fn sync_units(
    mut commands: Commands,
    state: Res<BevyGameState>,
    mut entity_map: ResMut<EntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

        let (px, pz) = hex_to_pixel(unit.x, unit.y);
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
                .insert(Transform::from_xyz(px, y_pos, pz));
        } else {
            let mesh = meshes.add(Capsule3d::new(radius, unit_height));
            let mat = materials.add(player_color(unit.player_slot));

            let entity = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(px, y_pos, pz),
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
    mut commands: Commands,
    state: Res<BevyGameState>,
    mut entity_map: ResMut<EntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut seen = std::collections::HashSet::new();

    for building in &state.buildings {
        seen.insert(building.id.clone());

        let (px, pz) = hex_to_pixel(building.x, building.y);
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
                .insert(Transform::from_xyz(px, y_pos, pz));
        } else {
            let mesh = meshes.add(Cuboid::new(building_w, building_h, building_w));
            let mat = materials.add(player_color(building.player_slot));

            let entity = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(px, y_pos, pz),
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
    mut commands: Commands,
    state: Res<BevyGameState>,
    mut entity_map: ResMut<EntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut seen = std::collections::HashSet::new();

    for resource in &state.resources {
        seen.insert(resource.id.clone());

        let (px, pz) = hex_to_pixel(resource.x, resource.y);
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
                .insert(Transform::from_xyz(px, terrain_h + 0.2, pz));
        } else {
            let mesh = meshes.add(Sphere::new(HEX_SIZE * 0.2));
            let mat = materials.add(Color::srgb(brightness, brightness * 0.9, 0.1));

            let entity = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(px, terrain_h + 0.2, pz),
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
