#[allow(dead_code)]
#[allow(nonstandard_style)]

mod core;
use core::constlib;
use std::env;
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let mut board = core::Board::new();
    // Return to normal behavior: load starting position and run perft
    board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    let mg = core::movegen::MoveGenerator::new();
    use std::time::Instant;
    println!("Running perft from starting position...");
    for i in 1..5 {
        let start = Instant::now();
        let res = constlib::perft(&mut board,i,&mg);
        println!("Perft: depth = {}, result = {} (time {}s)", i, res, start.elapsed().as_secs_f64());
    }
}
