use crate::diff::harness::DiffResult;
use crate::core::Board;
pub fn print(diff: DiffResult) {
    println!("=== MOVEGEN MISMATCH ===");
    println!("FEN: {}", diff.fen);
    let mut board = Board::new();
    board.from_fen(diff.fen.clone());
    board.print();

    if !diff.missing.is_empty() {
        println!("Missing moves (Stockfish has, you do not):");
        for m in diff.missing {
            println!("  {}", m);
        }
    }

    if !diff.extra.is_empty() {
        println!("Extra moves (you have, Stockfish does not):");
        for m in diff.extra {
            println!("  {}", m);
        }
    }
}
