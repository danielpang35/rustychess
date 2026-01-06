use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;

use serde::{Deserialize, Serialize};

use rustychess::core::movegen::MoveGenerator;
use rustychess::core::{Board, Move as EngineMove, PieceType};

use rustychess::evaluate::evaluate;
use rustychess::search::Search;
// ===== Your protocol types (as discussed) =====
use axum::{routing::get, Router};

use std::sync::{Arc, Mutex};
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "server up" }))
        .route("/ws", get(ws_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("bind failed");

    println!("http://0.0.0.0:3000  ws://0.0.0.0:3000/ws");

    axum::serve(listener, app).await.expect("serve failed");
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    NewGame {playerside: u8},
    SetPosition { fen: String },
    PlayMove { id: u16},
    
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ServerMsg {
    State(State),
    MoveResult { ok: bool, reason: String },
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Move {
    id:u16,
    from: u8,
    to: u8,
    promo: Option<char>,
}

#[derive(Debug, Serialize, Clone)]
pub struct State {
    pub board: Vec<String>,        // 64 chars: ".PNBRQKpnbrqk"
    pub turn: u8,
    pub legal_moves: Vec<Move>,
    pub thinking: bool,            // NEW: UI disables interaction when true
    pub eval: Option<i32>,         // NEW: engine evaluation (centipawns or mate score)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_move: Option<Move>,
}


// ===== Axum entrypoint =====

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}
use tokio::sync::mpsc;

async fn handle_socket(mut socket: WebSocket) {
    let movegen = MoveGenerator::new();
    let mut board = Board::new();
    let searcher = Arc::new(Mutex::new(Search::new(false)));
    //init neural network
    {
        board.set_startpos(&searcher.lock().unwrap().nnue);
    }

    // Engine result channel: search task -> socket loop
    let (engine_tx, mut engine_rx) = mpsc::unbounded_channel::<(EngineMove, i32)>();

    // (depth, move, score, thinking_flag_for_state)
    let mut playerside: u8 = 0;
    let mut thinking = false;

    // Initial state
    if send_json(&mut socket, &ServerMsg::State(make_state(&mut board, &movegen, thinking, Some(0), None)))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            // 1) Engine finished thinking
            maybe_best = engine_rx.recv() => {
                let Some((best_move, best_score)) = maybe_best else { return; };
                

                // // 1) Send a hint/update BEFORE mutating the board
                // let hint_state = make_state(
                //     &mut board,
                //     &movegen,
                //     true,                    // thinking
                //     Some(best_score),         // eval
                //     Some(best_move),          // best_move (EngineMove)
                // );

                // if send_json(&mut socket, &ServerMsg::State(hint_state)).await.is_err() {
                //     return;
                // }

                eprintln!("SERVER SEND eval_cp={}", -best_score);
                let final_state = {
                    let mut s = searcher.lock().unwrap();
                    board.push(best_move, &movegen, &s.nnue);
                    let nn = s.nnue.eval_cp_like(&board);
                    eprintln!("NNUE eval after push: {}", nn);
                    thinking = false;
                    make_state(&mut board, &movegen, thinking, Some(best_score), None)
                }; // 🔴 lock dropped HERE

                if send_json(&mut socket, &ServerMsg::State(final_state)).await.is_err() {
                    return;
                }
            }
            // 2) Incoming websocket frames
            maybe_frame = socket.recv() => {
                let Some(Ok(frame)) = maybe_frame else {
                    return;
                };
                
                match frame {
                    Message::Text(text) => {
                        let parsed: Result<ClientMsg, serde_json::Error> = serde_json::from_str(&text);

                        match parsed {
                            Ok(ClientMsg::NewGame {playerside}) => {
                                if thinking {
                                    // Optional: ignore or allow cancel. Keeping it strict for now.
                                    let _ = send_json(&mut socket, &ServerMsg::Error { message: "Engine is thinking".to_string() }).await;
                                    continue;
                                }
                                thinking = false;
                                board = Board::new();

                                let searcher_cloned = searcher.clone();
                                {
                                    let mut s = searcher.lock().unwrap();
                                    board.set_startpos(&s.nnue);
                                }
                                let text = serde_json::to_string(&ServerMsg::State(make_state(&mut board, &movegen, thinking, None, None))).unwrap();
                                println!("SENT: {}", text);
                                if send_json(&mut socket, &ServerMsg::State(make_state(&mut board, &movegen, false, None, None)))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }

                                //if playerside is black
                                if playerside == 1 {
                                    thinking = true;
                                    if send_json(&mut socket,
                                        &ServerMsg::State(make_state(&mut board, &movegen, true, None, None))
                                    ).await.is_err() {
                                        return;
                                    }

                                    let tx = engine_tx.clone();
                                    let mut board_for_search = board.clone_position();
                                    let depth: u8 = 5;
                                    let searcher = searcher.clone();

                                    tokio::spawn(async move {
                                        let best = tokio::task::spawn_blocking(move || {
                                            let mg = MoveGenerator::new();
                                            let mut s = searcher.lock().unwrap();
                                            s.search_root(&mut board_for_search, depth, &mg)
                                        }).await.unwrap();
                                        let (mv, score) = best;
                                        let _ = tx.send((mv, score));

                                    });
                                }
                            }

                            Ok(ClientMsg::SetPosition { fen }) => {
                                if thinking {
                                    let _ = send_json(&mut socket, &ServerMsg::Error { message: "Engine is thinking".to_string() }).await;
                                    continue;
                                }
                                board = Board::new();
                                let searcher = searcher.clone();
                                {
                                    let mut s = searcher.lock().unwrap();
                                    board.from_fen(fen, &s.nnue);
                                }
                                if send_json(&mut socket, &ServerMsg::State(make_state(&mut board, &movegen, false, None, None)))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                            }

                            Ok(ClientMsg::PlayMove { id }) => {
                                
                                if thinking {
                                    // Board is locked while engine thinks
                                    let _ = send_json(&mut socket, &ServerMsg::MoveResult { ok: false, reason: "Engine is thinking".to_string() }).await;
                                    continue;
                                }

                                // Recompute legal list now (authoritative)
                                let legal = movegen.generate(&mut board);

                                let Some(player_move) = legal.get(id as usize).copied() else {
                                    let _ = send_json(&mut socket, &ServerMsg::MoveResult { ok: false, reason: "Illegal move id".to_string() }).await;
                                    // Re-send state for UI consistency
                                    let _ = send_json(&mut socket, &ServerMsg::State(make_state(&mut board, &movegen, false, None, None))).await;
                                    continue;
                                };

                                // Apply player move
                                {
                                    let mut s = searcher.lock().unwrap();
                                    board.push(player_move, &movegen, &s.nnue);
                                    let nn = s.nnue.eval_cp_like(&board);
                                    eprintln!("NNUE eval after push: {}", nn);
                                } // 🔴 lock dropped

                                if send_json(&mut socket, &ServerMsg::MoveResult { ok: true, reason: String::new() })
                                    .await
                                    .is_err()
                                {
                                    return;
                                }

                                // Immediately send state with thinking=true (locks UI)
                                thinking = true;
                                if send_json(&mut socket, &ServerMsg::State(make_state(&mut board, &movegen, true, None, None)))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }

                                // Spawn search to full depth (blocking)
                                // IMPORTANT: we clone the board for search so we don't race the authoritative board.
                                let tx = engine_tx.clone();
                                let mut board_for_search = board.clone_position();
                                let depth: u8 = 7;            // hardcode for now; add to protocol later
                                let searcher = searcher.clone();
                                tokio::spawn(async move {
                                    let best: Result<(EngineMove, i32), tokio::task::JoinError> =
                                        tokio::task::spawn_blocking(move || {
                                            let mg_local = MoveGenerator::new();
                                            let mut s = searcher.lock().unwrap();
                                            s.search_root(&mut board_for_search, depth, &mg_local)
                                        })
                                        .await;

                                    match best {
                                        Ok((best_move, best_score)) => { let _ = tx.send((best_move, best_score)); }
                                        Err(e) => eprintln!("spawn_blocking join error: {e}"),
                                    }
                                });
                            }

                            Err(e) => {
                                let _ = send_json(&mut socket, &ServerMsg::Error {
                                    message: format!("Invalid JSON: {e}"),
                                }).await;
                            }
                        }
                    }

                    Message::Ping(p) => {
                        if socket.send(Message::Pong(p)).await.is_err() {
                            return;
                        }
                    }

                    Message::Close(_) => return,
                    _ => {}
                }
            }
        }
    }
}

