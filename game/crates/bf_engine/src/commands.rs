use bf_types::*;
use crate::{GameState, Unit};

pub fn execute_commands(state: &mut GameState, commands: Vec<Command>) {
    for cmd in commands {
        execute_command(state, cmd);
    }
}

fn execute_command(state: &mut GameState, cmd: Command) {
    match cmd {
        Command::SpawnUnit { player_slot, unit_type } => {
            execute_spawn_unit(state, player_slot, unit_type);
        }
        Command::MoveUnit { player_slot, unit_id, target_x, target_y } => {
            execute_move_unit(state, player_slot, &unit_id, target_x, target_y);
        }
        Command::AttackTarget { player_slot, unit_id, target_id } => {
            execute_attack_target(state, player_slot, &unit_id, &target_id);
        }
        Command::Harvest { player_slot, unit_id, resource_id } => {
            execute_harvest(state, player_slot, &unit_id, &resource_id);
        }
    }
}

fn execute_spawn_unit(state: &mut GameState, player_slot: u8, unit_type: UnitType) {
    let stats = unit_stats(unit_type);

    // Check energy and base before mutating
    let has_energy = state.players.iter().any(|p| p.slot == player_slot && p.energy >= stats.cost);
    if !has_energy { return; }

    let base = match state.get_player_base(player_slot) {
        Some(b) => (b.x, b.y),
        None => return,
    };

    // Now mutate
    let player = state.players.iter_mut().find(|p| p.slot == player_slot).unwrap();
    player.energy -= stats.cost;

    let id = uuid::Uuid::new_v4().to_string();
    state.units.push(Unit {
        id,
        player_slot,
        unit_type,
        x: base.0,
        y: base.1,
        hp: stats.hp,
        max_hp: stats.hp,
        status: UnitStatus::Idle,
        path: vec![],
        target_id: None,
        cargo: 0,
    });
}

fn execute_move_unit(state: &mut GameState, player_slot: u8, unit_id: &str, target_x: i32, target_y: i32) {
    let unit = match state.units.iter_mut().find(|u| u.id == unit_id && u.player_slot == player_slot) {
        Some(u) => u,
        None => return,
    };
    let start = Position::new(unit.x, unit.y);
    let end = Position::new(target_x, target_y);
    let path = crate::pathfinding::find_path(state, start, end);
    // Need to re-borrow unit after borrowing state
    if let Some(unit) = state.units.iter_mut().find(|u| u.id == unit_id) {
        unit.path = path;
        unit.status = UnitStatus::Moving;
        unit.target_id = None;
    }
}

fn execute_attack_target(state: &mut GameState, player_slot: u8, unit_id: &str, target_id: &str) {
    // Validate target exists
    let target_exists = state.units.iter().any(|u| u.id == target_id)
        || state.buildings.iter().any(|b| b.id == target_id);
    if !target_exists {
        return;
    }

    if let Some(unit) = state.units.iter_mut().find(|u| u.id == unit_id && u.player_slot == player_slot) {
        unit.target_id = Some(target_id.to_string());
        unit.status = UnitStatus::Attacking;
    }
}

fn execute_harvest(state: &mut GameState, player_slot: u8, unit_id: &str, resource_id: &str) {
    let resource_pos = state.resource_nodes.iter()
        .find(|r| r.id == resource_id)
        .map(|r| (r.x, r.y));

    let resource_pos = match resource_pos {
        Some(p) => p,
        None => return,
    };

    let unit = match state.units.iter_mut().find(|u| u.id == unit_id && u.player_slot == player_slot) {
        Some(u) => u,
        None => return,
    };

    if unit.unit_type != UnitType::Worker {
        return;
    }

    let dist = hex_distance(
        Position::new(unit.x, unit.y),
        Position::new(resource_pos.0, resource_pos.1),
    );

    if dist > 1 {
        // Need to pathfind to resource
        let start = Position::new(unit.x, unit.y);
        let end = Position::new(resource_pos.0, resource_pos.1);
        // Re-borrow for pathfinding
        let path = crate::pathfinding::find_path(state, start, end);
        if let Some(unit) = state.units.iter_mut().find(|u| u.id == unit_id) {
            unit.path = path;
            unit.status = UnitStatus::Moving;
            unit.target_id = Some(resource_id.to_string());
        }
    } else {
        unit.status = UnitStatus::Harvesting;
        unit.target_id = Some(resource_id.to_string());
    }
}

pub fn resolve_movement(state: &mut GameState) {
    for unit in state.units.iter_mut() {
        if unit.status != UnitStatus::Moving || unit.path.is_empty() {
            continue;
        }
        let stats = unit_stats(unit.unit_type);
        let mut steps = stats.speed;

        while steps > 0 && !unit.path.is_empty() {
            let next = unit.path[0];
            // Check if blocked (but allow moving through occupied tiles for simplicity)
            if state.terrain.get(next.y as usize)
                .and_then(|row| row.get(next.x as usize))
                .map(|t| tile_movement_cost(t.tile_type).is_infinite())
                .unwrap_or(true)
            {
                unit.path.clear();
                unit.status = UnitStatus::Idle;
                break;
            }
            unit.x = next.x;
            unit.y = next.y;
            unit.path.remove(0);
            steps -= 1;
        }

        if unit.path.is_empty() {
            if unit.target_id.is_some() && unit.status == UnitStatus::Moving {
                // Was moving to a harvest target
                unit.status = UnitStatus::Harvesting;
            } else if unit.status == UnitStatus::Moving {
                unit.status = UnitStatus::Idle;
            }
        }
    }
}

