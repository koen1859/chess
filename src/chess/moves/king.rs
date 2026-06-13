use crate::chess::utils::BitBoard;

pub const KING_MOVES: [BitBoard; 64] = generate_king_moves();

const fn generate_king_moves() -> [BitBoard; 64] {
    [0; 64]
}
