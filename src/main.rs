#[allow(dead_code)]
#[allow(nonstandard_style)]
mod core;
use core::constlib;
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let mut board = core::Board::new();
    let s = String::new();
    board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    
    // board.from_fen(String::from("k5r1/3P1P2/8/8/8/8/8/3K4 b - - 0 1"));
    let mg = core::movegen::MoveGenerator::new();
    
    println!("Running perft....");
    for i in 0..6 {
        let res = constlib::perft(&mut board,i,&mg);
        println!("Perft: depth = {}, result = {}",i, res);
    }
    // let ml = mg.generate(&board);
    // let mlc = ml.clone();
    // for bm in ml {
    //     bm.print();
    // }
    // board.push(mlc[1]);
    // let ml = mg.generate(&board);
    // println!("{}",ml.len());
    // let mlc = ml.clone();

    // for bm in ml {
    //     bm.print();
    // } 
    // board.push(mlc[17]);

    // board.pop();
    // board.pop();
    println!("{}",board.toStr());

}
