mod game;
mod utils;

use crate::{
    game::Game,
    utils::{is_own_piece, piece_svg},
};
use chess::{
    apply_undo_move::{Move, MoveFlags},
    color::Color,
    square::Square,
};
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let game = use_state(Game::new);

    let legal_moves: Vec<Move> = game.legal_moves();
    let destinations: Vec<usize> = game.destinations();
    let status: String = game.status();
    let advantage: i32 = game.count_material();
    let board_squares: Vec<(usize, usize)> = game.board_squares();
    let user_color_str: &'static str = game.user_color_str();

    {
        let game = game.clone();
        use_effect(move || {
            if game.chess.active_color != game.user_color {
                let mut current = game.chess.clone();
                let mut engine_handle = game.engine.clone();

                let game_history = game.game_history.clone();
                spawn_local(async move {
                    TimeoutFuture::new(0).await;
                    if let Some(best_move) =
                        engine_handle.get_best_move(&mut current, 6, &game_history)
                    {
                        let mut next = (*game).clone();
                        next.chess = current;
                        next.chess.apply_move(&best_move);
                        next.push_position();
                        next.engine = engine_handle;
                        game.set(next);
                    }
                });
            }
            || {}
        });
    }

    let on_square_click = {
        let game = game.clone();
        let legal_moves = legal_moves.clone();

        Callback::from(move |idx: usize| {
            let current = (*game).clone();

            if current.chess.active_color != current.user_color
                || current.pending_promotion.is_some()
            {
                return;
            }

            let mut next = current.clone();

            match next.selected {
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
                            next.pending_promotion = Some((from, idx));
                        } else {
                            next.chess.apply_move(matching_moves[0]);
                            next.push_position();
                            next.selected = None;
                        }
                    } else if idx != from
                        && is_own_piece(next.chess.squares[idx], next.chess.active_color)
                    {
                        next.selected = Some(idx);
                    } else {
                        next.selected = None;
                    }
                }
                None => {
                    if is_own_piece(next.chess.squares[idx], next.chess.active_color) {
                        next.selected = Some(idx);
                    }
                }
            }

            game.set(next);
        })
    };

    let on_promote = {
        let game = game.clone();
        let legal_moves = legal_moves.clone();

        Callback::from(move |flag: MoveFlags| {
            let current = (*game).clone();

            if let Some((from, to)) = current.pending_promotion {
                if let Some(mv) = legal_moves
                    .iter()
                    .find(|m| m.from == from && m.to == to && m.flags.contains(flag))
                {
                    let mut next = current.clone();
                    next.chess.apply_move(mv);
                    next.push_position();
                    next.selected = None;
                    next.pending_promotion = None;
                    game.set(next);
                }
            }
        })
    };

    let on_restart = {
        let game = game.clone();
        Callback::from(move |_| {
            game.set(Game::new());
        })
    };

    let promotion_modal = if let Some((from, _)) = game.pending_promotion {
        let is_white = is_own_piece(game.chess.squares[from], Color::White);
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
            let game = game.clone();
            Callback::from(move |_| {
                let mut next = (*game).clone();
                next.pending_promotion = None;
                game.set(next);
            })
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

    html! {
        <div class="page">
            <h1>{ "Chess" }</h1>
            <div class="user-color">{ format!("You are {}", user_color_str) }</div>
            <div class="status">{ status }</div>
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
                        let piece = game.chess.squares[idx];

                        let mut classes = vec!["square".to_string()];
                        classes.push(if (rank + file) % 2 == 0 { "dark".to_string() } else { "light".to_string() });
                        if game.selected == Some(idx) {
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
