use bevy::prelude::*;

use bf_types::hex::{hex_to_pixel, HEX_H, HEX_SIZE, HEX_W};
use bf_types::TileType;
use crate::game::GameSystems;
use crate::render_utils::{tile_color, tile_height};
use crate::{log, BevyGameState, EntityMap};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_terrain.in_set(GameSystems::EntitySync));
    }
}

#[derive(Component)]
pub struct TerrainTile;

fn sync_terrain(
    mut commands: Commands,
    state: Res<BevyGameState>,
    mut entity_map: ResMut<EntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if state.map_width == 0 || state.map_height == 0 {
        return;
    }
    if entity_map.terrain_spawned {
        return;
    }

    log!(
        "[game] Spawning 3D terrain: {}x{}, {} rows",
        state.map_width, state.map_height, state.terrain.len()
    );

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
        let mat = materials.add(tile_color(&t));
        (key, mat)
    })
    .collect();

    let default_mat = materials.add(tile_color(&TileType::Grass));

    for y in 0..state.map_height {
        for x in 0..state.map_width {
            let tile = state
                .terrain
                .get(y as usize)
                .and_then(|row| row.get(x as usize));

            let tile_type = tile.map(|t| &t.tile_type).unwrap_or(&TileType::Grass);
            let elevation = tile.map(|t| t.elevation).unwrap_or(1);

            let (px, pz) = hex_to_pixel(x as i32, y as i32);
            let height = tile_height(tile_type, elevation);
            let key = format!("{:?}", tile_type);
            let mat = tile_mats.get(&key).cloned().unwrap_or(default_mat.clone());

            if x < 3 && y < 3 {
                log!(
                    "[hex] ({},{}) -> world({:.2}, {:.2}) size={} w={:.2} h={:.2}",
                    x, y, px, pz, HEX_SIZE, HEX_W, HEX_H
                );
            }

            let hex_mesh = meshes.add(Extrusion::new(
                RegularPolygon::new(HEX_SIZE - 0.02, 6),
                height,
            ));

            commands.spawn((
                Mesh3d(hex_mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(px, height * 0.5, pz)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                TerrainTile,
            ));
        }
    }
    entity_map.terrain_spawned = true;
    log!("[game] 3D terrain spawned!");
}
