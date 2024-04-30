mod core;

fn main() {
    println!("Hello, world!");

    let board = core::Board::new();
    let s = String::new();
    println!("{}", board.toStr(s));
    println!("{}",board.piece_exists_at(0,0));
}
