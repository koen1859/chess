use crate::chess::{
    chess::Chess,
    color::Color::{self, Black, White},
};

pub fn minimax(board: &Chess, depth: u8) -> i32 {
    let moves = board.generate_moves();

    // Checkmate and Stalemate
    if moves.is_empty() {
        if board.is_color_in_check(board.active_color) {
            return if board.active_color == White {
                -100000 + depth as i32
            } else {
                100000 - depth as i32
            };
        } else {
            return 0;
        }
    }

    if depth == 0 {
        return board.evaluate();
    }

    match board.active_color {
        White => {
            let mut best_eval = i32::MIN;
            for m in moves {
                let mut next_board = *board;
                next_board.apply_move(&m);
                let eval = minimax(&next_board, depth - 1);
                best_eval = best_eval.max(eval);
            }
            best_eval
        }
        Black => {
            let mut best_eval = i32::MAX;
            for m in moves {
                let mut next_board = *board;
                next_board.apply_move(&m);
                // next_board.active_color is now White
                let eval = minimax(&next_board, depth - 1);
                best_eval = best_eval.min(eval);
            }
            best_eval
        }
    }
}
