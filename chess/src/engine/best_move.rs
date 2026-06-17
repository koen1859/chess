use std::time::Duration;

use crate::{
    apply_undo_move::Move, chess::Chess, color::Color::White, engine::engine::Engine,
    movelist::MoveList,
};
use instant::Instant;

const MAX_DEPTH: u8 = 50;

impl Engine {
    pub fn get_best_move(&mut self, board: &mut Chess, depth: u8) -> Option<Move> {
        let mut movelist = MoveList::new();
        board.generate_moves_into(board.active_color, &mut movelist);

        // Game is over (checkmate or stalemate)
        if movelist.is_empty() {
            return None;
        }

        // Initialize the search
        let mut best_move: Option<Move> = None;
        let is_white: bool = board.active_color == White;
        let mut best_eval: i32 = if is_white { i32::MIN } else { i32::MAX };

        for i in 0..movelist.len() {
            let m: &Move = movelist.get(i);
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
    use crate::{chess::Chess, engine::engine::Engine};
    use instant::Instant;

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
}
