mod core;
use std::env;
#[allow(dead_code)]

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let mut board = core::Board::new();
    let s = String::new();
    //board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    //
    board.from_fen(String::from("rnbqkbnr/pppp1ppp/8/8/3PpP2/N7/PPP1P1PP/R1BQKBNR b KQkq f3 0 3"));
    let mg = core::movegen::MoveGenerator::new();
    let ml = mg.generate(&board);
    for bm in ml {
        bm.print();
    }
    println!("{}",board.toStr(s));

    //core::constlib::print_bitboard(mg.king[23]); 
    println!("{}",board.piece_exists_at(0,0));
    let p = core::piece::Piece::make(0, core::piece::PieceType::P);
}
