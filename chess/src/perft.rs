use crate::chess::Chess;
use crate::movelist::MoveList;

/// Perft (performance test) — counts the number of legal moves at a given
/// depth. Used to verify move generation correctness against known values.
pub fn perft(board: &mut Chess, depth: u32) -> u64 {
    let mut moves = MoveList::new();
    board.generate_moves_into(board.active_color, &mut moves);

    if depth <= 1 {
        return moves.len() as u64;
    }

    let mut count = 0u64;
    for i in 0..moves.len() {
        let m = *moves.get(i);
        let history = board.apply_move(&m);
        count += perft(board, depth - 1);
        board.undo_move(&history);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    // Starting position
    // rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    #[test]
    fn perft_starting_d1() {
        let mut board = Chess::new();
        assert_eq!(perft(&mut board, 1), 20);
    }

    #[test]
    fn perft_starting_d2() {
        let mut board = Chess::new();
        assert_eq!(perft(&mut board, 2), 400);
    }

    #[test]
    fn perft_starting_d3() {
        let mut board = Chess::new();
        assert_eq!(perft(&mut board, 3), 8902);
    }

    #[test]
    fn perft_starting_d4() {
        let mut board = Chess::new();
        assert_eq!(perft(&mut board, 4), 197281);
    }

    // Position 2 (Kiwi Pete)
    // r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1
    #[test]
    fn perft_kiwipete_d1() {
        let mut board = Chess::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&mut board, 1), 48);
    }

    #[test]
    fn perft_kiwipete_d2() {
        let mut board = Chess::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&mut board, 2), 2039);
    }

    #[test]
    fn perft_kiwipete_d3() {
        let mut board = Chess::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&mut board, 3), 97862);
    }

    // Position 3
    // 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1
    #[test]
    fn perft_pos3_d1() {
        let mut board = Chess::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&mut board, 1), 14);
    }

    #[test]
    fn perft_pos3_d2() {
        let mut board = Chess::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&mut board, 2), 191);
    }

    #[test]
    fn perft_pos3_d3() {
        let mut board = Chess::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&mut board, 3), 2812);
    }

    #[test]
    fn perft_pos3_d4() {
        let mut board = Chess::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&mut board, 4), 43238);
    }

    // Position 4
    // r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1
    #[test]
    fn perft_pos4_d1() {
        let mut board =
            Chess::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(perft(&mut board, 1), 6);
    }

    #[test]
    fn perft_pos4_d2() {
        let mut board =
            Chess::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(perft(&mut board, 2), 264);
    }

    #[test]
    fn perft_pos4_d3() {
        let mut board =
            Chess::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(perft(&mut board, 3), 9467);
    }

    // Position 5
    // rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8
    #[test]
    fn perft_pos5_d1() {
        let mut board =
            Chess::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft(&mut board, 1), 44);
    }

    #[test]
    fn perft_pos5_d2() {
        let mut board =
            Chess::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft(&mut board, 2), 1486);
    }

    #[test]
    fn perft_pos5_d3() {
        let mut board =
            Chess::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft(&mut board, 3), 62379);
    }

    // Position 6
    // r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10
    #[test]
    fn perft_pos6_d1() {
        let mut board =
            Chess::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(perft(&mut board, 1), 46);
    }

    #[test]
    fn perft_pos6_d2() {
        let mut board =
            Chess::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(perft(&mut board, 2), 2079);
    }

    #[test]
    fn perft_pos6_d3() {
        let mut board =
            Chess::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(perft(&mut board, 3), 89890);
    }
}
