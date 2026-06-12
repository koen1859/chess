use crate::chess::knightattacks::KNIGHT_ATTACKS;
use crate::chess::utils::*;
use crate::chess::{
    castling_rights::CastlingRights,
    color::{
        Color,
        Color::{Black, White},
    },
    square::Square,
};

pub struct Chess {
    pub squares: [Square; 64],

    pub active_color: Color,
    pub castling_rights: CastlingRights,
    pub en_passent: Option<BitBoard>,
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
}

impl Chess {
    pub fn new() -> Chess {
        Chess::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn to_string(&self) -> String {
        let mut out: String = String::new();
        for row in (0..8).rev() {
            for col in 0..8 {
                let sq: usize = row * 8 + col;
                out.push(self.squares[sq].to_char());
                out.push(' ');
            }
            out.push('\n');
        }
        out
    }

    pub fn from_fen(fen: &str) -> Chess {
        let mut chess = Chess {
            squares: [Square::Empty; 64],
            active_color: Color::White,
            castling_rights: CastlingRights::NONE,
            en_passent: None,
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
            "-" => chess.en_passent = None,
            s => match pos_to_bit(s) {
                Err(msg) => panic!("{}", msg),
                Ok(bit) => chess.en_passent = Some(bit),
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

        chess
    }

    pub fn move_piece(&mut self, from: usize, to: usize) {
        let piece = self.squares[from];

        if matches!(piece, Square::Empty) {
            panic!("No piece on source square");
        }

        let from_bb = 1u64 << from;
        let to_bb = 1u64 << to;

        // Handle captures first
        self.remove_piece_at(to);

        // Move piece on board
        self.squares[from] = Square::Empty;
        self.squares[to] = piece;

        // Update bitboards
        self.move_bitboard(piece, from_bb, to_bb);
    }

    fn remove_piece_at(&mut self, sq: usize) {
        let piece = self.squares[sq];

        if matches!(piece, Square::Empty) {
            return;
        }

        let bitboard = 1u64 << sq;

        match piece {
            Square::WhitePawn => self.white_pawns &= !bitboard,
            Square::WhiteKnight => self.white_knights &= !bitboard,
            Square::WhiteBishop => self.white_bishops &= !bitboard,
            Square::WhiteRook => self.white_rooks &= !bitboard,
            Square::WhiteQueen => self.white_queens &= !bitboard,
            Square::WhiteKing => self.white_king &= !bitboard,

            Square::BlackPawn => self.black_pawns &= !bitboard,
            Square::BlackKnight => self.black_knights &= !bitboard,
            Square::BlackBishop => self.black_bishops &= !bitboard,
            Square::BlackRook => self.black_rooks &= !bitboard,
            Square::BlackQueen => self.black_queens &= !bitboard,
            Square::BlackKing => self.black_king &= !bitboard,

            Square::Empty => {}
        }

        self.squares[sq] = Square::Empty;
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_move_piece() {
        let mut chess: Chess = Chess::new();
        println!("{}", chess.to_string());
        chess.move_piece(1 << 15, 31);
        println!("{}", chess.to_string());
    }
}
