use crate::{
    castling_rights::CastlingRights,
    chess::Chess,
    color::Color::{self, Black, White},
    square::Square,
    utils::BitBoard,
};
use bitflags::bitflags;

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

pub struct History {
    pub m: Move,
    pub captured_square: Square,

    // State before the move was applied for restoration
    pub castling_rights: CastlingRights,
    pub en_passent: BitBoard,
    pub halfmove_clock: usize,
    pub previous_hash: u64,
}

impl History {
    pub fn new(m: Move) -> Self {
        History {
            m: m,
            captured_square: Square::Empty,
            castling_rights: CastlingRights::NONE,
            en_passent: 0,
            halfmove_clock: 0,
            previous_hash: 0,
        }
    }
}

impl Chess {
    // Move the piece on square idx from to square idx to
    pub fn apply_move(&mut self, m: &Move) -> History {
        let mut history: History = History::new(*m);
        history.castling_rights = self.castling_rights;
        history.en_passent = self.en_passent;
        history.halfmove_clock = self.halfmove_clock;
        history.previous_hash = self.hash;

        let moving_piece: Square = self.squares[m.from];
        let mut result_piece: Square = moving_piece;

        // Update halfmove clock: if pawn move / capture, reset else + 1
        self.halfmove_clock = if m.flags.contains(MoveFlags::CAPTURE)
            || matches!(moving_piece, Square::WhitePawn | Square::BlackPawn)
        {
            0
        } else {
            self.halfmove_clock + 1
        };

        // Handle en passent: reset and set if double push
        self.en_passent = 0;
        if matches!(moving_piece, Square::WhitePawn | Square::BlackPawn) {
            if (m.from as isize - m.to as isize).abs() == 16 {
                self.en_passent = 1u64 << ((m.from + m.to) / 2);
            }
        }

        // Handle captures
        let mut target_sq: usize = m.to;
        if m.flags.contains(MoveFlags::CAPTURE) {
            if m.flags.contains(MoveFlags::EN_PASSENT) {
                target_sq = match moving_piece.color() {
                    Some(White) => m.to - 8,
                    Some(Black) => m.to + 8,
                    None => unreachable!(),
                };
            }
            history.captured_square = self.squares[target_sq];
            self.remove_piece_at(target_sq);
        }

        // Handle promotions
        match moving_piece {
            Square::WhitePawn => {
                if m.flags.contains(MoveFlags::PROMOTION_QUEEN) {
                    result_piece = Square::WhiteQueen;
                } else if m.flags.contains(MoveFlags::PROMOTION_ROOK) {
                    result_piece = Square::WhiteRook;
                } else if m.flags.contains(MoveFlags::PROMOTION_BISHOP) {
                    result_piece = Square::WhiteBishop;
                } else if m.flags.contains(MoveFlags::PROMOTION_KNIGHT) {
                    result_piece = Square::WhiteKnight;
                }
            }
            Square::BlackPawn => {
                if m.flags.contains(MoveFlags::PROMOTION_QUEEN) {
                    result_piece = Square::BlackQueen;
                } else if m.flags.contains(MoveFlags::PROMOTION_ROOK) {
                    result_piece = Square::BlackRook;
                } else if m.flags.contains(MoveFlags::PROMOTION_BISHOP) {
                    result_piece = Square::BlackBishop;
                } else if m.flags.contains(MoveFlags::PROMOTION_KNIGHT) {
                    result_piece = Square::BlackKnight;
                }
            }
            _ => {}
        }

        // Handle Castling
        if m.flags.contains(MoveFlags::CASTLE_KINGSIDE) {
            match moving_piece.color() {
                Some(White) => {
                    self.squares[7] = Square::Empty;
                    self.squares[5] = Square::WhiteRook;

                    self.white_rooks &= !(1u64 << 7);
                    self.white_rooks |= 1u64 << 5;
                }
                Some(Black) => {
                    self.squares[63] = Square::Empty;
                    self.squares[61] = Square::BlackRook;

                    self.black_rooks &= !(1u64 << 63);
                    self.black_rooks |= 1u64 << 61;
                }
                _ => unreachable!(),
            }
        }
        if m.flags.contains(MoveFlags::CASTLE_QUEENSIDE) {
            match moving_piece.color() {
                Some(White) => {
                    self.squares[0] = Square::Empty;
                    self.squares[3] = Square::WhiteRook;

                    self.white_rooks &= !(1u64 << 0);
                    self.white_rooks |= 1u64 << 3;
                }
                Some(Black) => {
                    self.squares[56] = Square::Empty;
                    self.squares[59] = Square::BlackRook;

                    self.black_rooks &= !(1u64 << 56);
                    self.black_rooks |= 1u64 << 59;
                }
                _ => unreachable!(),
            }
        }

        // Handle castling rights updates
        if matches!(moving_piece, Square::WhiteKing) {
            self.castling_rights.remove(CastlingRights::WHITEKINGSIDE);
            self.castling_rights.remove(CastlingRights::WHITEQUEENSIDE);
        }
        if matches!(moving_piece, Square::BlackKing) {
            self.castling_rights.remove(CastlingRights::BLACKKINGSIDE);
            self.castling_rights.remove(CastlingRights::BLACKQUEENSIDE);
        }
        if matches!(moving_piece, Square::WhiteRook) {
            if m.from == 0 {
                self.castling_rights.remove(CastlingRights::WHITEQUEENSIDE);
            } else if m.from == 7 {
                self.castling_rights.remove(CastlingRights::WHITEKINGSIDE);
            }
        }
        if matches!(moving_piece, Square::BlackRook) {
            if m.from == 56 {
                self.castling_rights.remove(CastlingRights::BLACKQUEENSIDE);
            } else if m.from == 63 {
                self.castling_rights.remove(CastlingRights::BLACKKINGSIDE);
            }
        }

        // Move piece on board
        self.remove_piece_at(m.from);
        self.add_piece_at(m.to, result_piece);

        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => {
                self.fullmove_number += 1;
                Color::White
            }
        };

