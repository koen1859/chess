use crate::utils::*;
use crate::{castling_rights::CastlingRights, color::Color, square::Square};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chess {
    pub squares: [Square; 64],

    pub active_color: Color,
    pub castling_rights: CastlingRights,
    pub en_passent: BitBoard,
    pub halfmove_clock: usize,
    pub fullmove_number: usize,

    pub white_pawns: BitBoard,
    pub white_knights: BitBoard,
    pub white_bishops: BitBoard,
    pub white_rooks: BitBoard,
    pub white_queens: BitBoard,
    pub white_king: BitBoard,

    pub black_pawns: BitBoard,
    pub black_knights: BitBoard,
    pub black_bishops: BitBoard,
    pub black_rooks: BitBoard,
    pub black_queens: BitBoard,
    pub black_king: BitBoard,

    pub hash: u64,

    // Track the current search path positions (push in apply_move, pop in undo_move)
    // Used for repetition detection during search
    pub search_path: Vec<u64>,
}

impl Chess {
    pub fn new() -> Chess {
        Chess::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn from_fen(fen: &str) -> Chess {
        let mut chess = Chess {
            squares: [Square::Empty; 64],
            active_color: Color::White,
            castling_rights: CastlingRights::NONE,
            en_passent: 0,
            halfmove_clock: 0,
            fullmove_number: 1,

            white_pawns: 0,
            white_knights: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_queens: 0,
            white_king: 0,

            black_pawns: 0,
            black_knights: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_queens: 0,
            black_king: 0,

            hash: 0,
            search_path: Vec::with_capacity(200),
        };

        let (position, rest): (&str, &str) = split_on(fen, ' ');
        let mut row: usize = 7;
        let mut col: usize = 0;

        for ch in position.chars() {
            match ch {
                '/' => {
                    row -= 1;
                    col = 0;
                }

                '1'..='8' => {
                    col += ch.to_digit(10).unwrap() as usize;
                }

                _ => {
                    let square_idx = row * 8 + col;
                    let bitboard: BitBoard = 1 << square_idx;

                    match ch {
                        'P' => {
                            chess.squares[square_idx] = Square::WhitePawn;
                            chess.white_pawns |= bitboard;
                        }
                        'N' => {
                            chess.squares[square_idx] = Square::WhiteKnight;
                            chess.white_knights |= bitboard;
                        }
                        'B' => {
                            chess.squares[square_idx] = Square::WhiteBishop;
                            chess.white_bishops |= bitboard;
                        }
                        'R' => {
                            chess.squares[square_idx] = Square::WhiteRook;
                            chess.white_rooks |= bitboard;
                        }
                        'Q' => {
                            chess.squares[square_idx] = Square::WhiteQueen;
                            chess.white_queens |= bitboard;
                        }
                        'K' => {
                            chess.squares[square_idx] = Square::WhiteKing;
                            chess.white_king |= bitboard;
                        }

                        'p' => {
                            chess.squares[square_idx] = Square::BlackPawn;
                            chess.black_pawns |= bitboard;
                        }
                        'n' => {
                            chess.squares[square_idx] = Square::BlackKnight;
                            chess.black_knights |= bitboard;
                        }
                        'b' => {
                            chess.squares[square_idx] = Square::BlackBishop;
                            chess.black_bishops |= bitboard;
                        }
                        'r' => {
                            chess.squares[square_idx] = Square::BlackRook;
                            chess.black_rooks |= bitboard;
                        }
                        'q' => {
                            chess.squares[square_idx] = Square::BlackQueen;
                            chess.black_queens |= bitboard;
                        }
                        'k' => {
                            chess.squares[square_idx] = Square::BlackKing;
                            chess.black_king |= bitboard;
                        }

                        _ => panic!("Invalid FEN piece '{}'", ch),
                    }

                    col += 1;
                }
            }
        }

        let (color_to_move, rest): (&str, &str) = split_on(rest, ' ');
        chess.active_color = match color_to_move {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Unknown color designator: '{}'", color_to_move),
        };

        let (castling_str, rest): (&str, &str) = split_on(rest, ' ');
        for c in castling_str.chars() {
            match c {
                'K' => chess.castling_rights |= CastlingRights::WHITEKINGSIDE,
                'Q' => chess.castling_rights |= CastlingRights::WHITEQUEENSIDE,
                'k' => chess.castling_rights |= CastlingRights::BLACKKINGSIDE,
                'q' => chess.castling_rights |= CastlingRights::BLACKQUEENSIDE,
                '-' => (),
                _ => panic!("Invalid character in castling rights: '{}'", c),
            }
        }

        let (en_passent, rest): (&str, &str) = split_on(rest, ' ');
        match en_passent {
            "-" => chess.en_passent = 0,
            s => match pos_to_bit(s) {
                Err(msg) => panic!("{}", msg),
                Ok(bit) => chess.en_passent = bit,
            },
        }

        let (halfmove_clock, rest): (&str, &str) = split_on(rest, ' ');
        match halfmove_clock.parse() {
            Ok(number) => chess.halfmove_clock = number,
            Err(_) => panic!("Invalid halfmove: {}", halfmove_clock),
        }

        let (fullmove_number, _): (&str, &str) = split_on(rest, ' ');
        match fullmove_number.parse() {
            Ok(number) => chess.fullmove_number = number,
            Err(_) => panic!("Invalid full move number: {}", fullmove_number),
        }

        chess.hash = chess.zobrist_hash();

        chess
    }
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let idx = rank * 8 + file;
                match self.squares[idx] {
                    Square::Empty => empty += 1,
                    Square::WhitePawn => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('P');
                    }
                    Square::WhiteKnight => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('N');
                    }
                    Square::WhiteBishop => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('B');
                    }
                    Square::WhiteRook => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('R');
                    }
                    Square::WhiteQueen => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('Q');
                    }
                    Square::WhiteKing => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('K');
                    }
                    Square::BlackPawn => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('p');
                    }
                    Square::BlackKnight => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('n');
                    }
                    Square::BlackBishop => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('b');
                    }
                    Square::BlackRook => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('r');
                    }
                    Square::BlackQueen => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('q');
                    }
                    Square::BlackKing => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push('k');
                    }
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(if self.active_color == Color::White {
            'w'
        } else {
            'b'
        });
        fen.push(' ');

        let mut castling = String::new();
        if self.castling_rights.contains(CastlingRights::WHITEKINGSIDE) {
            castling.push('K');
        }
        if self
            .castling_rights
            .contains(CastlingRights::WHITEQUEENSIDE)
        {
            castling.push('Q');
        }
        if self.castling_rights.contains(CastlingRights::BLACKKINGSIDE) {
            castling.push('k');
        }
        if self
            .castling_rights
            .contains(CastlingRights::BLACKQUEENSIDE)
        {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }
        fen.push_str(&castling);
        fen.push(' ');

        if self.en_passent == 0 {
            fen.push('-');
        } else {
            let ep_idx = crate::utils::bit_scan(self.en_passent);
            fen.push_str(&crate::utils::index_to_algebraic(ep_idx));
        }

        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    pub fn white_occupancy(&self) -> BitBoard {
        self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_king
    }
    pub fn black_occupancy(&self) -> BitBoard {
        self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_king
    }
}
