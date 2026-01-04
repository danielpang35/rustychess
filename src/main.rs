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

    board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    //board.from_fen(String::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 "));
    
    let mg = movegen::MoveGenerator::new();
    
    use std::time::Instant;
    cli::interactive_cli(&mut board, &mg);
    let mut search = Search::new();
    let bm = search.search_root(&mut board, 4,&mg);
    println!("Best move found: ");
    bm.0.print();
    let movelist = mg.generate(&mut board);
    
    println!("Running perft from starting position...");

    let mut line: Vec<Move> = Vec::with_capacity(64);
    let res = constlib::perft_tracked(&mut board, 6, &mg, &mut line);
    eprintln!("Line:");
    for m in line.iter() { m.print(); }
    use std::collections::HashMap;

    fn divide(board: &mut Board, depth: u8, mg: &movegen::MoveGenerator) -> HashMap<String, u64> {
        let movelist = mg.generate(board);
        let mut out = HashMap::new();

        for m in movelist {
            let key = m.tostr(); // or your existing move-to-string method
            board.push(m, mg);
            let cnt = constlib::perft(board, depth - 1, mg);
            board.pop(mg);
            out.insert(key, cnt);
        }
        out
    }

    fn expected_divide_startpos_depth5() -> HashMap<&'static str, u64> {
    HashMap::from([
        ("a2a3", 181046), ("b2b3", 215255), ("c2c3", 222861), ("d2d3", 328511),
        ("e2e3", 402988), ("f2f3", 178889), ("g2g3", 217210), ("h2h3", 181044),
        ("a2a4", 217832), ("b2b4", 216145), ("c2c4", 240082), ("d2d4", 361790),
        ("e2e4", 405385), ("f2f4", 198473), ("g2g4", 214048), ("h2h4", 218829),
        ("b1a3", 198572), ("b1c3", 234656), ("g1f3", 233491), ("g1h3", 198502),
    ])
}

    fn compare_divide(actual: &HashMap<String, u64>, expected: &HashMap<&'static str, u64>) {
        let mut deltas: Vec<(String, i64)> = Vec::new();

        for (&mv, &exp) in expected.iter() {
            let act = *actual.get(mv).unwrap_or(&0);
            if act != exp {
                deltas.push((mv.to_string(), act as i64 - exp as i64));
            }
        }

        deltas.sort_by_key(|(_, d)| d.abs());
        for (mv, d) in deltas {
            println!("MISMATCH {mv}: delta {d}");
        }
    }
    board.set_startpos();
    let actual = divide(&mut board, 5, &mg);
    let expected = expected_divide_startpos_depth5();
    compare_divide(&actual, &expected);

    for i in 1..7 {
        let start = Instant::now();
        if i != 6 {
            let res = constlib::perft(&mut board,i,&mg);
            println!("Perft: depth = {}, result = {} (time {}s)", i, res, start.elapsed().as_secs_f64());
        } else {
            // Do a per-move divide at depth 4
            let movelist = mg.generate(&mut board);
            let mut total: u64 = 0;
            for m in movelist {
                m.print();
                board.push(m, &mg);
                let cnt = constlib::perft(&mut board, i-1, &mg);
                board.pop(&mg);
                println!("  -> {}", cnt);
                total += cnt;
            }
            println!("Perft: depth = {}, total = {} (time {}s)", i, total, start.elapsed().as_secs_f64());
        }
    }

    // Targeted perft-divide for debugging: inspect subtrees for c2c3 and f2f3
    fn perft_divide_for_move(board: &mut Board, mg: &movegen::MoveGenerator, src: u8, dst: u8, depth: u8) {
        use constlib;
        let mut found = false;
        let movelist = mg.generate(board);
        for m in movelist {
            if m.getSrc() == src && m.getDst() == dst {
                found = true;
                println!("\nPerft-divide for move {}{} (depth {})", constlib::squaretouci(src), constlib::squaretouci(dst), depth);
                board.push(m,mg);
                let children = mg.generate(board);
                let mut total: u64 = 0;
                for cm in children {
                    cm.print();
                    board.push(cm,&mg);
                    let cnt = constlib::perft(board, depth - 1, mg);
                    board.pop(mg);
                    println!("  -> {}", cnt);
                    total += cnt;
                }
                println!("Subtree total = {}\n", total);
                board.pop(mg);
                break;
            }
        }
        if !found {
            println!("Move {}{} not found in root move list", constlib::squaretouci(src), constlib::squaretouci(dst));
        }
    }


    //perft_divide_for_move(&mut board, &mg, 10, 18, 5); // c2c3
    //perft_divide_for_move(&mut board, &mg, 11, 19, 4); //d2d3
    // perft_divide_for_move(&mut board, &mg, 12, 20, 5); //e2e3

    // perft_divide_for_move(&mut board, &mg, 13, 21, 3); // f2f3
   //perft_divide_for_move(&mut board, &mg, 9, 17, 3); // b2b3
    // Dive one more ply into a specific reply: e7e6 (black) after each white move
    fn perft_divide_for_sequence(board: &mut Board, mg: &movegen::MoveGenerator, seq: &[(u8,u8)], depth: u8) {
        use constlib;
        // push all moves in seq, tracking how many were pushed so we can pop exactly that many on failure
        let mut pushed: usize = 0;
        for (src,dst) in seq.iter() {
            // find move in root-generated list (or current generated list)
            let movelist = mg.generate(board);
            let mut found = false;
            for m in movelist {
                if m.getSrc() == *src && m.getDst() == *dst {
                    board.push(m, mg);
                    pushed += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                println!("Sequence move {}{} not found", constlib::squaretouci(*src), constlib::squaretouci(*dst));
                // pop any that were pushed
                for _ in 0..pushed { board.pop(mg); }
                return;
            }
        }
        // remaining depth
        let remaining = depth as i32 - seq.len() as i32;
        println!("\nPerft-divide for sequence {:?} (remaining depth {})", seq.iter().map(|(s,d)| format!("{}{}", constlib::squaretouci(*s), constlib::squaretouci(*d))).collect::<Vec<_>>(), remaining);
        if remaining <= 0 {
            // just compute perft at this node
            let cnt = constlib::perft(board, 0, mg);
            println!("Leaf node count {}", cnt);
            for _ in 0..pushed { board.pop(mg); }
            return;
        }
        let children = mg.generate(board);
        let mut total: u64 = 0;
        for cm in children {
            cm.print();
            board.push(cm,mg);
            let cnt = constlib::perft(board, (remaining - 1) as u8, mg);
            board.pop(mg);
            println!("  -> {}", cnt);
            total += cnt;
        }
        println!("Sequence subtree total = {}\n", total);
        // pop seq moves
        for _ in 0..pushed { board.pop(mg); }
    }

    //perft_divide_for_sequence(&mut board, &mg, &[(10,18),(62,45),(3,24)], 4); // c2c3 then e7e6
    // perft_divide_for_sequence(&mut board, &mg, &[(13,21),(52,44)], 3); // f2f3 then e7e6
    //perft_divide_for_sequence(&mut board, &mg, &[(10,18),(51,35),(3,24),(57,42),(8,16)], 6); // c2c3 then d7d5
    //perft_divide_for_sequence(&mut board, &mg, &[(11,19),(50,42),(3,11),(59,32)], 5); // d2d3, c7c6,d1d2
    //perft_divide_for_sequence(&mut board, &mg, &[(12,20),(51,43),(3,21),(60,51),(21,53)], 6); // c2c3 then d7d5

    //perft_divide_for_sequence(&mut board, &mg, &[(10,18),(50,34),(3,24)], 5); // c2c3 then b7b5
    // //perft_divide_for_sequence(&mut board, &mg, &[(10,18),(49,33),(3,24)], 4); // c2c3 then b7b5 then a1d4
    //perft_divide_for_sequence(&mut board, &mg, &[(9,17),(52,36),(2,16)], 4); // b2b3 then e7e5 then c1a3
}

