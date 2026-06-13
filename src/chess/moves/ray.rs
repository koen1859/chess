use crate::chess::utils::{BitBoard, bit_scan, bit_scan_backward, set_bit};

const NORTH_RAYS: [BitBoard; 64] = generate_ray_family(1, 0);
const SOUTH_RAYS: [BitBoard; 64] = generate_ray_family(-1, 0);
const EAST_RAYS: [BitBoard; 64] = generate_ray_family(0, 1);
const WEST_RAYS: [BitBoard; 64] = generate_ray_family(0, -1);

const NE_RAYS: [BitBoard; 64] = generate_ray_family(1, 1);
const NW_RAYS: [BitBoard; 64] = generate_ray_family(1, -1);
const SE_RAYS: [BitBoard; 64] = generate_ray_family(-1, 1);
const SW_RAYS: [BitBoard; 64] = generate_ray_family(-1, -1);

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
    // Return the inverse of the intersection of the original ray and the blocked part
    ray ^ ray_family[blocker]
}

pub const fn straight_attacks(square: usize, occupancy: BitBoard) -> BitBoard {
    blocked_ray_attack(NORTH_RAYS[square], &NORTH_RAYS, true, occupancy)
        | blocked_ray_attack(SOUTH_RAYS[square], &SOUTH_RAYS, false, occupancy)
        | blocked_ray_attack(EAST_RAYS[square], &EAST_RAYS, true, occupancy)
        | blocked_ray_attack(WEST_RAYS[square], &WEST_RAYS, false, occupancy)
}

pub const fn diagonal_attacks(square: usize, occupancy: BitBoard) -> BitBoard {
    blocked_ray_attack(NE_RAYS[square], &NE_RAYS, true, occupancy)
        | blocked_ray_attack(NW_RAYS[square], &NW_RAYS, true, occupancy)
        | blocked_ray_attack(SE_RAYS[square], &SE_RAYS, false, occupancy)
        | blocked_ray_attack(SW_RAYS[square], &SW_RAYS, false, occupancy)
}
