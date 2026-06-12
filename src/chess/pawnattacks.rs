use crate::chess::color::Color;
use crate::chess::utils::{BitBoard, set_bit};

pub const WHITE_PAWN_ATTACKS: [BitBoard; 64] = generate_family(true, true);
pub const BLACK_PAWN_ATTACKS: [BitBoard; 64] = generate_family(false, true);

pub const WHITE_PAWN_MOVES: [BitBoard; 64] = generate_family(true, false);
pub const BLACK_PAWN_MOVES: [BitBoard; 64] = generate_family(false, false);

const fn generate_family(white: bool, diagonal: bool) -> [BitBoard; 64] {
    let mut attacks = [0; 64];

    let mut row = 1;
    while row <= 8 {
        let mut col = 1;

        while col <= 8 {
            let sq = ((row - 1) * 8 + (col - 1)) as usize;

            attacks[sq] = if diagonal {
                diagonal_move(row, col, white)
            } else {
                forward_move(row, col, white)
            };

            col += 1;
        }

        row += 1;
    }

    attacks
}

const fn forward_move(row: i32, col: i32, white: bool) -> BitBoard {
    let mut bitboard: BitBoard = 0;
    if row == 1 || row == 8 {
        return bitboard;
    }
    if white {
        bitboard = set_bit(bitboard, row + 1, col);
        if row == 2 {
            bitboard = set_bit(bitboard, row + 2, col);
        }
    } else {
        bitboard = set_bit(bitboard, row - 1, col);
        if row == 7 {
            bitboard = set_bit(bitboard, row - 2, col);
        }
    }
    bitboard
}

const fn diagonal_move(row: i32, col: i32, white: bool) -> BitBoard {
    let mut bitboard: BitBoard = 0;
    if row == 1 || row == 8 {
        return bitboard;
    }
    if white {
        bitboard = set_bit(bitboard, row + 1, col + 1);
        bitboard = set_bit(bitboard, row + 1, col - 1);
    } else {
        bitboard = set_bit(bitboard, row - 1, col + 1);
        bitboard = set_bit(bitboard, row - 1, col - 1);
    }
    bitboard
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard_to_string;

    #[test]
    fn print_pawn_attacks() {
        println!(
            "{}",
            bitboard_to_string(WHITE_PAWN_MOVES[9] | WHITE_PAWN_ATTACKS[9], Some(9))
        );
        println!(
            "{}",
            bitboard_to_string(WHITE_PAWN_MOVES[20] | WHITE_PAWN_ATTACKS[20], Some(20))
        );
    }
}
