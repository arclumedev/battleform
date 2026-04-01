use bevy::prelude::*;
use bf_types::TileType;

pub const PLAYER_COLORS: [Color; 8] = [
    Color::srgb(0.23, 0.51, 0.96),
    Color::srgb(0.94, 0.27, 0.27),
    Color::srgb(0.20, 0.83, 0.60),
    Color::srgb(0.95, 0.75, 0.18),
    Color::srgb(0.73, 0.33, 0.83),
    Color::srgb(0.98, 0.50, 0.20),
    Color::srgb(0.20, 0.80, 0.80),
    Color::srgb(0.80, 0.80, 0.80),
];

pub fn player_color(slot: u8) -> Color {
    PLAYER_COLORS.get(slot as usize).copied().unwrap_or(Color::WHITE)
}

pub fn tile_color(tile_type: &TileType) -> Color {
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

pub fn tile_height(tile_type: &TileType, elevation: u8) -> f32 {
    let base = match tile_type {
        TileType::WaterLake | TileType::WaterSea => 0.05,
        TileType::Grass | TileType::Desert => 0.15,
        TileType::Forest => 0.25,
        TileType::Snow => 0.3,
        TileType::Mountain => 0.6,
    };
    base + elevation as f32 * 0.1
}
