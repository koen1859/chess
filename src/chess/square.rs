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
    pub fn color(self) -> Option<Color> {
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
}
