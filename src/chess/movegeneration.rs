use crate::chess::{
    castling_rights::CastlingRights,
    chess::Chess,
    color::{
        Color,
        Color::{Black, White},
    },
    moves::{
        king::KING_MOVES,
        knight::KNIGHT_MOVES,
        pawn::{
            BLACK_PAWN_ATTACKS, BLACK_PAWN_MOVES_1, BLACK_PAWN_MOVES_2, WHITE_PAWN_ATTACKS,
            WHITE_PAWN_MOVES_1, WHITE_PAWN_MOVES_2,
        },
        ray::{diagonal_attacks, straight_attacks},
    },
    square::Square,
    utils::{BitBoard, bit_scan},
};
use bitflags::bitflags;

// Square indices for castling moves, used to check if squares between king and rook are empty and not attacked
const B1: usize = 1;
const C1: usize = 2;
const D1: usize = 3;

const F1: usize = 5;
const G1: usize = 6;

const B8: usize = 57;
const C8: usize = 58;
const D8: usize = 59;

const F8: usize = 61;
const G8: usize = 62;

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub flags: MoveFlags,
}

bitflags! {
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct MoveFlags: u8 {
        const CAPTURE=1<<0;
        const EN_PASSENT=1<<1;
        const CASTLE_KINGSIDE=1<<2;
        const CASTLE_QUEENSIDE=1<<3;
        const PROMOTION_QUEEN=1<<4;
        const PROMOTION_ROOK=1<<5;
        const PROMOTION_BISHOP=1<<6;
        const PROMOTION_KNIGHT=1<<7;
    }
}

