#[allow(dead_code)]
#[allow(nonstandard_style)]
mod core;
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let mut board = core::Board::new();
    let s = String::new();
    //board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    //
    board.from_fen(String::from("4k3/8/5p2/3b4/4N3/8/8/1K1R4 w - - 0 1
    "));
    let mg = core::movegen::MoveGenerator::new();
    let ml = mg.generate(&board);
    println!("{}",board.toStr());

    println!("Movelist length: {}", ml.len());
    let m = ml[7];
    for bm in ml {
        bm.print();
    }
    board.push(m);
    board.pop();
    println!("{}",board.toStr());

}
