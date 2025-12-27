#[cfg(test)]


mod tests {
    use crate::core::Board;
    use crate::core::movegen::MoveGenerator;
    use crate::core::constlib;
    fn boards_equal(a: &Board, b: &Board) -> bool {
        if a.occupied != b.occupied { return false; }
        if a.playerpieces != b.playerpieces { return false; }
        if a.turn != b.turn { return false; }

        for i in 0..12 {
            if a.pieces[i] != b.pieces[i] {
                return false;
            }
        }

        // Check piece locations square-by-square
        for sq in 0..64 {
            if a.piecelocs.piece_at(sq) != b.piecelocs.piece_at(sq) {
                return false;
            }
        }

        // Check board state
        let sa = &a.state;
        let sb = &b.state;

        if sa.castling_rights != sb.castling_rights { return false; }
        if sa.ep_square != sb.ep_square { return false; }
        if sa.pinned != sb.pinned { return false; }
        if sa.pinners != sb.pinners { return false; }
        if sa.attacked != sb.attacked { return false; }

        true
    }

    #[test]
    fn test_push_pop() {
        let mut board = Board::new();
        board.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        let mg = MoveGenerator::new();
        let mov = mg.generate(&mut board).pop().unwrap();
        let board_before = board.clone();
        board.push(mov, &mg);
        board.pop();
        assert!(boards_equal(&board_before, &board), "Board state mismatch after push/pop");
    }

    #[test]
    fn push_pop_invariant_tactical_positions() {
        use crate::core::{Board, movegen::MoveGenerator};

        let mg = MoveGenerator::new();

        let fens = [
            // Promotion
            "8/P7/8/8/8/8/7p/7K w - - 0 1",
            // Promotion capture
            "1r6/P7/8/8/8/8/8/7K w - - 0 1",
            // En passant available
            "rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 2",
            // Castling rights + pieces moved
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        ];

        for fen in fens {
            let mut board = Board::new();
            board.from_fen(fen.to_string());
            let original = board.clone();

            let moves = mg.generate(&mut board);
            assert!(!moves.is_empty(), "no legal moves for fen {}", fen);

            for m in moves {
                board.push(m, &mg);
                board.pop();

                if !boards_equal(&board, &original) {
                    m.print();
                    panic!(
                        "Push/pop invariant failed on fen {}\nmove ",
                        fen
                    );
                }
            }
        }
    }
    // Deterministic tiny RNG: xorshift64*
struct Rng {
    state: u64,
}
impl Rng {
    fn new(seed: u64) -> Self { Self { state: seed } }
    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn gen_usize(&mut self, n: usize) -> usize {
        // n must be > 0
        (self.next_u64() as usize) % n
    }
}

#[test]
fn push_pop_invariant_random_walk_small() {
    use crate::core::{Board, movegen::MoveGenerator};

    let mg = MoveGenerator::new();

    // Seeds chosen to cover: normal play, castling rights, EP, promotion.
    // Keep this small; it's meant to be fast but broad.
    let seed_fens = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
    "rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 2",
    // promotion seeds WITH both kings:
    "k7/P7/8/8/8/8/7p/7K w - - 0 1",
    "k1r5/P7/8/8/8/8/8/7K w - - 0 1",
];


    // If this test ever fails, print enough to reproduce deterministically.
    // Bump plies later if desired.
    let plies_per_seed: usize = 200;

    for (fen_i, fen) in seed_fens.iter().enumerate() {
        let mut board = Board::new();
        board.from_fen((*fen).to_string());

        // Seed RNG deterministically from fen index + a constant.
        let mut rng = Rng::new(0xC0FFEE_u64 ^ (fen_i as u64).wrapping_mul(0x9E3779B97F4A7C15));

        // Walk forward, checking push/pop on the chosen move each ply.
        for ply in 0..plies_per_seed {
         

            let moves = mg.generate(&mut board);

            if moves.is_empty() {
                // terminal node (mate/stalemate) - stop this walk
                break;
            }

            let idx = rng.gen_usize(moves.len());
            let m = moves[idx];

            // Invariant check for this move: push then pop must restore EXACT board.
            let before = board.clone();
            board.push(m, &mg);
            board.pop();

            if !boards_equal(&before, &board) {
                // Print minimal repro info
                eprintln!("Random invariant failed");
                eprintln!("FEN seed index: {fen_i}");
                eprintln!("Seed FEN: {fen}");
                eprintln!("Ply: {ply}");
                eprintln!("Move index: {idx} / {}", moves.len());
                m.print();
                // If you want, also print board:
                // before.print();
                // board.print();
                panic!("Push/pop invariant failed in random walk");
            }

            // Advance the walk: actually push and keep it.
            board.push(m, &mg);
        }

        // Rewind fully to ensure the state stack unwinds properly.
        // (This is optional; it ensures no stack corruption.)
        while board.state.prev.is_some() {
            board.pop();
        }
    }
}
    fn king_sanity(board: &Board) -> Result<(), String> {
        use crate::core::piece::PieceIndex;

        let wk = board.pieces[PieceIndex::K.index()];
        let bk = board.pieces[PieceIndex::k.index()];

        if wk.count_ones() != 1 {
            return Err(format!("White king bitboard has {} bits: {:016x}", wk.count_ones(), wk));
        }
        if bk.count_ones() != 1 {
            return Err(format!("Black king bitboard has {} bits: {:016x}", bk.count_ones(), bk));
        }
        Ok(())
    }

}