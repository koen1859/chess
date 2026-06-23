use std::time::Duration;

use crate::{
    apply_undo_move::Move,
    chess::Chess,
    color::Color,
    engine::{
        engine::{Engine, SearchInfo, VALUE_MAX, VALUE_MIN},
        move_ordering::order_moves,
    },
    movelist::MoveList,
};
use instant::Instant;

pub const MAX_DEPTH: u8 = 50;

impl Engine {
    pub fn get_best_move(&mut self, board: &mut Chess, depth: u8) -> Option<Move> {
        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);

        // Game is over
        if moves.is_empty() {
            return None;
        }

        // Order root moves with TT move
        let tt_best_move = self.tt.probe(board.hash).and_then(|e| e.best_move);
        order_moves(&mut moves, board, tt_best_move, None, None, &self.history);

        let mut best_move: Option<Move> = None;
        let mut alpha = VALUE_MIN;
        let beta = VALUE_MAX;

        for i in 0..moves.len() {
            let m = *moves.get(i);
            let hist = board.apply_move(&m);

            let eval = if i == 0 {
                -self.negamax(board, depth - 1, 1, -beta, -alpha, false)
            } else {
                let score = -self.negamax(board, depth - 1, 1, -alpha - 1, -alpha, false);
                if score > alpha {
                    -self.negamax(board, depth - 1, 1, -beta, -alpha, false)
                } else {
                    score
                }
            };

            board.undo_move(&hist);

            if self.time_up {
                break;
            }

            if eval > alpha {
                alpha = eval;
                best_move = Some(m);
            }
        }

        // Store score from White's perspective for UCI output
        self.best_score = if board.active_color == Color::White {
            alpha
        } else {
            -alpha
        };

        best_move
    }

    pub fn get_best_move_in_time(
        &mut self,
        board: &mut Chess,
        max_ms: u64,
        mut info_callback: Option<&mut dyn FnMut(SearchInfo)>,
    ) -> Option<Move> {
        let start = Instant::now();
        self.deadline = Some(start + Duration::from_millis(max_ms));
        self.time_up = false;
        self.nodes = 0;
        self.tt.new_generation();
        self.clear_search_stats();
        let mut best_move: Option<Move> = None;

        for depth in 1..=MAX_DEPTH {
            let m = self.get_best_move(board, depth);
            if self.time_up {
                break;
            }
            best_move = m;

            let elapsed = start.elapsed().as_millis();
            let nps = if elapsed > 0 {
                (self.nodes as u128 * 1000 / elapsed) as u64
            } else {
                0
            };

            if let Some(ref mut cb) = info_callback {
                cb(SearchInfo {
                    depth,
                    score: self.best_score,
                    nodes: self.nodes,
                    time_ms: elapsed,
                    nps,
                    best_move_uci: best_move
                        .as_ref()
                        .map(|m| crate::apply_undo_move::move_to_uci(m))
                        .unwrap_or_default(),
                    is_mate: self.best_score.abs() > 50000,
                    mate_in: if self.best_score.abs() > 50000 {
                        let plies = if self.best_score > 0 {
                            self.best_score - 100000
                        } else {
                            -self.best_score - 100000
                        };
                        if board.active_color == Color::White {
                            if self.best_score > 0 {
                                (plies + 1) / 2
                            } else {
                                -((plies + 1) / 2)
                            }
                        } else {
                            if self.best_score > 0 {
                                -((plies + 1) / 2)
                            } else {
                                (plies + 1) / 2
                            }
                        }
                    } else {
                        0
                    },
                });
            }
            println!("Evaluated depth {}, took {} nodes", depth, self.nodes);
        }

        self.deadline = None;

        best_move
    }
}

#[cfg(test)]
mod tests {
    use crate::{chess::Chess, engine::engine::Engine};
    use instant::Instant;

    #[test]
    fn find_best_move_time() {
        let mut chess_1 = Chess::new();
        let chess_2 = Chess::new();
        // let mut chess_1 = Chess::from_fen("7k/8/8/8/8/6r1/7r/K7 w - - 0 1");
        // let chess_2 = Chess::from_fen("7k/8/8/8/8/6r1/7r/K7 w - - 0 1");
        let mut engine = Engine::new();

        let start = Instant::now();

        let m = engine.get_best_move_in_time(&mut chess_1, 30_000, None);

        println!(
            "Completed search, evaluated {} nodes, took {} ms",
            engine.nodes,
            start.elapsed().as_millis()
        );
        println!("{:?}", m);

        assert_eq!(chess_1, chess_2);
    }

    // #[test]
    // fn find_best_move() {
    //     let mut chess_1 = Chess::new();
    //     let chess_2 = Chess::new();
    //     let mut engine = Engine::new();
    //
    //     let start = Instant::now();
    //
    //     let depth = 5;
    //     let m = engine.get_best_move(&mut chess_1, depth);
    //
    //     println!(
    //         "Completed search up to depth {}, evaluated {} nodes, took {} ms",
    //         depth,
    //         engine.nodes,
    //         start.elapsed().as_millis()
    //     );
    //     println!("{}", m.unwrap().to_san(&chess_1));
    //
    //     assert_eq!(chess_1, chess_2);
    // }
    //
    // #[test]
    // fn test_ladder_mate() {
    //     let mut board =
    //         Chess::from_fen("7k/8/8/8/8/6r1/7r/K7 w - - 0 1");
    //     let mut engine = Engine::new();
    //
    //     let best = engine.get_best_move_in_time(&mut board, 1_000, None);
    //     assert!(best.is_some(), "Engine should find a move for white");
    //     board.apply_move(&best.unwrap());
    //
    //     let mut engine2 = Engine::new();
    //     let best = engine2.get_best_move_in_time(&mut board, 1_000, None);
    //     assert!(best.is_some(), "Engine should find a move for black");
    //     assert_eq!(
    //         best.unwrap().to_san(&board),
    //         "Rg1#",
    //         "Black should play Rg1# for checkmate"
    //     );
    // }
}
