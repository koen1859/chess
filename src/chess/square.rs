use crate::chess::color::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Square {
    Empty,
    WhitePawn,
    WhiteRook,
    WhiteKnight,
    WhiteBishop,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackRook,
    BlackKnight,
    BlackBishop,
    BlackQueen,
    BlackKing,
}
impl Square {
    pub fn color(&self) -> Option<Color> {
        match self {
            Square::WhitePawn
            | Square::WhiteKnight
            | Square::WhiteBishop
            | Square::WhiteRook
            | Square::WhiteQueen
            | Square::WhiteKing => Some(Color::White),

            Square::BlackPawn
            | Square::BlackKnight
            | Square::BlackBishop
            | Square::BlackRook
            | Square::BlackQueen
            | Square::BlackKing => Some(Color::Black),

            Square::Empty => None,
        }
    }
    pub fn value(&self) -> i32 {
        match self {
            Square::WhitePawn | Square::BlackPawn => 1,
            Square::WhiteKnight | Square::BlackKnight => 3,
            Square::WhiteBishop | Square::BlackBishop => 3,
            Square::WhiteRook | Square::BlackRook => 5,
            Square::WhiteQueen | Square::BlackQueen => 9,
            Square::WhiteKing | Square::BlackKing => 0,
            Square::Empty => 0,
        }
    }
}