// ===== Helpers =====

fn normalize_board_cells(cells: Vec<String>) -> Result<Vec<String>, String> {
    // 1) Trim whitespace
    let mut trimmed: Vec<String> = cells.into_iter().map(|s| s.trim().to_string()).collect();

    // 2) If it's already 64 single-character cells, accept it
    if trimmed.len() == 64 && trimmed.iter().all(|s| s.chars().count() == 1) {
        return Ok(trimmed);
    }

    // 3) If it's 8 rank strings of length 8 (or with separators), flatten them
    if trimmed.len() == 8 {
        let mut out = Vec::with_capacity(64);

        for row in trimmed.drain(..) {
            // Keep only non-whitespace chars
            let row_chars: Vec<char> = row.chars().filter(|c| !c.is_whitespace()).collect();

            if row_chars.len() != 8 {
                return Err(format!("Rank string is not 8 chars after trimming: {:?}", row));
            }

            for ch in row_chars {
                out.push(ch.to_string());
            }
        }

        if out.len() == 64 && out.iter().all(|s| s.chars().count() == 1) {
            return Ok(out);
        }
        return Err("Flattened board did not produce 64 single-char cells".to_string());
    }

    // 4) Otherwise, fail with a clear message
    Err(format!(
        "Unexpected board_to_chars() shape: len={} sample={:?}",
        trimmed.len(),
        trimmed.get(0..std::cmp::min(trimmed.len(), 12))
    ))
}fn make_state(board: &mut Board, movegen: &MoveGenerator, thinking: bool, eval: Option<i32>, best_move: Option<EngineMove>) -> State {
    let raw = board.board_to_chars();
    let board_cells = normalize_board_cells(raw).expect("bad board_to_chars");

    let legal = if thinking {
        // Optional: you can send empty legal list while thinking to make UI simpler/safer.
        Vec::new()
    } else {
        movegen.generate(board)
    };

    let legal_moves: Vec<Move> = legal
        .iter()
        .enumerate()
        .map(|(i, em)| {
            let promo = if em.isprom() {
                Some(match em.prompiece() {
                    PieceType::N => 'n',
                    PieceType::B => 'b',
                    PieceType::R => 'r',
                    PieceType::Q => 'q',
                    _ => 'q',
                })
            } else {
                None
            };

            Move {
                id: i as u16,
                from: em.getSrc(),
                to: em.getDst(),
                promo,
            }
        })
        .collect();

    State {
        board: board_cells,
        turn: board.turn,
        legal_moves,
        thinking,
        eval,
        best_move: best_move.map(engine_move_to_ui),
    }
}
async fn send_json(socket: &mut WebSocket, msg: &ServerMsg) -> Result<(), ()> {
    let text = serde_json::to_string(msg).map_err(|_| ())?;
    
    socket.send(Message::Text(text.into())).await.map_err(|_| ())
}
fn engine_move_to_ui(m: rustychess::core::Move) -> Move {
    let promo = if m.isprom() {
        Some(match m.prompiece() {
            PieceType::N => 'n',
            PieceType::B => 'b',
            PieceType::R => 'r',
            PieceType::Q => 'q',
            _ => 'q',
        })
    } else {
        None
    };

    Move {
        id: 0,
        from: m.getSrc(),
        to: m.getDst(),
        promo,
    }
}