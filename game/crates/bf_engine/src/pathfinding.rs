use bf_types::*;
use crate::GameState;
use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;

#[derive(Clone, Debug)]
struct AStarNode {
    pos: Position,
    g: f64,
    f: f64,
    parent: Option<Box<AStarNode>>,
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}

/// A* pathfinding on the hex grid.
/// Returns path from start (exclusive) to end (inclusive).
pub fn find_path(state: &GameState, start: Position, end: Position) -> Vec<Position> {
    if start == end {
        return vec![];
    }
    if state.is_blocked(end.x, end.y) {
        return vec![];
    }

    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();

    let h = hex_distance(start, end) as f64;
    open_set.push(AStarNode {
        pos: start,
        g: 0.0,
        f: h,
        parent: None,
    });

    while let Some(current) = open_set.pop() {
        if current.pos == end {
            // Reconstruct path
            let mut path = vec![];
            let mut node = &current;
            path.push(node.pos);
            while let Some(ref parent) = node.parent {
                if parent.pos != start {
                    path.push(parent.pos);
                }
                node = parent;
            }
            path.reverse();
            return path;
        }

        let key = (current.pos.x, current.pos.y);
        if closed_set.contains(&key) {
            continue;
        }
        closed_set.insert(key);

        for neighbor in hex_neighbors(current.pos) {
            let nkey = (neighbor.x, neighbor.y);
            if closed_set.contains(&nkey) {
                continue;
            }
            if state.is_blocked(neighbor.x, neighbor.y) && neighbor != end {
                continue;
            }
            let move_cost = state.get_movement_cost(neighbor.x, neighbor.y);
            if move_cost.is_infinite() && neighbor != end {
                continue;
            }

            let g = current.g + if move_cost.is_infinite() { 1.0 } else { move_cost };
            let h = hex_distance(neighbor, end) as f64;

            open_set.push(AStarNode {
                pos: neighbor,
                g,
                f: g + h,
                parent: Some(Box::new(current.clone())),
            });
        }
    }

    vec![] // No path found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameState;

    fn make_state(width: u32, height: u32) -> GameState {
        let terrain = (0..height).map(|_| {
            (0..width).map(|_| Tile {
                tile_type: TileType::Grass,
                elevation: 1,
            }).collect()
        }).collect();

        GameState {
            phase: MatchPhase::Active,
            tick: 0,
            map_width: width,
            map_height: height,
            terrain,
            units: vec![],
            buildings: vec![],
            resource_nodes: vec![],
            players: vec![],
            visibility: vec![],
            command_queue: vec![],
            winner_slot: None,
            snapshot_units: vec![],
            snapshot_buildings: vec![],
        }
    }

    #[test]
    fn test_same_position() {
        let state = make_state(10, 10);
        let path = find_path(&state, Position::new(5, 5), Position::new(5, 5));
        assert!(path.is_empty());
    }

    #[test]
    fn test_adjacent_path() {
        let state = make_state(10, 10);
        let path = find_path(&state, Position::new(3, 3), Position::new(4, 3));
        assert!(!path.is_empty());
        assert_eq!(*path.last().unwrap(), Position::new(4, 3));
    }

    #[test]
    fn test_blocked_destination() {
        let mut state = make_state(10, 10);
        state.terrain[5][5] = Tile { tile_type: TileType::Mountain, elevation: 3 };
        let path = find_path(&state, Position::new(3, 3), Position::new(5, 5));
        assert!(path.is_empty());
    }
}
