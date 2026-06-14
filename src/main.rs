mod chess;

use crate::chess::{
    chess::Chess,
    color::Color,
    engine::engine::Engine,
    movegeneration::{Move, MoveFlags},
    square::Square,
};
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

/// Returns the file path to the SVG representing a piece.
fn piece_svg(square: Square) -> &'static str {
    match square {
        Square::WhitePawn => "pieces/pawn-w.svg",
        Square::WhiteKnight => "pieces/knight-w.svg",
        Square::WhiteBishop => "pieces/bishop-w.svg",
        Square::WhiteRook => "pieces/rook-w.svg",
        Square::WhiteQueen => "pieces/queen-w.svg",
        Square::WhiteKing => "pieces/king-w.svg",
        Square::BlackPawn => "pieces/pawn-b.svg",
        Square::BlackKnight => "pieces/knight-b.svg",
        Square::BlackBishop => "pieces/bishop-b.svg",
        Square::BlackRook => "pieces/rook-b.svg",
        Square::BlackQueen => "pieces/queen-b.svg",
        Square::BlackKing => "pieces/king-b.svg",
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
    let pending_promotion = use_state(|| None::<(usize, usize)>);
    let user_color = use_state(|| {
        if rand::random::<bool>() {
            Color::White
        } else {
            Color::Black
        }
    });

    let engine = use_state(|| Engine::new());

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

    let in_check = game.is_color_in_check(game.active_color);
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
                let mut engine_handle = (*engine).clone();

                spawn_local(async move {
                    TimeoutFuture::new(0).await;
                    if let Some(best_move) = engine_handle.get_best_move_in_time(&current, 1000) {
                        let mut next = current;
                        next.apply_move(&best_move);
                        game.set(next);
                        engine.set(engine_handle);
                    }
                });
            }
            || {}
        });
    }

    let on_square_click = {
        let game = game.clone();
        let selected = selected.clone();
        let legal_moves = legal_moves.clone();
        let pending_promotion = pending_promotion.clone();
        let user_color = *user_color;

        Callback::from(move |idx: usize| {
            let current = *game;
            if current.active_color != user_color || pending_promotion.is_some() {
                return;
            }

            match *selected {
                Some(from) => {
                    let matching_moves: Vec<_> = legal_moves
                        .iter()
                        .filter(|m| m.from == from && m.to == idx)
                        .collect();

                    if !matching_moves.is_empty() {
                        let is_promotion = matching_moves
                            .iter()
                            .any(|m| m.flags.contains(MoveFlags::PROMOTION_QUEEN));

                        if is_promotion {
                            pending_promotion.set(Some((from, idx)));
                        } else {
                            let mut next = current;
                            next.apply_move(matching_moves[0]);
                            game.set(next);
                            selected.set(None);
                        }
                    } else if idx != from
                        && is_own_piece(current.squares[idx], current.active_color)
                    {
                        selected.set(Some(idx));
                    } else {
                        selected.set(None);
                    }
                }
                None => {
                    if is_own_piece(current.squares[idx], current.active_color) {
                        selected.set(Some(idx));
                    }
                }
            }
        })
    };

    let on_promote = {
        let game = game.clone();
        let selected = selected.clone();
        let legal_moves = legal_moves.clone();
        let pending_promotion = pending_promotion.clone();

        Callback::from(move |flag: MoveFlags| {
            if let Some((from, to)) = *pending_promotion {
                if let Some(mv) = legal_moves
                    .iter()
                    .find(|m| m.from == from && m.to == to && m.flags.contains(flag))
                {
                    let mut next = *game;
                    next.apply_move(mv);
                    game.set(next);
                    selected.set(None);
                    pending_promotion.set(None);
                }
            }
        })
    };

    let on_restart = {
        let game = game.clone();
        let selected = selected.clone();
        let pending_promotion = pending_promotion.clone();
        let user_color = user_color.clone();
        Callback::from(move |_| {
            game.set(Chess::new());
            selected.set(None);
            pending_promotion.set(None);
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

    // Render the promotion modal with SVG elements
    let promotion_modal = if let Some((from, _)) = *pending_promotion {
        let is_white = is_own_piece(game.squares[from], Color::White);
        let (q, r, b, n) = if is_white {
            (
                Square::WhiteQueen,
                Square::WhiteRook,
                Square::WhiteBishop,
                Square::WhiteKnight,
            )
        } else {
            (
                Square::BlackQueen,
                Square::BlackRook,
                Square::BlackBishop,
                Square::BlackKnight,
            )
        };

        let on_q = on_promote.reform(|_| MoveFlags::PROMOTION_QUEEN);
        let on_r = on_promote.reform(|_| MoveFlags::PROMOTION_ROOK);
        let on_b = on_promote.reform(|_| MoveFlags::PROMOTION_BISHOP);
        let on_n = on_promote.reform(|_| MoveFlags::PROMOTION_KNIGHT);

        let on_cancel = {
            let pending = pending_promotion.clone();
            Callback::from(move |_| pending.set(None))
        };

        html! {
            <div class="promotion-modal">
                <div class="promotion-content">
                    <h3>{ "Promote to:" }</h3>
                    <div class="promotion-pieces">
                        <div class="promo-square" onclick={on_q}><img src={piece_svg(q)} class="piece-image" draggable="false"/></div>
                        <div class="promo-square" onclick={on_r}><img src={piece_svg(r)} class="piece-image" draggable="false"/></div>
                        <div class="promo-square" onclick={on_b}><img src={piece_svg(b)} class="piece-image" draggable="false"/></div>
                        <div class="promo-square" onclick={on_n}><img src={piece_svg(n)} class="piece-image" draggable="false"/></div>
                    </div>
                    <button class="cancel-btn" onclick={on_cancel}>{ "Cancel" }</button>
                </div>
            </div>
        }
    } else {
        html! {}
    };

    // Set the rendering order of squares based on player color to flip the board
    let board_squares = {
        let mut sqs = Vec::with_capacity(64);
        if *user_color == Color::White {
            for rank in (0..8).rev() {
                for file in 0..8 {
                    sqs.push((rank, file));
                }
            }
        } else {
            for rank in 0..8 {
                for file in (0..8).rev() {
                    sqs.push((rank, file));
                }
            }
        }
        sqs
    };

    let advantage = game.count_material();

    html! {
        <div class="page">
            <h1>{ "Rust Chess" }</h1>
            <div class="user-color">{ format!("You are {}", user_color_str) }</div>
            <div class="status">{ &status }</div>
            <div class="score-display">
                if advantage > 0 {
                    { format!("White is leading by: +{}", advantage) }
                } else if advantage < 0 {
                    { format!("Black is leading by: +{}", advantage.abs()) }
                } else {
                    { "Material is equal" }
                }
            </div>
            <div class="board-container">
                { promotion_modal }
                <div class="board">
                    { for board_squares.into_iter().map(|(rank, file)| {
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

                        let piece_html = match piece {
                            Square::Empty => html! {},
                            _ => html! { <img src={piece_svg(piece)} class="piece-image" draggable="false"/> }
                        };

                        let on_click = on_square_click.clone();
                        html! {
                            <div
                                class={classes.join(" ")}
                                onclick={Callback::from(move |_| on_click.emit(idx))}
                            >
                                { piece_html }
                            </div>
                        }
                    }) }
                </div>
            </div>
            <button onclick={on_restart}>{ "New Game" }</button>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
