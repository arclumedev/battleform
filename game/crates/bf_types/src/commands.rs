use serde::{Deserialize, Serialize};

use crate::UnitType;

/// A command issued by a player or agent during a match.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameCommand {
    pub player_slot: u8,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
}

/// Strongly-typed command variants for the engine.
#[derive(Clone, Debug)]
pub enum Command {
    SpawnUnit {
        player_slot: u8,
        unit_type: UnitType,
    },
    MoveUnit {
        player_slot: u8,
        unit_id: String,
        target_x: i32,
        target_y: i32,
    },
    AttackTarget {
        player_slot: u8,
        unit_id: String,
        target_id: String,
    },
    Harvest {
        player_slot: u8,
        unit_id: String,
        resource_id: String,
    },
}

/// Match configuration.
#[derive(Clone, Debug)]
pub struct MatchConfig {
    pub map_preset: MapPreset,
    pub players: Vec<PlayerConfig>,
    pub max_ticks: u32,
    pub tick_rate_ms: u32,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            map_preset: MapPreset::Duel,
            players: vec![],
            max_ticks: 2000,
            tick_rate_ms: 100,
        }
    }
}

/// Player slot configuration.
#[derive(Clone, Debug)]
pub struct PlayerConfig {
    pub slot: u8,
    pub kind: PlayerKind,
    pub name: String,
}

/// The type of player occupying a slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayerKind {
    Human,
    Bot,
    McpAgent,
}

/// Predefined map layouts.
#[derive(Clone, Debug, Default)]
pub enum MapPreset {
    #[default]
    Duel,
    TwoVTwo,
    FreeForAll4,
    FreeForAll8,
}

impl MapPreset {
    pub fn player_count(&self) -> usize {
        match self {
            MapPreset::Duel => 2,
            MapPreset::TwoVTwo | MapPreset::FreeForAll4 => 4,
            MapPreset::FreeForAll8 => 8,
        }
    }
}
