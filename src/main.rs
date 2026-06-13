mod chess;

use crate::chess::{chess::Chess, color::Color, movegeneration::Move, square::Square};
use rand::seq::IndexedRandom;
use yew::prelude::*;

/// Unicode glyph used to render a piece on the board.
fn piece_glyph(square: Square) -> &'static str {
    match square {
        Square::WhitePawn => "♙",
        Square::WhiteKnight => "♘",
        Square::WhiteBishop => "♗",
        Square::WhiteRook => "♖",
        Square::WhiteQueen => "♕",
        Square::WhiteKing => "♔",
        Square::BlackPawn => "♟",
        Square::BlackKnight => "♞",
        Square::BlackBishop => "♝",
        Square::BlackRook => "♜",
        Square::BlackQueen => "♛",
        Square::BlackKing => "♚",
        Square::Empty => "",
    }
}

/// Whether the piece on `square` belongs to `color`.
fn is_own_piece(square: Square, color: Color) -> bool {
    use Square::*;
    match color {
        Color::White => matches!(
            square,
            WhitePawn | WhiteKnight | WhiteBishop | WhiteRook | WhiteQueen | WhiteKing
        ),
        Color::Black => matches!(
            square,
            BlackPawn | BlackKnight | BlackBishop | BlackRook | BlackQueen | BlackKing
        ),
    }
}

#[function_component(App)]
fn app() -> Html {
    let game = use_state(|| Chess::new());
    let selected = use_state(|| None::<usize>);
    let user_color = use_state(|| {
        if rand::random::<bool>() {
            Color::White
        } else {
            Color::Black
        }
    });

    let legal_moves: Vec<Move> = game.generate_moves();

    // Squares the currently selected piece can move to.
    let destinations: Vec<usize> = match *selected {
        Some(from) => legal_moves
            .iter()
            .filter(|m| m.from == from)
            .map(|m| m.to)
            .collect(),
        None => Vec::new(),
    };

    let in_check = game.is_active_color_in_check();
    let no_moves = legal_moves.is_empty();

    let status = match (no_moves, in_check, game.active_color) {
        (true, true, Color::White) => "Checkmate — Black wins".to_owned(),
        (true, true, Color::Black) => "Checkmate — White wins".to_owned(),
        (true, false, _) => "Stalemate — draw".to_owned(),
        (false, true, Color::White) => "White to move — check!".to_owned(),
        (false, true, Color::Black) => "Black to move — check!".to_owned(),
        (false, false, Color::White) => "White to move".to_owned(),
        (false, false, Color::Black) => "Black to move".to_owned(),
    };

    // Make computer move after user moves
    {
        let game = game.clone();
        let user_color = *user_color;

        use_effect(move || {
            if game.active_color != user_color {
                let current = *game;
                let moves = current.generate_moves();

                if !moves.is_empty() {
                    let mut rng = rand::rng();

                    if let Some(&mv) = moves.choose(&mut rng) {
                        let mut next = current;
                        next.apply_move(&mv);
                        next.active_color = match next.active_color {
                            Color::White => Color::Black,
                            Color::Black => Color::White,
                        };
                        game.set(next);
                    }
                }
            }

            || {}
        });
    }

    let on_square_click = {
        let game = game.clone();
        let selected = selected.clone();
        let legal_moves = legal_moves.clone();
        let user_color = *user_color;

        Callback::from(move |idx: usize| {
            // Don't allow clicks if it's not the user's turn or if computer is thinking
            let current = *game;
            if current.active_color != user_color {
                return;
            }

            match *selected {
                // A piece is already selected: either play a move, switch
                // selection to another own piece, or deselect.
                Some(from) => {
                    if let Some(mv) = legal_moves.iter().find(|m| m.from == from && m.to == idx) {
                        let mut next = current;
                        next.apply_move(mv);
                        next.active_color = match next.active_color {
                            Color::White => Color::Black,
                            Color::Black => Color::White,
                        };
                        game.set(next);
                        selected.set(None);
                    } else if idx != from
                        && is_own_piece(current.squares[idx], current.active_color)
                    {
                        selected.set(Some(idx));
                    } else {
                        selected.set(None);
                    }
                }
                // Nothing selected yet: select the clicked square if it's
                // one of the active player's pieces.
                None => {
                    if is_own_piece(current.squares[idx], current.active_color) {
                        selected.set(Some(idx));
                    }
                }
            }
        })
    };

    let on_restart = {
        let game = game.clone();
        let selected = selected.clone();
        let user_color = user_color.clone();
        Callback::from(move |_| {
            game.set(Chess::new());
            selected.set(None);
            user_color.set(if rand::random::<bool>() {
                Color::White
            } else {
                Color::Black
            });
        })
    };

    let user_color_str = match *user_color {
        Color::White => "White",
        Color::Black => "Black",
    };

    html! {
        <div class="page">
            <h1>{ "Rust Chess" }</h1>
            <div class="user-color">{ format!("You are {}", user_color_str) }</div>
            <div class="status">{ &status }</div>
            <div class="board">
                { for (0..8).rev().flat_map(|rank| (0..8).map(move |file| (rank, file))).map(|(rank, file)| {
                    let idx = rank * 8 + file;
                    let piece = game.squares[idx];

                    let mut classes = vec!["square".to_string()];
                    classes.push(if (rank + file) % 2 == 0 { "dark".to_string() } else { "light".to_string() });
                    if *selected == Some(idx) {
                        classes.push("selected".to_string());
                    }
                    if destinations.contains(&idx) {
                        classes.push("highlight".to_string());
                    }

                    let on_click = on_square_click.clone();
                    html! {
                        <div
                            class={classes.join(" ")}
                            onclick={Callback::from(move |_| on_click.emit(idx))}
                        >
                            { piece_glyph(piece) }
                        </div>
                    }
                }) }
            </div>
            <button onclick={on_restart}>{ "New Game" }</button>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
