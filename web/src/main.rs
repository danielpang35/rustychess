use gloo_net::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use gloo_timers::future::TimeoutFuture;
use yew::classes;
use web_sys::HtmlAudioElement;


use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{SinkExt, StreamExt};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ServerMsg {
    State(State),
    MoveResult { ok: bool, reason: String },
    Error { message: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ClientMsg {
    NewGame{playerside: u8},
    PlayMove {id: u16},
}
#[derive(Debug, Deserialize, Clone)]
pub struct State {
    pub board: Vec<String>,
    pub turn: u8,
    pub legal_moves: Vec<Move>,

    #[serde(default)]
    pub thinking: bool,

    // NEW: engine evaluation (centipawns or mate score—see note below)
    #[serde(default)]
    pub eval: Option<i32>,

    #[serde(default)]
    pub best_move: Option<Move>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub id: u16,
    pub from: u8,
    pub to: u8,
    pub promo: Option<char>,
}


#[derive(Properties, PartialEq)]
struct SquareProps {
    pub engine_sq: u8,     // 0..63 in engine indexing (a1=0)
    pub ui_rank: u8,       // 0..7 from top
    pub file: u8,          // 0..7
    pub piece: String,     // ".", "P", ...
    pub is_selected: bool,
    pub is_dest: bool,
    pub is_last: bool,
    pub onclick: Callback<u8>,
}


#[function_component(Square)]
fn square(props: &SquareProps) -> Html {
    let is_dark = ((props.ui_rank + props.file) % 2) == 1;
    let mut class = classes!("sq", if is_dark { "dark" } else { "light" });
    
    
    if props.is_selected {
        class.push("sel");
    }
    if props.is_dest {
        class.push("dest");
    }
    if props.is_last {
        class.push("last");
    }


    let engine_sq = props.engine_sq;
    let onclick = {
        let onclick = props.onclick.clone();
        Callback::from(move |_| onclick.emit(engine_sq))
    };

    
    let piece_view = if let Some(src) = piece_svg(props.piece.as_str()) {
        html! {
            <img class="piece" src={src} draggable="false" />
        }
    } else {
            html! { <span class="piece-glyph">{ piece_to_glyph(props.piece.as_str()) }</span> }
    };

    html! {
        <div {class} {onclick}>
            { piece_view }
        </div>
    }
}

fn piece_to_glyph(p: &str) -> &'static str {
    match p {
        "P" => "♙", "N" => "♘", "B" => "♗", "R" => "♖", "Q" => "♕", "K" => "♔",
        "p" => "♟", "n" => "♞", "b" => "♝", "r" => "♜", "q" => "♛", "k" => "♚",
        _ => "",
    }
}



#[function_component(App)]
fn app() -> Html {

    // Outbound WS sender
    let tx = use_state(|| None::<UnboundedSender<Message>>);

    // Authoritative render state
    let state = use_state(|| State {
        board: vec![".".to_string(); 64],
        turn: 0,
        legal_moves: vec![],
        thinking: false,
        eval: None,
        best_move: None,
    });

    //audio

    let cap_audio = use_mut_ref(|| {
        let a = HtmlAudioElement::new().unwrap();
        a.set_src("/assets/sounds/capture.mp3"); // or .wav
        a.set_volume(0.4);
        a
    });

    let move2_audio = use_mut_ref(|| {
        let a = HtmlAudioElement::new().unwrap();
        a.set_src("/assets/sounds/move2.mp3"); // or .wav
        a.set_volume(0.70);
        a
    });
    // UI-only state
    let selected = use_state(|| None::<u8>);
    let status = use_state(|| "disconnected".to_string());
    let seq = use_state(|| 0u64); // increments on every received State (proves UI applied it)
    use std::cell::RefCell;
    use std::rc::Rc;

    let pending = use_mut_ref(|| None::<(u8, u8)>);    
    let last_from = use_state(|| None::<u8>);
    let last_to = use_state(|| None::<u8>);
    let prev_board = use_mut_ref(|| vec![".".to_string(); 64]);




    // --- WebSocket connect once ---
    {
        let tx_effect = tx.clone();
        let state_effect = state.clone();
        let selected_effect = selected.clone();
        let status_effect = status.clone();
        let seq_effect = seq.clone();
        let pending_effect = pending.clone();
        let last_from_effect = last_from.clone();
        let last_to_effect = last_to.clone();
        let prev_board_effect = prev_board.clone();

        

        use_effect_with((), move |_| {
            let socket = WebSocket::open("ws://127.0.0.1:3000/ws").expect("failed to open ws");
            status_effect.set("connected".to_string());

            let (mut write, mut read) = socket.split();

            let (out_tx, mut out_rx) = unbounded::<Message>();
            tx_effect.set(Some(out_tx.clone()));

            // Writer task
            spawn_local(async move {
                while let Some(msg) = out_rx.next().await {
                    if write.send(msg).await.is_err() {
                        break;
                    }
                }
            });

            
            // Reader task
            let state_for_task = state_effect.clone();
            let selected_for_task = selected_effect.clone();
            let status_for_task = status_effect.clone();
            let seq_for_task = seq_effect.clone();
            let pending_for_task = pending_effect.clone();
            let last_from_for_task = last_from_effect.clone();
            let last_to_for_task = last_to_effect.clone();
            let prev_board_for_task = prev_board_effect.clone();



            spawn_local(async move {
                while let Some(msg) = read.next().await {

                    match msg {
                        Ok(Message::Text(text)) => {

                            match serde_json::from_str::<ServerMsg>(&text) {
                                Ok(ServerMsg::State(s)) => {
                                    // 1) Snapshot the currently-rendered board BEFORE overwriting state
                                    let prev = prev_board_for_task.borrow().clone();

                                    // Helper: treat "." and "" as empty
                                    let is_empty = |x: &str| x == "." || x.is_empty();

                                    // 2) If UI hasn't been initialized yet, seed and bail
                                    let prev_has_any_piece = prev.iter().any(|x| !is_empty(x.as_str()));
                                    if !prev_has_any_piece {
                                        // Initialize UI + snapshot, then return (no capture detection possible yet)
                                        state_for_task.set(s.clone());
                                        *prev_board_for_task.borrow_mut() = s.board.clone();
                                        status_for_task.set("seeded baseline board".to_string());
                                        continue;
                                    }
                                     // 2.5) If no pending player move, infer engine move from prev->next diff
                                    {
                                        let mut pend = pending_for_task.borrow_mut();
                                        if pend.is_none() {
                                            if let Some((f, t)) = infer_last_move(&prev, &s.board) {
                                                *pend = Some((f, t));
                                                last_from_for_task.set(Some(f));
                                                last_to_for_task.set(Some(t));
                                            }
                                        }
                                    }
                                    // 3) Resolve pending move + capture using `prev` (real board)
                                    if let Some((pf, pt)) = pending_for_task.borrow_mut().take() {
                                        let from_piece = prev[pf as usize].as_str();
                                        let to_before  = prev[pt as usize].as_str();
                                        
                                        let move2audio = move2_audio.borrow();
                                        let _ = move2audio.set_current_time(0.0);


                                        let mut did_capture = !is_empty(to_before);

                                        // (optional) EP check here...

                                        status_for_task.set(format!(
                                            r#"cap={} pf={} pt={} from="{}" to_before="{}""#,
                                            did_capture, pf, pt, from_piece, to_before
                                        ));

                                        if did_capture {
                                            // Play capture sound
                                            let audio = cap_audio.borrow();
                                            let _ = audio.set_current_time(0.0);
                                            let _ = audio.play();
                                            move2audio.set_volume(0.4);

                                            spawn_local(async move {
                                                TimeoutFuture::new(420).await;
                                            });
                                        }
                                        let _ = move2audio.play();
                                        move2audio.set_volume(0.7);

                                    }

                                    // 4) Now apply new state
                                    state_for_task.set(s.clone());
                                    selected_for_task.set(None);

                                    // 5) Update snapshot for next message
                                    *prev_board_for_task.borrow_mut() = s.board.clone();
                                }

                                Ok(ServerMsg::MoveResult { ok, reason }) => {
                                    if !ok {
                                        status_for_task.set(format!("move rejected: {reason}"));
                                    }
                                }
                                Ok(ServerMsg::Error { message }) => {
                                    status_for_task.set(format!("server error: {message}"));
                                }
                                Err(e) => {
                                    status_for_task.set(format!("bad server msg: {e}\nRAW: {text}"));
                                }
                            }
                        }
                        Ok(Message::Bytes(_)) => {}
                        Err(e) => {
                            status_for_task.set(format!("ws error: {e}"));
                            break;
                        }
                    }
                }
            });

            || {}
        });
    }

    let send_client = {
        let tx = tx.clone();
        let status = status.clone();
        Callback::from(move |msg: ClientMsg| {
            if let Some(sender) = (*tx).as_ref() {
                let text = serde_json::to_string(&msg).unwrap();
                if sender.unbounded_send(Message::Text(text.into())).is_err() {
                    status.set("send failed".to_string());
                }
            } else {
                status.set("not connected".to_string());
            }
        })
    };

    let player_side = use_state(|| 0u8); // 0 = white, 1 = black

    let on_new_game = {
        let send_client = send_client.clone();
        let player_side = player_side.clone();
        Callback::from(move |_| {
            send_client.emit(ClientMsg::NewGame {
                playerside: *player_side,
            });
        })
    };

    // Derived values for render (read directly from state)
    let sel = *selected;
    let legal_moves = state.legal_moves.clone();

    let dests: Vec<u8> = sel
        .map(|s| legal_moves.iter().filter(|m| m.from == s).map(|m| m.to).collect())
        .unwrap_or_default();
    
    let lf = *last_from;
    let lt = *last_to;
    
    
    let on_square_click = {
        let selected = selected.clone();
        let pending = pending.clone();
        let send_client = send_client.clone();
        let status = status.clone();
        let state_for_click = state.clone(); // <-- add this


        Callback::from(move |sq: u8| {
            if state_for_click.thinking {
                // Hard lock: ignore clicks while engine thinks
                //keep selected tho
                let current_sel = *selected;
                return;
            }
            let current_sel = *selected;
            

            if let Some(from) = current_sel {
                if let Some(mv) = legal_moves.iter().find(|m| m.from == from && m.to == sq) {
                    *pending.borrow_mut() = Some((from, sq));
                    last_from.set(Some(from));
                    last_to.set(Some(sq));
                    send_client.emit(ClientMsg::PlayMove { id: mv.id });
                    return;
                }
            }

            selected.set(Some(sq));
        })
    };
    let eval_text = match state.eval {
        Some(cp) => format!("evaluation: {:.2}", cp as f32 / 100.0),
        None => "evaluation: --".to_string(),
    };


    html! {
        <div>
            <div class="row status">
                <button onclick={on_new_game}>{"New Game"}</button>
                <div class="mono">{eval_text}</div>
                <button onclick={{
                    let ps = player_side.clone();
                    Callback::from(move |_| ps.set(0))
                }}>{"Play White"}</button>

                <button onclick={{
                    let ps = player_side.clone();
                    Callback::from(move |_| ps.set(1))
                }}>{"Play Black"}</button>
                
                // <button class={hint_btn_class} onclick={on_toggle_hint}>
                //     { if hint_on { "Engine Hint: ON" } else { "Engine Hint: OFF" } }
                // </button>
                <div class="mono">  { format!("best_move: {:?}", state.best_move) }</div>

                <div class="mono">{format!("thinking: {}", state.thinking)}</div>
                <div class="mono">{format!("status: {}", (*status).clone())}</div>
                <div class="mono">{format!("turn: {}", if state.turn == 0 { "white" } else { "black" })}</div>
                <div class="mono">{format!("state seq: {}", *seq)}</div>

            </div>

            <div class="board">
                {
                    
                    for (0u8..64u8).map(|ui_sq| {
                        // UI: rank 8 at top, engine: a1=0
                        let file = ui_sq % 8;
                        let rank_from_top = ui_sq / 8;        // 0..7 top->bottom
                        let engine_sq = if *player_side == 0 {
                            // White perspective
                            (7 - rank_from_top) * 8 + file
                        } else {
                            // Black perspective (rotate 180 degrees)
                            rank_from_top * 8 + (7 - file)
                        };
                        let piece = state.board[engine_sq as usize].clone();
                        let bm = state.best_move;
                        html! {
                            <Square
                                key={ui_sq}                 // CRITICAL: stable per square
                                engine_sq={engine_sq}
                                ui_rank={rank_from_top}
                                file={file}
                                piece={piece}
                                is_selected={Some(engine_sq) == sel}
                                is_dest={dests.contains(&engine_sq)}
                                is_last={Some(engine_sq) == lf || Some(engine_sq) == lt}
                                onclick={on_square_click.clone()}
                            />
                        }
                    })
                }
            </div>

            // --- Debug dump: what the UI thinks it received ---
            <pre class="mono">
                { format_board_dump(&state.board) }
            </pre>
        </div>
    }
}

fn piece_svg(cell: &str) -> Option<&'static str> {
    match cell {
        "P" => Some("/assets/pieces/wp.svg"),
        "N" => Some("/assets/pieces/wn.svg"),
        "B" => Some("/assets/pieces/wb.svg"),
        "R" => Some("/assets/pieces/wr.svg"),
        "Q" => Some("/assets/pieces/wq.svg"),
        "K" => Some("/assets/pieces/wk.svg"),
        "p" => Some("/assets/pieces/bp.svg"),
        "n" => Some("/assets/pieces/bn.svg"),
        "b" => Some("/assets/pieces/bb.svg"),
        "r" => Some("/assets/pieces/br.svg"),
        "q" => Some("/assets/pieces/bq.svg"),
        "k" => Some("/assets/pieces/bk.svg"),

        "." => None,
        _ => None, // defensive
    }
}

