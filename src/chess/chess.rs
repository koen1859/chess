use std::collections::VecDeque;

use crate::chess::utils::*;
use crate::chess::{
    castling_rights::CastlingRights,
    color::Color,
    piece::{Piece, PiecePos, PieceType},
    square::Square,
};

pub struct Chess {
    pieces: Vec<Piece>,
    squares: Vec<Square>,
    active_color: Color,
    castling_rights: CastlingRights,
    en_passent: Option<PiecePos>,
    halfmove_clock: usize,
    fullmove_number: usize,
}

impl Chess {
    pub fn new() -> Chess {
        Chess::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
    pub fn to_string(&self) -> String {
        let mut board = String::new();
        let mut temp = String::new();

        for (i, square) in self.squares.iter().enumerate() {
            match square {
                Square::Empty => temp.push_str(&index_to_pos(i)),
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

    fn from_fen(fen: &str) -> Chess {
        let mut chess = Chess {
            pieces: vec![],
            squares: vec![],
            active_color: Color::White,
            castling_rights: CastlingRights::ALL,
            en_passent: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        };

        let (position, rest) = split_on(fen, ' ');
        let mut deque_squares = VecDeque::new();
        let mut piece_index = 0;
        let mut piece_position = 64;

        for row in position.splitn(8, |ch| ch == '/') {
            piece_position -= 8;
            let (pieces, squares) = parse_row(&row, piece_index, piece_position);

            for p in pieces {
                chess.pieces.push(p);
                piece_index += 1;
            }

            for s in squares {
                deque_squares.push_front(s);
            }
        }

        chess.squares = Vec::from(deque_squares);

        let (color_to_move, rest) = split_on(rest, ' ');
        chess.active_color = match color_to_move {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Unknown color designator: '{}'", color_to_move),
        };

        let (castling_str, rest) = split_on(rest, ' ');
        let mut castling_rights = CastlingRights::NONE;
        for c in castling_str.chars() {
            match c {
                'K' => castling_rights |= CastlingRights::WHITEKINGSIDE,
                'Q' => castling_rights |= CastlingRights::WHITEQUEENSIDE,
                'k' => castling_rights |= CastlingRights::BLACKKINGSIDE,
                'q' => castling_rights |= CastlingRights::BLACKQUEENSIDE,
                '-' => (),
                _ => panic!("Invalid character in castling rights: '{}'", c),
            }
        }
        chess.castling_rights = castling_rights;

        let (en_passent, rest) = split_on(rest, ' ');
        match en_passent {
            "-" => chess.en_passent = None,
            s => match pos_to_bit(s) {
                Err(msg) => panic!("{}", msg),
                Ok(bit) => chess.en_passent = Some(bit),
            },
        }

        let (halfmove_clock, rest) = split_on(rest, ' ');
        match halfmove_clock.parse() {
            Ok(number) => chess.halfmove_clock = number,
            Err(_) => panic!("Invalid halfmove: {}", halfmove_clock),
        }

        let (fullmove_number, _) = split_on(rest, ' ');
        match fullmove_number.parse() {
            Ok(number) => chess.fullmove_number = number,
            Err(_) => panic!("Invalid full move number: {}", fullmove_number),
        }

        chess
    }
    pub fn active_color(&self) -> Color {
        self.active_color
    }
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }
    pub fn en_passent(&self) -> Option<PiecePos> {
        self.en_passent
    }
    pub fn halfmove_clock(&self) -> usize {
        self.halfmove_clock
    }
    pub fn fullmove_number(&self) -> usize {
        self.fullmove_number
    }
}
