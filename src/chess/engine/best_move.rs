use crate::chess::{
    chess::Chess, color::Color::White, engine::engine::Engine, movegeneration::Move,
};
use instant::Instant;

impl Engine {
    pub fn get_best_move(&mut self, board: &Chess, depth: u8) -> Option<Move> {
        let moves = board.generate_moves();
        if moves.is_empty() {
            return None; // Game is over (checkmate or stalemate)
        }

        let mut best_move = None;
        let is_white = board.active_color == White;
        let mut best_eval = if is_white { i32::MIN } else { i32::MAX };

        for m in moves {
            let mut next_board = *board;
            next_board.apply_move(&m);

            // Find out what the opponent can achieve if we play this move
            let eval = self.minimax(&next_board, depth - 1, i32::MIN, i32::MAX);

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

    pub fn get_best_move_in_time(&mut self, board: &Chess, max_ms: u32) -> Option<Move> {
        let start = Instant::now();
        let mut best_move = None;
        let mut depth = 1;

        while depth <= 10 {
            if let Some(m) = self.get_best_move(board, depth as u8) {
                best_move = Some(m)
            }
            depth += 1;
            if start.elapsed().as_millis() as u32 > max_ms {
                break;
            }
        }
        best_move
    }
}
