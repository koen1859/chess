use crate::chess::{
    chess::Chess,
    color::{
        Color,
        Color::{Black, White},
    },
    moves::{
        king::KING_MOVES,
        knight::KNIGHT_MOVES,
        pawn::{BLACK_PAWN_ATTACKS, BLACK_PAWN_MOVES, WHITE_PAWN_ATTACKS, WHITE_PAWN_MOVES},
        ray::{diagonal_attacks, straight_attacks},
    },
    square::Square,
    utils::{BitBoard, bit_scan, bitboard_to_string},
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
        if m.flags.contains(MoveFlags::CAPTURE) {
            self.remove_piece_at(m.to);
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

        let (rooks, knights, bishops, queens, king, pawns, own_occ, enemy_occ) =
            match self.active_color {
                White => (
                    self.white_rooks,
                    self.white_knights,
                    self.white_bishops,
                    self.white_queens,
                    self.white_king,
                    self.white_pawns,
                    self.white_occupancy(),
                    self.black_occupancy(),
                ),
                Black => (
                    self.black_rooks,
                    self.black_knights,
                    self.black_bishops,
                    self.black_queens,
                    self.black_king,
                    self.black_pawns,
                    self.black_occupancy(),
                    self.white_occupancy(),
                ),
            };
        let all_occ: BitBoard = own_occ | enemy_occ;

        generate_moves_for_piece_type(
            knights,
            own_occ,
            enemy_occ,
            |sq| KNIGHT_MOVES[sq],
            &mut moves,
        );

        generate_moves_for_piece_type(
            bishops,
            own_occ,
            enemy_occ,
            |sq| diagonal_attacks(sq, all_occ),
            &mut moves,
        );

        generate_moves_for_piece_type(
            rooks,
            own_occ,
            enemy_occ,
            |sq| straight_attacks(sq, all_occ),
            &mut moves,
        );

        generate_moves_for_piece_type(
            queens,
            own_occ,
            enemy_occ,
            |sq| straight_attacks(sq, all_occ) | diagonal_attacks(sq, all_occ),
            &mut moves,
        );

        generate_moves_for_piece_type(king, own_occ, enemy_occ, |sq| KING_MOVES[sq], &mut moves);

        generate_pawn_moves(pawns, own_occ, enemy_occ, self.active_color, &mut moves);

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

pub fn generate_moves_for_piece_type<F>(
    mut pieces: BitBoard,
    own_occ: BitBoard,
    enemy_occ: BitBoard,
    attack_fn: F,
    moves: &mut Vec<Move>,
) where
    F: Fn(usize) -> BitBoard,
{
    while pieces != 0 {
        let from: usize = bit_scan(pieces);
        let mut targets: BitBoard = attack_fn(from) & !own_occ;

        while targets != 0 {
            let to = bit_scan(targets);

            let flags = if (enemy_occ & (1u64 << to)) != 0 {
                MoveFlags::CAPTURE
            } else {
                MoveFlags::empty()
            };
            moves.push(Move {
                from: from,
                to: to,
                flags: flags,
            });
            targets &= targets - 1;
        }
        pieces &= pieces - 1;
    }
}

fn generate_pawn_moves(
    pawns: BitBoard,
    own_occ: BitBoard,
    enemy_occ: BitBoard,
    active_color: Color,
    moves: &mut Vec<Move>,
) {
    let all_occ = own_occ | enemy_occ;

    let mut remaining = pawns;

    while remaining != 0 {
        let from = bit_scan(remaining);

        let quiets = match active_color {
            White => WHITE_PAWN_MOVES[from] & !all_occ,
            Black => BLACK_PAWN_MOVES[from] & !all_occ,
        };

        let captures = match active_color {
            White => WHITE_PAWN_ATTACKS[from] & enemy_occ,
            Black => BLACK_PAWN_ATTACKS[from] & enemy_occ,
        };

        let mut targets = quiets;

        while targets != 0 {
            let to = bit_scan(targets);

            moves.push(Move {
                from,
                to,
                flags: MoveFlags::empty(),
            });

            targets &= targets - 1;
        }

        let mut targets = captures;

        while targets != 0 {
            let to = bit_scan(targets);

            moves.push(Move {
                from,
                to,
                flags: MoveFlags::CAPTURE,
            });

            targets &= targets - 1;
        }

        remaining &= remaining - 1;
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
