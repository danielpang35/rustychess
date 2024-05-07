mod core;
use std::env;

fn main() {
    #[allow(dead_code)]
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let mut board = core::Board::new();
    let s = String::new();
    board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
"));
    let mg = core::movegen::MoveGenerator::new();
    let ml = mg.generate(&board);
    println!("{}{}",ml[0].getSrc(),ml[0].getDst());
    //core::constlib::print_bitboard(mg.king[23]); 
    println!("{}",board.piece_exists_at(0,0));
    let p = core::piece::Piece::make(0, core::piece::PieceType::P);
}
