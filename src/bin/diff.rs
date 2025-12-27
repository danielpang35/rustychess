use rustychess::uci::stockfish::Stockfish;
use rustychess::diff::{corpus, diff_fen, report};

// simple RNG (no external crate)
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn main() {
    let mut sf = Stockfish::new("/opt/homebrew/bin/stockfish");

    // Edge cases first
    for fen in corpus::edge_case_fens() {
        if let Err(diff) = diff_fen(fen, &mut sf) {
            report::print(diff);
            return;
        }
    }

    // Hard-coded test parameters
    let walks: usize = 1_000;      // number of random games
    let plies_min: usize = 0;
    let plies_max: usize = 80;

    let mut rng_state = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    for w in 0..walks {
        if w % 100 == 0 {
            eprintln!("walk {}", w);
        }

        let span = (plies_max - plies_min + 1) as u64;
        let plies = plies_min + (xorshift64(&mut rng_state) % span) as usize;

        let mut moves_line: Vec<String> = Vec::new();

        for ply in 0..=plies {
            // Diff the current position BEFORE choosing the next move
            let fen = sf.fen_from_startpos_moves(&moves_line);

            if let Err(diff) = diff_fen(&fen, &mut sf) {
                eprintln!("Mismatch on walk {} ply {}", w, ply);
                report::print(diff);
            }

            if ply == plies {
                break; // we already diffed the final position
            }

            // Choose a random legal move using Stockfish as the move source
            let legal = sf.legal_moves(&fen);
            if legal.is_empty() {
                break; // mate/stalemate
            }

            let idx = (xorshift64(&mut rng_state) % (legal.len() as u64)) as usize;
            moves_line.push(legal[idx].clone());
        }
    }

    println!("All positions matched (including intermediate positions).");
}
