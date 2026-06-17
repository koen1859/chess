use crate::{
    castling_rights::CastlingRights,
    chess::Chess,
    color::Color::{self, Black, White},
    square::Square,
    utils::BitBoard,
    zobrist_hash::{
        black_to_move_hash, castling_hash, en_passant_hash, get_en_passant_index, piece_hash,
        square_to_piece_index,
    },
};
use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

        // Store this game's hash before the move in the game_history
        self.game_history.push(self.hash);

        let moving_piece: Square = self.squares[m.from];
        debug_assert!(
            !matches!(moving_piece, Square::Empty),
            "apply_move: moving_piece is Empty at from={} active_color={:?} hash={:#x}",
            m.from,
            self.active_color,
            self.hash,
        );
        let mut result_piece: Square = moving_piece;

        // Update halfmove clock: if pawn move / capture, reset else + 1
        self.halfmove_clock = if m.flags.contains(MoveFlags::CAPTURE)
            || matches!(moving_piece, Square::WhitePawn | Square::BlackPawn)
        {
            0
        } else {
            self.halfmove_clock + 1
        };

        // XOR out old en passant hash
        self.hash ^= en_passant_hash(get_en_passant_index(history.en_passent));

        // Handle en passent: reset and set if double push
        self.en_passent = 0;
        if matches!(moving_piece, Square::WhitePawn | Square::BlackPawn) {
            if (m.from as isize - m.to as isize).abs() == 16 {
                self.en_passent = 1u64 << ((m.from + m.to) / 2);
            }
        }

        // XOR in new en passant hash
        self.hash ^= en_passant_hash(get_en_passant_index(self.en_passent));

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
            // XOR out captured piece
            if let Some((c, p)) = square_to_piece_index(history.captured_square) {
                self.hash ^= piece_hash(c, target_sq, p);
            }
            // Clear castling rights if a rook is captured on its starting square
            match target_sq {
                0 => self.castling_rights.remove(CastlingRights::WHITEQUEENSIDE),
                7 => self.castling_rights.remove(CastlingRights::WHITEKINGSIDE),
                56 => self.castling_rights.remove(CastlingRights::BLACKQUEENSIDE),
                63 => self.castling_rights.remove(CastlingRights::BLACKKINGSIDE),
                _ => {}
            }
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

        // XOR out old castling rights hash
        self.hash ^= castling_hash(history.castling_rights.bits() as usize);

        // Handle Castling
        if m.flags.contains(MoveFlags::CASTLE_KINGSIDE) {
            match moving_piece.color() {
                Some(White) => {
                    self.hash ^= piece_hash(0, 7, 3);
                    self.hash ^= piece_hash(0, 5, 3);
                    self.squares[7] = Square::Empty;
                    self.squares[5] = Square::WhiteRook;

                    self.white_rooks &= !(1u64 << 7);
                    self.white_rooks |= 1u64 << 5;
                }
                Some(Black) => {
                    self.hash ^= piece_hash(1, 63, 9);
                    self.hash ^= piece_hash(1, 61, 9);
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
                    self.hash ^= piece_hash(0, 0, 3);
                    self.hash ^= piece_hash(0, 3, 3);
                    self.squares[0] = Square::Empty;
                    self.squares[3] = Square::WhiteRook;

                    self.white_rooks &= !(1u64 << 0);
                    self.white_rooks |= 1u64 << 3;
                }
                Some(Black) => {
                    self.hash ^= piece_hash(1, 56, 9);
                    self.hash ^= piece_hash(1, 59, 9);
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

        // XOR in new castling rights hash
        self.hash ^= castling_hash(self.castling_rights.bits() as usize);

        // XOR out moving piece from source
        if let Some((c, p)) = square_to_piece_index(moving_piece) {
            self.hash ^= piece_hash(c, m.from, p);
        }

        // Move piece on board
        self.remove_piece_at(m.from);

        // XOR in result piece at destination
        if let Some((c, p)) = square_to_piece_index(result_piece) {
            self.hash ^= piece_hash(c, m.to, p);
        }

        self.add_piece_at(m.to, result_piece);

        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => {
                self.fullmove_number += 1;
                Color::White
            }
        };

        // XOR side to move hash
        self.hash ^= black_to_move_hash();

        history
    }
    pub fn undo_move(&mut self, history: &History) {
        let m: Move = history.m;

        // Remove the final entry from the game's history
        let hash = self.game_history.pop().unwrap();
        // If the hashes are not equal, we are trying to undo a move that is not the last move made
        debug_assert_eq!(hash, history.previous_hash);

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

pub fn uci_to_move(s: &str, board: &Chess) -> Option<Move> {
    if s.len() < 4 {
        return None;
    }
    let from = crate::utils::algebraic_to_index(&s[0..2]);
    let to = crate::utils::algebraic_to_index(&s[2..4]);

    let mut flags = MoveFlags::empty();

    let moving_piece = board.squares[from];
    if moving_piece == Square::Empty {
        return None;
    }

    // Check en passant
    if board.en_passent != 0
        && moving_piece.color() == Some(Color::White)
        && to == crate::utils::bit_scan(board.en_passent)
        && matches!(moving_piece, Square::WhitePawn)
    {
        flags |= MoveFlags::EN_PASSENT | MoveFlags::CAPTURE;
    } else if board.en_passent != 0
        && moving_piece.color() == Some(Color::Black)
        && to == crate::utils::bit_scan(board.en_passent)
        && matches!(moving_piece, Square::BlackPawn)
    {
        flags |= MoveFlags::EN_PASSENT | MoveFlags::CAPTURE;
    } else if board.squares[to] != Square::Empty {
        flags |= MoveFlags::CAPTURE;
    }

    // Check castling
    if matches!(moving_piece, Square::WhiteKing) {
        if from == 4 && to == 6 {
            flags |= MoveFlags::CASTLE_KINGSIDE;
        }
        if from == 4 && to == 2 {
            flags |= MoveFlags::CASTLE_QUEENSIDE;
        }
    }
    if matches!(moving_piece, Square::BlackKing) {
        if from == 60 && to == 62 {
            flags |= MoveFlags::CASTLE_KINGSIDE;
        }
        if from == 60 && to == 58 {
            flags |= MoveFlags::CASTLE_QUEENSIDE;
        }
    }

    // Check promotion
    if s.len() >= 5 {
        match s.as_bytes()[4] {
            b'q' => flags |= MoveFlags::PROMOTION_QUEEN,
            b'r' => flags |= MoveFlags::PROMOTION_ROOK,
            b'b' => flags |= MoveFlags::PROMOTION_BISHOP,
            b'n' => flags |= MoveFlags::PROMOTION_KNIGHT,
            _ => {}
        }
        if board.squares[to] != Square::Empty && !flags.contains(MoveFlags::CAPTURE) {
            flags |= MoveFlags::CAPTURE;
        }
    }

    Some(Move { from, to, flags })
}

pub fn move_to_uci(m: &Move) -> String {
    let mut s = String::with_capacity(5);
    s.push_str(&crate::utils::index_to_algebraic(m.from));
    s.push_str(&crate::utils::index_to_algebraic(m.to));
    if m.flags.contains(MoveFlags::PROMOTION_QUEEN) {
        s.push('q');
    } else if m.flags.contains(MoveFlags::PROMOTION_ROOK) {
        s.push('r');
    } else if m.flags.contains(MoveFlags::PROMOTION_BISHOP) {
        s.push('b');
    } else if m.flags.contains(MoveFlags::PROMOTION_KNIGHT) {
        s.push('n');
    }
    s
}

#[cfg(test)]
mod tests {
    use crate::chess::Chess;

    use super::*;

    #[test]
    fn test_uci_to_move_regular() {
        let board = Chess::new();
        let m = uci_to_move("e2e4", &board).unwrap();
        assert_eq!(m.from, 12);
        assert_eq!(m.to, 28);
        assert_eq!(m.flags, MoveFlags::empty());
    }

    #[test]
    fn test_uci_to_move_en_passant() {
        let board = Chess::from_fen("rnbqkbnr/pppppppp/8/8/3Pp3/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1");
        let m = uci_to_move("e4d3", &board).unwrap();
        assert_eq!(m.from, 28);
        assert_eq!(m.to, 19);
        assert!(m.flags.contains(MoveFlags::EN_PASSENT));
        assert!(m.flags.contains(MoveFlags::CAPTURE));
    }

    #[test]
    fn test_move_to_uci() {
        let m = Move {
            from: 12,
            to: 28,
            flags: MoveFlags::empty(),
        };
        assert_eq!(move_to_uci(&m), "e2e4");
    }

    #[test]
    fn test_move_to_uci_promotion() {
        let m = Move {
            from: 55,
            to: 63,
            flags: MoveFlags::PROMOTION_QUEEN,
        };
        assert_eq!(move_to_uci(&m), "h7h8q");
    }

    #[test]
    fn test_to_fen_roundtrip() {
        let board = Chess::new();
        let fen = board.to_fen();
        let board2 = Chess::from_fen(&fen);
        assert_eq!(board, board2);
    }

    #[test]
    fn test_to_fen_kiwipete() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = Chess::from_fen(fen);
        assert_eq!(board.to_fen(), fen);
    }

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
