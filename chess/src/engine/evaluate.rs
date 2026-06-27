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
    utils::{BitBoard, FILES, bit_scan, count_ones},
};

const D4: usize = 27;
const E4: usize = 28;
const D5: usize = 35;
const E5: usize = 36;

const CENTER: BitBoard = (1u64 << D4) | (1u64 << E4) | (1u64 << D5) | (1u64 << E5);

const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // First rank not possible
    10, 10, 10, 10, 10, 10, 10, 10, // Second rank and
    10, 10, 10, 10, 10, 10, 10, 10, // third rank are as far away from the end
    20, 20, 20, 20, 20, 20, 20, 20, // 4th rank
    30, 30, 30, 30, 30, 30, 30, 30, // 5th rank
    40, 40, 40, 40, 40, 40, 40, 40, // 6th rank
    50, 50, 50, 50, 50, 50, 50, 50, // 7th rank
    0, 0, 0, 0, 0, 0, 0, 0, // Last rank not possible
];

const KNIGHT_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, // 1st
    -40, -20, 0, 5, 5, 0, -20, -40, // 2nd
    -30, 0, 10, 15, 15, 10, 0, -30, // 3rd
    -30, 5, 15, 20, 20, 15, 5, -30, // 4th
    -30, 0, 15, 20, 20, 15, 0, -30, // 5th
    -30, 5, 10, 15, 15, 10, 5, -30, // 6th
    -40, -20, 0, 5, 5, 0, -20, -40, // 7th
    -50, -40, -30, -30, -30, -30, -40, -50, // 8th
];

const BISHOP_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, // 1st
    -10, 0, 0, 0, 0, 0, 0, -10, // 2nd
    -10, 0, 5, 10, 10, 5, 0, -10, // 3rd
    -10, 5, 5, 10, 10, 5, 5, -10, // 4th
    -10, 0, 10, 10, 10, 10, 0, -10, // 5th
    -10, 10, 10, 10, 10, 10, 10, -10, // 6th
    -10, 5, 0, 0, 0, 0, 5, -10, // 7th
    -20, -10, -10, -10, -10, -10, -10, -20, // 8th
];

const ROOK_TABLE: [i32; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0, // 1st
    -5, 0, 0, 0, 0, 0, 0, -5, // 2nd
    -5, 0, 0, 0, 0, 0, 0, -5, // 3rd
    -5, 0, 0, 0, 0, 0, 0, -5, // 4th
    -5, 0, 0, 0, 0, 0, 0, -5, // 5th
    -5, 0, 0, 0, 0, 0, 0, -5, // 6th
    5, 10, 10, 10, 10, 10, 10, 5, // 7th
    0, 0, 0, 0, 0, 0, 0, 0, // 8th
];

const QUEEN_TABLE: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, // 1st
    -10, 0, 0, 0, 0, 0, 0, -10, // 2nd
    -10, 0, 5, 5, 5, 5, 0, -10, // 3rd
    -5, 0, 5, 5, 5, 5, 0, -5, // 4th
    0, 0, 5, 5, 5, 5, 0, 0, // 5th
    -10, 5, 5, 5, 5, 5, 0, -10, // 6th
    -10, 0, 5, 0, 0, 0, 0, -10, // 7th
    -20, -10, -10, -5, -5, -10, -10, -20, // 8th
];

const MG_KING_TABLE: [i32; 64] = [
    20, 30, 10, 0, 0, 10, 30, 20, // 1st
    20, 20, 0, 0, 0, 0, 20, 20, // 2nd
    -10, -20, -20, -20, -20, -20, -20, -10, // 3rd
    -20, -30, -30, -40, -40, -30, -30, -20, // 4th
    -30, -40, -40, -50, -50, -40, -40, -30, // 5th
    -30, -40, -40, -50, -50, -40, -40, -30, // 6th
    -30, -40, -40, -50, -50, -40, -40, -30, // 7th
    -30, -40, -40, -50, -50, -40, -40, -30, // 8th
];

const EG_KING_TABLE: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, // 1st
    -30, -20, -10, 0, 0, -10, -20, -30, // 2nd
    -30, -10, 20, 30, 30, 20, -10, -30, // 3rd
    -30, -10, 30, 40, 40, 30, -10, -30, // 4th
    -30, -10, 30, 40, 40, 30, -10, -30, // 5th
    -30, -10, 20, 30, 30, 20, -10, -30, // 6th
    -30, -30, 0, 0, 0, 0, -30, -30, // 7th
    -50, -30, -30, -30, -30, -30, -30, -50, // 8th
];

