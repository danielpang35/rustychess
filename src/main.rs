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
    board.from_fen(String::from("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1    "));
    let mg = core::movegen::MoveGenerator::new();
    let ml = mg.generate(&board);
    for bm in ml {
        bm.print();
    }
    println!("{}",board.toStr(s));
    println!("{}",board.piece_exists_at(0,0));
    let p = core::piece::Piece::make(0, core::piece::PieceType::P);
}
