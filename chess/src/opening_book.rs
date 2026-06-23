use crate::apply_undo_move::{Move, uci_to_move};
use crate::chess::Chess;
use std::sync::OnceLock;

static BOOK: OnceLock<OpeningBook> = OnceLock::new();

#[derive(Clone, Copy)]
struct BookMove {
    mov: Move,
    weight: u8,
}

struct OpeningBook {
    entries: Vec<(u64, Vec<BookMove>)>,
}

const OPENINGS: &[&[&str]] = &[
    // ===== 1.e4 Openings =====
    // 1...e5
    &["e2e4", "e7e5"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5"],
    &[
        "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6",
    ],
    &[
        "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "d7d6",
    ],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5c6"],
    &[
        "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1g1", "f8e7",
    ],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "c2c3"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6"],
    &[
        "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "d2d4", "e5d4", "e4e5",
    ],
    &["e2e4", "e7e5", "g1f3", "b8c6", "d2d4"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "b1c3", "g8f6"],
    &["e2e4", "e7e5", "b1c3"],
    &["e2e4", "e7e5", "b1c3", "g8f6", "f2f4"],
    &["e2e4", "e7e5", "f2f4"],
    &["e2e4", "e7e5", "f2f4", "e5f4", "g1f3"],
    &["e2e4", "e7e5", "g1f3", "d7d6"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "c2c3"],
    // 1...c5 (Sicilian)
    &["e2e4", "c7c5"],
    &[
        "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3",
    ],
    &[
        "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6",
    ],
    &[
        "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "g7g6",
    ],
    &[
        "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "e7e6",
    ],
    &[
        "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "b8c6",
    ],
    &[
        "e2e4", "c7c5", "g1f3", "e7e6", "d2d4", "c5d4", "f3d4", "a7a6",
    ],
    &[
        "e2e4", "c7c5", "g1f3", "e7e6", "d2d4", "c5d4", "f3d4", "b8c6",
    ],
    &[
        "e2e4", "c7c5", "g1f3", "b8c6", "d2d4", "c5d4", "f3d4", "g7g6",
    ],
    &["e2e4", "c7c5", "c2c3"],
    &["e2e4", "c7c5", "c2c3", "d7d5", "e4d5", "d8d5"],
    &["e2e4", "c7c5", "d2d4", "c5d4", "c2c3"],
    // 1...e6 (French)
    &["e2e4", "e7e6", "d2d4", "d7d5"],
    &["e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "f8b4"],
    &["e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "g8f6"],
    &["e2e4", "e7e6", "d2d4", "d7d5", "b1d2"],
    &["e2e4", "e7e6", "d2d4", "d7d5", "e4e5"],
    // 1...c6 (Caro-Kann)
    &["e2e4", "c7c6", "d2d4", "d7d5"],
    &[
        "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4", "c8f5",
    ],
    &["e2e4", "c7c6", "d2d4", "d7d5", "e4e5"],
    &["e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "c2c4"],
    // 1...other
    &["e2e4", "d7d5"],
    &["e2e4", "d7d5", "e4d5", "d8d5", "b1c3"],
    &["e2e4", "g8f6"],
    &["e2e4", "g8f6", "e4e5", "f6d5", "d2d4", "d7d6"],
    &["e2e4", "d7d6", "d2d4", "g8f6", "b1c3", "g7g6"],
    &["e2e4", "g7g6", "d2d4", "f8g7"],
    &["e2e4", "b8c6"],
    &["e2e4", "a7a6"],
    // ===== 1.d4 Openings =====
    // 1.d4 d5
    &["d2d4", "d7d5"],
    &["d2d4", "d7d5", "c2c4"],
    &["d2d4", "d7d5", "c2c4", "e7e6"],
    &["d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6"],
    &[
        "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5", "f8e7",
    ],
    &[
        "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5", "f8e7", "e2e3", "e8g8",
    ],
    &[
        "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c4d5", "e6d5",
    ],
    &["d2d4", "d7d5", "c2c4", "d5c4"],
    &["d2d4", "d7d5", "c2c4", "d5c4", "e2e3"],
    &["d2d4", "d7d5", "c2c4", "c7c6"],
    &[
        "d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6", "b1c3", "e7e6",
    ],
    &["d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6", "e2e3"],
    &["d2d4", "d7d5", "c2c4", "c7c6", "c4d5", "c6d5"],
    // 1.d4 Nf6 (Indian)
    &["d2d4", "g8f6", "c2c4", "g7g6"],
    &[
        "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6",
    ],
    &["d2d4", "g8f6", "c2c4", "e7e6"],
    &["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4"],
    &["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "d1c2"],
    &["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "e2e3"],
    &["d2d4", "g8f6", "c2c4", "e7e6", "g1f3", "b7b6"],
    &["d2d4", "g8f6", "c2c4", "e7e6", "g1f3", "f8b4"],
    &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5"],
    &[
        "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5", "c4d5", "f6d5", "e2e4", "d5c3", "b2c3",
    ],
    &["d2d4", "g8f6", "c2c4", "c7c5", "d4d5", "e7e6"],
    &["d2d4", "g8f6", "c2c4", "e7e5"],
    // 1.d4 other
    &["d2d4", "f7f5"],
    &["d2d4", "f7f5", "c2c4", "g8f6", "g2g3", "g7g6"],
    &["d2d4", "g8f6", "c2c4", "e7e6", "g2g3"],
    &["d2d4", "d7d5", "c1f4", "g8f6", "e2e3"],
    &["d2d4", "g8f6", "c1g5"],
    // ===== Other first moves =====
    &["c2c4"],
    &["c2c4", "e7e5"],
    &["c2c4", "g8f6"],
    &["c2c4", "c7c5"],
    &["c2c4", "e7e6"],
    &["c2c4", "g7g6"],
    &["g1f3"],
    &["g1f3", "d7d5"],
    &["g1f3", "g8f6"],
    &["g1f3", "c7c5"],
    &["f2f4"],
    &["b2b3"],
    // Extra deeper main lines
    &[
        "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6", "e1g1", "f8e7", "f1e1",
        "b7b5", "a4b3", "d7d6", "c2c3", "e8g8", "h2h3",
    ],
    &[
        "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "c2c3", "g8f6", "d2d4", "e5d4", "c3d4",
        "c5b4",
    ],
    &[
        "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6", "c1g5",
        "e7e6", "f2f4",
    ],
    &[
        "d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "e2e3", "e8g8", "c1d3", "d7d5", "g1f3",
        "c7c5", "e1g1",
    ],
];

impl OpeningBook {
    fn build() -> Self {
        let mut raw: Vec<(u64, BookMove)> = Vec::new();

        for opening in OPENINGS {
            let mut board = Chess::new();
            for &uci in *opening {
                let m = uci_to_move(uci, &board)
                    .unwrap_or_else(|| panic!("Invalid move '{}' in opening book", uci));
                let bm = BookMove { mov: m, weight: 10 };
                raw.push((board.hash, bm));
                board.apply_move(&m);
            }
        }

        raw.sort_by_key(|&(hash, _)| hash);

        let mut entries: Vec<(u64, Vec<BookMove>)> = Vec::new();
        let mut i = 0;
        while i < raw.len() {
            let hash = raw[i].0;
            let mut moves: Vec<BookMove> = Vec::new();
            while i < raw.len() && raw[i].0 == hash {
                let bm = raw[i].1;
                if let Some(existing) = moves.iter_mut().find(|m: &&mut BookMove| m.mov == bm.mov) {
                    existing.weight = existing.weight.saturating_add(bm.weight);
                } else {
                    moves.push(bm);
                }
                i += 1;
            }
            entries.push((hash, moves));
        }

        OpeningBook { entries }
    }

    fn lookup(&self, hash: u64) -> Option<&[BookMove]> {
        let idx = self.entries.binary_search_by_key(&hash, |&(h, _)| h);
        match idx {
            Ok(i) => Some(&self.entries[i].1),
            Err(_) => None,
        }
    }
}

pub fn get_book_move(hash: u64) -> Option<Move> {
    let book = BOOK.get_or_init(OpeningBook::build);
    let moves = book.lookup(hash)?;
    let total: u32 = moves.iter().map(|m| m.weight as u32).sum();
    if total == 0 {
        return None;
    }
    let choice = rand::random_range(0..total);
    let mut cumulative = 0u32;
    for bm in moves {
        cumulative += bm.weight as u32;
        if choice < cumulative {
            return Some(bm.mov);
        }
    }
    None
}

pub fn init() {
    BOOK.get_or_init(OpeningBook::build);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::Chess;

    #[test]
    fn test_book_initializes() {
        let book = OpeningBook::build();
        assert!(!book.entries.is_empty(), "Book should have entries");
    }

    #[test]
    fn test_book_contains_start_position() {
        let board = Chess::new();
        let book = OpeningBook::build();
        let moves = book.lookup(board.hash);
        assert!(
            moves.is_some(),
            "Book should have moves from start position"
        );
    }

    #[test]
    fn test_book_lookup() {
        let mut board = Chess::new();
        let m = uci_to_move("e2e4", &board).unwrap();
        board.apply_move(&m);
        let book = OpeningBook::build();
        let moves = book.lookup(board.hash);
        assert!(moves.is_some(), "Book should have moves after 1.e4");
    }

    #[test]
    fn test_get_book_move_after_e4() {
        let mut board = Chess::new();
        board.apply_move(&uci_to_move("e2e4", &board).unwrap());
        let m = get_book_move(board.hash);
        assert!(m.is_some(), "Should find book move after 1.e4");
    }

    #[test]
    fn test_get_book_move_random() {
        let board = Chess::new();
        let m1 = get_book_move(board.hash);
        let m2 = get_book_move(board.hash);
        assert!(m1.is_some(), "Should get a move from start");
        assert!(m2.is_some(), "Should get a move from start");
    }
}
