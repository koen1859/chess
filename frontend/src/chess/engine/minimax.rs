use crate::chess::{
    chess::Chess,
    color::Color::{Black, White},
    engine::engine::Engine,
    movegeneration::{Move, MoveFlags},
};

impl Engine {
    pub fn minimax(&mut self, board: &Chess, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
        let hash = board.zobrist_hash();

        if let Some(&(score, stored_depth)) = self.tt.get(&hash) {
            if stored_depth >= depth {
                return score;
            }
        }

        let moves = board.generate_moves();

        // Checkmate and Stalemate
        if moves.is_empty() {
            if board.is_color_in_check(board.active_color) {
                return if board.active_color == White {
                    -100000 - depth as i32
                } else {
                    100000 + depth as i32
                };
            } else {
                return 0;
            }
        }

        if depth == 0 {
            return self.quiescence(board, i32::MIN, i32::MAX);
        }

        let mut best_eval = if board.active_color == White {
            i32::MIN
        } else {
            i32::MAX
        };

        for m in moves {
            let mut next_board = *board;
            next_board.apply_move(&m);

            let eval = self.minimax(&next_board, depth - 1, alpha, beta);

            if board.active_color == White {
                best_eval = best_eval.max(eval);
                alpha = alpha.max(best_eval);
            } else {
                best_eval = best_eval.min(eval);
                beta = beta.min(best_eval);
            }

            if beta <= alpha {
                break;
            }
        }

        self.tt.insert(hash, (best_eval, depth));
        best_eval
    }

    // Given a position, for all captures, play out all future trades until the position is "quiet" that no captures can be made.
    // This ensures that the engine does not see at the end of the minimax that it can take a queen, evaluate it very high while
    // its own queen can just be taken after
    fn quiescence(&self, board: &Chess, mut alpha: i32, mut beta: i32) -> i32 {
        let stand_pat = board.evaluate();
        match board.active_color {
            White => {
                if stand_pat >= beta {
                    return beta;
                }
                if stand_pat > alpha {
                    alpha = stand_pat;
                }
            }
            Black => {
                if stand_pat <= alpha {
                    return stand_pat;
                }
                if stand_pat < beta {
                    beta = stand_pat;
                }
            }
        }

        let captures: Vec<Move> = board
            .generate_moves()
            .into_iter()
            .filter(|m| m.flags.contains(MoveFlags::CAPTURE))
            .collect();

        let mut best_eval = stand_pat;

        // Only generate capture moves
        for m in captures {
            let mut next_board = *board;
            next_board.apply_move(&m);
            let score = self.quiescence(&next_board, alpha, beta);

            match board.active_color {
                White => {
                    best_eval = best_eval.max(score);
                    alpha = alpha.max(best_eval);
                }
                Black => {
                    best_eval = best_eval.min(score);
                    beta = beta.min(best_eval);
                }
            }

            if beta <= alpha {
                break;
            }
        }
        best_eval
    }
}
