use std::collections::VecDeque;

use crate::chess::knightattacks::KnightAttacks;
use crate::chess::utils::*;
use crate::chess::{
    castling_rights::CastlingRights,
    color::{
        Color,
        Color::{Black, White},
    },
    piece::{Piece, PieceType},
    square::Square,
};

pub struct Chess {
    pub pieces: Vec<Piece>,
    pub squares: Vec<Square>,
    pub active_color: Color,
    pub castling_rights: CastlingRights,
    pub en_passent: Option<BitBoard>,
    pub halfmove_clock: usize,
    pub fullmove_number: usize,

    pub white_occupancy: BitBoard,
    pub black_occupancy: BitBoard,

    pub knight_attacks: KnightAttacks,
}

impl Chess {
    pub fn new() -> Chess {
        Chess::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn to_string(&self) -> String {
        let mut board: String = String::new();
        let mut temp: String = String::new();

        for (i, square) in self.squares.iter().enumerate() {
            match square {
                Square::Empty => temp.push_str(". "),
                Square::Occupied(idx) => temp.push_str(&self.pieces[*idx].to_string()),
            }

            if (i + 1) % 8 == 0 {
                temp.push_str("\n");
                board.insert_str(0, &temp);
                temp.clear();
            }
        }

        board
    }

    pub fn from_fen(fen: &str) -> Chess {
        let mut chess = Chess {
            pieces: vec![],
            squares: vec![],
            active_color: Color::White,
            castling_rights: CastlingRights::NONE,
            en_passent: None,
            halfmove_clock: 0,
            fullmove_number: 1,

            white_occupancy: 0,
            black_occupancy: 0,

            knight_attacks: KnightAttacks::new(),
        };

        let (position, rest): (&str, &str) = split_on(fen, ' ');
        let mut deque_squares: VecDeque<Square> = VecDeque::new();
        let mut piece_index: usize = 0;
        let mut piece_position: usize = 64;

        for row in position.splitn(8, |ch| ch == '/') {
            piece_position -= 8;
            let (pieces, squares): (Vec<Piece>, VecDeque<Square>) =
                parse_row(&row, piece_index, piece_position);

            for p in pieces {
                match p.color {
                    Black => chess.black_occupancy |= p.position,
                    White => chess.white_occupancy |= p.position,
                }
                chess.pieces.push(p);
                piece_index += 1;
            }

            for s in squares {
                deque_squares.push_front(s);
            }
        }

        chess.squares = Vec::from(deque_squares);

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
}
