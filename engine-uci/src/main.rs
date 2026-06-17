use std::io::{self, BufRead, Write};

use chess::{
    apply_undo_move::{move_to_uci, uci_to_move},
    chess::Chess,
    color::Color,
    engine::engine::Engine,
};

struct UciState {
    board: Chess,
    engine: Engine,
}

impl UciState {
    fn new() -> Self {
        UciState {
            board: Chess::new(),
            engine: Engine::new(),
        }
    }

    fn set_position(&mut self, fen: Option<&str>, moves: &[String]) {
        self.board = match fen {
            Some(fen) => Chess::from_fen(fen),
            None => Chess::new(),
        };

        for m_str in moves {
            if let Some(m) = uci_to_move(m_str, &self.board) {
                self.board.apply_move(&m);
            }
        }
    }

    fn go_depth(&mut self, depth: u8) -> Option<String> {
        let mut best_move = None;
        let mut current_depth = 1u8;
        while current_depth <= depth {
            // Set a generous deadline so time_up won't trigger
            self.engine.deadline =
                Some(std::time::Instant::now() + std::time::Duration::from_secs(3600));
            self.engine.time_up = false;
            self.engine.nodes = 0;

            if let Some(m) = self.engine.get_best_move(&mut self.board, current_depth) {
                best_move = Some(move_to_uci(&m));
            } else {
                break;
            }
            current_depth += 1;
        }
        best_move
    }

    fn go_movetime(&mut self, ms: u64) -> Option<String> {
        self.engine
            .get_best_move_in_time(&mut self.board, ms)
            .map(|m| move_to_uci(&m))
    }
}

fn main() {
    let mut state = UciState::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "uci" => {
                println!("id name v0.5.0");
                println!("id author KoenStevens");
                println!("uciok");
            }
            "debug" => {}
            "isready" => {
                println!("readyok");
            }
            "setoption" => {}
            "register" => {}
            "ucinewgame" => {
                state = UciState::new();
            }
            "position" => {
                let mut fen: Option<String> = None;

                let mut i = 1;
                if i < parts.len() && parts[i] == "startpos" {
                    fen = None;
                    i += 1;
                } else if i < parts.len() && parts[i] == "fen" {
                    i += 1;
                    let mut fen_parts = Vec::new();
                    while i < parts.len() && parts[i] != "moves" {
                        fen_parts.push(parts[i]);
                        i += 1;
                    }
                    if !fen_parts.is_empty() {
                        fen = Some(fen_parts.join(" "));
                    }
                }

                if i < parts.len() && parts[i] == "moves" {
                    i += 1;
                }

                let moves: Vec<String> = parts[i..].iter().map(|s| s.to_string()).collect();
                state.set_position(fen.as_deref(), &moves);
            }
            "go" => {
                let mut depth = 10u8;
                let mut movetime = None;

                let mut i = 1;
                while i < parts.len() {
                    match parts[i] {
                        "depth" => {
                            i += 1;
                            depth = parts[i].parse().unwrap_or(10);
                        }
                        "movetime" => {
                            i += 1;
                            movetime = Some(parts[i].parse().unwrap_or(100)); // 0.1 sec per move
                        }
                        "wtime" => {
                            i += 1;
                            let white_time: u64 =
                                parts.get(i).and_then(|s| s.parse().ok()).unwrap_or(60000);
                            i += 1;
                            // Parts are interleaved: wtime <val> btime <val> winc <val> binc <val>
                            // We need to handle all time parameters together
                            let black_time = if i + 1 < parts.len() && parts[i] == "btime" {
                                let v: u64 = parts[i + 1].parse().unwrap_or(60000);
                                i += 2;
                                v
                            } else {
                                60000
                            };
                            let winc = if i + 1 < parts.len() && parts[i] == "winc" {
                                let v: u64 = parts[i + 1].parse().unwrap_or(0);
                                i += 2;
                                v
                            } else {
                                0
                            };
                            let binc = if i + 1 < parts.len() && parts[i] == "binc" {
                                let v: u64 = parts[i + 1].parse().unwrap_or(0);
                                i += 2;
                                v
                            } else {
                                0
                            };
                            if state.board.active_color == Color::White {
                                movetime = Some((white_time / 40 + winc).max(100));
                            } else {
                                movetime = Some((black_time / 40 + binc).max(100));
                            }
                        }
                        "infinite" => {
                            movetime = Some(3600_000);
                        }
                        _ => {}
                    }
                    i += 1;
                }

                let best = if let Some(ms) = movetime {
                    state.go_movetime(ms)
                } else {
                    state.go_depth(depth)
                };

                match best {
                    Some(m) => println!("bestmove {}", m),
                    None => println!("bestmove 0000"),
                }
            }
            "stop" => {
                // Synchronous search can't be interrupted, just print nothing
            }
            "quit" => {
                break;
            }
            _ => {}
        }

        stdout.flush().ok();
    }
}
