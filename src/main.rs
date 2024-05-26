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
    board.from_fen(String::from("8/8/8/4k3/8/4n3/3P4/K7 w - - 0 1   "));
    let mg = core::movegen::MoveGenerator::new();
    let ml = mg.generate(&board);
    println!("Movelist length: {}", ml.len());
    let m = ml[2];
    for bm in ml {
        bm.print();
    }
    board.push(m);
    println!("{}",board.toStr(s));

}
