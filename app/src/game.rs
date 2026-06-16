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
    pub game_history: Vec<u64>,
    pub selected: Option<usize>,
    pub pending_promotion: Option<(usize, usize)>,
    pub user_color: Color,
    pub engine: Engine,
}

impl Game {
    pub fn new() -> Self {
        let chess = Chess::new();
        Self {
            game_history: vec![chess.hash],
            chess,
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
    pub fn push_position(&mut self) {
        self.game_history.push(self.chess.hash);
    }
    pub fn is_threefold_repetition(&self) -> bool {
        let current = self.chess.hash;
        self.game_history.iter().filter(|&&h| h == current).count() >= 3
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
        let no_moves = self.legal_moves().is_empty();
        let in_check = self.chess.is_color_in_check(self.chess.active_color);

        match (no_moves, in_check) {
            (true, true) => {
                return match self.chess.active_color {
                    White => "Checkmate — Black wins",
                    Black => "Checkmate — White wins",
                }
                .to_owned();
            }
            (true, false) => return "Stalemate — draw".to_owned(),
            (false, _) => {}
        }

        if self.is_threefold_repetition() {
            return "Draw by threefold repetition".to_owned();
        }
        if self.chess.halfmove_clock >= 100 {
            return "Draw by fifty-move rule".to_owned();
        }

        match (in_check, self.chess.active_color) {
            (true, White) => "White to move — check!",
            (true, Black) => "Black to move — check!",
            (false, White) => "White to move",
            (false, Black) => "Black to move",
        }
        .to_owned()
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
