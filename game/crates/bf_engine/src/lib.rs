pub mod commands;
pub mod pathfinding;
pub mod fog;
pub mod maps;
pub mod bot_ai;
pub mod mcp;

use bf_types::*;

/// The core game engine — runs the simulation.
pub struct GameEngine {
    pub state: GameState,
    pub config: MatchConfig,
    pub tick_count: u32,
    pub finished: bool,
    pub winner: Option<u8>,
    pub autopilot_slots: Vec<u8>,
}

impl GameEngine {
    /// Create a new game engine from config.
    pub fn new(config: MatchConfig) -> Self {
        let map_config = maps::map_config_for_preset(&config.map_preset);
        let state = maps::create_game_state(&map_config, &config.players);

        let autopilot_slots: Vec<u8> = config.players.iter()
            .filter(|p| p.kind == PlayerKind::Bot)
            .map(|p| p.slot)
            .collect();

        let mut engine = Self {
            state,
            config,
            tick_count: 0,
            finished: false,
            winner: None,
            autopilot_slots,
        };

        // Compute initial fog
        fog::compute_fog(&mut engine.state);
        engine
    }

    /// Queue a command from a player.
    pub fn queue_command(&mut self, cmd: Command) {
        if self.state.phase != MatchPhase::Active {
            return;
        }
        self.state.command_queue.push(cmd);
    }

    /// Run a single simulation tick.
    pub fn tick(&mut self) -> TickResult {
        if self.finished {
            return TickResult {
                diff: StateDiff {
                    tick: self.state.tick,
                    units_moved: vec![],
                    units_spawned: vec![],
                    units_killed: vec![],
                    buildings_built: vec![],
                    buildings_destroyed: vec![],
                    combat_events: vec![],
                    resources_changed: vec![],
                    visibility_updates: vec![],
                    msg_type: None,
                },
                combat_events: vec![],
            };
        }

        self.state.snapshot_for_diff();

        // 1. Generate autopilot commands (every 5 ticks)
        if self.state.tick.is_multiple_of(5) {
            for &slot in &self.autopilot_slots.clone() {
                // Only if base is alive
                if self.state.get_player_base(slot).is_some() {
                    let bot_commands = bot_ai::generate_bot_commands(&self.state, slot);
                    for cmd in bot_commands {
                        self.state.command_queue.push(cmd);
                    }
                }
            }
        }

        // 2. Drain and execute commands
        let queued: Vec<Command> = self.state.command_queue.drain(..).collect();
        commands::execute_commands(&mut self.state, queued);

        // 3. Resolve movement
        commands::resolve_movement(&mut self.state);

        // 4. Resolve combat
        let combat_events = commands::resolve_combat(&mut self.state);

        // 5. Resolve harvesting
        commands::resolve_harvesting(&mut self.state);

        // 6. Compute fog
        fog::compute_fog(&mut self.state);

        // 7. Check win conditions
        self.check_win_conditions();

        // 8. Generate diff
        let mut diff = self.state.compute_diff();
        diff.combat_events = combat_events.clone();

        self.state.tick += 1;
        self.tick_count += 1;

        TickResult { diff, combat_events }
    }

