use crate::{
    chess::Chess,
    color::Color,
    color::Color::{Black, White},
    moves::{
        king::KING_MOVES,
        knight::KNIGHT_MOVES,
        pawn::{
            BLACK_PAWN_ATTACKS, BLACK_PAWN_MOVES_1, BLACK_PAWN_MOVES_2, WHITE_PAWN_ATTACKS,
            WHITE_PAWN_MOVES_1, WHITE_PAWN_MOVES_2,
        },
        ray::{diagonal_attacks, straight_attacks},
    },
    utils::{BitBoard, bit_scan, count_ones},
};

const D4: usize = 27;
const E4: usize = 28;
const D5: usize = 35;
const E5: usize = 36;

const CENTER: BitBoard = (1u64 << D4) | (1u64 << E4) | (1u64 << D5) | (1u64 << E5);

const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];

const QUEEN_TABLE: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const MG_KING_TABLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

const EG_KING_TABLE: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
];

impl Chess {
    // Returns negative if black is better and positive if white is better
    pub fn evaluate(&self) -> i32 {
        let mut score = 0;
        score += self.material_score();

        // Calculate for both middle game and end game
        let mg_score = self.calculate_pst_score(true);
        let eg_score = self.calculate_pst_score(false);
        let phase = self.get_game_phase();
        score += (mg_score * phase + eg_score * (24 - phase)) / 24;

        // Mobility score
        score += self.mobility_score() / 2;

        // Development bonus in opening
        if phase > 18 {
            score += 10 * count_ones(self.white_knights & (1 << 1 | 1 << 6)) as i32;
            score += 10 * count_ones(self.white_bishops & (1 << 2 | 1 << 5)) as i32;
            score -= 10 * count_ones(self.black_knights & (1 << 57 | 1 << 62)) as i32;
            score -= 10 * count_ones(self.black_bishops & (1 << 58 | 1 << 61)) as i32;

            score += 20 * count_ones(self.white_pawns & CENTER) as i32;
            score -= 20 * count_ones(self.black_pawns & CENTER) as i32;
        }

        // bonus for bishop pair
        if count_ones(self.white_bishops) >= 2 {
            score += 30;
        }
        if count_ones(self.black_bishops) >= 2 {
            score -= 30;
        }

        score
    }
    // Game phase (0-24)
    fn get_game_phase(&self) -> i32 {
        i32::clamp(
            count_ones(self.white_knights | self.black_knights)
                + count_ones(self.white_bishops | self.black_bishops)
                + 2 * count_ones(self.white_rooks | self.black_rooks)
                + 4 * count_ones(self.white_queens | self.black_queens),
            0,
            24,
        )
    }
    pub fn material_score(&self) -> i32 {
        let mut score: i32 = 0;

        score += 100 * (count_ones(self.white_pawns) - count_ones(self.black_pawns));
        score += 350 * (count_ones(self.white_knights) - count_ones(self.black_knights));
        score += 350 * (count_ones(self.white_bishops) - count_ones(self.black_bishops));
        score += 525 * (count_ones(self.white_rooks) - count_ones(self.black_rooks));
        score += 1000 * (count_ones(self.white_queens) - count_ones(self.black_queens));

        score
    }
    fn mobility_score(&self) -> i32 {
        // Count pseudolegal moves without allocating any Move structs
        let white_count = self.count_pseudolegal_moves(White);
        let black_count = self.count_pseudolegal_moves(Black);
        white_count - black_count
    }

