use gloo_net::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

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
    NewGame,
    PlayMove {id: u16},
}

#[derive(Debug, Deserialize, Clone)]
pub struct State {
    pub board: Vec<String>, // length 64, each entry ".", "P", "n", etc.
    pub turn: u8,           // 0 white, 1 black
    pub legal_moves: Vec<Move>,
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

    let engine_sq = props.engine_sq;
    let onclick = {
        let onclick = props.onclick.clone();
        Callback::from(move |_| onclick.emit(engine_sq))
    };

    html! {
        <div {class} {onclick}>
            { piece_to_display(&props.piece) }
        </div>
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
    });

    // UI-only state
    let selected = use_state(|| None::<u8>);
    let status = use_state(|| "disconnected".to_string());
    let seq = use_state(|| 0u64); // increments on every received State (proves UI applied it)

    // --- WebSocket connect once ---
    {
        let tx_effect = tx.clone();
        let state_effect = state.clone();
        let selected_effect = selected.clone();
        let status_effect = status.clone();
        let seq_effect = seq.clone();

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

            spawn_local(async move {
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            match serde_json::from_str::<ServerMsg>(&text) {
                                Ok(ServerMsg::State(s)) => {
                                    if s.board.len() == 64 {
                                        // Prove state application
                                        seq_for_task.set(*seq_for_task + 1);

                                        // Authoritative render update
                                        state_for_task.set(s);

                                        // Clear selection on authoritative update
                                        selected_for_task.set(None);

                                        status_for_task.set("synced".to_string());
                                    } else {
                                        status_for_task.set(format!("bad state: board len {}", s.board.len()));
                                    }
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
                                    status_for_task.set(format!("bad server msg: {e}"));
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

    let on_new_game = {
        let send_client = send_client.clone();
        Callback::from(move |_| {
            send_client.emit(ClientMsg::NewGame);
            // do not mutate board locally; wait for next State
        })
    };

    // Derived values for render (read directly from state)
    let sel = *selected;
    let legal_moves = state.legal_moves.clone();

    let dests: Vec<u8> = sel
        .map(|s| legal_moves.iter().filter(|m| m.from == s).map(|m| m.to).collect())
        .unwrap_or_default();

    let on_square_click = {
        let selected = selected.clone();
        let send_client = send_client.clone();
        let status = status.clone();
        let legal_moves = state.legal_moves.clone(); // safe: handler recreated on rerender

        Callback::from(move |sq: u8| {
            let current_sel = *selected;

            if let Some(from) = current_sel {
                if let Some(mv) = legal_moves.iter().find(|m| m.from == from && m.to == sq) {
                    send_client.emit(ClientMsg::PlayMove { id: mv.id });
                    status.set("sent move".to_string());
                    return;
                }
            }

            selected.set(Some(sq));
        })
    };

    html! {
        <div>
            <div class="row status">
                <button onclick={on_new_game}>{"New Game"}</button>
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
                        let engine_rank = 7 - rank_from_top;  // 7..0
                        let engine_sq = engine_rank * 8 + file;

                        let piece = state.board[engine_sq as usize].clone();

                        html! {
                            <Square
                                key={engine_sq}                 // CRITICAL: stable per square
                                engine_sq={engine_sq}
                                ui_rank={rank_from_top}
                                file={file}
                                piece={piece}
                                is_selected={Some(engine_sq) == sel}
                                is_dest={dests.contains(&engine_sq)}
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

fn piece_to_display(p: &str) -> String {
    if p == "." { "".to_string() } else { p.to_string() }
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
