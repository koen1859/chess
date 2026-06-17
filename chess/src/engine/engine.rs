use crate::apply_undo_move::Move;
use instant::Instant;
use std::collections::HashMap;

#[derive(Clone)]
pub enum StorageFlag {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone)]
pub struct Engine {
    // Map hash of a position to the depth the position was analyzed on, the score and the best move given this position
    pub storage: HashMap<u64, (u8, i32, Option<Move>, StorageFlag)>,

    pub deadline: Option<Instant>,
    pub time_up: bool,
    pub nodes: u64,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            storage: HashMap::with_capacity(1_000_000),
            deadline: None,
            time_up: false,
            nodes: 0,
        }
    }
}
