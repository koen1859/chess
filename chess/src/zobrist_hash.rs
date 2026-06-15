use crate::{chess::Chess, color::Color, square::Square, utils::BitBoard};
use lazy_static::lazy_static;

lazy_static! {
    // 64 squares × 12 piece types = 768 numbers
    static ref PIECE_HASHES: [[[u64; 12]; 64]; 2] = {
        let mut hashes = [[[0u64; 12]; 64]; 2];

        // Use a deterministic seed so the hashes are always the same
        let mut seed = 0x9e3779b97f4a7c15u64;

        for color in 0..2 {
            for square in 0..64 {
                for piece in 0..12 {
                    // Simple LCG-style hash generation (deterministic)
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    hashes[color][square][piece] = seed;
                }
            }
        }
        hashes
    };

    // Hashes for castling rights (4 bits = 16 combinations)
    static ref CASTLING_HASHES: [u64; 16] = {
        let mut hashes = [0u64; 16];
        let mut seed = 0x85ebca6b3c0c0e3fu64;

        for i in 0..16 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            hashes[i] = seed;
        }
        hashes
    };

    // Hashes for en passant files (8 files, +1 for "none")
    static ref EN_PASSANT_HASHES: [u64; 9] = {
        let mut hashes = [0u64; 9];
        let mut seed = 0xc3d2e1f0a1b2c3d4u64;

        for i in 0..9 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            hashes[i] = seed;
        }
        hashes
    };

    // Hash for black to move
    static ref BLACK_TO_MOVE_HASH: u64 = {
        let mut seed = 0x1122334455667788u64;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed
    };
}

/// Convert a Square enum to a piece index (0-11)
/// White pieces: 0-5, Black pieces: 6-11
fn square_to_piece_index(square: Square) -> Option<(usize, usize)> {
    // (color_index: 0 for white, 1 for black, piece_index: 0-5)
    match square {
        Square::WhitePawn => Some((0, 0)),
        Square::WhiteKnight => Some((0, 1)),
        Square::WhiteBishop => Some((0, 2)),
        Square::WhiteRook => Some((0, 3)),
        Square::WhiteQueen => Some((0, 4)),
        Square::WhiteKing => Some((0, 5)),
        Square::BlackPawn => Some((1, 6)),
        Square::BlackKnight => Some((1, 7)),
        Square::BlackBishop => Some((1, 8)),
        Square::BlackRook => Some((1, 9)),
        Square::BlackQueen => Some((1, 10)),
        Square::BlackKing => Some((1, 11)),
        Square::Empty => None,
    }
}

/// Get the en passant file from a bitboard
/// Returns 0-7 for files a-h, or 8 if no en passant
fn get_en_passant_index(en_passant: BitBoard) -> usize {
    if en_passant == 0 {
        return 8; // no en passant
    }

    // Find the first set bit and determine which file it's on
    let square = en_passant.trailing_zeros() as usize;
    square % 8
}

impl Chess {
    /// Compute the zobrist hash for this position
    pub fn zobrist_hash(&self) -> u64 {
        let mut hash = 0u64;

        // Hash all pieces on the board
        for (square_idx, &piece) in self.squares.iter().enumerate() {
            if let Some((color_idx, piece_idx)) = square_to_piece_index(piece) {
                hash ^= PIECE_HASHES[color_idx][square_idx][piece_idx];
            }
        }

        // Hash castling rights
        hash ^= CASTLING_HASHES[self.castling_rights.bits() as usize];

        // Hash en passant
        let ep_index = get_en_passant_index(self.en_passent);
        hash ^= EN_PASSANT_HASHES[ep_index];

        // Hash active color
        if self.active_color == Color::Black {
            hash ^= *BLACK_TO_MOVE_HASH;
        }

        hash
    }
}