    /// Get current game state reference.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Get full snapshot for rendering.
    pub fn full_snapshot(&self) -> GameStateView {
        self.state.full_snapshot()
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn winner(&self) -> Option<u8> {
        self.winner
    }

    fn check_win_conditions(&mut self) {
        // Count players with surviving bases
        let mut alive_players: Vec<u8> = Vec::new();
        for player in &self.state.players {
            if self.state.buildings.iter().any(|b| {
                b.player_slot == player.slot && b.building_type == BuildingType::Base
            }) {
                alive_players.push(player.slot);
            }
        }

        if alive_players.len() == 1 {
            // Single survivor wins
            self.winner = Some(alive_players[0]);
            self.finished = true;
            self.state.phase = MatchPhase::Finished;
            self.state.winner_slot = self.winner;
        } else if alive_players.is_empty() {
            // Draw
            self.winner = None;
            self.finished = true;
            self.state.phase = MatchPhase::Finished;
            self.state.winner_slot = None;
        } else {
            // Check max ticks
            let max_ticks = self.state.map_width * 100;
            if self.state.tick >= max_ticks {
                // Score: base HP + unit count × 10 + energy
                let mut best_score = i32::MIN;
                let mut best_slot = None;

                for player in &self.state.players {
                    if !alive_players.contains(&player.slot) {
                        continue;
                    }
                    let base_hp = self.state.buildings.iter()
                        .filter(|b| b.player_slot == player.slot && b.building_type == BuildingType::Base)
                        .map(|b| b.hp)
                        .sum::<i32>();
                    let unit_count = self.state.units.iter()
                        .filter(|u| u.player_slot == player.slot)
                        .count() as i32;
                    let score = base_hp + unit_count * 10 + player.energy;

                    if score > best_score {
                        best_score = score;
                        best_slot = Some(player.slot);
                    } else if score == best_score {
                        best_slot = None; // Tie
                    }
                }

                self.winner = best_slot;
                self.finished = true;
                self.state.phase = MatchPhase::Finished;
                self.state.winner_slot = self.winner;
            }
        }
    }
}

/// Internal game state (authoritative, not the view).
pub struct GameState {
    pub phase: MatchPhase,
    pub tick: u32,
    pub map_width: u32,
    pub map_height: u32,
    pub terrain: Vec<Vec<Tile>>,
    pub units: Vec<Unit>,
    pub buildings: Vec<Building>,
    pub resource_nodes: Vec<ResourceNode>,
    pub players: Vec<PlayerState>,
    pub visibility: Vec<Vec<Vec<VisibilityState>>>,
    pub command_queue: Vec<Command>,
    pub winner_slot: Option<u8>,
    // Snapshot for diff computation
    snapshot_units: Vec<Unit>,
    snapshot_buildings: Vec<Building>,
}

/// Internal unit representation (engine-side, not the view).
#[derive(Clone, Debug)]
pub struct Unit {
    pub id: String,
    pub player_slot: u8,
    pub unit_type: UnitType,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub status: UnitStatus,
    pub path: Vec<Position>,
    pub target_id: Option<String>,
    pub cargo: i32,
}

/// Internal building representation.
#[derive(Clone, Debug)]
pub struct Building {
    pub id: String,
    pub player_slot: u8,
    pub building_type: BuildingType,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
}

/// Internal resource node.
#[derive(Clone, Debug)]
pub struct ResourceNode {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub remaining: i32,
}

/// Per-player state.
#[derive(Clone, Debug)]
pub struct PlayerState {
    pub slot: u8,
    pub energy: i32,
    pub base_id: String,
}

/// Result of a single engine tick.
pub struct TickResult {
    pub diff: StateDiff,
    pub combat_events: Vec<CombatEvent>,
}

impl GameState {
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.map_width as i32 || y >= self.map_height as i32 {
            return true;
        }
        let tile = &self.terrain[y as usize][x as usize];
        let cost = tile_movement_cost(tile.tile_type);
        if cost.is_infinite() {
            return true;
        }
        // Check for buildings blocking
        self.buildings.iter().any(|b| b.x == x && b.y == y)
    }

    pub fn get_movement_cost(&self, x: i32, y: i32) -> f64 {
        if x < 0 || y < 0 || x >= self.map_width as i32 || y >= self.map_height as i32 {
            return f64::INFINITY;
        }
        let tile = &self.terrain[y as usize][x as usize];
        tile_movement_cost(tile.tile_type)
    }

    pub fn get_player_base(&self, slot: u8) -> Option<&Building> {
        self.buildings
            .iter()
            .find(|b| b.player_slot == slot && b.building_type == BuildingType::Base)
    }

    pub fn snapshot_for_diff(&mut self) {
        self.snapshot_units = self.units.clone();
        self.snapshot_buildings = self.buildings.clone();
    }

    pub fn full_snapshot(&self) -> GameStateView {
        GameStateView {
            tick: self.tick,
            map_width: self.map_width,
            map_height: self.map_height,
            terrain: self.terrain.clone(),
            units: self.units.iter().map(|u| UnitView {
                id: u.id.clone(),
                player_slot: u.player_slot,
                unit_type: u.unit_type,
                x: u.x,
                y: u.y,
                hp: u.hp,
                max_hp: u.max_hp,
                status: u.status,
                path: u.path.clone(),
                target_id: u.target_id.clone(),
                cargo: u.cargo,
            }).collect(),
            buildings: self.buildings.iter().map(|b| {
                let energy = self.players.iter()
                    .find(|p| p.slot == b.player_slot)
                    .map(|p| p.energy);
                BuildingView {
                    id: b.id.clone(),
                    player_slot: b.player_slot,
                    building_type: b.building_type,
                    x: b.x,
                    y: b.y,
                    hp: b.hp,
                    max_hp: b.max_hp,
                    energy,
                }
            }).collect(),
            resources: self.resource_nodes.iter().map(|r| ResourceView {
                id: r.id.clone(),
                x: r.x,
                y: r.y,
                remaining: r.remaining,
            }).collect(),
            combat_events: vec![],
            visibility: if self.visibility.is_empty() {
                vec![]
            } else {
                // Return player 0's visibility by default
                self.visibility.first().cloned().unwrap_or_default()
            },
            msg_type: None,
            players: None,
        }
    }

