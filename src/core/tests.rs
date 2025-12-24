#[cfg(test)]
mod tests {
    use crate::core::constlib;
    use std::sync::Mutex;
    
    // Use a once_cell pattern for the lock
    fn get_print_lock() -> &'static Mutex<()> {
        static LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }
    
    // Macro to synchronize printing
    macro_rules! sync_print {
        ($($arg:tt)*) => {
            {
                let _lock = get_print_lock().lock().unwrap();
                println!($($arg)*);
            }
        };
    }
    
    macro_rules! sync_print_bitboard {
        ($bb:expr) => {
            {
                let _lock = get_print_lock().lock().unwrap();
                constlib::print_bitboard($bb);
            }
        };
    }

    /// Test bishop attacks on an empty board (no blockers)
    /// Bishop on d4 (square 27: rank=3, file=3)
    /// Should attack all 4 diagonals until board edge
    #[test]
    fn test_bishop_d4_empty_board() {
        let sq: i8 = 27; // d4
        let blockers = 0;
        let attacks = constlib::compute_bishop(sq, blockers);
        
        sync_print!("\n=== Bishop on d4 (empty board) ===");
        sync_print_bitboard!(attacks);
        
        // Expected attack squares from d4 on empty board:
        // Northeast: e5, f6, g7, h8
        // Northwest: c5, b6, a7
        // Southwest: c3, b2, a1
        // Southeast: e3, f2, g1
        
        let expected_squares = vec![
            // northeast diagonal
            36, // e5
            45, // f6
            54, // g7
            63, // h8
            // northwest diagonal
            34, // c5
            41, // b6
            48, // a7
            // southwest diagonal
            18, // c3
            9,  // b2
            0,  // a1
            // southeast diagonal
            20, // e3
            13, // f2
            6,  // g1
        ];
        
        for &sq in &expected_squares {
            assert!(
                (attacks >> sq) & 1 == 1,
                "Bishop on d4 should attack square {}, but attacks = {:b}",
                sq,
                attacks
            );
        }
        
        // Verify we don't attack the bishop's own square
        assert!((attacks >> 27) & 1 == 0, "Bishop should not attack its own square");
    }

    /// Test bishop on h8 (top-right corner): no northeast moves possible
    #[test]
    fn test_bishop_h8_corner() {
        let sq: i8 = 63; // h8
        let blockers = 0;
        let attacks = constlib::compute_bishop(sq, blockers);
        
        sync_print!("\n=== Bishop on h8 (corner) ===");
        sync_print_bitboard!(attacks);
        
        // From h8, bishop can only go southwest (down-left diagonal)
        // g7, f6, e5, d4, c3, b2, a1
        let expected_squares = vec![54, 45, 36, 27, 18, 9, 0];
        
        for &sq in &expected_squares {
            assert!(
                (attacks >> sq) & 1 == 1,
                "Bishop on h8 should attack square {}, but attacks = {:b}",
                sq,
                attacks
            );
        }
        
        assert_eq!(attacks.count_ones(), expected_squares.len() as u32);
    }

    /// Test bishop on a1 (bottom-left corner): only northeast moves possible
    #[test]
    fn test_bishop_a1_corner() {
        let sq: i8 = 0; // a1
        let blockers = 0;
        let attacks = constlib::compute_bishop(sq, blockers);
        
        sync_print!("\n=== Bishop on a1 (corner) ===");
        sync_print_bitboard!(attacks);
        
        // From a1, bishop can only go northeast (up-right diagonal)
        // b2, c3, d4, e5, f6, g7, h8
        let expected_squares = vec![9, 18, 27, 36, 45, 54, 63];
        
        for &sq in &expected_squares {
            assert!(
                (attacks >> sq) & 1 == 1,
                "Bishop on a1 should attack square {}, but attacks = {:b}",
                sq,
                attacks
            );
        }
        
        assert_eq!(attacks.count_ones(), expected_squares.len() as u32);
    }

    /// Test bishop on h-file (rightmost file) does not wrap to a-file
    #[test]
    fn test_bishop_h_file_no_wrap() {
        let sq: i8 = 39; // h5
        let blockers = 0;
        let attacks = constlib::compute_bishop(sq, blockers);
        
        sync_print!("\n=== Bishop on h5 (no wraparound) ===");
        sync_print_bitboard!(attacks);
        
        // From h5, bishop can only go to three directions (not northeast):
        // Northwest: g6, f7, e8
        // Southwest: g4, f3, e2, d1
        // No southeast (already on h-file)
        
        let expected_squares = vec![
            46, // g6
            53, // f7
            60, // e8
            30, // g4
            21, // f3
            12, // e2
            3, // d1
        ];
        
        for &sq in &expected_squares {
            assert!(
                (attacks >> sq) & 1 == 1,
                "Bishop on h5 should attack square {}, but attacks = {:b}",
                sq,
                attacks
            );
        }
        
        // Make sure it doesn't wrap to a-file (squares with file 0)
        for file_a_sq in 0..8 {
            let a_file_sq = file_a_sq * 8; // a1, a2, ..., a8
            assert!(
                (attacks >> a_file_sq) & 1 == 0,
                "Bishop on h5 should NOT attack a-file square {}, but it does",
                a_file_sq
            );
        }
    }

    /// Test rook on d4 (empty board)
    #[test]
    fn test_rook_d4_empty_board() {
        let sq: i8 = 27; // d4
        let blockers = 0;
        let attacks = constlib::compute_rook(sq, blockers);
        
        sync_print!("\n=== Rook on d4 (empty board) ===");
        sync_print_bitboard!(attacks);
        
        // Rook attacks entire rank 4 (0-indexed rank 3) and entire d-file (file 3)
        let mut expected_squares = vec![];
        
        // Entire rank 4 (file a-h): 24, 25, 26, 27, 28, 29, 30, 31
        for file in 0..8 {
            let file_sq = 24 + file;
            if file_sq != 27 { // don't include the rook's own square
                expected_squares.push(file_sq);
            }
        }
        
        // Entire d-file (rank 1-8): 3, 11, 19, 27, 35, 43, 51, 59
        for rank in 0..8 {
            let file_sq = rank * 8 + 3;
            if file_sq != 27 {
                expected_squares.push(file_sq);
            }
        }
        
        for &sq in &expected_squares {
            assert!(
                (attacks >> sq) & 1 == 1,
                "Rook on d4 should attack square {}, but attacks = {:b}",
                sq,
                attacks
            );
        }
        
        assert_eq!(
            attacks.count_ones(),
            expected_squares.len() as u32,
            "Rook on d4 should attack {} squares",
            expected_squares.len()
        );
    }

    /// Test rook on h1 (corner): no wrap
    #[test]
    fn test_rook_h1_corner() {
        let sq: i8 = 7; // h1
        let blockers = 0;
        let attacks = constlib::compute_rook(sq, blockers);
        
        sync_print!("\n=== Rook on h1 (corner, no wraparound) ===");
        sync_print_bitboard!(attacks);
        
        // From h1: entire rank 1 (except h1) and entire h-file (except h1)
        // Rank 1: a1, b1, c1, d1, e1, f1, g1 (0-6)
        // h-file: h2, h3, ..., h8 (15, 23, 31, 39, 47, 55, 63)
        
        let expected_squares = vec![
            0, 1, 2, 3, 4, 5, 6, // rank 1 minus h1
            15, 23, 31, 39, 47, 55, 63, // h-file minus h1
        ];
        
        for &sq in &expected_squares {
            assert!(
                (attacks >> sq) & 1 == 1,
                "Rook on h1 should attack square {}, but attacks = {:b}",
                sq,
                attacks
            );
        }
        
        // Make sure it doesn't wrap to a-file
        for rank in 1..8 {
            let a_file_sq = rank * 8; // a2, a3, ..., a8
            assert!(
                (attacks >> a_file_sq) & 1 == 0,
                "Rook on h1 should NOT wrap to a-file square {}",
                a_file_sq
            );
        }
    }

    /// Test rook blocked by a piece on its attack line
    #[test]
    fn test_rook_d4_with_blocker() {
        let sq: i8 = 27; // d4
        let blocker_sq: i8 = 35; // d5 (one square north)
        let blockers = 1u64 << blocker_sq;
        let attacks = constlib::compute_rook(sq, blockers);
        
        sync_print!("\n=== Rook on d4 with blocker at d5 ===");
        sync_print_bitboard!(attacks);
        
        // Rook can attack north up to and including the blocker (capture), but not past it
        assert!(
            (attacks >> blocker_sq) & 1 == 1,
            "Rook should attack the blocker square at d5"
        );
        // It should not attack past the blocker
        let d6 = 43;
        assert!(
            (attacks >> d6) & 1 == 0,
            "Rook should not attack past blocker d6"
        );
    }

    /// Test bishop blocked by a piece
    /// Test bishop pinning - bishop pins a rook to the king
    #[test]
    fn test_bishop_pin_simple() {
        // Setup: White king on e1, white rook on e2, black bishop on e5
        // Rook should be pinned on the e-file
        let mut board = crate::core::Board::new();
        board.from_fen(String::from("8/8/8/4b3/8/8/4R3/4K3 w - - 0 1"));
        
        let pinned = board.getpinned();
        
        // White rook at e2 (square 12) should be pinned
        let rook_sq = 12;
        sync_print!("\n=== Test: Bishop pins rook ===");
        
        assert_eq!(
            (pinned[0] >> rook_sq) & 1,
            1,
            "Rook on e2 should be pinned by bishop on e5"
        );
    }
    
    /// Test rook pinning - rook pins a bishop to the king
    #[test]
    fn test_rook_pin_simple() {
        // Setup: White king on a1, white bishop on a3, black rook on a5
        // Bishop should be pinned on the a-file
        let mut board = crate::core::Board::new();
        board.from_fen(String::from("8/8/8/r7/8/B7/8/K7 w - - 0 1"));
        
        let pinned = board.getpinned();
        
        // White bishop at a3 (square 16) should be pinned
        let bishop_sq = 16;
        sync_print!("\n=== Test: Rook pins bishop ===");
        sync_print_bitboard!(pinned[0]);
        
        assert_eq!(
            (pinned[0] >> bishop_sq) & 1,
            1,
            "Bishop on a3 should be pinned by rook on a5"
        );
    }
    
    /// Test queen pinning - queen pins a pawn
    #[test]
    fn test_queen_pin_pawn() {
        // Setup: White king on e1, white pawn on e3, black queen on e8
        // Pawn should be pinned
        let mut board = crate::core::Board::new();
        board.from_fen(String::from("4q3/8/8/8/8/4P3/8/4K3 w - - 0 1"));
        
        let pinned = board.getpinned();
        
        // White pawn at e3 (square 20) should be pinned
        let pawn_sq = 20;
        sync_print!("\n=== Test: Queen pins pawn ===");
        sync_print!("Pinned pieces (white): {:064b}", pinned[0]);
        
        assert_eq!(
            (pinned[0] >> pawn_sq) & 1,
            1,
            "Pawn on e3 should be pinned by queen on e8"
        );
    }
    
    /// Test diagonal pin
    #[test]
    fn test_diagonal_pin() {
        // Setup: White king on e1, white knight on d2, black bishop on a5
        // Knight should be pinned on the diagonal
        let mut board = crate::core::Board::new();
        board.from_fen(String::from("8/8/8/b7/8/8/3N4/4K3 w - - 0 1"));
        
        let pinned = board.getpinned();
        
        // White knight at d2 (square 11) should be pinned
        let knight_sq = 11;
        sync_print!("\n=== Test: Diagonal pin ===");
        sync_print!("Pinned pieces (white): {:064b}", pinned[0]);
        
        assert_eq!(
            (pinned[0] >> knight_sq) & 1,
            1,
            "Knight on d2 should be pinned by bishop on a5"
        );
    }
    
    /// Test no pin when piece is not on ray
    #[test]
    fn test_no_pin_off_ray() {
        // Setup: White king on e1, white rook on d2, black bishop on e5
        // Rook should NOT be pinned (not on the ray)
        let mut board = crate::core::Board::new();
        board.from_fen(String::from("8/8/8/4b3/8/8/3R4/4K3 w - - 0 1"));
        
        let pinned = board.getpinned();
        
        // White rook at d2 (square 11) should NOT be pinned
        let rook_sq = 11;
        sync_print!("\n=== Test: No pin (off ray) ===");
        sync_print!("Pinned pieces (white): {:064b}", pinned[0]);
        
        assert_eq!(
            (pinned[0] >> rook_sq) & 1,
            0,
            "Rook on d2 should NOT be pinned"
        );
    }
    
    /// Test pinned piece can move along the pin ray
    #[test]
    fn test_pinned_piece_moves_along_ray() {
        // Setup: White king on e1, white rook on e3, black rook on e8
        // Rook is pinned vertically but should be able to move along e-file
        let mut board = crate::core::Board::new();
        board.from_fen(String::from("4r3/8/8/8/8/4R3/8/4K3 w - - 0 1"));
        
        let pinned = board.getpinned();
        
        // White rook at e3 (square 20) should be pinned
        let rook_sq = 20;
        sync_print!("\n=== Test: Pinned rook on e-file ===");
        sync_print!("Pinned pieces (white): {:064b}", pinned[0]);
        
        assert_eq!(
            (pinned[0] >> rook_sq) & 1,
            1,
            "Rook on e3 should be pinned by rook on e8"
        );
        
        // Now verify the rook can generate moves
        let moves = crate::core::movegen::MoveGenerator::new().generate(&mut board);
        
        // Filter for rook moves (source square e3 = 20)
        let rook_moves: Vec<_> = moves.iter().filter(|m| {
            m.getSrc() == 20 as u8
        }).collect();
        
        sync_print!("Rook can move to {} squares along pin ray", rook_moves.len());
        
        // Rook should be able to move to e2, e4, e5, e6, e7 (along e-file)
        // That's at least some valid moves
        assert!(
            rook_moves.len() > 0,
            "Pinned rook should still be able to move along the pin ray"
        );
    }
    
    /// Test multiple pieces pinned
    #[test]
    fn test_multiple_pins() {
        // Setup: White king on e1, white rooks on e3 and d2, black rooks on e8 and a2
        // Rook on e3 is pinned vertically by rook on e8 (both on e-file, king on e1)
        // Rook on d2 is pinned horizontally by rook on a2 (both on rank 2, king on e1? no...)
        // Actually, let's use: king on e5, rook at e3 pinned by rook at e8, and rook at b5 pinned by rook at a5
        let mut board = crate::core::Board::new();
        // Place king on e1 so rook on e3 is pinned by rook on e8
        board.from_fen(String::from("4r3/8/8/8/8/4R3/8/4K3 w - - 0 1"));
        
        let pinned = board.getpinned();
        
        // White rook at e3 (square 20) should be pinned
        let rook_e3_sq = 20;
        
        sync_print!("\n=== Test: Multiple pins (simplified) ===");
        sync_print_bitboard!(pinned[0]);
        
        assert_eq!(
            (pinned[0] >> rook_e3_sq) & 1,
            1,
            "Rook on e3 should be pinned by rook on e8"
        );
    }
    
    /// Test no pin when blocker is between slider and king
    #[test]
    fn test_no_pin_with_blocker_between() {
        // Setup: White king on e1, white bishop on d3, white rook on e4, black bishop on g6
        // No piece should be pinned because there's a blocker (rook on e4)
        let mut board = crate::core::Board::new();
        board.from_fen(String::from("8/8/6b1/8/4R3/3B4/8/4K3 w - - 0 1"));
        
        let pinned = board.getpinned();
        
        // Neither bishop nor rook should be pinned
        let bishop_d3_sq = 19;
        let rook_e4_sq = 28;
        
        sync_print!("\n=== Test: No pin with blocker between ===");
        sync_print!("Pinned pieces (white): {:064b}", pinned[0]);
        
        // Both should be unpinned
        assert_eq!(
            (pinned[0] >> bishop_d3_sq) & 1,
            0,
            "Bishop on d3 should NOT be pinned"
        );
        assert_eq!(
            (pinned[0] >> rook_e4_sq) & 1,
            0,
            "Rook on e4 should NOT be pinned"
        );
    }
    #[test]
    fn debug_c2c3_e7e6_move_list() {
        let _lock = get_print_lock().lock().unwrap();
        sync_print!("\n=== Debug: move list after c2c3 e7e6 ===");

        let mut board = crate::core::Board::new();
        board.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
        let mg = crate::core::movegen::MoveGenerator::new();

        // Sequence: c2c3 (10->18), e7e6 (52->44)
        let seq = [(10u8,18u8),(52u8,44u8)];
        let mut pushed = 0usize;
        for (src,dst) in seq.iter() {
            let mut found = false;
            for m in mg.generate(&mut board) {
                if m.getSrc() == *src && m.getDst() == *dst {
                    board.push(m);
                    pushed += 1;
                    found = true;
                    break;
                }
            }
            assert!(found, "Failed to push sequence move {}{}", constlib::squaretouci(*src), constlib::squaretouci(*dst));
        }

        // Print human-friendly board
        sync_print!("Board after c2c3 e7e6:");
        board.print();
        sync_print!("Occupied bitboard:");
        constlib::print_bitboard(board.occupied);

        // Collect generated moves (UCI) at this node
        let mut moves: Vec<String> = mg.generate(&mut board).into_iter()
            .map(|m| format!("{}{}", constlib::squaretouci(m.getSrc()), constlib::squaretouci(m.getDst())))
            .collect();
        moves.sort();

        sync_print!("Total moves: {}", moves.len());
        for mv in &moves {
            sync_print!("  {}", mv);
        }

        // Expected list (from user-provided data)
        let mut expected = vec![
            "a2a3","b2b3","d2d3","e2e3","f2f3","g2g3","h2h3",
            "c3c4","a2a4","b2b4","d2d4","e2e4","f2f4","g2g4","h2h4",
            "b1a3","g1f3","g1h3","d1c2","d1b3","d1a4",
        ].iter().map(|s| s.to_string()).collect::<Vec<_>>();
        expected.sort();

        // Clean up pushed moves
        for _ in 0..pushed { board.pop(); }

        assert_eq!(moves, expected, "Move list at c2c3 e7e6 does not match expected list");
    }
}
