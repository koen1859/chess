use crate::chess::{
    chess::Chess,
    color::Color::{Black, White},
    knightattacks::KnightAttacks,
    piece::{Piece, PieceType::*},
    utils::{BitBoard, bit_scan, bitboard_to_string},
};

// Take a chess game (full position), and return a vector of all possible next positions
pub fn generate_moves(game: &Chess) -> Vec<Chess> {
    let mut new_positions: Vec<BitBoard> = vec![];

    for piece in &game.pieces {
        if piece.color == game.active_color {
            match piece.piece_type {
                Knight => {
                    let positions = generate_knight_moves(&piece, &game);
                    new_positions.extend(positions);
                }
                typ => panic!("Unimplemented!"),
            }
        }
    }

    vec![]
}

fn generate_knight_moves(piece: &Piece, game: &Chess) -> Vec<BitBoard> {
    let mut attacks: BitBoard = game.knight_attacks.attacks[bit_scan(piece.position)];

    // Find all the squares occupied by our own pieces
    let own_occupancy: BitBoard = match piece.color {
        White => game.white_occupancy,
        Black => game.black_occupancy,
    };

    // attacks &= -own_occupancy;

    println!(
        "{}",
        bitboard_to_string(attacks, Some(bit_scan(piece.position)))
    );
    vec![attacks]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movegen() {
        let chess: Chess = Chess::from_fen("8/4n3/8/3N1n2/8/2N5/8/8 w - - 0 1");

        println!("{}", chess.to_string());

        println!("{}", bitboard_to_string(chess.white_occupancy, None));

        println!("{}", bitboard_to_string(chess.black_occupancy, None));

        generate_moves(&chess);
    }
}
