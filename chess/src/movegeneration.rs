use crate::{
    apply_undo_move::{Move, MoveFlags},
    castling_rights::CastlingRights,
    chess::Chess,
    color::{
        Color,
        Color::{Black, White},
    },
    movelist::MoveList,
    moves::{
        king::KING_MOVES,
        knight::KNIGHT_MOVES,
        pawn::{
            BLACK_PAWN_ATTACKS, BLACK_PAWN_MOVES_1, BLACK_PAWN_MOVES_2, WHITE_PAWN_ATTACKS,
            WHITE_PAWN_MOVES_1, WHITE_PAWN_MOVES_2,
        },
        ray::{diagonal_attacks, straight_attacks},
    },
    utils::{BitBoard, bit_scan},
};

// Square indices for castling moves, used to check if squares between king and rook are empty and not attacked
const B1: usize = 1;
const C1: usize = 2;
const D1: usize = 3;

const F1: usize = 5;
const G1: usize = 6;

const B8: usize = 57;
const C8: usize = 58;
const D8: usize = 59;

const F8: usize = 61;
const G8: usize = 62;

impl Chess {
    // Engine hot path - no heap allocation, uses apply/undo for legality
    pub fn generate_moves_into(&mut self, color: Color, moves: &mut MoveList) {
        moves.clear();
        self.generate_pseudolegal_moves_into(color, moves);

        let mut write = 0;
        for read in 0..moves.len() {
            let m = moves.get(read);
            let history = self.apply_move(m);
            if !self.is_color_in_check(color) {
                if write != read {
                    moves.swap(write, read);
                }
                write += 1;
            }
            self.undo_move(&history);
        }
        moves.truncate(write);

        // Sort by victim value - attacker value (MVV-LVA)
        let squares = &self.squares;
        moves.sort_by(|a, b| {
            let score_a: i32 = squares[a.to].value() - squares[a.from].value();
            let score_b: i32 = squares[b.to].value() - squares[b.from].value();
            score_b.cmp(&score_a)
        });
    }

    // Public API for UI/tests - returns Vec (heap alloc, not hot path)
    pub fn generate_moves(&self, color: Color) -> Vec<Move> {
        let mut buf = MoveList::new();
        self.generate_pseudolegal_moves_into(color, &mut buf);
        let mut legal: Vec<Move> = buf
            .as_slice()
            .iter()
            .filter(|m| !self.leaves_king_in_check(m))
            .copied()
            .collect();
        legal.sort_by(|a, b| {
            let score_a: i32 = self.squares[a.to].value() - self.squares[a.from].value();
            let score_b: i32 = self.squares[b.to].value() - self.squares[b.from].value();
            score_b.cmp(&score_a)
        });
        legal
    }

