use crate::{
    apply_undo_move::{History, Move, MoveFlags},
    chess::Chess,
    color::Color::{Black, White},
    engine::engine::{
        Engine, TTFlag,
        TTFlag::{Alpha, Beta, Exact},
    },
};
use instant::Instant;

impl Engine {
    pub fn minimax(&mut self, board: &mut Chess, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
        // Increment node count and check time periodically
        self.nodes += 1;
        if self.nodes % 2048 == 0 {
            if let Some(deadline) = self.deadline {
                if Instant::now() >= deadline {
                    self.time_up = true;
                }
            }
        }

        if self.time_up {
            return 0; // does not matter what we return, will be discarded
        }

        let hash: u64 = board.hash;

        if let Some(&(score, stored_depth, flag)) = self.tt.get(&hash) {
            if stored_depth >= depth {
                match flag {
                    Exact => return score,
                    Alpha => {
                        if score <= alpha {
                            return score;
                        }
                    }
                    Beta => {
                        if score >= beta {
                            return score;
                        }
                    }
                }
            }
        }

        // moves, sorted by victim value - attacker value
        let moves: Vec<Move> = board.generate_moves(board.active_color);

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
            return self.quiescence(board, alpha, beta);
        }

        let original_alpha = alpha;
        let original_beta = beta;
        let mut best_eval: i32 = if board.active_color == White {
            i32::MIN
        } else {
            i32::MAX
        };

        for m in moves {
            let history: History = board.apply_move(&m);
            let eval: i32 = self.minimax(board, depth - 1, alpha, beta);
            board.undo_move(&history);

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

        if !self.time_up {
            let flag = if best_eval <= original_alpha {
                TTFlag::Alpha
            } else if best_eval >= original_beta {
                TTFlag::Beta
            } else {
                TTFlag::Exact
            };
            self.tt.insert(hash, (best_eval, depth, flag));
        }
        best_eval
    }

    // Given a position, for all captures, play out all future trades until the position is "quiet" that no captures can be made.
    // This ensures that the engine does not see at the end of the minimax that it can take a queen, evaluate it very high while
    // its own queen can just be taken after
    fn quiescence(&mut self, board: &Chess, mut alpha: i32, mut beta: i32) -> i32 {
        self.nodes += 1;
        if self.nodes % 2048 == 0 {
            if let Some(deadline) = self.deadline {
                if instant::Instant::now() >= deadline {
                    self.time_up = true;
                }
            }
        }

        if self.time_up {
            return 0;
        }

        let stand_pat: i32 = board.evaluate();
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
                    return alpha;
                }
                if stand_pat < beta {
                    beta = stand_pat;
                }
            }
        }

        let captures: Vec<Move> = board
            .generate_moves(board.active_color)
            .into_iter()
            .filter(|m| m.flags.contains(MoveFlags::CAPTURE))
            .collect();

        let mut best_eval: i32 = stand_pat;

        for m in captures {
            let mut next_board: Chess = *board;
            next_board.apply_move(&m);
            let score: i32 = self.quiescence(&next_board, alpha, beta);

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
