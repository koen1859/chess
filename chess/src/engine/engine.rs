use crate::apply_undo_move::Move;
use instant::Instant;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone)]
// Entry to the transposition table
pub struct TTEntry {
    pub depth: u8,
    pub score: i32,
    pub flag: Bound,
    pub best_move: Option<Move>,
}

#[derive(Clone)]
pub struct Engine {
    // Map hash of a position to the depth the position was analyzed on, the score and the best move given this position
    pub tt: HashMap<u64, TTEntry>,

    pub deadline: Option<Instant>,
    pub time_up: bool,
    pub nodes: u64,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            tt: HashMap::with_capacity(1_000_000),
            deadline: None,
            time_up: false,
            nodes: 0,
        }
    }
}
