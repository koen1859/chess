use crate::{
    apply_undo_move::{Move, MoveFlags},
    chess::Chess,
    color::Color::{Black, White},
    engine::engine::{
        Engine, TTFlag,
        TTFlag::{Alpha, Beta, Exact},
    },
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
            return 0;
        }

        // TT probe
        if let Some(entry) = self.tt_probe(board.hash) {
            if entry.depth >= depth {
                match entry.flag {
                    Exact => return entry.score,
                    Alpha => {
                        if entry.score <= alpha {
                            return entry.score;
                        }
                    }
                    Beta => {
                        if entry.score >= beta {
                            return entry.score;
                        }
                    }
                }
            }
        }

        // Generate moves into stack-allocated buffer (no heap alloc)
        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);

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

        // Check extension: at depth 0, if king is in check, search 1 ply deeper
        if depth == 0 {
            if board.is_color_in_check(board.active_color) {
                return self.minimax(board, 1, alpha, beta);
            }
            return self.quiescence(board, alpha, beta);
        }

        // Move ordering: bring TT best move to front
        if let Some(entry) = self.tt_probe(board.hash) {
            if let Some(bm) = entry.best_move {
                for i in 0..moves.len() {
                    if *moves.get(i) == bm {
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
            let m: Move = *moves.get(i);
            let history = board.apply_move(&m);
            let eval: i32 = self.minimax(board, depth - 1, alpha, beta);
            board.undo_move(&history);

            if self.time_up {
                return 0;
            }

            if board.active_color == White {
                if eval > best_eval {
                    best_eval = eval;
                    best_move = Some(m);
                    alpha = alpha.max(best_eval);
                }
            } else {
                if eval < best_eval {
                    best_eval = eval;
                    best_move = Some(m);
                    beta = beta.min(best_eval);
                }
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
            self.tt_store(board.hash, best_eval, depth, flag, best_move);
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

        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);
        let mut best_eval: i32 = stand_pat;

        match board.active_color {
            White => {
                for i in 0..moves.len() {
                    let m: Move = *moves.get(i);
                    if !m.flags.contains(MoveFlags::CAPTURE) {
                        continue;
                    }
                    let history = board.apply_move(&m);
                    let score: i32 = self.quiescence(board, alpha, beta);
                    board.undo_move(&history);

                    if self.time_up {
                        return 0;
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
                    let m: Move = *moves.get(i);
                    if !m.flags.contains(MoveFlags::CAPTURE) {
                        continue;
                    }
                    let history = board.apply_move(&m);
                    let score: i32 = self.quiescence(board, alpha, beta);
                    board.undo_move(&history);

                    if self.time_up {
                        return 0;
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
