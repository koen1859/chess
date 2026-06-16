use crate::apply_undo_move::{Move, MoveFlags};

const MAX_MOVES: usize = 256;

pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move { from: 0, to: 0, flags: MoveFlags::empty() }; MAX_MOVES],
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn push(&mut self, m: Move) {
        debug_assert!(self.len < MAX_MOVES, "MoveList overflow");
        self.moves[self.len] = m;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    pub fn get(&self, index: usize) -> &Move {
        &self.moves[index]
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.moves.swap(i, j);
    }

    pub fn truncate(&mut self, new_len: usize) {
        self.len = new_len;
    }

    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&Move, &Move) -> std::cmp::Ordering,
    {
        self.moves[..self.len].sort_by(&mut compare);
    }
}
