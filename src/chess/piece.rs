use crate::chess::color::Color;

pub type PiecePos = u64;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, PartialEq)]
pub struct Piece {
    position: PiecePos,
    color: Color,
    piece_type: PieceType,
}

impl Piece {
    pub fn new(position: PiecePos, color: Color, piece_type: PieceType) -> Piece {
        Piece {
            position: position,
            color: color,
            piece_type: piece_type,
        }
    }
    pub fn to_string(&self) -> String {
        let mut result: String = match self.piece_type {
            PieceType::Pawn => String::from("p "),
            PieceType::Rook => String::from("r "),
            PieceType::Knight => String::from("n "),
            PieceType::Bishop => String::from("b "),
            PieceType::Queen => String::from("q "),
            PieceType::King => String::from("k "),
        };

        if self.color == Color::White {
            result.make_ascii_uppercase();
        }

        result
    }

    pub fn position(&self) -> PiecePos {
        self.position
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }
}