fn format_board_dump(board: &Vec<String>) -> String {
    if board.len() != 64 {
        return format!("board.len() = {} (expected 64)", board.len());
    }
    let mut out = String::new();
    // Print ranks 8->1 (engine a1=0)
    for r in (0..8).rev() {
        for f in 0..8 {
            let sq = (r * 8 + f) as usize;
            out.push_str(&board[sq]);
        }
        out.push('\n');
    }
    out
}

fn main() {
    yew::Renderer::<App>::new().render();
}

fn is_empty_cell(x: &str) -> bool {
    x == "." || x.is_empty()
}

/// Infer last move (from,to) from previous and next 64-cell boards.
/// Handles normal moves (2 diffs), EP (3 diffs), castling (4 diffs).
fn infer_last_move(prev: &[String], next: &[String]) -> Option<(u8, u8)> {
    if prev.len() != 64 || next.len() != 64 {
        return None;
    }

    let mut changed: Vec<u8> = Vec::new();
    for sq in 0u8..64u8 {
        if prev[sq as usize] != next[sq as usize] {
            changed.push(sq);
        }
    }

    match changed.len() {
        0 => None,

        // Normal move or promotion: from emptied, to filled/changed
        2 | 3 => {
            let mut from_sq: Option<u8> = None;
            let mut to_sq: Option<u8> = None;

            for &sq in &changed {
                let p = prev[sq as usize].as_str();
                let n = next[sq as usize].as_str();

                if !is_empty_cell(p) && is_empty_cell(n) {
                    // something moved away (or got captured from this square in EP/castling)
                    // Prefer the square that was occupied by a moving piece:
                    // In EP (3 diffs) there are two empties; one is the pawn's from, one is captured pawn.
                    // We'll identify "to" first and then choose "from" as the empty that matches mover color if needed.
                    if from_sq.is_none() {
                        from_sq = Some(sq);
                    }
                } else if !is_empty_cell(n) && (is_empty_cell(p) || p != n) {
                    to_sq = Some(sq);
                }
            }

            // EP refinement: if 3 diffs, there are 2 empties; choose the empty whose prev piece
            // matches the color of the moved piece now on `to`.
            if changed.len() == 3 {
                if changed.len() == 3 {
                    if let Some(t) = to_sq {
                        let mover_is_white = next[t as usize]
                            .as_bytes()
                            .first()
                            .map(|b| (*b as char).is_ascii_uppercase())
                            .unwrap_or(false);

                        let empties: Vec<u8> = changed
                            .iter()
                            .copied()
                            .filter(|&sq| !is_empty_cell(prev[sq as usize].as_str()) && is_empty_cell(next[sq as usize].as_str()))
                            .collect();

                        if empties.len() == 2 {
                            for &sq in &empties {
                                let pc_is_white = prev[sq as usize]
                                    .as_bytes()
                                    .first()
                                    .map(|b| (*b as char).is_ascii_uppercase())
                                    .unwrap_or(false);
                                if pc_is_white == mover_is_white {
                                    from_sq = Some(sq);
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            match (from_sq, to_sq) {
                (Some(f), Some(t)) => Some((f, t)),
                _ => None,
            }
        }

        // Castling: 4 squares change. Prefer the king move as (from,to).
        4 => {
            // Find king destination (square where next has 'K' or 'k' and prev didn't)
            let mut king_to: Option<u8> = None;
            let mut king_from: Option<u8> = None;

            for &sq in &changed {
                let p = prev[sq as usize].as_str();
                let n = next[sq as usize].as_str();

                if (n == "K" || n == "k") && p != n {
                    king_to = Some(sq);
                }
                if (p == "K" || p == "k") && p != n {
                    king_from = Some(sq);
                }
            }

            match (king_from, king_to) {
                (Some(f), Some(t)) => Some((f, t)),
                _ => None,
            }
        }

        _ => None,
    }
}
