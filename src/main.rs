mod core;
use crate::core::board as Board;

fn main() {
    println!("Hello, world!");

    let board = Board::new();
    println!(
        "{}",board.toStr()
    );
}
