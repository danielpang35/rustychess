use std::collections::BTreeSet;

use crate::core::constlib;
use crate::core::movegen::MoveGenerator;
use crate::core::r#move::Move;
use crate::core::Board;
use crate::uci::stockfish::Stockfish;

pub struct DiffResult {
    pub fen: String,
    pub missing: Vec<String>,
    pub extra: Vec<String>,
}

pub fn diff_fen(fen: &str, sf: &mut Stockfish) -> Result<(), DiffResult> {
    let mut board = Board::new();
    board.from_fen(fen.to_string());

    let mg = MoveGenerator::new();
    let our_vec = mg.generate(&mut board);

    let our_moves: BTreeSet<String> = our_vec.into_iter().map(move_to_uci).collect();
    let sf_moves: BTreeSet<String> = sf.legal_moves(fen).into_iter().collect();

    if our_moves != sf_moves {
        println!("FEN: {}", fen);
        eprintln!("castling_rights bits: {:04b}", board.state.castling_rights);
        let missing = sf_moves.difference(&our_moves).cloned().collect();
        let extra = our_moves.difference(&sf_moves).cloned().collect();

        Err(DiffResult {
            fen: fen.to_string(),
            missing,
            extra,
        })
    } else {
        Ok(())
    }
}



#[inline]
fn square_to_uci(sq: u8) -> String {
    constlib::squaretouci(sq)
}

pub fn move_to_uci(mv: Move) -> String {
    let flag = mv.flag();

    // Castling: your engine encodes it as e1h1/e1a1 etc.
    // UCI requires e1g1/e1c1 etc.
    if flag == 0b0010 || flag == 0b0011 {
        let src = mv.getSrc();
        let dst = match (src, flag) {
            (4,  0b0010) => 6,   // e1 -> g1
            (4,  0b0011) => 2,   // e1 -> c1
            (60, 0b0010) => 62,  // e8 -> g8
            (60, 0b0011) => 58,  // e8 -> c8
            _ => panic!("unexpected castling move: src={} flag={:b}", src, flag),
        };

        let mut s = String::with_capacity(4);
        s.push_str(&crate::core::constlib::squaretouci(src));
        s.push_str(&crate::core::constlib::squaretouci(dst));
        return s;
    }

    // Normal move
    let mut s = String::with_capacity(5);
    s.push_str(&crate::core::constlib::squaretouci(mv.getSrc()));
    s.push_str(&crate::core::constlib::squaretouci(mv.getDst()));

    if mv.isprom() {
        let ch = match mv.prompiece() {
            crate::core::piece::PieceType::N => 'n',
            crate::core::piece::PieceType::B => 'b',
            crate::core::piece::PieceType::R => 'r',
            crate::core::piece::PieceType::Q => 'q',
            _ => unreachable!("invalid promotion piece"),
        };
        s.push(ch);
    }

    s
}
