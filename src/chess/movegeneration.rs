use crate::chess::{
    chess::Chess,
    color::Color::{Black, White},
    knightattacks::KNIGHT_ATTACKS,
    utils::{BitBoard, bit_scan, extract_bits},
};
use bitflags::bitflags;

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub flags: MoveFlags,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MoveFlags: u8 {
        const CAPTURE=1;
        const EN_PASSENT=2;
        const CASTLE=4;
        const PROMOTION=8;
    }
}

impl Chess {
    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        self.generate_knight_moves(&mut moves);
        moves
    }

    fn generate_knight_moves(&self, moves: &mut Vec<Move>) {
        let (knights_unmut, own_occ, enemy_occ) = match self.active_color {
            White => (
                self.white_knights,
                self.white_occupancy(),
                self.black_occupancy(),
            ),
            Black => (
                self.black_knights,
                self.black_occupancy(),
                self.white_occupancy(),
            ),
        };
        let mut knights: BitBoard = knights_unmut;

        while knights != 0 {
            let from: usize = bit_scan(knights);
            let mut targets: BitBoard = KNIGHT_ATTACKS[from] & !own_occ;

            while targets != 0 {
                let to: usize = bit_scan(targets);
                let flags = if (enemy_occ >> to) & 1 != 0 {
                    MoveFlags::CAPTURE
                } else {
                    MoveFlags::empty()
                };
                moves.push(Move {
                    from: from as u8,
                    to: to as u8,
                    flags: flags,
                });
                targets &= targets - 1;
            }
            knights &= knights - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard_to_string;

    #[test]
    fn test_movegen() {
        let chess: Chess = Chess::from_fen("8/4n3/8/3N1n2/8/2N5/8/8 w - - 0 1");

        println!("{}", chess.to_string());

        println!("{}", bitboard_to_string(chess.white_occupancy(), None));

        println!("{}", bitboard_to_string(chess.black_occupancy(), None));

        println!("{:?}", chess.generate_moves());
    }
}
