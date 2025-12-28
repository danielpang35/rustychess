use std::io::{self, Write};
use crate::core::constlib;
use std::env;
use crate::core::Board;
use crate::core::movegen::MoveGenerator;
use crate::core::r#move::Move;
use crate::core::piece::PieceType;
use crate::core::Piece;
use crate::search;
/// Convert UCI string (e.g., "e2e4", "e7e8q") into a Move
pub fn uci_to_move(board: &Board, uci: &str) -> Option<Move> {
    if uci.len() < 4 { return None; }
    let src = constlib::square_from_string(&uci[0..2]);
    let dst = constlib::square_from_string(&uci[2..4]);

    // Determine flags dynamically
    let flag = if board.piecelocs.piece_at(dst) != Piece::None { Move::FLAG_CAPTURE }
               else { Move::FLAG_QUIET };

    // Handle promotions
    let mv = if uci.len() == 5 {
        let promo_piece = match &uci[4..5] {
            "n" | "N" => PieceType::N,
            "b" | "B" => PieceType::B,
            "r" | "R" => PieceType::R,
            "q" | "Q" => PieceType::Q,
            _ => return None,
        };
        Move::makeProm(src, dst, promo_piece)
    } else {
        Move::make(src, dst, flag)
    };

    Some(mv)
}

/// Interactive command line tester for the chess engine
pub fn interactive_cli(board: &mut Board, generator: &MoveGenerator) {
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

        let mv = match uci_to_move(board, mvstr) {
            Some(m) => m,
            None => { println!("Invalid move format."); continue; }
        };

        board.push(mv, generator);
        println!("Move applied: {}{}", &uci_from_square(mv.getSrc()), &uci_from_square(mv.getDst()));

        // Generate legal moves
        let moves = generator.generate(board);
        for mv in &moves {
            mv.print();
        }
        let mut searcher = search::Search::new();
        let bm = searcher.search_root(board,6, generator);
        board.push(bm, generator);
        println!("Move applied: {}{}", &uci_from_square(mv.getSrc()), &uci_from_square(mv.getDst()));

        println!("Legal moves: {} total", moves.len());

    }
}

/// Utility: convert square index to UCI string
fn uci_from_square(square: u8) -> String {
    constlib::squaretouci(square)
}
