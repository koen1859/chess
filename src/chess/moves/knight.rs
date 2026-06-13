use crate::chess::utils::{BitBoard, set_bit};

pub const KNIGHT_MOVES: [BitBoard; 64] = generate_knight_attacks();

const fn generate_knight_attacks() -> [BitBoard; 64] {
    let mut attacks = [0; 64];

    let mut row = 1;
    while row <= 8 {
        let mut col = 1;
        while col <= 8 {
            attacks[((row - 1) * 8 + (col - 1)) as usize] = knight_attacks(row, col);
            col += 1;
        }
        row += 1;
    }

    attacks
}

const fn knight_attacks(row: i32, col: i32) -> BitBoard {
    let mut bitboard = 0;

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

    let mut i = 0;
    while i < attack_pairs.len() {
        let (r, c) = attack_pairs[i];
        bitboard = set_bit(bitboard, row + r, col + c);
        i += 1;
    }

    bitboard
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard_to_string;

    #[test]
    fn print_knight_attacks() {
        println!("{}", bitboard_to_string(KNIGHT_MOVES[3], Some(3)));
        println!("{}", bitboard_to_string(KNIGHT_MOVES[30], Some(30)));
        println!("{}", bitboard_to_string(KNIGHT_MOVES[39], Some(39)));
    }
}
