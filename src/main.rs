#[allow(dead_code)]
#[allow(nonstandard_style)]

use rustychess::core::{cli, constlib};
use rustychess::search::Search;
use rustychess::core::{Board, movegen, Move};

use std::env;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

// fn main() {
//     // Fixed seed → deterministic output
//     let mut rng = StdRng::seed_from_u64(0xCAFEBABEDEADBEEF);

//     println!("// AUTO-GENERATED ZOBRIST CONSTANTS");
//     println!("// Generated with fixed seed");
//     println!();

//     // Piece-square keys [12][64]
//     println!("pub const Z_PIECE_SQ: [[u64; 64]; 12] = [");
//     for p in 0..12 {
//         print!("    [");
//         for s in 0..64 {
//             let v: u64 = rng.gen();
//             if s % 4 == 0 { print!("\n        "); }
//             print!("0x{:016X}, ", v);
//         }
//         println!("\n    ],");
//     }
//     println!("];\n");

//     // Side to move
//     let z_side: u64 = rng.gen();
//     println!("pub const Z_SIDE: u64 = 0x{:016X};\n", z_side);

//     // Castling rights [16]
//     println!("pub const Z_CASTLING: [u64; 16] = [");
//     for i in 0..16 {
//         let v: u64 = rng.gen();
//         if i % 4 == 0 { print!("\n    "); }
//         print!("0x{:016X}, ", v);
//     }
//     println!("\n];\n");

//     // En-passant file [8]
//     println!("pub const Z_EP_FILE: [u64; 8] = [");
//     for i in 0..8 {
//         let v: u64 = rng.gen();
//         print!("0x{:016X}, ", v);
//     }
//     println!("];");
// }
fn main() {
    
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let mut board = Board::new();

    //board.from_fen(String::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 "));
    
    let mg = movegen::MoveGenerator::new();
    use std::time::Instant;
    let mut search = Search::new(false);
    board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), &search.nnue);

    cli::interactive_cli(&mut board, &mg, &search.nnue);

}

