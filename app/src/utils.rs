use chess::{
    color::{Color, Color::*},
    square::{Square, Square::*},
};

pub fn piece_svg(square: Square) -> &'static str {
    match square {
        WhitePawn => "pieces/pawn-w.svg",
        WhiteKnight => "pieces/knight-w.svg",
        WhiteBishop => "pieces/bishop-w.svg",
        WhiteRook => "pieces/rook-w.svg",
        WhiteQueen => "pieces/queen-w.svg",
        WhiteKing => "pieces/king-w.svg",
        BlackPawn => "pieces/pawn-b.svg",
        BlackKnight => "pieces/knight-b.svg",
        BlackBishop => "pieces/bishop-b.svg",
        BlackRook => "pieces/rook-b.svg",
        BlackQueen => "pieces/queen-b.svg",
        BlackKing => "pieces/king-b.svg",
        Empty => "",
    }
}

pub fn is_own_piece(square: Square, color: Color) -> bool {
    match color {
        White => matches!(
            square,
            WhitePawn | WhiteKnight | WhiteBishop | WhiteRook | WhiteQueen | WhiteKing
        ),
        Black => matches!(
            square,
            BlackPawn | BlackKnight | BlackBishop | BlackRook | BlackQueen | BlackKing
        ),
    }
}