    pub fn compute_diff(&self) -> StateDiff {
        let units_moved: Vec<UnitMove> = self.units.iter().filter_map(|u| {
            if let Some(old) = self.snapshot_units.iter().find(|o| o.id == u.id) {
                if old.x != u.x || old.y != u.y || old.status != u.status {
                    return Some(UnitMove {
                        id: u.id.clone(),
                        x: u.x,
                        y: u.y,
                        status: u.status,
                    });
                }
            }
            None
        }).collect();

        let old_ids: std::collections::HashSet<&str> =
            self.snapshot_units.iter().map(|u| u.id.as_str()).collect();
        let units_spawned: Vec<UnitView> = self.units.iter()
            .filter(|u| !old_ids.contains(u.id.as_str()))
            .map(|u| UnitView {
                id: u.id.clone(),
                player_slot: u.player_slot,
                unit_type: u.unit_type,
                x: u.x,
                y: u.y,
                hp: u.hp,
                max_hp: u.max_hp,
                status: u.status,
                path: u.path.clone(),
                target_id: u.target_id.clone(),
                cargo: u.cargo,
            })
            .collect();

        let new_ids: std::collections::HashSet<&str> =
            self.units.iter().map(|u| u.id.as_str()).collect();
        let units_killed: Vec<String> = self.snapshot_units.iter()
            .filter(|u| !new_ids.contains(u.id.as_str()))
            .map(|u| u.id.clone())
            .collect();

        let old_building_ids: std::collections::HashSet<&str> =
            self.snapshot_buildings.iter().map(|b| b.id.as_str()).collect();
        let buildings_built: Vec<BuildingView> = self.buildings.iter()
            .filter(|b| !old_building_ids.contains(b.id.as_str()))
            .map(|b| BuildingView {
                id: b.id.clone(),
                player_slot: b.player_slot,
                building_type: b.building_type,
                x: b.x,
                y: b.y,
                hp: b.hp,
                max_hp: b.max_hp,
                energy: None,
            })
            .collect();

        let new_building_ids: std::collections::HashSet<&str> =
            self.buildings.iter().map(|b| b.id.as_str()).collect();
        let buildings_destroyed: Vec<String> = self.snapshot_buildings.iter()
            .filter(|b| !new_building_ids.contains(b.id.as_str()))
            .map(|b| b.id.clone())
            .collect();

        let resources_changed: Vec<(u8, i32)> = self.players.iter()
            .map(|p| (p.slot, p.energy))
            .collect();

        StateDiff {
            tick: self.tick,
            units_moved,
            units_spawned,
            units_killed,
            buildings_built,
            buildings_destroyed,
            combat_events: vec![],
            resources_changed,
            visibility_updates: vec![],
            msg_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_match_bot_vs_bot() {
        let config = MatchConfig {
            map_preset: MapPreset::Duel,
            players: vec![
                PlayerConfig { slot: 0, kind: PlayerKind::Bot, name: "Bot A".into() },
                PlayerConfig { slot: 1, kind: PlayerKind::Bot, name: "Bot B".into() },
            ],
            max_ticks: 2000,
            tick_rate_ms: 100,
        };

        let mut engine = GameEngine::new(config);

        // Both bases should exist
        assert_eq!(engine.state.buildings.len(), 2);
        assert_eq!(engine.state.players.len(), 2);
        assert_eq!(engine.state.players[0].energy, STARTING_ENERGY);

        // Run match to completion (max_ticks triggers at map_width*100 = 3200)
        let mut tick_count = 0;
        while !engine.is_finished() && tick_count < 3500 {
            engine.tick();
            tick_count += 1;
        }

        assert!(engine.is_finished(), "Match should complete within max ticks (ran {} ticks)", tick_count);
        assert!(tick_count > 0, "At least one tick should have run");

        // Snapshot should be valid
        let snapshot = engine.full_snapshot();
        assert_eq!(snapshot.map_width, 32);
        assert_eq!(snapshot.map_height, 32);
    }

    #[test]
    fn test_engine_spawn_and_move() {
        let config = MatchConfig {
            map_preset: MapPreset::Duel,
            players: vec![
                PlayerConfig { slot: 0, kind: PlayerKind::Human, name: "P1".into() },
                PlayerConfig { slot: 1, kind: PlayerKind::Human, name: "P2".into() },
            ],
            max_ticks: 2000,
            tick_rate_ms: 100,
        };

        let mut engine = GameEngine::new(config);

        // Spawn a worker for player 0
        engine.queue_command(Command::SpawnUnit {
            player_slot: 0,
            unit_type: UnitType::Worker,
        });

        let result = engine.tick();
        assert_eq!(result.diff.units_spawned.len(), 1);
        assert_eq!(engine.state.units.len(), 1);
        assert_eq!(engine.state.units[0].unit_type, UnitType::Worker);

        // Player energy should have decreased
        let p0_energy = engine.state.players[0].energy;
        assert_eq!(p0_energy, STARTING_ENERGY - 50);
    }
}
