use crate::chess::Chess;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    positions: Vec<u64>,
}

impl History {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
        }
    }

    pub fn push(&mut self, hash: u64) {
        self.positions.push(hash);
    }

    pub fn pop(&mut self) {
        self.positions.pop();
    }

    pub fn is_repetition(&self, board: &Chess) -> bool {
        self.positions
            .iter()
            .rev()
            .take(board.halfmove_clock)
            .filter(|&&h| h == board.hash)
            .count()
            >= 2
    }
}
