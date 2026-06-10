mod chess;
use crate::chess::chess::Chess;

fn main() {
    let chess: Chess = Chess::new();

    println!("{}", chess.to_string());
    println!("Active Color: {:?}", chess.active_color());
    println!("En passent: {:?}", chess.en_passent());
    println!("Full move number: {:?}", chess.fullmove_number());
}
