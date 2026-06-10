use std::{char, collections::VecDeque};
pub type BitBoard = u64;

use crate::chess::{
    color::Color,
    piece::{Piece, PiecePos, PieceType},
    square::Square,
};

pub fn bit_to_pos(bit: PiecePos) -> Result<String, String> {
    if bit == 0 {
        Err("No piece present!".to_string())
    } else {
        let onebit_index = bit_scan(bit);
        Ok(index_to_pos(onebit_index))
    }
}

pub fn pos_to_bit(pos: &str) -> Result<PiecePos, String> {
    if pos.len() != 2 {
        return Err(format!(
            "Invalid length for piece position: {}, string: '{}'",
            pos.len(),
            pos
        ));
    }

    let bytes = pos.as_bytes();

    // The letter of the pos string, a-h. 97 = a, 98 = b, etc. So byte0 must be >= 97 and < 97 + 8
    let byte0 = bytes[0];
    if byte0 < 97 || byte0 >= 97 + 8 {
        return Err(format!(
            "Invalid column character: {}, string '{}'",
            byte0 as char, pos
        ));
    }

    let column = (byte0 - 97) as u32;
    let row: u32;

    let byte1 = bytes[1];
    match (byte1 as char).to_digit(10) {
        Some(number) => {
            if number < 1 || number > 8 {
                return Err(format!(
                    "Invalid row character: {}, string: '{}'",
                    byte1 as char, pos
                ));
            } else {
                row = number - 1;
            }
        }
        None => {
            return Err(format!(
                "Invalid row character: {}, string: '{}'",
                byte1 as char, pos
            ));
        }
    }

    let square_number = row * 8 + column;
    let bit = (1 as BitBoard) << square_number;

    Ok(bit)
}

pub fn index_to_pos(index: usize) -> String {
    let column: usize = index % 8;
    let row: usize = index / 8 + 1;
    format!("{}{}", char::from_u32((97 + column) as u32).unwrap(), row)
}

// Returns the number of trailing zeros of a BitBoard
// So: 00101110 -> 1
pub fn bit_scan(bit: BitBoard) -> usize {
    bit.trailing_zeros() as usize
}

// Returns number of bits after the highest nonzero bit
// So: 00101110 -> 5
pub fn bit_scan_backward(bit: BitBoard) -> usize {
    63 - bit.leading_zeros() as usize
}

// s: "ABCDEF", sep: 'C' -> ("AB", "DEF")
pub fn split_on(s: &str, sep: char) -> (&str, &str) {
    s.chars()
        .enumerate()
        .find(|&(_, c)| c == sep)
        .map(|(index, _)| (&s[0..index], &s[index + 1..]))
        .unwrap_or((&s[..], ""))
}

pub fn parse_row(
    row: &str,
    mut piece_index: usize,
    mut piece_position: usize,
) -> (Vec<Piece>, VecDeque<Square>) {
    let mut pieces = Vec::new();
    let mut squares = VecDeque::new();

    let mut color: Color;

    macro_rules! add_piece {
        ($piece_type:ident) => {{
            pieces.push(Piece::new(
                (1 as BitBoard) << piece_position,
                color,
                PieceType::$piece_type,
            ));
            squares.push_front(Square::Occupied(piece_index));
            piece_position += 1;
            piece_index += 1;
        }};
    }

    for c in row.chars() {
        let is_upper = c.is_ascii_uppercase();
        color = if is_upper { Color::White } else { Color::Black };
        match c.to_ascii_lowercase() {
            'r' => add_piece!(Rook),
            'n' => add_piece!(Knight),
            'b' => add_piece!(Bishop),
            'q' => add_piece!(Queen),
            'k' => add_piece!(King),
            'p' => add_piece!(Pawn),
            num => match num.to_digit(10) {
                None => panic!("Invalid input: {}", num),
                Some(number) => {
                    for _ in 0..number {
                        squares.push_front(Square::Empty);
                        piece_position += 1;
                    }
                }
            },
        }
    }

    (pieces, squares)
}

pub fn set_bit(bitboard: BitBoard, row: i32, col: i32) -> BitBoard {
    if row < 1 || row > 8 || col < 1 || col > 8 {
        return bitboard;
    }
    bitboard | (1 << ((col - 1) + (row - 1) * 8))
}

pub fn bitboard_to_string(bitboard: BitBoard, mark: Option<usize>) -> String {
    let mut row = String::new();
    let mut board = String::new();

    for i in 0..64 {
        let value = (bitboard >> i) & 1; // Get the bit value of each board position

        let s = if value == 0 {
            String::from(".")
        } else {
            value.to_string()
        };

        match mark {
            Some(idx) => {
                if i == idx {
                    row.push_str("X");
                } else {
                    row.push_str(&s);
                }
            }
            None => row.push_str(&s),
        }

        if (i + 1) % 8 == 0 {
            row.push_str("\n");
            board.insert_str(0, &row);
            row.clear();
        }
    }

    board
}
