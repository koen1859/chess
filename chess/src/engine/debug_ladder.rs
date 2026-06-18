use crate::chess::Chess;
use crate::engine::engine::Engine;

pub fn debug() {
    let fen = "7k/8/8/8/8/6r1/7r/1K6 b - - 1 1";
    let mut board = Chess::from_fen(fen);
    let mut engine = Engine::new();
    let best = engine.get_best_move(&mut board, 4);
    println!("Best move: {:?}", best.map(|m| m.to_san(&board)));
}
