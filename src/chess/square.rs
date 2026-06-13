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
    pub fn to_char(self) -> char {
        match self {
            Square::Empty => '.',

            Square::WhitePawn => 'P',
            Square::WhiteKnight => 'N',
            Square::WhiteBishop => 'B',
            Square::WhiteRook => 'R',
            Square::WhiteQueen => 'Q',
            Square::WhiteKing => 'K',

            Square::BlackPawn => 'p',
            Square::BlackKnight => 'n',
            Square::BlackBishop => 'b',
            Square::BlackRook => 'r',
            Square::BlackQueen => 'q',
            Square::BlackKing => 'k',
        }
    }

    pub fn is_empty(self) -> bool {
        matches!(self, Square::Empty)
    }

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
