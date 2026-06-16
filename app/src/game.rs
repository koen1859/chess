use chess::{
    apply_undo_move::Move,
    chess::Chess,
    color::Color::{self, Black, White},
    engine::engine::Engine,
    square::{Square, Square::*},
    utils::count_ones,
};

#[derive(Clone)]
pub struct Game {
    pub chess: Chess,
    pub selected: Option<usize>,
    pub pending_promotion: Option<(usize, usize)>,
    pub user_color: Color,
    pub engine: Engine,
}

impl Game {
    pub fn new() -> Self {
        Self {
            chess: Chess::new(),
            selected: None,
            pending_promotion: None,
            user_color: if rand::random::<bool>() { White } else { Black },
            engine: Engine::new(),
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn legal_moves(&self) -> Vec<Move> {
        self.chess.generate_moves(self.chess.active_color)
    }
    pub fn destinations(&self) -> Vec<usize> {
        match self.selected {
            Some(from) => self
                .legal_moves()
                .iter()
                .filter(|m| m.from == from)
                .map(|m| m.to)
                .collect(),
            None => Vec::new(),
        }
    }
    pub fn status(&self) -> String {
        // let legal_moves = self.legal_moves();
        let in_check = self.chess.is_color_in_check(self.chess.active_color);
        let no_moves = self.legal_moves().is_empty();

        match (no_moves, in_check, self.chess.active_color) {
            (true, true, White) => "Checkmate — Black wins".to_owned(),
            (true, true, Black) => "Checkmate — White wins".to_owned(),
            (true, false, _) => "Stalemate — draw".to_owned(),
            (false, true, White) => "White to move — check!".to_owned(),
            (false, true, Black) => "Black to move — check!".to_owned(),
            (false, false, White) => "White to move".to_owned(),
            (false, false, Black) => "Black to move".to_owned(),
        }
    }
    pub fn user_color_str(&self) -> &'static str {
        match self.user_color {
            White => "White",
            Black => "Black",
        }
    }
    pub fn board_squares(&self) -> Vec<(usize, usize)> {
        let mut sqs = Vec::with_capacity(64);
        if self.user_color == White {
            for rank in (0..8).rev() {
                for file in 0..8 {
                    sqs.push((rank, file));
                }
            }
        } else {
            for rank in 0..8 {
                for file in (0..8).rev() {
                    sqs.push((rank, file));
                }
            }
        }
        sqs
    }
    pub fn active_square_color(&self, idx: usize) -> Color {
        let piece = self.chess.squares[idx];
        if matches!(
            piece,
            WhitePawn | WhiteKnight | WhiteBishop | WhiteRook | WhiteQueen | WhiteKing
        ) {
            White
        } else {
            Black
        }
    }
    pub fn count_material(&self) -> i32 {
        let mut score: i32 = 0;

        score += 1 * (count_ones(self.chess.white_pawns) - count_ones(self.chess.black_pawns));
        score += 3 * (count_ones(self.chess.white_knights) - count_ones(self.chess.black_knights));
        score += 3 * (count_ones(self.chess.white_bishops) - count_ones(self.chess.black_bishops));
        score += 5 * (count_ones(self.chess.white_rooks) - count_ones(self.chess.black_rooks));
        score += 9 * (count_ones(self.chess.white_queens) - count_ones(self.chess.black_queens));

        score
    }
}
