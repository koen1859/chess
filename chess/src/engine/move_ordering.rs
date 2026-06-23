use crate::{
    apply_undo_move::{Move, MoveFlags},
    chess::Chess,
    movelist::MoveList,
};

pub fn score_move(
    m: &Move,
    board: &Chess,
    tt_move: Option<Move>,
    killer1: Option<(usize, usize)>,
    killer2: Option<(usize, usize)>,
    history: &[[i32; 64]; 64],
) -> i32 {
    if let Some(tm) = tt_move {
        if tm.from == m.from && tm.to == m.to && tm.flags == m.flags {
            return 2_000_000;
        }
    }

    if m.flags.contains(MoveFlags::CAPTURE) {
        let victim = board.squares[m.to].value();
        let attacker = board.squares[m.from].value();
        return 1_000_000 + victim * 10 - attacker;
    }

    if let Some((kf, kt)) = killer1 {
        if m.from == kf && m.to == kt {
            return 900_000;
        }
    }
    if let Some((kf, kt)) = killer2 {
        if m.from == kf && m.to == kt {
            return 800_000;
        }
    }

    history[m.from][m.to]
}

pub fn order_moves(
    moves: &mut MoveList,
    board: &Chess,
    tt_move: Option<Move>,
    killer1: Option<(usize, usize)>,
    killer2: Option<(usize, usize)>,
    history: &[[i32; 64]; 64],
) {
    moves.sort_by(|a, b| {
        let a_score = score_move(a, board, tt_move, killer1, killer2, history);
        let b_score = score_move(b, board, tt_move, killer1, killer2, history);
        b_score.cmp(&a_score)
    });
}
