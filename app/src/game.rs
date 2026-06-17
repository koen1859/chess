use chess::{
    apply_undo_move::Move,
    chess::Chess,
    color::Color::{self, Black, White},
    engine::engine::Engine,
    utils::count_ones,
};

use crate::utils::bitboard_squares;

pub const DEFAULT_ENGINE_THINK_TIME_MS: u64 = 3000;

/// Which piece bitboard (if any) the debug overlay should currently highlight.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DebugBitboard {
    WhitePawns,
    WhiteKnights,
    WhiteBishops,
    WhiteRooks,
    WhiteQueens,
    WhiteKing,
    BlackPawns,
    BlackKnights,
    BlackBishops,
    BlackRooks,
    BlackQueens,
    BlackKing,
}

impl DebugBitboard {
    pub const ALL: [DebugBitboard; 12] = [
        DebugBitboard::WhitePawns,
        DebugBitboard::WhiteKnights,
        DebugBitboard::WhiteBishops,
        DebugBitboard::WhiteRooks,
        DebugBitboard::WhiteQueens,
        DebugBitboard::WhiteKing,
        DebugBitboard::BlackPawns,
        DebugBitboard::BlackKnights,
        DebugBitboard::BlackBishops,
        DebugBitboard::BlackRooks,
        DebugBitboard::BlackQueens,
        DebugBitboard::BlackKing,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            DebugBitboard::WhitePawns => "White Pawns",
            DebugBitboard::WhiteKnights => "White Knights",
            DebugBitboard::WhiteBishops => "White Bishops",
            DebugBitboard::WhiteRooks => "White Rooks",
            DebugBitboard::WhiteQueens => "White Queens",
            DebugBitboard::WhiteKing => "White King",
            DebugBitboard::BlackPawns => "Black Pawns",
            DebugBitboard::BlackKnights => "Black Knights",
            DebugBitboard::BlackBishops => "Black Bishops",
            DebugBitboard::BlackRooks => "Black Rooks",
            DebugBitboard::BlackQueens => "Black Queens",
            DebugBitboard::BlackKing => "Black King",
        }
    }

    /// Pulls the matching raw bitboard out of the Chess struct.
    /// NOTE: assumes `Chess` exposes `white_king` / `black_king` bitboards in
    /// addition to the pawn/knight/bishop/rook/queen ones already used in
    /// `count_material`. If those fields don't exist (or are named
    /// differently) under the hood, adjust these two arms accordingly.
    pub fn bitboard(&self, chess: &Chess) -> u64 {
        match self {
            DebugBitboard::WhitePawns => chess.white_pawns,
            DebugBitboard::WhiteKnights => chess.white_knights,
            DebugBitboard::WhiteBishops => chess.white_bishops,
            DebugBitboard::WhiteRooks => chess.white_rooks,
            DebugBitboard::WhiteQueens => chess.white_queens,
            DebugBitboard::WhiteKing => chess.white_king,
            DebugBitboard::BlackPawns => chess.black_pawns,
            DebugBitboard::BlackKnights => chess.black_knights,
            DebugBitboard::BlackBishops => chess.black_bishops,
            DebugBitboard::BlackRooks => chess.black_rooks,
            DebugBitboard::BlackQueens => chess.black_queens,
            DebugBitboard::BlackKing => chess.black_king,
        }
    }
}

#[derive(Clone)]
pub struct GameState {
    pub chess: Chess,
    pub game_history: Vec<u64>,
    pub selected: Option<usize>,
    pub pending_promotion: Option<(usize, usize)>,
    pub user_color: Color,
    pub engine: Engine,
    pub engine_think_time_ms: u64,
    pub debug_overlay: Option<DebugBitboard>,
    pub last_move: Option<(usize, usize)>,
}

impl GameState {
    pub fn new() -> Self {
        let chess = Chess::new();
        Self {
            game_history: vec![chess.hash],
            chess,
            selected: None,
            pending_promotion: None,
            user_color: if rand::random::<bool>() { White } else { Black },
            engine: Engine::new(),
            engine_think_time_ms: DEFAULT_ENGINE_THINK_TIME_MS,
            debug_overlay: None,
            last_move: None,
        }
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

    pub fn count_material(&self) -> i32 {
        let mut score: i32 = 0;

        score += 1 * (count_ones(self.chess.white_pawns) - count_ones(self.chess.black_pawns));
        score += 3 * (count_ones(self.chess.white_knights) - count_ones(self.chess.black_knights));
        score += 3 * (count_ones(self.chess.white_bishops) - count_ones(self.chess.black_bishops));
        score += 5 * (count_ones(self.chess.white_rooks) - count_ones(self.chess.black_rooks));
        score += 9 * (count_ones(self.chess.white_queens) - count_ones(self.chess.black_queens));

        score
    }

    /// Squares to highlight for the currently selected debug bitboard overlay,
    /// empty if no overlay is active.
    pub fn debug_overlay_squares(&self) -> Vec<usize> {
        match self.debug_overlay {
            Some(bb) => bitboard_squares(bb.bitboard(&self.chess)),
            None => Vec::new(),
        }
    }
}
