use crate::chess::{
    chess::Chess, color::Color::White, engine::minimax::minimax, movegeneration::Move,
};

pub fn get_best_move(board: &Chess, depth: u8) -> Option<Move> {
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
        let eval = minimax(&next_board, depth - 1);

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
