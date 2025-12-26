#[allow(dead_code)]
#[allow(nonstandard_style)]

mod core;
use core::constlib;
use std::env;
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let mut board = core::Board::new();
    // Return to normal behavior: load starting position and run perft
    board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    let mg = core::movegen::MoveGenerator::new();
    use std::time::Instant;
    println!("Running perft from starting position...");

    for i in 1..6 {
        let start = Instant::now();
        if i != 5 {
            let res = constlib::perft(&mut board,i,&mg);
            println!("Perft: depth = {}, result = {} (time {}s)", i, res, start.elapsed().as_secs_f64());
        } else {
            // Do a per-move divide at depth 4
            let movelist = mg.generate(&mut board);
            let mut total: u64 = 0;
            for m in movelist {
                m.print();
                board.push(m);
                let cnt = constlib::perft(&mut board, i-1, &mg);
                board.pop();
                println!("  -> {}", cnt);
                total += cnt;
            }
            println!("Perft: depth = {}, total = {} (time {}s)", i, total, start.elapsed().as_secs_f64());
        }
    }

    // Targeted perft-divide for debugging: inspect subtrees for c2c3 and f2f3
    fn perft_divide_for_move(board: &mut core::Board, mg: &core::movegen::MoveGenerator, src: u8, dst: u8, depth: u8) {
        use core::constlib;
        let mut found = false;
        let movelist = mg.generate(board);
        for m in movelist {
            if m.getSrc() == src && m.getDst() == dst {
                found = true;
                println!("\nPerft-divide for move {}{} (depth {})", constlib::squaretouci(src), constlib::squaretouci(dst), depth);
                board.push(m);
                let children = mg.generate(board);
                let mut total: u64 = 0;
                for cm in children {
                    cm.print();
                    board.push(cm);
                    let cnt = constlib::perft(board, depth - 1, mg);
                    board.pop();
                    println!("  -> {}", cnt);
                    total += cnt;
                }
                println!("Subtree total = {}\n", total);
                board.pop();
                break;
            }
        }
        if !found {
            println!("Move {}{} not found in root move list", constlib::squaretouci(src), constlib::squaretouci(dst));
        }
    }

    //perft_divide_for_move(&mut board, &mg, 10, 18, 3); // c2c3
    //perft_divide_for_move(&mut board, &mg, 11, 27, 3);
    //#TODO: TEST OUT C2C3, THEN C7C5. our engine: c2c3 c7c5 11205, stockfish: 11129
    // perft_divide_for_move(&mut board, &mg, 13, 21, 3); // f2f3
   //perft_divide_for_move(&mut board, &mg, 9, 17, 3); // b2b3
    // Dive one more ply into a specific reply: e7e6 (black) after each white move
    fn perft_divide_for_sequence(board: &mut core::Board, mg: &core::movegen::MoveGenerator, seq: &[(u8,u8)], depth: u8) {
        use core::constlib;
        // push all moves in seq, tracking how many were pushed so we can pop exactly that many on failure
        let mut pushed: usize = 0;
        for (src,dst) in seq.iter() {
            // find move in root-generated list (or current generated list)
            let movelist = mg.generate(board);
            let mut found = false;
            for m in movelist {
                if m.getSrc() == *src && m.getDst() == *dst {
                    board.push(m);
                    pushed += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                println!("Sequence move {}{} not found", constlib::squaretouci(*src), constlib::squaretouci(*dst));
                // pop any that were pushed
                for _ in 0..pushed { board.pop(); }
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
            for _ in 0..pushed { board.pop(); }
            return;
        }
        let children = mg.generate(board);
        let mut total: u64 = 0;
        for cm in children {
            cm.print();
            board.push(cm);
            let cnt = constlib::perft(board, (remaining - 1) as u8, mg);
            board.pop();
            println!("  -> {}", cnt);
            total += cnt;
        }
        println!("Sequence subtree total = {}\n", total);
        // pop seq moves
        for _ in 0..pushed { board.pop(); }
    }

    //perft_divide_for_sequence(&mut board, &mg, &[(10,18),(62,45),(3,24)], 4); // c2c3 then e7e6
    // perft_divide_for_sequence(&mut board, &mg, &[(13,21),(52,44)], 3); // f2f3 then e7e6
    // perft_divide_for_sequence(&mut board, &mg, &[(10,18),(51,35),(3,24)], 4); // c2c3 then d7d5
    //perft_divide_for_sequence(&mut board, &mg, &[(11,27),(52,36),(2,20)], 4); // d2d4, e7e5 then b7b5

    //perft_divide_for_sequence(&mut board, &mg, &[(10,18),(50,34),(3,24)], 5); // c2c3 then b7b5
    // //perft_divide_for_sequence(&mut board, &mg, &[(10,18),(49,33),(3,24)], 4); // c2c3 then b7b5 then a1d4
    //perft_divide_for_sequence(&mut board, &mg, &[(9,17),(52,36),(2,16)], 4); // b2b3 then e7e5 then c1a3
}