    fn count_pseudolegal_moves(&self, color: Color) -> i32 {
        if self.halfmove_clock >= 100 {
            return 0;
        }

        let (rooks, knights, bishops, queens, king, pawns, own_occ, enemy_occ) = match color {
            White => (
                self.white_rooks,
                self.white_knights,
                self.white_bishops,
                self.white_queens,
                self.white_king,
                self.white_pawns,
                self.white_occupancy(),
                self.black_occupancy(),
            ),
            Black => (
                self.black_rooks,
                self.black_knights,
                self.black_bishops,
                self.black_queens,
                self.black_king,
                self.black_pawns,
                self.black_occupancy(),
                self.white_occupancy(),
            ),
        };
        let all_occ: BitBoard = own_occ | enemy_occ;

        let mut count: i32 = 0;

        // Knights
        count += count_attacks(knights, own_occ, |sq| KNIGHT_MOVES[sq]);
        // Bishops
        count += count_attacks(bishops, own_occ, |sq| diagonal_attacks(sq, all_occ));
        // Rooks
        count += count_attacks(rooks, own_occ, |sq| straight_attacks(sq, all_occ));
        // Queens
        count += count_attacks(queens, own_occ, |sq| {
            straight_attacks(sq, all_occ) | diagonal_attacks(sq, all_occ)
        });
        // King
        count += count_attacks(king, own_occ, |sq| KING_MOVES[sq]);
        // Pawns
        count += count_pawn_moves(pawns, own_occ, enemy_occ, color);
        // En passant
        if self.en_passent != 0 {
            count += count_ep_moves(pawns, self.en_passent, color);
        }

        count
    }
    fn calculate_pst_score(&self, middlegame: bool) -> i32 {
        let (pawn_t, knight_t, bishop_t, rook_t, queen_t, king_t) = if middlegame {
            (
                &PAWN_TABLE,
                &KNIGHT_TABLE,
                &BISHOP_TABLE,
                &ROOK_TABLE,
                &QUEEN_TABLE,
                &MG_KING_TABLE,
            )
        } else {
            (
                &PAWN_TABLE,
                &KNIGHT_TABLE,
                &BISHOP_TABLE,
                &ROOK_TABLE,
                &QUEEN_TABLE,
                &EG_KING_TABLE,
            )
        };

        let mut score = 0;

        // Add White
        score += self.evaluate_piece_pst(self.white_pawns, pawn_t, false);
        score += self.evaluate_piece_pst(self.white_knights, knight_t, false);
        score += self.evaluate_piece_pst(self.white_bishops, bishop_t, false);
        score += self.evaluate_piece_pst(self.white_rooks, rook_t, false);
        score += self.evaluate_piece_pst(self.white_queens, queen_t, false);
        score += self.evaluate_piece_pst(self.white_king, king_t, false);

        // Subtract Black (flip indices to match the table orientation)
        score -= self.evaluate_piece_pst(self.black_pawns, pawn_t, true);
        score -= self.evaluate_piece_pst(self.black_knights, knight_t, true);
        score -= self.evaluate_piece_pst(self.black_bishops, bishop_t, true);
        score -= self.evaluate_piece_pst(self.black_rooks, rook_t, true);
        score -= self.evaluate_piece_pst(self.black_queens, queen_t, true);
        score -= self.evaluate_piece_pst(self.black_king, king_t, true);

        score
    }
    fn evaluate_piece_pst(&self, mut pieces: BitBoard, table: &[i32; 64], flip: bool) -> i32 {
        let mut score = 0;
        while pieces != 0 {
            let sq = bit_scan(pieces);
            let table_idx = if flip { sq ^ 56 } else { sq };
            score += table[table_idx];
            pieces &= pieces - 1;
        }
        score
    }
}

fn count_attacks<F>(mut pieces: BitBoard, own_occ: BitBoard, attack_fn: F) -> i32
where
    F: Fn(usize) -> BitBoard,
{
    let mut count = 0;
    while pieces != 0 {
        let from = bit_scan(pieces);
        count += count_ones(attack_fn(from) & !own_occ) as i32;
        pieces &= pieces - 1;
    }
    count
}

fn count_pawn_moves(
    mut pawns: BitBoard,
    own_occ: BitBoard,
    enemy_occ: BitBoard,
    color: Color,
) -> i32 {
    let all_occ = own_occ | enemy_occ;
    let mut count = 0i32;

    while pawns != 0 {
        let from = bit_scan(pawns);

        // Single pushes
        let mut single = match color {
            White => WHITE_PAWN_MOVES_1[from],
            Black => BLACK_PAWN_MOVES_1[from],
        } & !all_occ;

        while single != 0 {
            let to = bit_scan(single);
            count += if is_promotion_rank(to, color) { 4 } else { 1 };
            single &= single - 1;
        }

        // Double pushes
        let double_mask = match color {
            White => WHITE_PAWN_MOVES_2[from],
            Black => BLACK_PAWN_MOVES_2[from],
        };
        if double_mask != 0 {
            let to = bit_scan(double_mask);
            let intermediate = match color {
                White => from + 8,
                Black => from - 8,
            };
            if (all_occ & (1u64 << intermediate)) == 0 && (all_occ & (1u64 << to)) == 0 {
                count += 1;
            }
        }

        // Captures
        let mut captures = match color {
            White => WHITE_PAWN_ATTACKS[from] & enemy_occ,
            Black => BLACK_PAWN_ATTACKS[from] & enemy_occ,
        };
        while captures != 0 {
            let to = bit_scan(captures);
            count += if is_promotion_rank(to, color) { 4 } else { 1 };
            captures &= captures - 1;
        }

        pawns &= pawns - 1;
    }

    count
}

fn count_ep_moves(mut pawns: BitBoard, en_passent: BitBoard, color: Color) -> i32 {
    if en_passent == 0 {
        return 0;
    }
    let mut count = 0i32;
    while pawns != 0 {
        let from = bit_scan(pawns);
        let attacks = match color {
            White => WHITE_PAWN_ATTACKS[from],
            Black => BLACK_PAWN_ATTACKS[from],
        };
        if attacks & en_passent != 0 {
            count += 1;
        }
        pawns &= pawns - 1;
    }
    count
}

fn is_promotion_rank(sq: usize, color: Color) -> bool {
    match color {
        White => sq >= 56,
        Black => sq < 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_starting_position() {
        let chess = Chess::new();
        let score = chess.evaluate();
        assert_eq!(score, 0, "Starting position should be evaluated as 0");
    }
}
