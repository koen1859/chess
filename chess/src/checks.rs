use crate::{
    apply_undo_move::{History, Move},
    chess::Chess,
    color::Color::{self, Black, White},
    moves::{
        king::KING_MOVES,
        knight::KNIGHT_MOVES,
        pawn::{BLACK_PAWN_ATTACKS, WHITE_PAWN_ATTACKS},
        ray::{diagonal_attacks, straight_attacks},
    },
    utils::{BitBoard, bit_scan},
};

impl Chess {
    pub fn leaves_king_in_check_slow(&self, m: &Move) -> bool {
        let mut clone = self.clone();
        let king_color: Color = clone.active_color;
        clone.apply_move(m);
        clone.is_color_in_check(king_color)
    }
    pub fn leaves_king_in_check(&mut self, m: &Move) -> bool {
        let king_color: Color = self.active_color;
        let history: History = self.apply_move(m);
        let check: bool = self.is_color_in_check(king_color);
        self.undo_move(&history);
        check
    }
    pub fn is_check(&mut self, m: &Move) -> bool {
        let history: History = self.apply_move(m);
        let check: bool = self.is_color_in_check(self.active_color);
        self.undo_move(&history);
        check
    }
    pub fn is_color_in_check(&self, color: Color) -> bool {
        let (own_king_bb, opp_color) = match color {
            White => (self.white_king, Black),
            Black => (self.black_king, White),
        };

        if own_king_bb == 0 {
            return false;
        }

        self.is_square_attacked_by_color(bit_scan(own_king_bb), opp_color)
    }
    pub fn is_square_attacked_by_color(&self, target: usize, att_color: Color) -> bool {
        self.is_attacked_by_pawn(target, att_color)
            || self.is_attacked_by_knight(target, att_color)
            || self.is_attacked_by_ray(target, att_color)
            || self.is_attacked_by_king(target, att_color)
    }
    pub fn is_attacked_by_pawn(&self, target: usize, att_color: Color) -> bool {
        let (pawns, attacks) = match att_color {
            White => (self.white_pawns, BLACK_PAWN_ATTACKS[target]),
            Black => (self.black_pawns, WHITE_PAWN_ATTACKS[target]),
        };

        (attacks & pawns) != 0
    }
    pub fn is_attacked_by_knight(&self, target: usize, att_color: Color) -> bool {
        let knights: BitBoard = match att_color {
            White => self.white_knights,
            Black => self.black_knights,
        };
        // Knight moves are symmetric, so can use the attack map in reverse
        (KNIGHT_MOVES[target] & knights) != 0
    }
    pub fn is_attacked_by_ray(&self, target: usize, att_color: Color) -> bool {
        let (bishops, rooks, queens) = match att_color {
            White => (self.white_bishops, self.white_rooks, self.white_queens),
            Black => (self.black_bishops, self.black_rooks, self.black_queens),
        };
        let occupancy: BitBoard = self.white_occupancy() | self.black_occupancy();

        let diagonal_attackers: BitBoard = diagonal_attacks(target, occupancy) & (bishops | queens);
        if diagonal_attackers != 0 {
            return true;
        }

        let straight_attackers: BitBoard = straight_attacks(target, occupancy) & (rooks | queens);

        straight_attackers != 0
    }
    pub fn is_attacked_by_king(&self, target: usize, att_color: Color) -> bool {
        let att_king: BitBoard = match att_color {
            White => self.white_king,
            Black => self.black_king,
        };

        // Same as for knights, king moves are symmetric
        (KING_MOVES[target] & att_king) != 0
    }
}
