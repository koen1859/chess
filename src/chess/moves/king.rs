use crate::chess::utils::{BitBoard, set_bit};

pub const KING_MOVES: [BitBoard; 64] = generate_king_moves();

const fn generate_king_moves() -> [BitBoard; 64] {
    let mut attacks = [0; 64];

    let mut row = 1;
    while row <= 8 {
        let mut col = 1;
        while col <= 8 {
            attacks[((row - 1) * 8 + (col - 1)) as usize] = king_move(row, col);
            col += 1
        }
        row += 1;
    }

    attacks
}

const fn king_move(row: i32, col: i32) -> BitBoard {
    let mut bitboard: BitBoard = 0;

    bitboard = set_bit(bitboard, row + 1, col);
    bitboard = set_bit(bitboard, row - 1, col);

    bitboard = set_bit(bitboard, row, col + 1);
    bitboard = set_bit(bitboard, row, col - 1);

    bitboard = set_bit(bitboard, row + 1, col + 1);
    bitboard = set_bit(bitboard, row - 1, col - 1);

    bitboard = set_bit(bitboard, row - 1, col + 1);
    bitboard = set_bit(bitboard, row + 1, col - 1);

    bitboard
}
