use crate::apply_undo_move::Move;
use instant::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    Alpha,
    Beta,
}

const TT_SIZE: usize = 1 << 20; // ~1M entries

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,
    pub score: i32,
    pub depth: u8,
    pub flag: TTFlag,
    pub best_move: Option<Move>,
}

#[derive(Clone)]
pub struct Engine {
    pub tt: Vec<TTEntry>,
    pub deadline: Option<Instant>,
    pub time_up: bool,
    pub nodes: u64,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            tt: vec![
                TTEntry {
                    hash: 0,
                    score: 0,
                    depth: 0,
                    flag: TTFlag::Exact,
                    best_move: None,
                };
                TT_SIZE
            ],
            deadline: None,
            time_up: false,
            nodes: 0,
        }
    }

    pub fn tt_probe(&self, hash: u64) -> Option<&TTEntry> {
        let entry = &self.tt[hash as usize & (TT_SIZE - 1)];
        if entry.hash == hash {
            Some(entry)
        } else {
            None
        }
    }

    pub fn tt_store(&mut self, hash: u64, score: i32, depth: u8, flag: TTFlag, best_move: Option<Move>) {
        let idx = hash as usize & (TT_SIZE - 1);
        // Always replace (simple strategy)
        self.tt[idx] = TTEntry {
            hash,
            score,
            depth,
            flag,
            best_move,
        };
    }
}