impl Chess {
    // Returns positive if better for the side to move, negative if worse
    pub fn evaluate_stm(&self) -> i32 {
        let score = self.evaluate();
        if self.active_color == Color::White {
            score
        } else {
            -score
        }
    }

    // Returns negative if black is better and positive if white is better
    pub fn evaluate(&self) -> i32 {
        let mut score = 0;

        // Draw detection: 50-move rule
        if self.halfmove_clock >= 100 {
            return 0;
        }

        // Draw detection: threefold repetition
        if self.history.is_repetition(self) {
            return 0;
        }

        score += self.material_score();

        // Calculate positional score for both middle game and end game
        let mg_score = self.calculate_pst_score(true);
        let eg_score = self.calculate_pst_score(false);
        let phase = self.get_game_phase();
        score += (mg_score * phase + eg_score * (24 - phase)) / 24;

        // Mobility score
        // Square root since it is bad if we only have very little moves, but when we have 50 or 30 moves it matters much less.
        score += self.count_pseudolegal_moves(White).isqrt();
        score -= self.count_pseudolegal_moves(Black).isqrt();

        // King safety: Higher score is more safe
        score += self.king_safety(&Color::White);
        score -= self.king_safety(&Color::Black);

        // Development bonus in opening
        if phase > 18 {
            score -= 10 * count_ones(self.white_knights & (1 << 1 | 1 << 6)) as i32;
            score -= 10 * count_ones(self.white_bishops & (1 << 2 | 1 << 5)) as i32;
            score += 10 * count_ones(self.black_knights & (1 << 57 | 1 << 62)) as i32;
            score += 10 * count_ones(self.black_bishops & (1 << 58 | 1 << 61)) as i32;

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

        score += self.count_passed_pawns() * 50;

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
    fn count_passed_pawns(&self) -> i32 {
        let mut score = 0;
        let mut white_passed = 0;
        let mut black_passed = 0;

        let mut white_pawns = self.white_pawns;
        while white_pawns != 0 {
            let sq = bit_scan(white_pawns);
            if is_passed(sq, White, self.black_pawns) {
                white_passed |= 1 << sq;
            }
            white_pawns &= white_pawns - 1;
        }

        let mut black_pawns = self.black_pawns;
        while black_pawns != 0 {
            let sq = bit_scan(black_pawns);
            if is_passed(sq, Black, self.white_pawns) {
                black_passed |= 1 << sq;
            }
            black_pawns &= black_pawns - 1;
        }

        score += count_ones(white_passed);
        score -= count_ones(black_passed);
        score
    }
    fn count_pseudolegal_moves(&self, color: Color) -> i32 {
        let (rooks, knights, bishops, queens, pawns, own_occ, enemy_occ) = match color {
            White => (
                self.white_rooks,
                self.white_knights,
                self.white_bishops,
                self.white_queens,
                self.white_pawns,
                self.white_occupancy(),
                self.black_occupancy(),
            ),
            Black => (
                self.black_rooks,
                self.black_knights,
                self.black_bishops,
                self.black_queens,
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
        // Pawns
        count += count_pawn_moves(pawns, own_occ, enemy_occ, color);
        // En passant
        if self.en_passent != 0 {
            count += count_ep_moves(pawns, self.en_passent, color);
        }

        count
    }
    fn king_safety(&self, color: &Color) -> i32 {
        let (
            king,
            own_pawns,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            enemy_queens,
            enemy_king,
            own_occ,
            enemy_occ,
        ) = match color {
            White => (
                self.white_king,
                self.white_pawns,
                self.black_pawns,
                self.black_knights,
                self.black_bishops,
                self.black_rooks,
                self.black_queens,
                self.black_king,
                self.white_occupancy(),
                self.black_occupancy(),
            ),
            Black => (
                self.black_king,
                self.black_pawns,
                self.white_pawns,
                self.white_knights,
                self.white_bishops,
                self.white_rooks,
                self.white_queens,
                self.white_king,
                self.black_occupancy(),
                self.white_occupancy(),
            ),
        };
        let all_occ: BitBoard = own_occ | enemy_occ;

        let king_sq = bit_scan(king);
        let king_file = king_sq % 8;
        let king_rank = king_sq / 8;
        let kf = king_file as i32;
        let kr = king_rank as i32;

        let king_zone = KING_MOVES[king_sq] | king;

        let mut score: i32 = 0;

        // Files adjacent to king (including king's own file)
        let adj_files: [Option<usize>; 3] = [
            if king_file > 0 {
                Some(king_file - 1)
            } else {
                None
            },
            Some(king_file),
            if king_file < 7 {
                Some(king_file + 1)
            } else {
                None
            },
        ];

        // Front ranks for pawn shield (one and two steps forward)
        let (rank1, rank2): (Option<i32>, Option<i32>) = match color {
            White => (
                if kr + 1 < 8 { Some(kr + 1) } else { None },
                if kr + 2 < 8 { Some(kr + 2) } else { None },
            ),
            Black => (
                if kr - 1 >= 0 { Some(kr - 1) } else { None },
                if kr - 2 >= 0 { Some(kr - 2) } else { None },
            ),
        };

        // Build shield masks
        let mut front_shield: BitBoard = 0;
        let mut deep_shield: BitBoard = 0;
        let mut shield_mask: BitBoard = 0;

        for f_opt in adj_files.iter() {
            if let Some(f) = f_opt {
                if let Some(r) = rank1 {
                    let sq = (r * 8 + *f as i32) as usize;
                    front_shield |= 1 << sq;
                    shield_mask |= 1 << sq;
                }
                if let Some(r) = rank2 {
                    let sq = (r * 8 + *f as i32) as usize;
                    deep_shield |= 1 << sq;
                    shield_mask |= 1 << sq;
                }
            }
        }

        // 1. Pawn shield: friendly pawns in front of king
        let shield_count = count_ones(own_pawns & shield_mask);
        score += shield_count * 15;

        // 5. Storm damage: enemy pawns in shield zone
        let storm_front = count_ones(enemy_pawns & front_shield);
        let storm_deep = count_ones(enemy_pawns & deep_shield);
        score -= storm_front * 30;
        score -= storm_deep * 10;

        // 2. King zone attack count
        // Pawn attacks into zone
        let mut pawn_attacks_bb: BitBoard = 0;
        let mut p = enemy_pawns;
        while p != 0 {
            let sq = bit_scan(p);
            pawn_attacks_bb |= match color {
                White => BLACK_PAWN_ATTACKS[sq],
                Black => WHITE_PAWN_ATTACKS[sq],
            };
            p &= p - 1;
        }
        score -= count_ones(pawn_attacks_bb & king_zone) * 10;

        // Knight attacks into zone
        let mut k = enemy_knights;
        while k != 0 {
            let sq = bit_scan(k);
            if KNIGHT_MOVES[sq] & king_zone != 0 {
                score -= 25;
            }
            k &= k - 1;
        }

        // Bishop attacks into zone
        let mut b = enemy_bishops;
        while b != 0 {
            let sq = bit_scan(b);
            if diagonal_attacks(sq, all_occ) & king_zone != 0 {
                score -= 20;
            }
            b &= b - 1;
        }

        // Rook attacks into zone
        let mut r = enemy_rooks;
        while r != 0 {
            let sq = bit_scan(r);
            if straight_attacks(sq, all_occ) & king_zone != 0 {
                score -= 30;
            }
            r &= r - 1;
        }

        // Queen attacks into zone
        let mut q = enemy_queens;
        while q != 0 {
            let sq = bit_scan(q);
            if (straight_attacks(sq, all_occ) | diagonal_attacks(sq, all_occ)) & king_zone != 0 {
                score -= 45;
            }
            q &= q - 1;
        }

        // King attack into zone
        if KING_MOVES[king_sq] & enemy_king != 0 {
            score -= 10;
        }

        // 3. King tropism: enemy pieces close to the king are weighted more heavily
        let chebyshev = |sq: usize| -> i32 {
            let df = (sq as i32 % 8 - kf).abs();
            let dr = (sq as i32 / 8 - kr).abs();
            df.max(dr)
        };

        let mut tropism = 0;

        let mut pieces = enemy_knights;
        while pieces != 0 {
            let sq = bit_scan(pieces);
            tropism += (6 - chebyshev(sq).min(6)) * 10;
            pieces &= pieces - 1;
        }

        let mut pieces = enemy_bishops;
        while pieces != 0 {
            let sq = bit_scan(pieces);
            tropism += (6 - chebyshev(sq).min(6)) * 10;
            pieces &= pieces - 1;
        }

        let mut pieces = enemy_rooks;
        while pieces != 0 {
            let sq = bit_scan(pieces);
            tropism += (6 - chebyshev(sq).min(6)) * 15;
            pieces &= pieces - 1;
        }

        let mut pieces = enemy_queens;
        while pieces != 0 {
            let sq = bit_scan(pieces);
            tropism += (6 - chebyshev(sq).min(6)) * 30;
            pieces &= pieces - 1;
        }

        score -= tropism;

        // 4. Open file penalty (adjacent files too)
        for f_opt in adj_files.iter() {
            if let Some(f) = f_opt {
                let file_bb = FILES[*f];
                let has_friendly = (own_pawns & file_bb) != 0;
                if !has_friendly {
                    let is_king_file = *f == king_file;
                    let has_enemy = (enemy_pawns & file_bb) != 0;
                    if !has_enemy {
                        // Fully open
                        score -= if is_king_file { 30 } else { 15 };
                    } else {
                        // Semi-open
                        score -= if is_king_file { 15 } else { 8 };
                    }
                }
            }
        }

        // 6. Safe checks: count enemy pieces that can directly check the king
        let diag_atk = diagonal_attacks(king_sq, all_occ);
        let strt_atk = straight_attacks(king_sq, all_occ);

        let pawn_checks = count_ones(
            enemy_pawns
                & match color {
                    White => BLACK_PAWN_ATTACKS[king_sq],
                    Black => WHITE_PAWN_ATTACKS[king_sq],
                },
        );
        score -= pawn_checks * 15;

        let knight_checks = count_ones(enemy_knights & KNIGHT_MOVES[king_sq]);
        score -= knight_checks * 35;

        let bishop_checks = count_ones(diag_atk & enemy_bishops);
        score -= bishop_checks * 25;

        let rook_checks = count_ones(strt_atk & enemy_rooks);
        score -= rook_checks * 35;

        let queen_checks = count_ones((diag_atk | strt_atk) & enemy_queens);
        score -= queen_checks * 50;

        if KING_MOVES[king_sq] & enemy_king != 0 {
            score -= 10;
        }

        score
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

fn is_passed(sq: usize, color: Color, opponent_pawns: BitBoard) -> bool {
    let file_idx = sq % 8;
    let rank_idx = sq / 8;

    let mut check_mask: BitBoard = FILES[file_idx];
    if file_idx > 0 {
        check_mask |= FILES[file_idx - 1];
    }
    if file_idx < 7 {
        check_mask |= FILES[file_idx + 1];
    }

    let mut forward_rank_mask: BitBoard = 0;
    match color {
        White => {
            for r in (rank_idx + 1)..8 {
                forward_rank_mask |= 0xFF << (r * 8);
            }
        }
        Black => {
            for r in 0..rank_idx {
                forward_rank_mask |= 0xFF << (r * 8);
            }
        }
    }

    (opponent_pawns & check_mask & forward_rank_mask) == 0
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
    use crate::apply_undo_move::{Move, MoveFlags};

    use super::*;

    #[test]
    fn test_evaluate_starting_position() {
        let chess = Chess::new();
        let score = chess.evaluate();
        assert_eq!(score, 0, "Starting position should be evaluated as 0");
    }

    #[test]
    fn test_opening_move() {
        let mut chess = Chess::new();

        let move1 = Move {
            from: 12,
            to: 28,
            flags: MoveFlags::empty(),
        };
        let history1 = chess.apply_move(&move1);
        let score1 = chess.evaluate();
        chess.undo_move(&history1);
        println!("Score for King's pawn opening: {}", score1);

        let move2 = Move {
            from: 1,
            to: 18,
            flags: MoveFlags::empty(),
        };
        let history2 = chess.apply_move(&move2);
        let score2 = chess.evaluate();
        chess.undo_move(&history2);
        println!("Score for Knight opening: {}", score2);
    }
}
