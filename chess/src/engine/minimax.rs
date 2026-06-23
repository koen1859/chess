use crate::{
    apply_undo_move::{Move, MoveFlags},
    chess::Chess,
    engine::{
        engine::{Bound, Engine, MAX_PLY, VALUE_MIN},
        move_ordering::order_moves,
    },
    movelist::MoveList,
};
use crate::zobrist_hash::{black_to_move_hash, en_passant_hash, get_en_passant_index};

impl Engine {
    pub fn negamax(
        &mut self,
        board: &mut Chess,
        depth: u8,
        ply: u8,
        mut alpha: i32,
        mut beta: i32,
        can_null: bool,
    ) -> i32 {
        self.nodes += 1;
        if self.nodes % 2048 == 0 {
            if let Some(deadline) = self.deadline {
                if instant::Instant::now() >= deadline {
                    self.time_up = true;
                }
            }
        }
        if self.time_up {
            return board.evaluate_stm();
        }

        if board.halfmove_clock >= 100 {
            return 0;
        }
        if board.history.is_repetition(board) {
            return 0;
        }

        let tt_entry = self.tt.probe(board.hash);
        let tt_best_move = tt_entry.and_then(|e| e.best_move);

        if let Some(entry) = tt_entry {
            if entry.depth >= depth {
                match entry.flag {
                    Bound::Exact => return entry.score,
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        if depth == 0 {
            if board.is_color_in_check(board.active_color) {
                return self.negamax(board, 1, ply + 1, alpha, beta, false);
            }
            return self.quiescence(board, alpha, beta);
        }

        // Null move pruning
        if can_null && depth >= 3 && !board.is_color_in_check(board.active_color) {
            let old_en_passent = board.en_passent;
            let old_halfmove = board.halfmove_clock;

            board.hash ^= en_passant_hash(get_en_passant_index(board.en_passent));
            board.en_passent = 0;
            board.hash ^= en_passant_hash(8);
            board.active_color = board.active_color.other();
            board.hash ^= black_to_move_hash();

            let null_alpha = beta.wrapping_neg();
            let null_beta = -(beta.wrapping_sub(1));
            let eval = -self.negamax(
                board,
                depth.saturating_sub(3),
                ply + 1,
                null_alpha,
                null_beta,
                false,
            );

            board.active_color = board.active_color.other();
            board.hash ^= black_to_move_hash();
            board.en_passent = old_en_passent;
            board.hash ^= en_passant_hash(8);
            board.hash ^= en_passant_hash(get_en_passant_index(old_en_passent));
            board.halfmove_clock = old_halfmove;

            if eval >= beta {
                return beta;
            }
        }

        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);

        let (killer1, killer2) = if (ply as usize) < MAX_PLY {
            (
                self.killer_moves[ply as usize][0],
                self.killer_moves[ply as usize][1],
            )
        } else {
            (None, None)
        };
        order_moves(
            &mut moves,
            board,
            tt_best_move,
            killer1,
            killer2,
            &self.history,
        );

        if moves.is_empty() {
            if board.is_color_in_check(board.active_color) {
                return -100000 - depth as i32;
            }
            return 0;
        }

        let original_alpha = alpha;
        let mut best_eval = VALUE_MIN;
        let mut best_move: Option<Move> = None;

        for i in 0..moves.len() {
            let m = *moves.get(i);
            let is_capture = m.flags.contains(MoveFlags::CAPTURE);

            let reduction = if i >= 3 && depth >= 3 && !is_capture {
                ((i - 3) as u8).min(3)
            } else {
                0
            };

            let hist = board.apply_move(&m);

            let eval = if i == 0 {
                -self.negamax(board, depth - 1, ply + 1, -beta, -alpha, true)
            } else if reduction > 0 {
                let rd = depth.saturating_sub(reduction);
                let nd = rd.saturating_sub(1);
                let score =
                    -self.negamax(board, nd, ply + 1, -alpha - 1, -alpha, true);
                if score > alpha {
                    -self.negamax(board, depth - 1, ply + 1, -beta, -alpha, true)
                } else {
                    score
                }
            } else {
                let score =
                    -self.negamax(board, depth - 1, ply + 1, -alpha - 1, -alpha, true);
                if score > alpha && score < beta {
                    -self.negamax(board, depth - 1, ply + 1, -beta, -alpha, true)
                } else {
                    score
                }
            };

            board.undo_move(&hist);

            if self.time_up {
                return if best_move.is_none() {
                    board.evaluate_stm()
                } else {
                    best_eval
                };
            }

            if eval > best_eval {
                best_eval = eval;
                best_move = Some(m);
            }

            if eval > alpha {
                alpha = eval;
            }

            if alpha >= beta {
                if !is_capture && (ply as usize) < MAX_PLY {
                    if self.killer_moves[ply as usize][0] != Some((m.from, m.to)) {
                        self.killer_moves[ply as usize][1] =
                            self.killer_moves[ply as usize][0];
                        self.killer_moves[ply as usize][0] = Some((m.from, m.to));
                    }
                    self.history[m.from][m.to] += depth as i32 * depth as i32;
                }
                break;
            }
        }

        if !self.time_up {
            let flag = if best_eval <= original_alpha {
                Bound::Upper
            } else if best_eval >= beta {
                Bound::Lower
            } else {
                Bound::Exact
            };
            self.tt
                .insert(board.hash, depth, best_eval, flag, best_move);
        }

        best_eval
    }

    fn quiescence(&mut self, board: &mut Chess, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.nodes % 2048 == 0 {
            if let Some(deadline) = self.deadline {
                if instant::Instant::now() >= deadline {
                    self.time_up = true;
                }
            }
        }

        let stand_pat = board.evaluate_stm();

        if self.time_up {
            return stand_pat;
        }

        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);
        order_moves(&mut moves, board, None, None, None, &self.history);

        let mut best_eval = stand_pat;

        for i in 0..moves.len() {
            let m = *moves.get(i);
            if !m.flags.contains(MoveFlags::CAPTURE) {
                continue;
            }

            let hist = board.apply_move(&m);
            let score = -self.quiescence(board, -beta, -alpha);
            board.undo_move(&hist);

            if self.time_up {
                return best_eval;
            }

            if score > best_eval {
                best_eval = score;
            }
            if score > alpha {
                alpha = score;
            }

            if alpha >= beta {
                break;
            }
        }

        best_eval
    }
}
