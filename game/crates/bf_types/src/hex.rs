use crate::Position;

/// Hex grid constants for pointy-top layout.
pub const HEX_SIZE: f32 = 2.0;
/// Width = sqrt(3) * size.
pub const HEX_W: f32 = HEX_SIZE * 1.732;
/// Height = 2 * size.
pub const HEX_H: f32 = HEX_SIZE * 2.0;

/// Convert hex grid (col, row) using pointy-top odd-r offset layout to pixel coords.
/// Odd rows shift right by half a hex width.
/// Returns (x, z) where z is negated for Bevy's coordinate system.
pub fn hex_to_pixel(col: i32, row: i32) -> (f32, f32) {
    let offset = if row % 2 != 0 { HEX_W * 0.5 } else { 0.0 };
    let px = col as f32 * HEX_W + offset;
    let pz = -(row as f32 * HEX_H * 0.75);
    (px, pz)
}

/// Cube coordinates for hex math.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CubeCoord {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}

/// Convert odd-r offset coordinates to cube coordinates.
pub fn offset_to_cube(col: i32, row: i32) -> CubeCoord {
    let q = col - (row - (row & 1)) / 2;
    let r = row;
    let s = -q - r;
    CubeCoord { q, r, s }
}

/// Hex distance between two positions using cube coordinates.
pub fn hex_distance(a: Position, b: Position) -> u32 {
    let ac = offset_to_cube(a.x, a.y);
    let bc = offset_to_cube(b.x, b.y);
    ((ac.q - bc.q).unsigned_abs() + (ac.r - bc.r).unsigned_abs() + (ac.s - bc.s).unsigned_abs())
        / 2
}

/// Direction offsets for hex neighbors in odd-r offset coordinates.
/// Index 0 = even rows, index 1 = odd rows.
const EVEN_ROW_DIRS: [(i32, i32); 6] = [
    (1, 0),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
];

const ODD_ROW_DIRS: [(i32, i32); 6] = [
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, 0),
    (0, 1),
    (1, 1),
];

/// Get the 6 hex neighbors of a position (odd-r offset coordinates).
pub fn hex_neighbors(pos: Position) -> [Position; 6] {
    let dirs = if pos.y & 1 == 0 {
        &EVEN_ROW_DIRS
    } else {
        &ODD_ROW_DIRS
    };
    let mut result = [Position::default(); 6];
    for (i, &(dx, dy)) in dirs.iter().enumerate() {
        result[i] = Position::new(pos.x + dx, pos.y + dy);
    }
    result
}

/// Get all hex positions within a given radius of a center position.
pub fn hexes_in_radius(center: Position, radius: u32) -> Vec<Position> {
    let cc = offset_to_cube(center.x, center.y);
    let r = radius as i32;
    let mut result = Vec::new();

    for dq in -r..=r {
        let dr_min = (-r).max(-dq - r);
        let dr_max = r.min(-dq + r);
        for dr in dr_min..=dr_max {
            let q = cc.q + dq;
            let r_val = cc.r + dr;
            // Convert cube back to offset
            let col = q + (r_val - (r_val & 1)) / 2;
            let row = r_val;
            result.push(Position::new(col, row));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_distance_same() {
        let a = Position::new(5, 5);
        assert_eq!(hex_distance(a, a), 0);
    }

    #[test]
    fn test_hex_distance_adjacent() {
        let a = Position::new(3, 3);
        for neighbor in hex_neighbors(a) {
            assert_eq!(hex_distance(a, neighbor), 1);
        }
    }

    #[test]
    fn test_hex_neighbors_count() {
        let pos = Position::new(5, 5);
        let neighbors = hex_neighbors(pos);
        assert_eq!(neighbors.len(), 6);
    }

    #[test]
    fn test_hexes_in_radius_center() {
        let center = Position::new(5, 5);
        let hexes = hexes_in_radius(center, 0);
        assert_eq!(hexes.len(), 1);
        assert_eq!(hexes[0], center);
    }

    #[test]
    fn test_hexes_in_radius_1() {
        let center = Position::new(5, 5);
        let hexes = hexes_in_radius(center, 1);
        // Center + 6 neighbors = 7
        assert_eq!(hexes.len(), 7);
        assert!(hexes.contains(&center));
    }

    #[test]
    fn test_offset_to_cube_roundtrip() {
        // Verify cube coordinate constraint q + r + s = 0
        for row in 0..10 {
            for col in 0..10 {
                let c = offset_to_cube(col, row);
                assert_eq!(c.q + c.r + c.s, 0, "Cube constraint violated at ({}, {})", col, row);
            }
        }
    }
}
