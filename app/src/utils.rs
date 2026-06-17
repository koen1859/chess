use chess::{
    color::{Color, Color::*},
    square::{Square, Square::*},
    utils::BitBoard,
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

/// Turns a bitboard (u64) into the list of board indices (0..64) whose bit is set.
/// Assumes bit `i` corresponds to `idx = rank * 8 + file`, matching the indexing
/// already used for `chess.squares` / `board_squares()` elsewhere in this app.
pub fn bitboard_squares(bitboard: BitBoard) -> Vec<usize> {
    (0..64).filter(|i| (bitboard >> i) & 1 == 1).collect()
}
