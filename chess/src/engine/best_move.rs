use std::time::Duration;

use crate::{
    apply_undo_move::{History, Move},
    chess::Chess,
    color::Color::White,
    engine::engine::Engine,
};
use instant::Instant;

impl Engine {
    pub fn get_best_move(&mut self, board: &mut Chess, depth: u8) -> Option<Move> {
        let moves = board.generate_moves();
        if moves.is_empty() {
            return None; // Game is over (checkmate or stalemate)
        }

        let mut best_move: Option<Move> = None;
        let is_white: bool = board.active_color == White;
        let mut best_eval: i32 = if is_white { i32::MIN } else { i32::MAX };

        for m in moves {
            let history: History = board.apply_move(&m);

            // Find out what the opponent can achieve if we play this move
            let eval: i32 = self.minimax(board, depth - 1, i32::MIN, i32::MAX);
            board.undo_move(&history);

            // If time ran out during eval, abort entire depth
            if self.time_up {
                return None;
            }

            if is_white {
                if eval > best_eval {
                    best_eval = eval;
                    best_move = Some(m);
                }
            } else {
                if eval < best_eval {
                    best_eval = eval;
                    best_move = Some(m);
                }
            }
        }

        best_move
    }

    pub fn get_best_move_in_time(&mut self, board: &mut Chess, max_ms: u64) -> Option<Move> {
        let start = Instant::now();

        self.deadline = Some(start + Duration::from_millis(max_ms));
        self.time_up = false;
        self.nodes = 0;

        let mut best_move: Option<Move> = None;
        let mut depth: u8 = 1;

        while depth <= 10 {
            if let Some(m) = self.get_best_move(board, depth) {
                best_move = Some(m)
            } else {
                break;
            }
            if self.time_up {
                break;
            }
            depth += 1;
        }

        println!(
            "Completed search up to depth {}, evaluated {} nodes, took {} ms",
            depth - 1,
            self.nodes,
            start.elapsed().as_millis()
        );

        self.deadline = None;

        best_move
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn find_best_move_time() {
        let mut chess_1 = Chess::new();
        let chess_2 = Chess::new();
        let mut engine = Engine::new();

        let _m = engine.get_best_move_in_time(&mut chess_1, 15000);

        assert_eq!(chess_1, chess_2);
    }

    #[test]
    fn find_best_move() {
        let mut chess_1 = Chess::new();
        let chess_2 = Chess::new();
        let mut engine = Engine::new();

        let _m = engine.get_best_move(&mut chess_1, 5);

        assert_eq!(chess_1, chess_2);
    }
}
