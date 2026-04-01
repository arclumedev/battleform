use bf_types::*;
use crate::GameState;

/// Generate bot commands for a given autopilot player slot.
pub fn generate_bot_commands(state: &GameState, bot_slot: u8) -> Vec<Command> {
    let mut commands = Vec::new();

    let my_units: Vec<&crate::Unit> = state.units.iter().filter(|u| u.player_slot == bot_slot).collect();
    let workers: Vec<&&crate::Unit> = my_units.iter().filter(|u| u.unit_type == UnitType::Worker).collect();
    let soldiers: Vec<&&crate::Unit> = my_units.iter().filter(|u| u.unit_type == UnitType::Soldier).collect();
    let scouts: Vec<&&crate::Unit> = my_units.iter().filter(|u| u.unit_type == UnitType::Scout).collect();

    let idle_workers: Vec<&&crate::Unit> = workers.iter().filter(|u| u.status == UnitStatus::Idle).copied().collect();
    let idle_soldiers: Vec<&&crate::Unit> = soldiers.iter().filter(|u| u.status == UnitStatus::Idle).copied().collect();
    let idle_scouts: Vec<&&crate::Unit> = scouts.iter().filter(|u| u.status == UnitStatus::Idle).copied().collect();

    let base = state.get_player_base(bot_slot);
    let base_pos = base.map(|b| Position::new(b.x, b.y));

    let resources: Vec<&crate::ResourceNode> = state.resource_nodes.iter().filter(|r| r.remaining > 0).collect();
    let nearest_resource = base_pos.and_then(|bp| {
        resources.iter().min_by_key(|r| hex_distance(bp, Position::new(r.x, r.y)))
    });

    let enemy_buildings: Vec<&crate::Building> = state.buildings.iter()
        .filter(|b| b.player_slot != bot_slot).collect();
    let enemy_units: Vec<&crate::Unit> = state.units.iter()
        .filter(|u| u.player_slot != bot_slot).collect();

    // Spawning priorities
    if base.is_some() {
        if workers.len() < 2 {
            commands.push(Command::SpawnUnit { player_slot: bot_slot, unit_type: UnitType::Worker });
        } else if scouts.is_empty() {
            commands.push(Command::SpawnUnit { player_slot: bot_slot, unit_type: UnitType::Scout });
        } else if soldiers.len() < 4 {
            commands.push(Command::SpawnUnit { player_slot: bot_slot, unit_type: UnitType::Soldier });
        } else if workers.len() < 4 && soldiers.len() >= 2 {
            commands.push(Command::SpawnUnit { player_slot: bot_slot, unit_type: UnitType::Worker });
        } else {
            commands.push(Command::SpawnUnit { player_slot: bot_slot, unit_type: UnitType::Soldier });
        }
    }

    // Worker orders — send idle workers to harvest
    if let Some(resource) = nearest_resource {
        for worker in &idle_workers {
            commands.push(Command::Harvest {
                player_slot: bot_slot,
                unit_id: worker.id.clone(),
                resource_id: resource.id.clone(),
            });
        }
    }

    // Scout orders — patrol toward enemy base
    let nearest_enemy_base = enemy_buildings.iter()
        .filter(|b| b.building_type == BuildingType::Base)
        .min_by_key(|b| {
            base_pos.map(|bp| hex_distance(bp, Position::new(b.x, b.y))).unwrap_or(0)
        });

    for scout in &idle_scouts {
        if let Some(enemy_base) = nearest_enemy_base {
            let tx = (enemy_base.x + 3).clamp(0, state.map_width as i32 - 1);
            let ty = (enemy_base.y + 3).clamp(0, state.map_height as i32 - 1);
            commands.push(Command::MoveUnit {
                player_slot: bot_slot,
                unit_id: scout.id.clone(),
                target_x: tx,
                target_y: ty,
            });
        } else {
            commands.push(Command::MoveUnit {
                player_slot: bot_slot,
                unit_id: scout.id.clone(),
                target_x: 16,
                target_y: 16,
            });
        }
    }

    // Soldier orders — attack nearby enemies or push toward enemy base
    for soldier in &idle_soldiers {
        let soldier_pos = Position::new(soldier.x, soldier.y);
        let nearest_enemy = enemy_units.iter()
            .min_by_key(|e| hex_distance(soldier_pos, Position::new(e.x, e.y)));

        if let Some(enemy) = nearest_enemy {
            let dist = hex_distance(soldier_pos, Position::new(enemy.x, enemy.y));
            if dist < 10 {
                commands.push(Command::AttackTarget {
                    player_slot: bot_slot,
                    unit_id: soldier.id.clone(),
                    target_id: enemy.id.clone(),
                });
                continue;
            }
        }

        if let Some(enemy_base) = nearest_enemy_base {
            commands.push(Command::AttackTarget {
                player_slot: bot_slot,
                unit_id: soldier.id.clone(),
                target_id: enemy_base.id.clone(),
            });
        } else {
            // Move toward center
            commands.push(Command::MoveUnit {
                player_slot: bot_slot,
                unit_id: soldier.id.clone(),
                target_x: 16,
                target_y: 16,
            });
        }
    }

    commands
}
