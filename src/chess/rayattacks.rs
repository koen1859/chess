use crate::chess::utils::{BitBoard, bit_scan, bit_scan_backward, set_bit};

pub const RAYS: Rays = Rays::new();

pub struct Rays {
    pub north: [BitBoard; 64],
    pub south: [BitBoard; 64],
    pub east: [BitBoard; 64],
    pub west: [BitBoard; 64],

    pub northeast: [BitBoard; 64],
    pub northwest: [BitBoard; 64],
    pub southeast: [BitBoard; 64],
    pub southwest: [BitBoard; 64],
}

impl Rays {
    const fn new() -> Self {
        Rays {
            north: generate_ray_family(1, 0),
            south: generate_ray_family(-1, 0),
            east: generate_ray_family(0, 1),
            west: generate_ray_family(0, -1),

            northeast: generate_ray_family(1, 1),
            northwest: generate_ray_family(1, -1),
            southeast: generate_ray_family(-1, 1),
            southwest: generate_ray_family(-1, -1),
        }
    }
}

// Ray starting from (row, col) with dr=1 north, dr=-1 south, dc=1 east, dc=-1 west
const fn ray(row: i32, col: i32, dr: i32, dc: i32) -> BitBoard {
    let mut bitboard = 0;

    let mut r: i32 = row as i32 + dr;
    let mut c: i32 = col as i32 + dc;

    while r >= 1 && r <= 8 && c >= 1 && c <= 8 {
        bitboard = set_bit(bitboard, r, c);
        r += dr;
        c += dc;
    }

    bitboard
}

const fn generate_ray_family(dr: i32, dc: i32) -> [BitBoard; 64] {
    let mut rays = [0; 64];

    let mut row = 1;
    while row <= 8 {
        let mut col = 1;
        while col <= 8 {
            let sq = ((row - 1) * 8 + (col - 1)) as usize;
            rays[sq] = ray(row, col, dr, dc);
            col += 1;
        }
        row += 1;
    }
    rays
}

// Forward ray indicates whether bit indices increase or decrease when moving away from the source square over the ray.
const fn blocked_ray_attack(
    ray: BitBoard,
    ray_family: &[BitBoard; 64],
    forward_ray: bool,
    occupancy: BitBoard,
) -> BitBoard {
    // Find the overlap between the ray and the occupied squares
    let overlap: u64 = ray & occupancy;

    // No overlap means return the full ray.
    if overlap == 0 {
        return ray;
    }

    // Find the index of the square that is blocked
    let blocker: usize = if forward_ray {
        bit_scan(overlap)
    } else {
        bit_scan_backward(overlap)
    };

    // Get a ray in the same direction starting from the blocking square (So not including the blocking square)
    // This is logical since if a piece blocks the ray we can take that piece, so we want this square to be included in the final ray
    let ray_after = ray_family[blocker];

    // Return the inverse of the intersection of the original ray and the blocked part
    ray ^ ray_after
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard_to_string;

    #[test]
    fn make_n_ray() {
        println!(
            "{}",
            bitboard_to_string(ray(4, 4, 1, 1), Some((4 - 1) * 8 + 4 - 1))
        );
        println!(
            "{}",
            bitboard_to_string(ray(4, 4, -1, 1), Some((4 - 1) * 8 + 4 - 1))
        );
    }

    #[test]
    fn blocked_ray_forward_single_blocker() {
        let ray = ray(4, 4, 1, 1);

        // The ray squares are:
        // (5,5), (6,6), (7,7), (8,8)
        // Place a blocker at (6,6)
        let blocker = set_bit(0u64, 6, 6);
        let occupancy = blocker;

        let rays = Rays::new();
        let result = blocked_ray_attack(ray, &rays.northeast, true, occupancy);

        // Expected: only squares before and including blocker remain:
        // (5,5) only

        let mut expected = set_bit(0u64, 5, 5);
        expected = set_bit(expected, 6, 6);

        println!(
            "Ray:\n{}",
            bitboard_to_string(ray, Some((4 - 1) * 8 + 4 - 1))
        );
        println!(
            "Blocker:\n{}",
            bitboard_to_string(blocker, Some((6 - 1) * 8 + 6 - 1))
        );
        println!(
            "Expected:\n{}",
            bitboard_to_string(expected, Some((4 - 1) * 8 + 4 - 1))
        );
        println!(
            "Result:\n{}",
            bitboard_to_string(result, Some((4 - 1) * 8 + 4 - 1))
        );

        assert_eq!(
            result,
            expected,
            "\nExpected only squares before blocker\nresult:\n{}\nexpected:\n{}",
            bitboard_to_string(result, Some((4 - 1) * 8 + 4 - 1)),
            bitboard_to_string(expected, Some((4 - 1) * 8 + 4 - 1)),
        );
    }
}