pub fn resolve_combat(state: &mut GameState) -> Vec<CombatEvent> {
    let mut damage_queue: Vec<(String, i32, bool)> = Vec::new(); // (targetId, damage, isUnit)
    let mut events: Vec<CombatEvent> = Vec::new();

    // Collect attacking units info
    let attackers: Vec<(String, u8, UnitType, i32, i32, Option<String>)> = state.units.iter()
        .filter(|u| u.status == UnitStatus::Attacking && u.target_id.is_some())
        .map(|u| (u.id.clone(), u.player_slot, u.unit_type, u.x, u.y, u.target_id.clone()))
        .collect();

    for (attacker_id, _player_slot, unit_type, ax, ay, target_id_opt) in &attackers {
        let target_id = match target_id_opt {
            Some(id) => id,
            None => continue,
        };
        let stats = unit_stats(*unit_type);

        // Check if target is a unit
        if let Some(target) = state.units.iter().find(|u| u.id == *target_id) {
            let dist = hex_distance(
                Position::new(*ax, *ay),
                Position::new(target.x, target.y),
            );
            if dist <= stats.range {
                damage_queue.push((target_id.clone(), stats.damage, true));
                events.push(CombatEvent {
                    attacker_id: attacker_id.clone(),
                    target_id: target_id.clone(),
                    damage: stats.damage,
                    x: target.x,
                    y: target.y,
                });
            } else {
                // Out of range — pathfind toward target
                let start = Position::new(*ax, *ay);
                let end = Position::new(target.x, target.y);
                let path = crate::pathfinding::find_path(state, start, end);
                if let Some(unit) = state.units.iter_mut().find(|u| u.id == *attacker_id) {
                    unit.path = path;
                    unit.status = UnitStatus::Moving;
                }
            }
        } else if let Some(target) = state.buildings.iter().find(|b| b.id == *target_id) {
            let dist = hex_distance(
                Position::new(*ax, *ay),
                Position::new(target.x, target.y),
            );
            if dist <= stats.range {
                damage_queue.push((target_id.clone(), stats.damage, false));
                events.push(CombatEvent {
                    attacker_id: attacker_id.clone(),
                    target_id: target_id.clone(),
                    damage: stats.damage,
                    x: target.x,
                    y: target.y,
                });
            } else {
                let start = Position::new(*ax, *ay);
                let end = Position::new(target.x, target.y);
                let path = crate::pathfinding::find_path(state, start, end);
                if let Some(unit) = state.units.iter_mut().find(|u| u.id == *attacker_id) {
                    unit.path = path;
                    unit.status = UnitStatus::Moving;
                }
            }
        } else {
            // Target gone
            if let Some(unit) = state.units.iter_mut().find(|u| u.id == *attacker_id) {
                unit.status = UnitStatus::Idle;
                unit.target_id = None;
            }
        }
    }

    // Apply damage simultaneously
    for (target_id, damage, is_unit) in damage_queue {
        if is_unit {
            if let Some(unit) = state.units.iter_mut().find(|u| u.id == target_id) {
                unit.hp -= damage;
            }
        } else {
            if let Some(building) = state.buildings.iter_mut().find(|b| b.id == target_id) {
                building.hp -= damage;
            }
        }
    }

    // Remove dead units and buildings
    state.units.retain(|u| u.hp > 0);
    state.buildings.retain(|b| b.hp > 0);

    events
}

pub fn resolve_harvesting(state: &mut GameState) {
    // Collect worker harvest actions
    let harvesters: Vec<(String, u8, i32, i32, Option<String>, UnitStatus)> = state.units.iter()
        .filter(|u| u.unit_type == UnitType::Worker && (u.status == UnitStatus::Harvesting || u.status == UnitStatus::Returning))
        .map(|u| (u.id.clone(), u.player_slot, u.x, u.y, u.target_id.clone(), u.status))
        .collect();

    for (unit_id, player_slot, ux, uy, target_id_opt, status) in harvesters {
        if status == UnitStatus::Harvesting {
            if let Some(resource_id) = &target_id_opt {
                if let Some(resource) = state.resource_nodes.iter_mut().find(|r| r.id == *resource_id) {
                    if resource.remaining > 0 {
                        let harvest = HARVEST_AMOUNT.min(resource.remaining);
                        resource.remaining -= harvest;
                        if let Some(unit) = state.units.iter_mut().find(|u| u.id == unit_id) {
                            unit.cargo += harvest;
                            // Start returning to base
                            unit.status = UnitStatus::Returning;
                            if let Some(base) = state.get_player_base(player_slot) {
                                let base_pos = (base.x, base.y);
                                let start = Position::new(ux, uy);
                                let end = Position::new(base_pos.0, base_pos.1);
                                let path = crate::pathfinding::find_path(state, start, end);
                                // Re-borrow
                                if let Some(unit) = state.units.iter_mut().find(|u| u.id == unit_id) {
                                    unit.path = path;
                                }
                            }
                        }
                    }
                }
            }
        } else if status == UnitStatus::Returning {
            // Check if at base
            if let Some(base) = state.get_player_base(player_slot) {
                let dist = hex_distance(
                    Position::new(ux, uy),
                    Position::new(base.x, base.y),
                );
                if dist <= 1 {
                    // Deposit cargo
                    let cargo = state.units.iter().find(|u| u.id == unit_id).map(|u| u.cargo).unwrap_or(0);
                    if let Some(player) = state.players.iter_mut().find(|p| p.slot == player_slot) {
                        player.energy += cargo;
                    }
                    if let Some(unit) = state.units.iter_mut().find(|u| u.id == unit_id) {
                        unit.cargo = 0;
                        unit.status = UnitStatus::Idle;
                        unit.target_id = None;
                    }
                }
            }
        }
    }
}
