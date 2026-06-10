use crate::chess::utils::{BitBoard, bitboard_to_string, set_bit};

pub struct KnightAttacks(Vec<BitBoard>);

impl KnightAttacks {
    pub fn new() -> Self {
        let mut attacks: Vec<BitBoard> = vec![];
        for row in 1..=8 {
            for col in 1..=8 {
                let attacks_from_this_square = knight_attacks(row, col);
                attacks.push(attacks_from_this_square);
            }
        }
        Self(attacks)
    }
}

fn knight_attacks(row: i32, col: i32) -> BitBoard {
    let mut bitboard: BitBoard = 0;
    let attack_pairs = [
        (1, 2),
        (1, -2),
        (-1, 2),
        (-1, -2),
        (2, 1),
        (2, -1),
        (-2, 1),
        (-2, -1),
    ];

    for (r, c) in attack_pairs {
        bitboard = set_bit(bitboard, row + r, col + c)
    }
    bitboard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_knight_attacks() {
        let knight_attacks = KnightAttacks::new();
        println!("{}", bitboard_to_string(knight_attacks.0[3], Some(3)));
        println!("{}", bitboard_to_string(knight_attacks.0[30], Some(30)));
        println!("{}", bitboard_to_string(knight_attacks.0[39], Some(39)));
    }
}