        self.hash = self.zobrist_hash();

        history
    }
    pub fn undo_move(&mut self, history: &History) {
        let m: Move = history.m;
        let moving_piece: Square = self.squares[m.to];

        // Handle Castling
        if m.flags.contains(MoveFlags::CASTLE_KINGSIDE) {
            match moving_piece.color() {
                Some(White) => {
                    self.squares[7] = Square::WhiteRook;
                    self.squares[5] = Square::Empty;

                    self.white_rooks |= 1u64 << 7;
                    self.white_rooks &= !(1u64 << 5);
                }
                Some(Black) => {
                    self.squares[63] = Square::BlackRook;
                    self.squares[61] = Square::Empty;

                    self.black_rooks |= 1u64 << 63;
                    self.black_rooks &= !(1u64 << 61);
                }
                _ => unreachable!(),
            }
        }
        if m.flags.contains(MoveFlags::CASTLE_QUEENSIDE) {
            match moving_piece.color() {
                Some(White) => {
                    self.squares[0] = Square::WhiteRook;
                    self.squares[3] = Square::Empty;

                    self.white_rooks |= 1u64 << 0;
                    self.white_rooks &= !(1u64 << 3);
                }
                Some(Black) => {
                    self.squares[56] = Square::BlackRook;
                    self.squares[59] = Square::Empty;

                    self.black_rooks |= 1u64 << 56;
                    self.black_rooks &= !(1u64 << 59);
                }
                _ => unreachable!(),
            }
        }

        // Handle promotions: demote back to pawn
        let demoted_piece: Square = match moving_piece {
            Square::WhiteQueen if m.flags.contains(MoveFlags::PROMOTION_QUEEN) => Square::WhitePawn,
            Square::WhiteRook if m.flags.contains(MoveFlags::PROMOTION_ROOK) => Square::WhitePawn,
            Square::WhiteBishop if m.flags.contains(MoveFlags::PROMOTION_BISHOP) => {
                Square::WhitePawn
            }
            Square::WhiteKnight if m.flags.contains(MoveFlags::PROMOTION_KNIGHT) => {
                Square::WhitePawn
            }

            Square::BlackQueen if m.flags.contains(MoveFlags::PROMOTION_QUEEN) => Square::BlackPawn,
            Square::BlackRook if m.flags.contains(MoveFlags::PROMOTION_ROOK) => Square::BlackPawn,
            Square::BlackBishop if m.flags.contains(MoveFlags::PROMOTION_BISHOP) => {
                Square::BlackPawn
            }
            Square::BlackKnight if m.flags.contains(MoveFlags::PROMOTION_KNIGHT) => {
                Square::BlackPawn
            }

            _ => moving_piece,
        };

        // Move piece back on board
        self.remove_piece_at(m.to);
        self.add_piece_at(m.from, demoted_piece);

        // Restore captured piece
        if m.flags.contains(MoveFlags::CAPTURE) {
            let target_sq: usize = if m.flags.contains(MoveFlags::EN_PASSENT) {
                match moving_piece.color() {
                    Some(White) => m.to - 8,
                    Some(Black) => m.to + 8,
                    None => unreachable!(),
                }
            } else {
                m.to
            };
            self.add_piece_at(target_sq, history.captured_square);
        }

        // Restore state
        self.castling_rights = history.castling_rights;
        self.en_passent = history.en_passent;
        self.halfmove_clock = history.halfmove_clock;
        self.hash = history.previous_hash;
        self.active_color = match self.active_color {
            Color::White => {
                self.fullmove_number -= 1;
                Color::Black
            }
            Color::Black => Color::White,
        };
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
    fn add_piece_at(&mut self, sq_idx: usize, piece: Square) {
        self.squares[sq_idx] = piece;
        let bb: BitBoard = 1u64 << sq_idx;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_undo_move() {
        let mut chess = Chess::new();

        let m = Move {
            from: 12, // e2
            to: 28,   // e4
            flags: MoveFlags::empty(),
        };

        let history = chess.apply_move(&m);
        assert_eq!(chess.squares[12], Square::Empty);
        assert_eq!(chess.squares[28], Square::WhitePawn);

        chess.undo_move(&history);
        assert_eq!(chess.squares[12], Square::WhitePawn);
        assert_eq!(chess.squares[28], Square::Empty);
    }

    #[test]
    fn test_undo_castle() {
        let mut chess_1 = Chess::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        let chess_2 = Chess::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

        let m = Move {
            from: 4, // e1
            to: 6,   // g1
            flags: MoveFlags::CASTLE_KINGSIDE,
        };

        let history = chess_1.apply_move(&m);
        chess_1.undo_move(&history);

        assert_eq!(chess_1, chess_2);
    }

    #[test]
    fn test_undo_en_passent() {
        let mut chess_1 =
            Chess::from_fen("rnbqkbnr/pppppppp/8/8/3Pp3/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1");
        let chess_2 =
            Chess::from_fen("rnbqkbnr/pppppppp/8/8/3Pp3/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1");

        let m = Move {
            from: 28, // e4
            to: 19,   // d3
            flags: MoveFlags::EN_PASSENT | MoveFlags::CAPTURE,
        };

        let history = chess_1.apply_move(&m);
        chess_1.undo_move(&history);

        assert_eq!(chess_1, chess_2);
    }

    #[test]
    fn test_undo_promotion() {
        // Corrected FEN: White pawn on h7, empty square on h8
        let mut chess_1 =
            Chess::from_fen("rnbqkbn1/pppppppP/8/8/8/8/PPPPPPPP/RNBQKBNR w KQq - 0 1");
        let chess_2 = Chess::from_fen("rnbqkbn1/pppppppP/8/8/8/8/PPPPPPPP/RNBQKBNR w KQq - 0 1");

        let m = Move {
            from: 55, // h7
            to: 63,   // h8
            flags: MoveFlags::PROMOTION_QUEEN,
        };

        let history = chess_1.apply_move(&m);
        chess_1.undo_move(&history);

        assert_eq!(chess_1, chess_2);
    }
}
