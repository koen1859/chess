use crate::apply_undo_move::Move;
use instant::Instant;

const TT_BUCKETS: usize = 1_048_576; // 2**20 = 16MB
const BUCKET_SIZE: usize = 4;

#[derive(Clone, Copy)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone, Copy)]
// Entry to the transposition table
pub struct TTEntry {
    pub hash: u64,
    pub depth: u8,
    pub score: i32,
    pub flag: Bound,
    pub best_move: Option<Move>,
    pub age: u8,
}

impl TTEntry {
    fn new() -> Self {
        Self {
            hash: 0,
            depth: 0,
            score: 0,
            flag: Bound::Exact,
            best_move: None,
            age: 0,
        }
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    buckets: Vec<[TTEntry; BUCKET_SIZE]>,
    generation: u8,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            buckets: vec![[TTEntry::new(); BUCKET_SIZE]; TT_BUCKETS],
            generation: 0,
        }
    }
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let idx = (hash as usize) & (TT_BUCKETS - 1);
        self.buckets[idx].iter().find(|entry| entry.hash == hash)
    }
    pub fn insert(
        &mut self,
        hash: u64,
        depth: u8,
        score: i32,
        flag: Bound,
        best_move: Option<Move>,
    ) {
        let idx = (hash as usize) & (TT_BUCKETS - 1);
        // Replace: prefer empty entries, then entries with lower depth, then older entries
        let mut replace_idx = 0;
        for i in 0..BUCKET_SIZE {
            let entry = &self.buckets[idx][i];
            if entry.hash == 0 {
                replace_idx = i;
                break;
            }
            if entry.depth < depth {
                replace_idx = i;
                break;
            }
            if entry.age != self.generation {
                replace_idx = i;
                break;
            }
        }
        self.buckets[idx][replace_idx] = TTEntry {
            hash,
            depth,
            score,
            flag,
            best_move,
            age: self.generation,
        };
    }
    pub fn new_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}

#[derive(Clone)]
pub struct Engine {
    // Map hash of a position to the depth the position was analyzed on, the score and the best move given this position
    pub tt: TranspositionTable,

    pub deadline: Option<Instant>,
    pub time_up: bool,
    pub nodes: u64,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            tt: TranspositionTable::new(),
            deadline: None,
            time_up: false,
            nodes: 0,
        }
    }
}
