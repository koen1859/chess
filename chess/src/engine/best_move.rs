use std::time::Duration;

use crate::{
    apply_undo_move::Move,
    chess::Chess,
    color::Color::White,
    engine::{engine::Engine, move_ordering::sort_moves_mvv_lva},
    movelist::MoveList,
};
use instant::Instant;

const MAX_DEPTH: u8 = 50;

impl Engine {
    pub fn get_best_move(&mut self, board: &mut Chess, depth: u8) -> Option<Move> {
        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);
        sort_moves_mvv_lva(&mut moves, board);

        // Game is over (checkmate or stalemate)
        if moves.is_empty() {
            return None;
        }

        // Threefold repetition is a draw
        if board.history.is_repetition(board) {
            return None;
        }

        // Move ordering: bring stored best move to front
        if let Some(entry) = self.tt.probe(board.hash) {
            if let Some(m) = entry.best_move {
                for i in 0..moves.len() {
                    if *moves.get(i) == m {
                        moves.swap(0, i);
                        break;
                    }
                }
            }
        }

        // Initialize the search
        let mut best_move: Option<Move> = None;
        let is_white: bool = board.active_color == White;
        let mut best_eval: i32 = if is_white { i32::MIN } else { i32::MAX };

        for i in 0..moves.len() {
            let m: &Move = moves.get(i);
            let history = board.apply_move(m);

            // Find out what the opponent can achieve if we play this move
            let eval: i32 = self.minimax(board, depth - 1, i32::MIN, i32::MAX);
            board.undo_move(&history);

            if self.time_up {
                break;
            }

            if is_white {
                if eval > best_eval {
                    best_eval = eval;
                    best_move = Some(*m);
                }
            } else {
                if eval < best_eval {
                    best_eval = eval;
                    best_move = Some(*m);
                }
            }
        }
        best_move
    }

    pub fn get_best_move_in_time(&mut self, board: &mut Chess, max_ms: u64) -> Option<Move> {
        // Initialize the search
        let start = Instant::now();
        self.deadline = Some(start + Duration::from_millis(max_ms));
        self.time_up = false;
        self.nodes = 0;
        self.tt.new_generation();
        let mut best_move: Option<Move> = None;

        // Iterative deepening search
        for depth in 1..=MAX_DEPTH {
            let m = self.get_best_move(board, depth);
            if self.time_up {
                break;
            }
            best_move = m;
            println!(
                "Completed search up to depth {}, evaluated {} nodes, took {} ms.",
                depth,
                self.nodes,
                start.elapsed().as_millis()
            );
        }

        // Reset the deadline after the search is complete
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
        let mut engine = Engine::new();

        let start = Instant::now();

        let m = engine.get_best_move_in_time(&mut chess_1, 30_000);

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
    //     let fen = "7k/8/8/8/8/6r1/7r/K7 w - - 0 1";
    //     let mut board = Chess::from_fen(fen);
    //     let mut engine = Engine::new();
    //
    //     // White to move, Kb1 is forced
    //     let best = engine.get_best_move_in_time(&mut board, 1_000);
    //     assert!(best.is_some(), "Engine should find a move for white");
    //     board.apply_move(&best.unwrap());
    //
    //     let mut engine2 = Engine::new();
    //     // Black to move: should find Rg1#
    //     let best = engine2.get_best_move_in_time(&mut board, 1_000);
    //     assert!(best.is_some(), "Engine should find a move for black");
    //     assert_eq!(
    //         best.unwrap().to_san(&board),
    //         "Rg1#",
    //         "Black should play Rg1# for checkmate"
    //     );
    // }
}
