use std::time::Duration;

use crate::{
    apply_undo_move::Move, chess::Chess, color::Color::White, engine::engine::Engine,
    movelist::MoveList,
};
use instant::Instant;

const MAX_DEPTH: u8 = 50;

impl Engine {
    pub fn get_best_move(&mut self, board: &mut Chess, depth: u8) -> Option<Move> {
        let mut moves = MoveList::new();
        board.generate_moves_into(board.active_color, &mut moves);

        // Game is over (checkmate or stalemate)
        if moves.is_empty() {
            return None;
        }

        // Threefold repetition is a draw
        if board.history.is_repetition(board) {
            return None;
        }

        // Move ordering: bring stored best move to front
        if let Some(entry) = self.tt.get(&board.hash) {
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
        self.tt.clear();
        let mut best_move: Option<Move> = None;

        // Iterative deepening search
        for depth in 1..=MAX_DEPTH {
            let m = self.get_best_move(board, depth);
            if self.time_up {
                break;
            }
            best_move = m;
        }

        // Reset the deadline after the search is complete
        self.deadline = None;

        best_move
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        apply_undo_move::uci_to_move, chess::Chess, engine::engine::Engine, movelist::MoveList,
    };
    use instant::Instant;

    fn apply_moves(board: &mut Chess, moves: &[&str]) {
        for m_str in moves {
            if let Some(m) = uci_to_move(m_str, board) {
                board.apply_move(&m);
            }
        }
    }

    #[test]
    fn test_pgn1_no_illegal_null_move() {
        let fen = "r3k2r/ppqn1pp1/2pbpn2/3p3p/3P4/1PP1PBP1/PB1N1P1P/R2QK2R w KQkq - 0 1";
        // UCI-encoded moves:
        let moves = [
            "a2a4", "h5h4", "g3h4", "d6h2", "h4h5", "h2d6", "e3e4", "e6e5", "a4a5", "e8f8", "c3c4",
            "a8e8", "c4c5", "d6e7", "d1c2", "e8a8",
        ];
        let mut board = Chess::from_fen(fen);
        apply_moves(&mut board, &moves);

        // First check: board should have legal moves
        let mut movelist = MoveList::new();
        board.generate_moves_into(board.active_color, &mut movelist);
        assert!(
            !movelist.is_empty(),
            "PGN 1: generate_moves_into returned empty at a non-terminal position\n{}",
            board.to_fen()
        );

        // Second check: get_best_move should return Some
        let mut engine = Engine::new();
        let best = engine.get_best_move(&mut board, 3);
        assert!(
            best.is_some(),
            "PGN 1: get_best_move returned None (bestmove 0000)"
        );

        let mut engine2 = Engine::new();
        let best_time = engine2.get_best_move_in_time(&mut board, 1000);
        assert!(
            best_time.is_some(),
            "PGN 1: get_best_move_in_time returned None"
        );
    }

    #[test]
    fn test_pgn2_no_illegal_null_move() {
        let fen = "r2q1rk1/ppp1bpp1/3p1n1p/1b2p3/4P3/2PPBN1P/PPQ2PP1/RN3RK1 w - - 0 1";
        // UCI-encoded moves:
        let moves = [
            "b1d2", "c7c5", "g1h2", "g8h7", "d2b3", "g7g5", "a1b1", "a7a5", "b1a1", "a5a4", "b3d2",
            "d8c7", "c3c4", "b5c6", "c2c3", "b7b5", "f1e1", "b5b4", "c3c2", "a8a7", "e1e2", "a7b7",
            "a1c1", "a4a3", "c1b1", "g5g4", "h3g4", "f6g4", "h2g3", "g4e3", "e2e3", "b7g8", "g3h2",
            "h7h5", "e3e2", "h5h4", "b3d2", "g8g7", "c2d2", "f6e7", "d2e3", "b6e3", "h2h3", "e7d7",
            "h3h2", "d7e6", "e2c2", "c6c2", "d2b3", "a4a3", "b3c1",
        ];
        let mut board = Chess::from_fen(fen);
        apply_moves(&mut board, &moves);

        let mut movelist = MoveList::new();
        board.generate_moves_into(board.active_color, &mut movelist);
        assert!(
            !movelist.is_empty(),
            "PGN 2: generate_moves_into returned empty at a non-terminal position"
        );

        let mut engine = Engine::new();
        let best = engine.get_best_move(&mut board, 3);
        assert!(
            best.is_some(),
            "PGN 2: get_best_move returned None (bestmove 0000)"
        );

        let mut engine2 = Engine::new();
        let best_time = engine2.get_best_move_in_time(&mut board, 1000);
        assert!(
            best_time.is_some(),
            "PGN 2: get_best_move_in_time returned None"
        );
    }

    #[test]
    fn find_best_move_time() {
        let mut chess_1 = Chess::new();
        let chess_2 = Chess::new();
        let mut engine = Engine::new();

        let start = Instant::now();

        let m = engine.get_best_move_in_time(&mut chess_1, 6_000);

        println!(
            "Completed search, evaluated {} nodes, took {} ms",
            engine.nodes,
            start.elapsed().as_millis()
        );
        println!("{:?}", m);

        assert_eq!(chess_1, chess_2);
    }

    #[test]
    fn find_best_move() {
        let mut chess_1 = Chess::new();
        let chess_2 = Chess::new();
        let mut engine = Engine::new();

        let start = Instant::now();

        let depth = 5;
        let m = engine.get_best_move(&mut chess_1, depth);

        println!(
            "Completed search up to depth {}, evaluated {} nodes, took {} ms",
            depth,
            engine.nodes,
            start.elapsed().as_millis()
        );
        println!("{:?}", m);

        assert_eq!(chess_1, chess_2);
    }

    #[test]
    fn test_ladder_mate() {
        let fen = "7k/8/8/8/8/6r1/7r/K7 w - - 0 1";
        let mut board = Chess::from_fen(fen);
        let mut engine = Engine::new();

        // White to move, Kb1 is forced
        let best = engine.get_best_move_in_time(&mut board, 1_000);
        assert!(best.is_some(), "Engine should find a move for white");
        board.apply_move(&best.unwrap());

        let mut engine2 = Engine::new();
        // Black to move: should find Rg1#
        let best = engine2.get_best_move_in_time(&mut board, 1_000);
        assert!(best.is_some(), "Engine should find a move for black");
        assert_eq!(
            best.unwrap().to_san(&board),
            "Rg1#",
            "Black should play Rg1# for checkmate"
        );
    }
}
