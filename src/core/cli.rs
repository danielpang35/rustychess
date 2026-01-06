use std::io::{self, Write};
use crate::core::constlib;
use std::env;
use crate::core::Board;
use crate::core::movegen::MoveGenerator;
use crate::core::r#move::Move;
use crate::core::piece::PieceType;
use crate::core::Piece;
use crate::search;
use crate::evaluate::nnue::Nnue;
/// Convert UCI string (e.g., "e2e4", "e7e8q") into a Move
pub fn uci_to_move(board: &mut Board, gen: &MoveGenerator, uci: &str) -> Option<Move> {
    let mut uci = uci.trim().to_ascii_lowercase();
    if uci.len() < 4 { return None; }

    let src = constlib::square_from_string(&uci[0..2]);
    let dst = constlib::square_from_string(&uci[2..4]);

    let promo = if uci.len() == 5 {
        Some(match &uci[4..5] {
            "n" | "N" => PieceType::N,
            "b" | "B" => PieceType::B,
            "r" | "R" => PieceType::R,
            "q" | "Q" => PieceType::Q,
            _ => return None,
        })
    } else {
        None
    };

    let mut moves: Vec<Move> = Vec::new();
    // Whatever your generator entrypoint is called; adjust if needed:
    // gen.generate(board, &mut moves, /*evasions?*/ false);
    moves = gen.generate(board);

    for mv in moves {
        if mv.getSrc() == src && mv.getDst() == dst {
            match promo {
                None => {
                    if !mv.isprom() { return Some(mv); }
                }
                Some(p) => {
                    if mv.isprom() && mv.prompiece() == p { return Some(mv); }
                }
            }
        }
    }
    None
}

/// Interactive command line tester for the chess engine
pub fn interactive_cli(board: &mut Board, generator: &MoveGenerator, nnue: &Nnue) {
    let mut input = String::new();
    loop {
        board.print();
        println!("Enter move in UCI (or 'quit'):");
        input.clear();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let mvstr = input.trim();

        if mvstr == "quit" {
            break;
        }

        let mv = match uci_to_move(board, generator, input.trim()) {
            Some(m) => m,
            None => { println!("Invalid move format."); continue; }
        };

        board.push(mv, generator, nnue);
        println!("Move applied: {}{}", &uci_from_square(mv.getSrc()), &uci_from_square(mv.getDst()));

        // Generate legal moves
        let moves = generator.generate(board);
        for mv in &moves {
            mv.print();
        }
        let mut searcher = search::Search::new();
        let bm = searcher.search_iterative(board,7, generator).0;
        board.push(bm, generator, nnue);
        println!("Move applied: ");
        bm.print();
        println!("Legal moves: {} total", moves.len());

    }
}


/// Interactive command line tester for the chess engine
pub fn interactive_cli_test(board: &mut Board, generator: &MoveGenerator, nnue: &Nnue) {
    let mut input = String::new();
    loop {
        board.print();
        println!("Enter move in UCI (or 'quit'):");
        input.clear();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let mvstr = input.trim();

        if mvstr == "quit" {
            break;
        }

        let mv = match uci_to_move(board, generator, mvstr) {
            Some(m) => m,
            None => { println!("Invalid move format."); continue; }
        };

        board.push(mv, generator, nnue);
        println!("Move applied: {}{}", &uci_from_square(mv.getSrc()), &uci_from_square(mv.getDst()));

        // Generate legal moves
        let moves = generator.generate(board);
        for mv in &moves {
            mv.print();
        }
        println!("Legal moves: {} total", moves.len());

    }
}

/// Utility: convert square index to UCI string
fn uci_from_square(square: u8) -> String {
    constlib::squaretouci(square)
}
