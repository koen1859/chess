use crate::{chess::Chess, movelist::MoveList};

pub fn sort_moves_mvv_lva(moves: &mut MoveList, board: &Chess) {
    moves.sort_by(|a, b| {
        let a_score = if a.flags.contains(crate::apply_undo_move::MoveFlags::CAPTURE) {
            let captured_piece = board.squares[a.to];
            let attacker_piece = board.squares[a.from];
            captured_piece.value() as i32 - attacker_piece.value() as i32
        } else {
            0
        };
        let b_score = if b.flags.contains(crate::apply_undo_move::MoveFlags::CAPTURE) {
            let captured_piece = board.squares[b.to];
            let attacker_piece = board.squares[b.from];
            captured_piece.value() as i32 - attacker_piece.value() as i32
        } else {
            0
        };
        b_score.cmp(&a_score) // Sort in descending order
    });
}
