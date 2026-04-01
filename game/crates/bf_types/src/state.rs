use serde::{Deserialize, Serialize};

/// Complete game state snapshot as seen by the renderer / spectator.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameStateView {
    pub tick: u32,
    pub map_width: u32,
    pub map_height: u32,
    #[serde(default)]
    pub terrain: Vec<Vec<Tile>>,
    #[serde(default)]
    pub units: Vec<UnitView>,
    #[serde(default)]
    pub buildings: Vec<BuildingView>,
    #[serde(default)]
    pub resources: Vec<ResourceView>,
    #[serde(default)]
    pub combat_events: Vec<CombatEvent>,
    #[serde(default)]
    pub visibility: Vec<Vec<VisibilityState>>,
    #[serde(default, rename = "type")]
    pub msg_type: Option<String>,
    #[serde(default)]
    pub players: Option<serde_json::Value>,
}

impl GameStateView {
    pub fn apply(&mut self, diff: StateDiff) {
        self.tick = diff.tick;

        for unit_move in diff.units_moved {
            if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_move.id) {
                unit.x = unit_move.x;
                unit.y = unit_move.y;
                unit.status = unit_move.status;
            }
        }

        for spawn in diff.units_spawned {
            self.units.push(spawn);
        }

        self.units.retain(|u| !diff.units_killed.contains(&u.id));

        for building in diff.buildings_built {
            self.buildings.push(building);
        }

        self.buildings
            .retain(|b| !diff.buildings_destroyed.contains(&b.id));

        self.combat_events = diff.combat_events;

        for (player_slot, new_energy) in diff.resources_changed {
            for building in self.buildings.iter_mut() {
                if building.building_type == BuildingType::Base
                    && building.player_slot == player_slot
                {
                    building.energy = Some(new_energy);
                }
            }
        }

        for vis_update in diff.visibility_updates {
            if let Some(row) = self.visibility.get_mut(vis_update.y as usize) {
                if let Some(cell) = row.get_mut(vis_update.x as usize) {
                    *cell = vis_update.state;
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StateDiff {
    pub tick: u32,
    #[serde(default)]
    pub units_moved: Vec<UnitMove>,
    #[serde(default)]
    pub units_spawned: Vec<UnitView>,
    #[serde(default)]
    pub units_killed: Vec<String>,
    #[serde(default)]
    pub buildings_built: Vec<BuildingView>,
    #[serde(default)]
    pub buildings_destroyed: Vec<String>,
    #[serde(default)]
    pub combat_events: Vec<CombatEvent>,
    #[serde(default)]
    pub resources_changed: Vec<(u8, i32)>,
    #[serde(default)]
    pub visibility_updates: Vec<VisUpdate>,
    #[serde(default, rename = "type")]
    pub msg_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnitMove {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub status: UnitStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnitView {
    pub id: String,
    pub player_slot: u8,
    pub unit_type: UnitType,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub status: UnitStatus,
    #[serde(default)]
    pub path: Vec<Position>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub cargo: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BuildingView {
    pub id: String,
    pub player_slot: u8,
    pub building_type: BuildingType,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    #[serde(default)]
    pub energy: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceView {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub remaining: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombatEvent {
    pub attacker_id: String,
    pub target_id: String,
    pub damage: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VisUpdate {
    pub x: i32,
    pub y: i32,
    pub state: VisibilityState,
}

/// A single hex tile in the terrain grid.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Tile {
    #[serde(rename = "type")]
    pub tile_type: TileType,
    #[serde(default = "default_elevation")]
    pub elevation: u8,
}

fn default_elevation() -> u8 {
    1
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TileType {
    #[default]
    Grass,
    Desert,
    Forest,
    Mountain,
    WaterLake,
    WaterSea,
    Snow,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "lowercase")]
pub enum UnitType {
    Worker,
    Soldier,
    Scout,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "lowercase")]
pub enum UnitStatus {
    #[default]
    Idle,
    Moving,
    Attacking,
    Harvesting,
    Returning,
    Dead,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BuildingType {
    Base,
    Turret,
    Wall,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityState {
    #[default]
    Unseen,
    PreviouslySeen,
    Visible,
}

/// A hex grid position (col, row) in odd-r offset coordinates.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, Hash, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Unit stat table — cost, HP, speed, range, damage, vision.
#[derive(Clone, Debug)]
pub struct UnitStats {
    pub cost: i32,
    pub hp: i32,
    pub speed: u32,
    pub range: u32,
    pub damage: i32,
    pub vision: u32,
}

pub const UNIT_STATS_WORKER: UnitStats = UnitStats {
    cost: 50,
    hp: 30,
    speed: 2,
    range: 1,
    damage: 5,
    vision: 3,
};

pub const UNIT_STATS_SOLDIER: UnitStats = UnitStats {
    cost: 100,
    hp: 80,
    speed: 2,
    range: 1,
    damage: 20,
    vision: 3,
};

pub const UNIT_STATS_SCOUT: UnitStats = UnitStats {
    cost: 75,
    hp: 40,
    speed: 4,
    range: 1,
    damage: 10,
    vision: 6,
};

pub fn unit_stats(unit_type: UnitType) -> &'static UnitStats {
    match unit_type {
        UnitType::Worker => &UNIT_STATS_WORKER,
        UnitType::Soldier => &UNIT_STATS_SOLDIER,
        UnitType::Scout => &UNIT_STATS_SCOUT,
    }
}

/// Building constants.
pub const BASE_HP: i32 = 500;
pub const BASE_VISION: u32 = 4;
pub const HARVEST_AMOUNT: i32 = 25;
pub const STARTING_ENERGY: i32 = 200;

/// Match phase.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MatchPhase {
    #[default]
    Lobby,
    Active,
    Finished,
}

/// Tile movement costs (Infinity for impassable).
pub fn tile_movement_cost(tile_type: TileType) -> f64 {
    match tile_type {
        TileType::Grass => 1.0,
        TileType::Desert => 1.5,
        TileType::Forest => 1.5,
        TileType::Mountain => f64::INFINITY,
        TileType::WaterLake => f64::INFINITY,
        TileType::WaterSea => f64::INFINITY,
        TileType::Snow => 2.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_json() {
        let json = r#"{"type":"grass","elevation":1}"#;
        let tile: Tile = serde_json::from_str(json).unwrap();
        assert_eq!(tile.tile_type, TileType::Grass);
        assert_eq!(tile.elevation, 1);
    }

    #[test]
    fn test_unit_stats_lookup() {
        let stats = unit_stats(UnitType::Worker);
        assert_eq!(stats.cost, 50);
        assert_eq!(stats.hp, 30);
        assert_eq!(stats.speed, 2);
    }

    #[test]
    fn test_tile_movement_costs() {
        assert_eq!(tile_movement_cost(TileType::Grass), 1.0);
        assert!(tile_movement_cost(TileType::Mountain).is_infinite());
    }

    #[test]
    fn test_snapshot_json() {
        let json = r#"{"tick":5,"mapWidth":2,"mapHeight":2,"terrain":[[{"type":"grass","elevation":1}],[{"type":"mountain","elevation":3}]],"units":[],"buildings":[],"resources":[],"combatEvents":[],"visibility":[]}"#;
        let state: GameStateView = serde_json::from_str(json).unwrap();
        assert_eq!(state.map_width, 2);
        assert_eq!(state.terrain.len(), 2);
        assert_eq!(state.terrain[0][0].tile_type, TileType::Grass);
        assert_eq!(state.terrain[1][0].tile_type, TileType::Mountain);
    }
}
