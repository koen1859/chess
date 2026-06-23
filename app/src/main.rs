mod game;
mod utils;

use crate::{
    game::{DebugBitboard, GameState},
    utils::{is_own_piece, piece_svg},
};
use chess::{
    apply_undo_move::{move_to_uci, MoveFlags},
    color::Color,
    engine::best_move::MAX_DEPTH,
    engine::engine::SearchInfo,
    opening_book,
    square::Square,
};
use gloo_timers::future::TimeoutFuture;
use instant::Instant;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlInputElement;
use yew::TargetCast;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let game_state = use_state(GameState::new);
    let legal_moves = game_state.legal_moves();
    let destinations = game_state.destinations();
    let status = game_state.status();
    let advantage = game_state.count_material();
    let board_squares = game_state.board_squares();
    let user_color_str = game_state.user_color_str();
    let debug_squares = game_state.debug_overlay_squares();
    let last_move = game_state.last_move;
    let copied = use_state(|| false);

    // Engine move effect — only fires when the turn changes, with a guard
    // against concurrent searches if the component re-renders during search.
    let is_searching: Rc<RefCell<bool>> = use_mut_ref(|| false);
    {
        let game_state = game_state.clone();
        let is_searching = is_searching.clone();
        use_effect_with(
            (game_state.chess.active_color, game_state.user_color),
            move |_| {
                if game_state.chess.active_color != game_state.user_color && !*is_searching.borrow()
                {
                    *is_searching.borrow_mut() = true;
                    spawn_local(async move {
                        TimeoutFuture::new(0).await;
                        let mut new_state = (*game_state).clone();

                        // Check opening book first
                        if let Some(book_move) =
                            opening_book::get_book_move(new_state.chess.hash)
                        {
                            new_state.chess.apply_move(&book_move);
                            new_state.last_move = Some((book_move.from, book_move.to));
                            new_state.push_position();
                            new_state.engine.deadline = None;
                            new_state.search_info = None;
                            game_state.set(new_state);
                            *is_searching.borrow_mut() = false;
                            return;
                        }

                        let think_time = new_state.engine_think_time_ms;

                        let start = Instant::now();
                        new_state.engine.deadline = Some(start + Duration::from_millis(think_time));
                        new_state.engine.time_up = false;
                        new_state.engine.nodes = 0;
                        new_state.engine.tt.new_generation();

                        let mut best_move = None;
                        let mut depth: u8 = 1;
                        while depth <= MAX_DEPTH {
                            let m = new_state.engine.get_best_move(&mut new_state.chess, depth);
                            if new_state.engine.time_up {
                                break;
                            }
                            best_move = m;

                            let elapsed = start.elapsed().as_millis();
                            let nps = if elapsed > 0 {
                                (new_state.engine.nodes as u128 * 1000 / elapsed) as u64
                            } else {
                                0
                            };

                            new_state.search_info = Some(SearchInfo {
                                depth,
                                score: new_state.engine.best_score,
                                nodes: new_state.engine.nodes,
                                time_ms: elapsed,
                                nps,
                                best_move_uci: m.map(|m| move_to_uci(&m)).unwrap_or_default(),
                                is_mate: new_state.engine.best_score.abs() > 50000,
                                mate_in: 0,
                            });
                            game_state.set(new_state.clone());

                            depth += 1;
                            TimeoutFuture::new(0).await;
                        }

                        if let Some(m) = best_move {
                            new_state.chess.apply_move(&m);
                            new_state.last_move = Some((m.from, m.to));
                            new_state.push_position();
                        }
                        new_state.engine.deadline = None;
                        new_state.search_info = None;
                        game_state.set(new_state);
                        *is_searching.borrow_mut() = false;
                    });
                }
                || {}
            },
        );
    }

    // Handle user click on a square
    let on_square_click = {
        let game = game_state.clone();
        let legal_moves = legal_moves.clone();
        Callback::from(move |idx: usize| {
            let mut new_state = (*game).clone();

            if new_state.chess.active_color != new_state.user_color
                || new_state.pending_promotion.is_some()
            {
                return;
            }

            match new_state.selected {
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
                            new_state.pending_promotion = Some((from, idx));
                        } else {
                            new_state.chess.apply_move(matching_moves[0]);
                            new_state.last_move = Some((from, idx));
                            new_state.push_position();
                            new_state.selected = None;
                        }
                    } else if idx != from
                        && is_own_piece(new_state.chess.squares[idx], new_state.chess.active_color)
                    {
                        new_state.selected = Some(idx);
                    } else {
                        new_state.selected = None;
                    }
                }
                None => {
                    if is_own_piece(new_state.chess.squares[idx], new_state.chess.active_color) {
                        new_state.selected = Some(idx);
                    }
                }
            }

            game.set(new_state);
        })
    };

    // Handle promotions
    let on_promote = {
        let game = game_state.clone();
        let legal_moves = legal_moves.clone();
        Callback::from(move |flag: MoveFlags| {
            let mut new_state = (*game).clone();

            if let Some((from, to)) = new_state.pending_promotion {
                if let Some(mv) = legal_moves
                    .iter()
                    .find(|m| m.from == from && m.to == to && m.flags.contains(flag))
                {
                    new_state.chess.apply_move(mv);
                    new_state.last_move = Some((from, to));
                    new_state.push_position();
                    new_state.selected = None;
                    new_state.pending_promotion = None;
                    game.set(new_state);
                }
            }
        })
    };

    let on_restart = {
        let game = game_state.clone();
        Callback::from(move |_| {
            game.set(GameState::new());
        })
    };

    let on_copy_pgn = {
        let game_state = game_state.clone();
        let copied = copied.clone();
        Callback::from(move |_| {
            let pgn = game_state.to_pgn();
            let copied = copied.clone();
            spawn_local(async move {
                if let Some(window) = web_sys::window() {
                    let promise = window.navigator().clipboard().write_text(&pgn);
                    let _ = JsFuture::from(promise).await;
                    copied.set(true);
                    TimeoutFuture::new(1500).await;
                    copied.set(false);
                }
            });
        })
    };

    // Engine think-time slider
    let on_think_time_input = {
        let game = game_state.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<u64>() {
                    let mut new_state = (*game).clone();
                    new_state.engine_think_time_ms = value;
                    game.set(new_state);
                }
            }
        })
    };

    // Debug bitboard overlay buttons (click again to clear)
    let on_debug_select = {
        let game = game_state.clone();
        Callback::from(move |bb: DebugBitboard| {
            let mut new_state = (*game).clone();
            new_state.debug_overlay = if new_state.debug_overlay == Some(bb) {
                None
            } else {
                Some(bb)
            };
            game.set(new_state);
        })
    };

    let promotion_modal = if let Some((from, _)) = game_state.pending_promotion {
        let is_white = is_own_piece(game_state.chess.squares[from], Color::White);
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
            let game = game_state.clone();
            Callback::from(move |_| {
                let mut new_state = (*game).clone();
                new_state.pending_promotion = None;
                game.set(new_state);
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

    let search_info_html = match &game_state.search_info {
        Some(info) => {
            let score_str = if info.is_mate {
                format!("Mate in {}", info.mate_in.abs())
            } else {
                let sign = if info.score > 0 { "+" } else { "" };
                format!("{}{}", sign, info.score)
            };
            html! {
                <div class="search-info">
                    <div class="search-stat">{ format!("Depth: {}", info.depth) }</div>
                    <div class="search-stat">{ format!("Score: {}", score_str) }</div>
                    <div class="search-stat">{ format!("Nodes: {}", info.nodes) }</div>
                    <div class="search-stat">{ format!("NPS: {}", info.nps) }</div>
                    <div class="search-stat">{ format!("Time: {} ms", info.time_ms) }</div>
                    <div class="search-stat">{ format!("Best: {}", info.best_move_uci) }</div>
                </div>
            }
        }
        None => {
            html! { <div class="search-info search-info-idle">{ "Idle" }</div> }
        }
    };

    let menu = html! {
        <div class="menu">
            <div class="menu-section">
                <label for="think-time">
                    { format!("Engine think time: {} ms", game_state.engine_think_time_ms) }
                </label>
                <input
                    id="think-time"
                    type="range"
                    min="100"
                    max="10000"
                    step="100"
                    value={game_state.engine_think_time_ms.to_string()}
                    oninput={on_think_time_input}
                />
            </div>
            <div class="menu-section">
                <div class="menu-label">{ "Search info:" }</div>
                { search_info_html }
            </div>
            <div class="menu-section">
                <div class="menu-label">{ "Debug bitboards:" }</div>
                <div class="debug-buttons">
                    { for DebugBitboard::ALL.iter().map(|bb| {
                        let bb = *bb;
                        let onclick = on_debug_select.reform(move |_| bb);
                        let active = game_state.debug_overlay == Some(bb);
                        let class = if active { "debug-btn active" } else { "debug-btn" };
                        html! {
                            <button class={class} onclick={onclick}>{ bb.label() }</button>
                        }
                    }) }
                </div>
            </div>
        </div>
    };

    let pgn = game_state.to_pgn();

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
            { menu }
            <div class="board-container">
                { promotion_modal }
                <div class="board">
                    { for board_squares.into_iter().map(|(rank, file)| {
                        let idx = rank * 8 + file;
                        let piece = game_state.chess.squares[idx];

                        let mut classes = vec!["square".to_string()];
                        classes.push(if (rank + file) % 2 == 0 { "dark".to_string() } else { "light".to_string() });
                        if game_state.selected == Some(idx) {
                            classes.push("selected".to_string());
                        }
                        if destinations.contains(&idx) {
                            classes.push("highlight".to_string());
                        }
                        if let Some((from, to)) = last_move {
                            if idx == from || idx == to {
                                classes.push("last-move".to_string());
                            }
                        }
                        if debug_squares.contains(&idx) {
                            classes.push("debug-highlight".to_string());
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
            <div class="pgn-section">
                <textarea class="pgn-text" readonly={true} value={pgn} rows={4} spellcheck="false" />
                <div class="pgn-actions">
                    <button class="copy-pgn-btn" onclick={on_copy_pgn}>
                        { if *copied { "Copied!" } else { "Copy PGN" } }
                    </button>
                </div>
            </div>
            <button onclick={on_restart}>{ "New Game" }</button>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