    // Generate pseudo legal moves: Does not account for checks
    fn generate_pseudolegal_moves_into(&self, color: Color, moves: &mut MoveList) {
        if self.halfmove_clock >= 100 {
            return;
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

        generate_moves_for_piece_type(knights, own_occ, enemy_occ, |sq| KNIGHT_MOVES[sq], moves);

        generate_moves_for_piece_type(
            bishops,
            own_occ,
            enemy_occ,
            |sq| diagonal_attacks(sq, all_occ),
            moves,
        );

        generate_moves_for_piece_type(
            rooks,
            own_occ,
            enemy_occ,
            |sq| straight_attacks(sq, all_occ),
            moves,
        );

        generate_moves_for_piece_type(
            queens,
            own_occ,
            enemy_occ,
            |sq| straight_attacks(sq, all_occ) | diagonal_attacks(sq, all_occ),
            moves,
        );

        generate_moves_for_piece_type(king, own_occ, enemy_occ, |sq| KING_MOVES[sq], moves);

        generate_pawn_moves(pawns, own_occ, enemy_occ, self.active_color, moves);

        generate_en_passent_moves(pawns, self.en_passent, self.active_color, moves);

        generate_castling_moves(self, moves);
    }
}

pub fn generate_moves_for_piece_type<F>(
    mut pieces: BitBoard,
    own_occ: BitBoard,
    enemy_occ: BitBoard,
    attack_fn: F,
    moves: &mut MoveList,
) where
    F: Fn(usize) -> BitBoard,
{
    while pieces != 0 {
        let from: usize = bit_scan(pieces);
        let mut targets: BitBoard = attack_fn(from) & !own_occ;

        while targets != 0 {
            let to = bit_scan(targets);

            let flags = if (enemy_occ & (1u64 << to)) != 0 {
                MoveFlags::CAPTURE
            } else {
                MoveFlags::empty()
            };
            moves.push(Move {
                from: from,
                to: to,
                flags: flags,
            });
            targets &= targets - 1;
        }
        pieces &= pieces - 1;
    }
}

fn push_pawn_move(
    moves: &mut MoveList,
    from: usize,
    to: usize,
    active_color: Color,
    capture: bool,
) {
    let is_promotion = match active_color {
        White => to >= 56,
        Black => to < 8,
    };

    let base_flags = if capture {
        MoveFlags::CAPTURE
    } else {
        MoveFlags::empty()
    };

    if is_promotion {
        moves.push(Move {
            from,
            to,
            flags: base_flags | MoveFlags::PROMOTION_QUEEN,
        });
        moves.push(Move {
            from,
            to,
            flags: base_flags | MoveFlags::PROMOTION_ROOK,
        });
        moves.push(Move {
            from,
            to,
            flags: base_flags | MoveFlags::PROMOTION_BISHOP,
        });
        moves.push(Move {
            from,
            to,
            flags: base_flags | MoveFlags::PROMOTION_KNIGHT,
        });
    } else {
        moves.push(Move {
            from,
            to,
            flags: base_flags,
        });
    }
}

fn generate_pawn_moves(
    pawns: BitBoard,
    own_occ: BitBoard,
    enemy_occ: BitBoard,
    active_color: Color,
    moves: &mut MoveList,
) {
    let all_occ = own_occ | enemy_occ;

    let mut remaining = pawns;

    while remaining != 0 {
        let from = bit_scan(remaining);

        // ---------- Single push ----------
        let mut single_pushes = match active_color {
            White => WHITE_PAWN_MOVES_1[from],
            Black => BLACK_PAWN_MOVES_1[from],
        };

        single_pushes &= !all_occ;

        while single_pushes != 0 {
            let to = bit_scan(single_pushes);

            push_pawn_move(moves, from, to, active_color, false);

            single_pushes &= single_pushes - 1;
        }

        let double_pushes = match active_color {
            White => WHITE_PAWN_MOVES_2[from],
            Black => BLACK_PAWN_MOVES_2[from],
        };

        if double_pushes != 0 {
            let to = bit_scan(double_pushes);

            let intermediate = match active_color {
                White => from + 8,
                Black => from - 8,
            };

            let intermediate_bb = 1u64 << intermediate;
            let destination_bb = 1u64 << to;

            if (all_occ & intermediate_bb) == 0 && (all_occ & destination_bb) == 0 {
                moves.push(Move {
                    from,
                    to,
                    flags: MoveFlags::empty(),
                });
            }
        }

        let mut captures = match active_color {
            White => WHITE_PAWN_ATTACKS[from] & enemy_occ,
            Black => BLACK_PAWN_ATTACKS[from] & enemy_occ,
        };

        while captures != 0 {
            let to = bit_scan(captures);

            push_pawn_move(moves, from, to, active_color, true);

            captures &= captures - 1;
        }

        remaining &= remaining - 1;
    }
}

fn generate_en_passent_moves(
    mut pawns: BitBoard,
    en_passent: BitBoard,
    active_color: Color,
    moves: &mut MoveList,
) {
    if en_passent == 0 {
        return;
    }

    let ep_square: usize = bit_scan(en_passent);
    while pawns != 0 {
        let from: usize = bit_scan(pawns);
        let attacks = match active_color {
            White => WHITE_PAWN_ATTACKS[from],
            Black => BLACK_PAWN_ATTACKS[from],
        };

        if attacks & en_passent != 0 {
            moves.push(Move {
                from: from,
                to: ep_square,
                flags: MoveFlags::EN_PASSENT | MoveFlags::CAPTURE,
            });
        }
        pawns &= pawns - 1;
    }
}

fn generate_castling_moves(chess: &Chess, moves: &mut MoveList) {
    if chess.white_king == 0 || chess.black_king == 0 {
        return;
    }

    let (king_sq, enemy_color) = match chess.active_color {
        White => (bit_scan(chess.white_king), Color::Black),
        Black => (bit_scan(chess.black_king), Color::White),
    };

    if chess.is_square_attacked_by_color(king_sq, enemy_color) {
        return;
    }

    let all_occ = chess.white_occupancy() | chess.black_occupancy();
    match chess.active_color {
        White => {
            if chess
                .castling_rights
                .contains(CastlingRights::WHITEKINGSIDE)
            {
                if (all_occ & ((1u64 << F1) | (1u64 << G1))) == 0 {
                    if !chess.is_square_attacked_by_color(F1, enemy_color)
                        && !chess.is_square_attacked_by_color(G1, enemy_color)
                    {
                        moves.push(Move {
                            from: king_sq,
                            to: G1,
                            flags: MoveFlags::CASTLE_KINGSIDE,
                        });
                    }
                }
            }
            if chess
                .castling_rights
                .contains(CastlingRights::WHITEQUEENSIDE)
            {
                if (all_occ & ((1u64 << B1) | (1u64 << C1) | (1u64 << D1))) == 0 {
                    if !chess.is_square_attacked_by_color(C1, enemy_color)
                        && !chess.is_square_attacked_by_color(D1, enemy_color)
                    {
                        moves.push(Move {
                            from: king_sq,
                            to: C1,
                            flags: MoveFlags::CASTLE_QUEENSIDE,
                        });
                    }
                }
            }
        }
        Black => {
            if chess
                .castling_rights
                .contains(CastlingRights::BLACKKINGSIDE)
            {
                if (all_occ & ((1u64 << F8) | (1u64 << G8))) == 0 {
                    if !chess.is_square_attacked_by_color(F8, enemy_color)
                        && !chess.is_square_attacked_by_color(G8, enemy_color)
                    {
                        moves.push(Move {
                            from: king_sq,
                            to: G8,
                            flags: MoveFlags::CASTLE_KINGSIDE,
                        });
                    }
                }
            }
            if chess
                .castling_rights
                .contains(CastlingRights::BLACKQUEENSIDE)
            {
                if (all_occ & ((1u64 << B8) | (1u64 << C8) | (1u64 << D8))) == 0 {
                    if !chess.is_square_attacked_by_color(C8, enemy_color)
                        && !chess.is_square_attacked_by_color(D8, enemy_color)
                    {
                        moves.push(Move {
                            from: king_sq,
                            to: C8,
                            flags: MoveFlags::CASTLE_QUEENSIDE,
                        });
                    }
                }
            }
        }
    }
}
