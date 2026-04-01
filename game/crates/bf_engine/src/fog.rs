use bf_types::*;
use crate::GameState;

/// Recompute fog of war for all players.
pub fn compute_fog(state: &mut GameState) {
    let num_players = state.players.len();
    let w = state.map_width as usize;
    let h = state.map_height as usize;

    // Ensure visibility arrays exist for all players
    while state.visibility.len() < num_players {
        state.visibility.push(
            vec![vec![VisibilityState::Unseen; w]; h]
        );
    }

    for slot_idx in 0..num_players {
        let slot = state.players[slot_idx].slot;

        // Step 1: Downgrade all 'visible' to 'previously_seen'
        for row in state.visibility[slot_idx].iter_mut() {
            for cell in row.iter_mut() {
                if *cell == VisibilityState::Visible {
                    *cell = VisibilityState::PreviouslySeen;
                }
            }
        }

        // Collect positions to mark visible
        let mut visible_positions: Vec<(Position, u32)> = Vec::new();

        // Step 2: Units grant visibility
        for unit in &state.units {
            if unit.player_slot == slot {
                let stats = unit_stats(unit.unit_type);
                visible_positions.push((Position::new(unit.x, unit.y), stats.vision));
            }
        }

        // Step 3: Buildings grant visibility
        for building in &state.buildings {
            if building.player_slot == slot {
                visible_positions.push((Position::new(building.x, building.y), BASE_VISION));
            }
        }

        // Mark visible hexes
        for (center, radius) in visible_positions {
            for pos in hexes_in_radius(center, radius) {
                if pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < w && (pos.y as usize) < h {
                    state.visibility[slot_idx][pos.y as usize][pos.x as usize] = VisibilityState::Visible;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameState, Unit, PlayerState};

    #[test]
    fn test_fog_marks_visible_around_unit() {
        let mut state = GameState {
            phase: MatchPhase::Active,
            tick: 0,
            map_width: 10,
            map_height: 10,
            terrain: vec![vec![Tile { tile_type: TileType::Grass, elevation: 1 }; 10]; 10],
            units: vec![Unit {
                id: "u1".to_string(),
                player_slot: 0,
                unit_type: UnitType::Scout,
                x: 5,
                y: 5,
                hp: 40,
                max_hp: 40,
                status: UnitStatus::Idle,
                path: vec![],
                target_id: None,
                cargo: 0,
            }],
            buildings: vec![],
            resource_nodes: vec![],
            players: vec![PlayerState { slot: 0, energy: 200, base_id: String::new() }],
            visibility: vec![],
            command_queue: vec![],
            winner_slot: None,
            snapshot_units: vec![],
            snapshot_buildings: vec![],
        };

        compute_fog(&mut state);

        // Scout has vision 6 — center should be visible
        assert_eq!(state.visibility[0][5][5], VisibilityState::Visible);
        // Far corner should still be unseen
        assert_eq!(state.visibility[0][0][0], VisibilityState::Unseen);
    }
}