impl Chess {
    // Move the piece on square idx from to square idx to
    pub fn apply_move(&mut self, m: &Move) {
        let moving_piece: Square = self.squares[m.from];
        let mut result_piece: Square = moving_piece;

        // Handle en passent
        self.en_passent = 0;
        if moving_piece == Square::WhitePawn && m.to == m.from + 16 {
            self.en_passent = 1u64 << (m.from + 8);
        }
        if moving_piece == Square::BlackPawn && m.from == m.to + 16 {
            self.en_passent = 1u64 << (m.from - 8);
        }

        // Handle castling
        self.update_castling_rights(m.from, moving_piece);

        if matches!(moving_piece, Square::Empty) {
            panic!("No piece on source square");
        }

        let from_bb: BitBoard = 1u64 << m.from;
        let to_bb: BitBoard = 1u64 << m.to;

        if m.flags.contains(MoveFlags::CAPTURE) {
            self.remove_piece_at(m.to);
        }
        if m.flags.contains(MoveFlags::PROMOTION_QUEEN) {
            if moving_piece.color() == Some(White) {
                result_piece = Square::WhiteQueen;
            } else {
                result_piece = Square::BlackQueen;
            }
        }
        if m.flags.contains(MoveFlags::PROMOTION_ROOK) {
            if moving_piece.color() == Some(White) {
                result_piece = Square::WhiteRook;
            } else {
                result_piece = Square::BlackRook;
            }
        }
        if m.flags.contains(MoveFlags::PROMOTION_BISHOP) {
            if moving_piece.color() == Some(White) {
                result_piece = Square::WhiteBishop;
            } else {
                result_piece = Square::BlackBishop;
            }
        }
        if m.flags.contains(MoveFlags::PROMOTION_KNIGHT) {
            if moving_piece.color() == Some(White) {
                result_piece = Square::WhiteKnight;
            } else {
                result_piece = Square::BlackKnight;
            }
        }
        if m.flags.contains(MoveFlags::CASTLE_KINGSIDE) {
            match self.active_color {
                White => {
                    // Move the rook as well
                    self.squares[7] = Square::Empty;
                    self.squares[5] = Square::WhiteRook;

                    self.white_rooks &= !(1u64 << 7);
                    self.white_rooks |= 1u64 << 5;
                }
                Black => {
                    // Move the rook as well
                    self.squares[63] = Square::Empty;
                    self.squares[61] = Square::BlackRook;

                    self.black_rooks &= !(1u64 << 63);
                    self.black_rooks |= 1u64 << 61;
                }
            }
        }
        if m.flags.contains(MoveFlags::CASTLE_QUEENSIDE) {
            match self.active_color {
                White => {
                    // Move the rook as well
                    self.squares[0] = Square::Empty;
                    self.squares[3] = Square::WhiteRook;

                    self.white_rooks &= !(1u64 << 0);
                    self.white_rooks |= 1u64 << 3;
                }
                Black => {
                    // Move the rook as well
                    self.squares[56] = Square::Empty;
                    self.squares[59] = Square::BlackRook;

                    self.black_rooks &= !(1u64 << 56);
                    self.black_rooks |= 1u64 << 59;
                }
            }
        }
        if m.flags.contains(MoveFlags::EN_PASSENT) {
            let captured_sq = match moving_piece.color() {
                Some(White) => m.to - 8,
                Some(Black) => m.to + 8,
                None => unreachable!(),
            };
            self.remove_piece_at(captured_sq);
        }

        // Move piece on board
        self.squares[m.from] = Square::Empty;
        self.squares[m.to] = result_piece;

        // Update bitboards
        self.remove_from_bitboard(moving_piece, from_bb);
        self.add_to_bitboard(result_piece, to_bb);

        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }
    // Remove the piece at a given square index from the board
    fn remove_piece_at(&mut self, sq_idx: usize) {
        let square: Square = self.squares[sq_idx];
        let bb: BitBoard = 1u64 << sq_idx;

        self.update_castling_rights(sq_idx, square);

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
    fn remove_from_bitboard(&mut self, piece: Square, bb: BitBoard) {
        match piece {
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

            Square::Empty => unreachable!(),
        }
    }
    fn add_to_bitboard(&mut self, piece: Square, bb: BitBoard) {
        match piece {
            Square::WhitePawn => self.white_pawns |= bb,
            Square::WhiteKnight => self.white_knights |= bb,
            Square::WhiteBishop => self.white_bishops |= bb,
            Square::WhiteRook => self.white_rooks |= bb,
            Square::WhiteQueen => self.white_queens |= bb,
            Square::WhiteKing => self.white_king |= bb,

            Square::BlackPawn => self.black_pawns |= bb,
            Square::BlackKnight => self.black_knights |= bb,
            Square::BlackBishop => self.black_bishops |= bb,
            Square::BlackRook => self.black_rooks |= bb,
            Square::BlackQueen => self.black_queens |= bb,
            Square::BlackKing => self.black_king |= bb,

            Square::Empty => unreachable!(),
        }
    }
    // Generate pseudo legal moves: Does not account for checks
    fn generate_pseudolegal_moves(&self) -> Vec<Move> {
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

        generate_en_passent_moves(pawns, self.en_passent, self.active_color, &mut moves);

        generate_castling_moves(self, &mut moves);

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

fn push_pawn_move(
    moves: &mut Vec<Move>,
    from: usize,
    to: usize,
    active_color: Color,
    capture: bool,
) {
    let is_promotion = match active_color {
        White => to >= 56,
        Black => to < 8,
    };

    let base_flags = if capture {
        MoveFlags::CAPTURE
    } else {
        MoveFlags::empty()
    };

    if is_promotion {
        moves.push(Move {
            from,
            to,
            flags: base_flags | MoveFlags::PROMOTION_QUEEN,
        });
        moves.push(Move {
            from,
            to,
            flags: base_flags | MoveFlags::PROMOTION_ROOK,
        });
        moves.push(Move {
            from,
            to,
            flags: base_flags | MoveFlags::PROMOTION_BISHOP,
        });
        moves.push(Move {
            from,
            to,
            flags: base_flags | MoveFlags::PROMOTION_KNIGHT,
        });
    } else {
        moves.push(Move {
            from,
            to,
            flags: base_flags,
        });
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

        // ---------- Single push ----------
        let mut single_pushes = match active_color {
            White => WHITE_PAWN_MOVES_1[from],
            Black => BLACK_PAWN_MOVES_1[from],
        };

        single_pushes &= !all_occ;

        while single_pushes != 0 {
            let to = bit_scan(single_pushes);

            push_pawn_move(moves, from, to, active_color, false);

            single_pushes &= single_pushes - 1;
        }

        let double_pushes = match active_color {
            White => WHITE_PAWN_MOVES_2[from],
            Black => BLACK_PAWN_MOVES_2[from],
        };

        if double_pushes != 0 {
            let to = bit_scan(double_pushes);

            let intermediate = match active_color {
                White => from + 8,
                Black => from - 8,
            };

            let intermediate_bb = 1u64 << intermediate;
            let destination_bb = 1u64 << to;

            if (all_occ & intermediate_bb) == 0 && (all_occ & destination_bb) == 0 {
                moves.push(Move {
                    from,
                    to,
                    flags: MoveFlags::empty(),
                });
            }
        }

        let mut captures = match active_color {
            White => WHITE_PAWN_ATTACKS[from] & enemy_occ,
            Black => BLACK_PAWN_ATTACKS[from] & enemy_occ,
        };

        while captures != 0 {
            let to = bit_scan(captures);

            push_pawn_move(moves, from, to, active_color, true);

            captures &= captures - 1;
        }

        remaining &= remaining - 1;
    }
}

fn generate_en_passent_moves(
    mut pawns: BitBoard,
    en_passent: BitBoard,
    active_color: Color,
    moves: &mut Vec<Move>,
) {
    if en_passent == 0 {
        return;
    }

    let ep_square: usize = bit_scan(en_passent);
    while pawns != 0 {
        let from: usize = bit_scan(pawns);
        let attacks = match active_color {
            White => WHITE_PAWN_ATTACKS[from],
            Black => BLACK_PAWN_ATTACKS[from],
        };

        if attacks & en_passent != 0 {
            moves.push(Move {
                from: from,
                to: ep_square,
                flags: MoveFlags::EN_PASSENT,
            });
        }
        pawns &= pawns - 1;
    }
}

fn generate_castling_moves(chess: &Chess, moves: &mut Vec<Move>) {
    let (king_sq, enemy_color) = match chess.active_color {
        White => (bit_scan(chess.white_king), Color::Black),
        Black => (bit_scan(chess.black_king), Color::White),
    };

    if chess.is_square_attacked_by_color(king_sq, enemy_color) {
        return; // Can't castle out of check
    }

    let all_occ = chess.white_occupancy() | chess.black_occupancy();
    match chess.active_color {
        White => {
            // White King side
            if chess
                .castling_rights
                .contains(CastlingRights::WHITEKINGSIDE)
            {
                if (all_occ & ((1u64 << F1) | (1u64 << G1))) == 0 {
                    if !chess.is_square_attacked_by_color(F1, enemy_color)
                        && !chess.is_square_attacked_by_color(G1, enemy_color)
                    {
                        moves.push(Move {
                            from: king_sq,
                            to: G1,
                            flags: MoveFlags::CASTLE_KINGSIDE,
                        });
                    }
                }
            }
            // White Queen side
            if chess
                .castling_rights
                .contains(CastlingRights::WHITEQUEENSIDE)
            {
                if (all_occ & ((1u64 << B1) | (1u64 << C1) | (1u64 << D1))) == 0 {
                    if !chess.is_square_attacked_by_color(C1, enemy_color)
                        && !chess.is_square_attacked_by_color(D1, enemy_color)
                    {
                        moves.push(Move {
                            from: king_sq,
                            to: C1,
                            flags: MoveFlags::CASTLE_QUEENSIDE,
                        });
                    }
                }
            }
        }
        Black => {
            // Black King side
            if chess
                .castling_rights
                .contains(CastlingRights::BLACKKINGSIDE)
            {
                if (all_occ & ((1u64 << F8) | (1u64 << G8))) == 0 {
                    if !chess.is_square_attacked_by_color(F8, enemy_color)
                        && !chess.is_square_attacked_by_color(G8, enemy_color)
                    {
                        moves.push(Move {
                            from: king_sq,
                            to: G8,
                            flags: MoveFlags::CASTLE_KINGSIDE,
                        });
                    }
                }
            }
            // Black Queen side
            if chess
                .castling_rights
                .contains(CastlingRights::BLACKQUEENSIDE)
            {
                if (all_occ & ((1u64 << B8) | (1u64 << C8) | (1u64 << D8))) == 0 {
                    if !chess.is_square_attacked_by_color(C8, enemy_color)
                        && !chess.is_square_attacked_by_color(D8, enemy_color)
                    {
                        moves.push(Move {
                            from: king_sq,
                            to: C8,
                            flags: MoveFlags::CASTLE_QUEENSIDE,
                        });
                    }
                }
            }
        }
    }
}
