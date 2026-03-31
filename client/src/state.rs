use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameStateView {
    pub tick: u32,
    pub map_width: u32,
    pub map_height: u32,
    #[serde(default)]
    pub terrain: Vec<Vec<TileType>>,
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
    pub path: Vec<serde_json::Value>,
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

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TileType {
    #[default]
    Open,
    Blocked,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UnitType {
    Worker,
    Soldier,
    Scout,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BuildingType {
    Base,
    Turret,
    Wall,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityState {
    #[default]
    Unseen,
    PreviouslySeen,
    Visible,
}
