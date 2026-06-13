use crate::chess::{
    chess::Chess,
    square::Square,
    utils::{BitBoard, bitboard_to_string},
};
use bitflags::bitflags;

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub flags: MoveFlags,
}

impl Move {
    pub fn print(&self) {
        println!("From:\n{}", bitboard_to_string(1u64 << self.from, None));
        println!("To:\n{}", bitboard_to_string(1u64 << self.to, None));
        println!("Flags: {:?}", self.flags);
    }
}

bitflags! {
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct MoveFlags: u8 {
        const CAPTURE=1;
        const EN_PASSENT=2;
        const CASTLE=4;
        const PROMOTION=8;
    }
}

impl Chess {
    // Move the piece on square idx from to square idx to
    pub fn apply_move(&mut self, m: &Move) {
        let from_sq = self.squares[m.from];

        if matches!(from_sq, Square::Empty) {
            panic!("No piece on source square");
        }

        let from_bb: BitBoard = 1u64 << m.from;
        let to_bb: BitBoard = 1u64 << m.to;

        // Handle captures first
        match m.flags {
            MoveFlags::CAPTURE => self.remove_piece_at(m.to),
            flag => {}
        }

        // Move piece on board
        self.squares[m.from] = Square::Empty;
        self.squares[m.to] = from_sq;

        // Update bitboards
        self.move_bitboard(from_sq, from_bb, to_bb);
    }
    // Remove the piece at a given square index from the board
    fn remove_piece_at(&mut self, sq_idx: usize) {
        let square: Square = self.squares[sq_idx];
        let bb: BitBoard = 1u64 << sq_idx;

        match square {
            Square::WhitePawn => self.white_pawns &= !bb,
            Square::WhiteKnight => self.white_knights &= !bb,
            Square::WhiteBishop => self.white_bishops &= !bb,
            Square::WhiteRook => self.white_rooks &= !bb,
            Square::WhiteQueen => self.white_queens &= !bb,
            Square::WhiteKing => self.white_king &= !bb,

            Square::BlackPawn => self.black_pawns &= !bb,
            Square::BlackKnight => self.black_knights &= !bb,
            Square::BlackBishop => self.black_bishops &= !bb,
            Square::BlackRook => self.black_rooks &= !bb,
            Square::BlackQueen => self.black_queens &= !bb,
            Square::BlackKing => self.black_king &= !bb,

            Square::Empty => {}
        }

        self.squares[sq_idx] = Square::Empty;
    }
    // Given a move update the bitboards
    fn move_bitboard(&mut self, piece: Square, from_bb: BitBoard, to_bb: BitBoard) {
        let update = |bb: &mut BitBoard| {
            *bb &= !from_bb;
            *bb |= to_bb;
        };

        match piece {
            Square::WhitePawn => update(&mut self.white_pawns),
            Square::WhiteKnight => update(&mut self.white_knights),
            Square::WhiteBishop => update(&mut self.white_bishops),
            Square::WhiteRook => update(&mut self.white_rooks),
            Square::WhiteQueen => update(&mut self.white_queens),
            Square::WhiteKing => update(&mut self.white_king),

            Square::BlackPawn => update(&mut self.black_pawns),
            Square::BlackKnight => update(&mut self.black_knights),
            Square::BlackBishop => update(&mut self.black_bishops),
            Square::BlackRook => update(&mut self.black_rooks),
            Square::BlackQueen => update(&mut self.black_queens),
            Square::BlackKing => update(&mut self.black_king),

            Square::Empty => unreachable!(),
        }
    }
    // Generate pseudo legal moves: Does not account for checks
    pub fn generate_pseudolegal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        self.generate_knight_moves(&mut moves);
        // self.generate_pawn_moves(&mut moves);
        // self.generate_ray_moves(&mut moves);
        // self.generate_king_moves(&mut moves);
        moves
    }
    // Filters out any illegal moves
    pub fn generate_moves(&self) -> Vec<Move> {
        self.generate_pseudolegal_moves()
            .into_iter()
            .filter(|m| !self.leaves_king_in_check(m))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::color::Color;

    #[test]
    fn test_check_detection() {
        // Position: 7r/5k2/8/8/8/8/R3K2R w - - 0 1
        // White king on e1, Black king on f7
        // White is not in check
        let chess = Chess::from_fen("7r/5k2/8/8/8/8/R3K2R w - - 0 1");
        assert!(!chess.is_active_color_in_check());

        // Position: 7r/5k2/8/8/8/8/4K2R w - - 0 1
        // White king on e2, Black rook on h8
        // White is not in check (rook doesn't attack on diagonal)
        let chess = Chess::from_fen("7r/5k2/8/8/8/8/4K2R w - - 0 1");
        assert!(!chess.is_active_color_in_check());

        // Position: r6r/5k2/8/8/8/8/4K2R w - - 0 1
        // White king on e2, Black rooks on a8 and h8
        // White is not in check
        let chess = Chess::from_fen("r6r/5k2/8/8/8/8/4K2R w - - 0 1");
        assert!(!chess.is_active_color_in_check());
    }

    #[test]
    fn test_move_legality_filters_check() {
        // Position where a piece move would expose king
        // This should be filtered out by generate_moves()
        let chess = Chess::from_fen("4k3/8/8/8/8/8/r3K2R w - - 0 1");
        // Rook on a2 pins the king if the rook moves, it exposes check

        let moves = chess.generate_moves();
        // All generated moves should be legal (none leave king in check)
        for mv in &moves {
            let mut temp = chess.clone();
            temp.apply_move(mv);
            temp.active_color = Color::White; // Simulate other side's turn to check
            assert!(
                !temp.is_active_color_in_check(),
                "Move {:?} was generated but leaves king in check!",
                mv
            );
        }
    }
}
