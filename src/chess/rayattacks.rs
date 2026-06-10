use crate::chess::utils::{BitBoard, bit_scan, bit_scan_backward, bitboard_to_string, set_bit};

pub struct Rays {
    north_rays: Vec<BitBoard>,
    south_rays: Vec<BitBoard>,
    east_rays: Vec<BitBoard>,
    west_rays: Vec<BitBoard>,

    northeast_rays: Vec<BitBoard>,
    northwest_rays: Vec<BitBoard>,
    southeast_rays: Vec<BitBoard>,
    southwest_rays: Vec<BitBoard>,
}

impl Rays {
    fn new() -> Self {
        let mut north_rays = vec![];
        let mut south_rays = vec![];
        let mut east_rays = vec![];
        let mut west_rays = vec![];
        let mut northeast_rays = vec![];
        let mut northwest_rays = vec![];
        let mut southeast_rays = vec![];
        let mut southwest_rays = vec![];

        for row in 1..=8 {
            for col in 1..=8 {
                north_rays.push(ray(row, col, 1, 0));
                south_rays.push(ray(row, col, -1, 0));
                east_rays.push(ray(row, col, 0, 1));
                west_rays.push(ray(row, col, 0, -1));
                northeast_rays.push(ray(row, col, 1, 1));
                northwest_rays.push(ray(row, col, 1, -1));
                southeast_rays.push(ray(row, col, -1, 1));
                southwest_rays.push(ray(row, col, -1, -1));
            }
        }

        Self {
            north_rays: north_rays,
            south_rays: south_rays,
            east_rays: east_rays,
            west_rays: west_rays,
            northeast_rays: northeast_rays,
            northwest_rays: northwest_rays,
            southeast_rays: southeast_rays,
            southwest_rays: southwest_rays,
        }
    }
}

// Ray starting from (row, col) with dr=1 north, dr=-1 south, dc=1 east, dc=-1 west
fn ray(row: usize, col: usize, dr: i32, dc: i32) -> BitBoard {
    let mut bb = 0;

    let mut r: i32 = row as i32 + dr;
    let mut c: i32 = col as i32 + dc;

    while (1..=8).contains(&r) && (1..=8).contains(&c) {
        bb = set_bit(bb, r, c);
        r += dr;
        c += dc;
    }

    bb
}

// Forward ray indicates whether bit indices increase or decrease when moving away from the source square over the ray.
fn blocked_ray_attack(
    ray: BitBoard,
    ray_family: &Vec<BitBoard>,
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
    let blocking_bit_index: usize = if forward_ray {
        bit_scan(overlap)
    } else {
        bit_scan_backward(overlap)
    };

    // Get a ray in the same direction starting from the blocking square (So not including the blocking square)
    // This is logical since if a piece blocks the ray we can take that piece, so we want this square to be included in the final ray
    let ray_after = ray_family[blocking_bit_index];

    // Return the inverse of the intersection of the original ray and the blocked part
    ray ^ ray_after
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = blocked_ray_attack(ray, &rays.northeast_rays, true, occupancy);

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
