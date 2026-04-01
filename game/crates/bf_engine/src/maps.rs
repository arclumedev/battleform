use bf_types::*;
use crate::{Building, ResourceNode, PlayerState, GameState};

/// Map configuration produced by generation.
pub struct MapConfig {
    pub width: u32,
    pub height: u32,
    pub terrain: Vec<Vec<Tile>>,
    pub start_positions: Vec<Position>,
    pub resource_nodes: Vec<ResourceNodeConfig>,
}

pub struct ResourceNodeConfig {
    pub x: i32,
    pub y: i32,
    pub energy: i32,
}

/// Generate a complete map for a given player count.
pub fn generate_map_config(player_count: usize) -> MapConfig {
    let width = 32u32;
    let height = 32u32;
    let start_positions = get_start_positions(player_count, width, height);
    let terrain = generate_terrain(width, height, &start_positions);
    let resource_nodes = get_resource_nodes(player_count, width, height);

    MapConfig {
        width,
        height,
        terrain,
        start_positions,
        resource_nodes,
    }
}

fn pseudo_noise(x: f64, y: f64) -> f64 {
    let v = (x * 127.1 + y * 311.7).sin() * 43758.5453;
    v - v.floor()
}

fn generate_terrain(width: u32, height: u32, start_positions: &[Position]) -> Vec<Vec<Tile>> {
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    (0..height).map(|y| {
        (0..width).map(|x| {
            let fx = x as f64;
            let fy = y as f64;

            let dist_from_center = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() / max_dist;
            let dist_from_edge = (x.min(y).min(width - 1 - x).min(height - 1 - y)) as f64;

            let n = pseudo_noise(fx, fy);
            let n2 = pseudo_noise(fx + 100.0, fy + 100.0);

            let near_start = start_positions.iter().any(|sp| {
                ((sp.x - x as i32).abs() + (sp.y - y as i32).abs()) <= 4
            });

            if near_start {
                Tile { tile_type: TileType::Grass, elevation: 1 }
            } else if dist_from_edge <= 1.0 {
                Tile { tile_type: TileType::WaterSea, elevation: 0 }
            } else if dist_from_edge <= 3.0 && n < 0.4 {
                Tile { tile_type: TileType::WaterLake, elevation: 0 }
            } else if dist_from_center < 0.2 && n < 0.3 {
                Tile { tile_type: TileType::Mountain, elevation: 3 }
            } else if dist_from_center < 0.35 && n < 0.25 {
                Tile { tile_type: TileType::Forest, elevation: 2 }
            } else if n2 < 0.12 {
                Tile { tile_type: TileType::Desert, elevation: 1 }
            } else if n2 < 0.18 {
                Tile { tile_type: TileType::Snow, elevation: 2 }
            } else if n < 0.2 {
                Tile { tile_type: TileType::Forest, elevation: 1 }
            } else {
                Tile { tile_type: TileType::Grass, elevation: 1 }
            }
        }).collect()
    }).collect()
}

fn get_start_positions(player_count: usize, width: u32, height: u32) -> Vec<Position> {
    let m = 2i32;
    let w = width as i32;
    let h = height as i32;

    match player_count {
        1 => vec![Position::new(m, m)],
        2 => vec![Position::new(m, m), Position::new(w - 1 - m, h - 1 - m)],
        3 => vec![
            Position::new(m, m),
            Position::new(w - 1 - m, m),
            Position::new(w / 2, h - 1 - m),
        ],
        4 => vec![
            Position::new(m, m),
            Position::new(w - 1 - m, m),
            Position::new(m, h - 1 - m),
            Position::new(w - 1 - m, h - 1 - m),
        ],
        5 => vec![
            Position::new(m, m),
            Position::new(w - 1 - m, m),
            Position::new(m, h - 1 - m),
            Position::new(w - 1 - m, h - 1 - m),
            Position::new(w / 2, m),
        ],
        6 => vec![
            Position::new(m, m),
            Position::new(w - 1 - m, m),
            Position::new(m, h - 1 - m),
            Position::new(w - 1 - m, h - 1 - m),
            Position::new(w / 2, m),
            Position::new(w / 2, h - 1 - m),
        ],
        7 => vec![
            Position::new(m, m),
            Position::new(w - 1 - m, m),
            Position::new(m, h - 1 - m),
            Position::new(w - 1 - m, h - 1 - m),
            Position::new(w / 2, m),
            Position::new(w / 2, h - 1 - m),
            Position::new(m, h / 2),
        ],
        _ => vec![
            Position::new(m, m),
            Position::new(w - 1 - m, m),
            Position::new(m, h - 1 - m),
            Position::new(w - 1 - m, h - 1 - m),
            Position::new(w / 2, m),
            Position::new(w / 2, h - 1 - m),
            Position::new(m, h / 2),
            Position::new(w - 1 - m, h / 2),
        ],
    }
}

fn get_resource_nodes(player_count: usize, width: u32, height: u32) -> Vec<ResourceNodeConfig> {
    let cx = (width / 2) as i32;
    let cy = (height / 2) as i32;
    let mut nodes = Vec::new();

    // Central resources
    for (dx, dy) in [(2, 2), (-2, 2), (2, -2), (-2, -2)] {
        nodes.push(ResourceNodeConfig { x: cx + dx, y: cy + dy, energy: 500 });
    }

    // Per-player nearby resources
    let starts = get_start_positions(player_count, width, height);
    for start in &starts {
        let dx = if start.x < cx { 4 } else { -4 };
        let dy = if start.y < cy { 4 } else { -4 };
        nodes.push(ResourceNodeConfig {
            x: start.x + dx,
            y: start.y + dy,
            energy: 500,
        });
    }

    // Extra resources for 4+ players
    if player_count >= 4 {
        for (dx, dy) in [(8, 0), (-8, 0), (0, 8), (0, -8)] {
            nodes.push(ResourceNodeConfig { x: cx + dx, y: cy + dy, energy: 500 });
        }
    }

    nodes
}

/// Create a full GameState from a MapConfig.
pub fn create_game_state(config: &MapConfig, player_configs: &[PlayerConfig]) -> GameState {
    let mut buildings = Vec::new();
    let mut players = Vec::new();

    for (i, start) in config.start_positions.iter().enumerate() {
        if i >= player_configs.len() { break; }

        let slot = player_configs[i].slot;
        let base_id = uuid::Uuid::new_v4().to_string();

        buildings.push(Building {
            id: base_id.clone(),
            player_slot: slot,
            building_type: BuildingType::Base,
            x: start.x,
            y: start.y,
            hp: BASE_HP,
            max_hp: BASE_HP,
        });

        players.push(PlayerState {
            slot,
            energy: STARTING_ENERGY,
            base_id,
        });
    }

    let resource_nodes: Vec<ResourceNode> = config.resource_nodes.iter().map(|r| {
        ResourceNode {
            id: uuid::Uuid::new_v4().to_string(),
            x: r.x,
            y: r.y,
            remaining: r.energy,
        }
    }).collect();

    GameState {
        phase: MatchPhase::Active,
        tick: 0,
        map_width: config.width,
        map_height: config.height,
        terrain: config.terrain.clone(),
        units: vec![],
        buildings,
        resource_nodes,
        players,
        visibility: vec![],
        command_queue: vec![],
        winner_slot: None,
        snapshot_units: vec![],
        snapshot_buildings: vec![],
    }
}

/// Preset map configs.
pub fn map_config_for_preset(preset: &MapPreset) -> MapConfig {
    generate_map_config(preset.player_count())
}
