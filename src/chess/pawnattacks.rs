use crate::chess::color::Color;
use crate::chess::utils::{BitBoard, bitboard_to_string, set_bit};

pub struct PawnAttacks {
    white_forward_moves: Vec<BitBoard>,
    white_diagonal_moves: Vec<BitBoard>,
    black_forward_moves: Vec<BitBoard>,
    black_diagonal_moves: Vec<BitBoard>,
}

impl PawnAttacks {
    pub fn new() -> Self {
        let mut w_forward: Vec<BitBoard> = vec![];
        let mut w_diagonal: Vec<BitBoard> = vec![];
        let mut b_forward: Vec<BitBoard> = vec![];
        let mut b_diagonal: Vec<BitBoard> = vec![];

        for row in 1..=8 {
            for col in 1..=8 {
                let w_f: BitBoard = forward_move(row, col, Color::White);
                let w_d: BitBoard = diagonal_move(row, col, Color::White);
                let b_f: BitBoard = forward_move(row, col, Color::Black);
                let b_d: BitBoard = diagonal_move(row, col, Color::Black);

                w_forward.push(w_f);
                w_diagonal.push(w_d);
                b_forward.push(b_f);
                b_diagonal.push(b_d);
            }
        }

        Self {
            white_forward_moves: w_forward,
            white_diagonal_moves: w_diagonal,

            black_forward_moves: b_forward,
            black_diagonal_moves: b_diagonal,
        }
    }
}

fn forward_move(row: i32, col: i32, color: Color) -> BitBoard {
    let mut bitboard: BitBoard = 0;
    if row == 1 || row == 8 {
        return bitboard;
    }
    match color {
        Color::White => {
            bitboard = set_bit(bitboard, row + 1, col);
            if row == 2 {
                bitboard = set_bit(bitboard, row + 2, col);
            }
        }
        Color::Black => {
            bitboard = set_bit(bitboard, row - 1, col);
            if row == 7 {
                bitboard = set_bit(bitboard, row - 2, col);
            }
        }
    }
    bitboard
}

fn diagonal_move(row: i32, col: i32, color: Color) -> BitBoard {
    let mut bitboard: BitBoard = 0;
    if row == 1 || row == 8 {
        return bitboard;
    }
    match color {
        Color::White => {
            bitboard = set_bit(bitboard, row + 1, col + 1);
            bitboard = set_bit(bitboard, row + 1, col - 1);
        }
        Color::Black => {
            bitboard = set_bit(bitboard, row - 1, col + 1);
            bitboard = set_bit(bitboard, row - 1, col - 1);
        }
    }
    bitboard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_pawn_attacks() {
        let pawn_attacks = PawnAttacks::new();
        println!(
            "{}",
            bitboard_to_string(
                pawn_attacks.white_forward_moves[9] | pawn_attacks.white_diagonal_moves[9],
                Some(9)
            )
        );
        println!(
            "{}",
            bitboard_to_string(
                pawn_attacks.white_forward_moves[20] | pawn_attacks.white_diagonal_moves[20],
                Some(20)
            )
        );
        println!(
            "{}",
            bitboard_to_string(
                pawn_attacks.white_forward_moves[55] | pawn_attacks.white_diagonal_moves[55],
                Some(55)
            )
        );
        println!(
            "{}",
            bitboard_to_string(
                pawn_attacks.white_forward_moves[56] | pawn_attacks.white_diagonal_moves[56],
                Some(56)
            )
        );
    }
}
