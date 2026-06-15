use crate::chess::utils::{BitBoard, set_bit};

pub const WHITE_PAWN_ATTACKS: [BitBoard; 64] = generate_diagonal(true);
pub const BLACK_PAWN_ATTACKS: [BitBoard; 64] = generate_diagonal(false);

pub const WHITE_PAWN_MOVES_1: [BitBoard; 64] = generate_forward_1(true);
pub const BLACK_PAWN_MOVES_1: [BitBoard; 64] = generate_forward_1(false);

pub const WHITE_PAWN_MOVES_2: [BitBoard; 64] = generate_forward_2(true);
pub const BLACK_PAWN_MOVES_2: [BitBoard; 64] = generate_forward_2(false);

const fn generate_forward_1(white: bool) -> [BitBoard; 64] {
    let mut boards = [0; 64];

    let mut row = 1;
    while row <= 8 {
        let mut col = 1;

        while col <= 8 {
            let sq = ((row - 1) * 8 + (col - 1)) as usize;

            boards[sq] = forward_move_1(row, col, white);

            col += 1;
        }

        row += 1;
    }

    boards
}

const fn generate_forward_2(white: bool) -> [BitBoard; 64] {
    let mut boards = [0; 64];

    let mut row = 1;
    while row <= 8 {
        let mut col = 1;

        while col <= 8 {
            let sq = ((row - 1) * 8 + (col - 1)) as usize;

            boards[sq] = forward_move_2(row, col, white);

            col += 1;
        }

        row += 1;
    }

    boards
}

const fn generate_diagonal(white: bool) -> [BitBoard; 64] {
    let mut boards = [0; 64];

    let mut row = 1;
    while row <= 8 {
        let mut col = 1;

        while col <= 8 {
            let sq = ((row - 1) * 8 + (col - 1)) as usize;

            boards[sq] = diagonal_move(row, col, white);

            col += 1;
        }

        row += 1;
    }

    boards
}

const fn forward_move_1(row: i32, col: i32, white: bool) -> BitBoard {
    let mut bitboard: BitBoard = 0;
    if row == 1 || row == 8 {
        return bitboard;
    }
    if white {
        bitboard = set_bit(bitboard, row + 1, col);
    } else {
        bitboard = set_bit(bitboard, row - 1, col);
    }
    bitboard
}
const fn forward_move_2(row: i32, col: i32, white: bool) -> BitBoard {
    let mut bitboard: BitBoard = 0;
    if row == 1 || row == 8 {
        return bitboard;
    }
    if white {
        if row == 2 {
            bitboard = set_bit(bitboard, row + 2, col);
        }
    } else {
        if row == 7 {
            bitboard = set_bit(bitboard, row - 2, col);
        }
    }
    bitboard
}
const fn diagonal_move(row: i32, col: i32, white: bool) -> BitBoard {
    let mut bitboard: BitBoard = 0;
    if white {
        bitboard = set_bit(bitboard, row + 1, col + 1);
        bitboard = set_bit(bitboard, row + 1, col - 1);
    } else {
        bitboard = set_bit(bitboard, row - 1, col + 1);
        bitboard = set_bit(bitboard, row - 1, col - 1);
    }
    bitboard
}
