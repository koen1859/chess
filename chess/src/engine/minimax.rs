use crate::{
    apply_undo_move::{Move, MoveFlags},
    chess::Chess,
    color::Color::{Black, White},
    engine::engine::{Engine, StorageFlag, StorageFlag::*},
    movelist::MoveList,
};

impl Engine {
    pub fn minimax(&mut self, board: &mut Chess, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
        self.nodes += 1;
        if self.nodes % 2048 == 0 {
            if let Some(deadline) = self.deadline {
                if instant::Instant::now() >= deadline {
                    self.time_up = true;
                }
            }
        }

        if self.time_up {
            return board.evaluate();
        }

        // Draw detection: 50-move rule
        if board.halfmove_clock >= 100 {
            return 0;
        }

        // Draw detection: threefold repetition
        if board.history.is_repetition(board) {
            return 0;
        }

        // Check if we have already analyzed this position
        if let Some((stored_depth, stored_score, _stored_best_move, flag)) =
            self.storage.get(&board.hash)
        {
            // Check if we analyzed this position on a higher depth already
            if *stored_depth >= depth {
                match *flag {
                    // If the stored score is exact, return it
                    Exact => {
                        return *stored_score;
                    }
                    // If the stored score is a lower bound, and it is lower than our current lower bound, return it
                    Lower => {
                        if *stored_score <= alpha {
                            return *stored_score;
                        }
                    }
                    // If the stored score is an upper bound, and it is higher than our current lower bound, return it
                    Upper => {
                        if *stored_score >= beta {
                            return *stored_score;
                        }
                    }
                }
            }
        }

        // Check extension: at depth 0, if king is in check, search 1 ply deeper
        if depth == 0 {
            if board.is_color_in_check(board.active_color) {
                return self.minimax(board, 1, alpha, beta);
            }
            return self.quiescence(board, alpha, beta);
        }

        // Generate moves
        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);

        // Checkmate and Stalemate
        if moves.is_empty() {
            if board.is_color_in_check(board.active_color) {
                let score = if board.active_color == White {
                    -100000 - depth as i32
                } else {
                    100000 + depth as i32
                };
                return score;
            } else {
                return 0;
            }
        }

        // Move ordering: bring stored best move to front
        if let Some((_stored_depth, _stored_score, stored_best_move, _flag)) =
            self.storage.get(&board.hash)
        {
            if let Some(m) = stored_best_move {
                for i in 0..moves.len() {
                    if *moves.get(i) == *m {
                        moves.swap(0, i);
                        break;
                    }
                }
            }
        }

        let original_alpha = alpha;
        let original_beta = beta;
        let mut best_eval: i32 = if board.active_color == White {
            i32::MIN
        } else {
            i32::MAX
        };
        let mut best_move: Option<Move> = None;

        for i in 0..moves.len() {
            let m: &Move = moves.get(i);
            let history = board.apply_move(m);
            let eval: i32 = self.minimax(board, depth - 1, alpha, beta);
            board.undo_move(&history);

            if self.time_up {
                return if best_eval == i32::MIN || best_eval == i32::MAX {
                    board.evaluate()
                } else {
                    best_eval
                };
            }

            if board.active_color == White {
                if eval > best_eval {
                    best_eval = eval;
                    best_move = Some(*m);
                    alpha = alpha.max(best_eval);
                }
            } else {
                if eval < best_eval {
                    best_eval = eval;
                    best_move = Some(*m);
                    beta = beta.min(best_eval);
                }
            }

            if beta <= alpha {
                break;
            }
        }

        // If we are not out of time, store this position with the search depth, its eval and best move
        if !self.time_up {
            let flag: StorageFlag = if best_eval <= original_alpha {
                Lower
            } else if best_eval >= original_beta {
                Upper
            } else {
                Exact
            };
            self.storage
                .insert(board.hash, (depth, best_eval, best_move, flag));
        }

        best_eval
    }

    fn quiescence(&mut self, board: &mut Chess, mut alpha: i32, mut beta: i32) -> i32 {
        self.nodes += 1;
        if self.nodes % 2048 == 0 {
            if let Some(deadline) = self.deadline {
                if instant::Instant::now() >= deadline {
                    self.time_up = true;
                }
            }
        }

        let stand_pat: i32 = board.evaluate();

        if self.time_up {
            return stand_pat;
        }
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

        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);
        let mut best_eval: i32 = stand_pat;

        match board.active_color {
            White => {
                for i in 0..moves.len() {
                    let m: &Move = moves.get(i);
                    if !m.flags.contains(MoveFlags::CAPTURE) && !board.is_check(m) {
                        continue;
                    }
                    let history = board.apply_move(m);
                    let score: i32 = self.quiescence(board, alpha, beta);
                    board.undo_move(&history);

                    if self.time_up {
                        return best_eval;
                    }

                    best_eval = best_eval.max(score);
                    alpha = alpha.max(best_eval);

                    if beta <= alpha {
                        break;
                    }
                }
            }
            Black => {
                for i in 0..moves.len() {
                    let m: &Move = moves.get(i);
                    if !m.flags.contains(MoveFlags::CAPTURE) && !board.is_check(m) {
                        continue;
                    }
                    let history = board.apply_move(m);
                    let score: i32 = self.quiescence(board, alpha, beta);
                    board.undo_move(&history);

                    if self.time_up {
                        return best_eval;
                    }

                    best_eval = best_eval.min(score);
                    beta = beta.min(best_eval);

                    if beta <= alpha {
                        break;
                    }
                }
            }
        }

        best_eval
    }
}
