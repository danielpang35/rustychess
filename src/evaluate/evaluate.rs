use crate::core::{Board, PieceIndex, constlib, movegen::MoveGenerator};
use crate::evaluate::evaluate::constlib::KNIGHT_PST;
use crate::evaluate::evaluate::constlib::PAWN_PST;
pub fn evaluate(board: &Board, mg: &MoveGenerator) -> i32 {
    let mut score: i32 = 0;

    for color in 0usize..2usize {
        let factor: i32 = if color as u8 == board.turn { 1 } else { -1 };

        let base = 6 * color;

        let mut qbb = board.pieces[base + PieceIndex::Q.index() as usize];
        let mut rbb = board.pieces[base + PieceIndex::R.index() as usize];
        let mut bbb = board.pieces[base + PieceIndex::B.index() as usize];
        let mut nbb = board.pieces[base + PieceIndex::N.index() as usize];
        let mut pbb = board.pieces[base + PieceIndex::P.index() as usize];

        let q = qbb.count_ones() as i32;
        let r = rbb.count_ones() as i32;
        let b = bbb.count_ones() as i32;
        let n = nbb.count_ones() as i32;
        let p = pbb.count_ones() as i32;
        


        // ---- Knights ----
        // ALSO GIVE BONUSES FOR CERTAIN SQUARES
        while nbb != 0 {
            let sq = constlib::poplsb(&mut nbb) as usize;
            score += factor * pst_value(&KNIGHT_PST, sq, color);

        }

        // ---- Pawns (captures only) ----
        // ALSO GIVE BONUSES FOR CERTAIN SQUARES
        while pbb != 0 {
            let sq = constlib::poplsb(&mut pbb) as usize;
            score += factor * pst_value(&PAWN_PST, sq, color);
        }

        score += factor * (100 * p + 300 * n + 330 * b + 500 * r + 900 * q );
        }

    
    // let control = board.attacked[if board.turn == 0 {1} else {0}].count_ones() as i32;
    // let enemy_control = board.attacked[board.turn as usize].count_ones() as i32;
    // score -= enemy_control;
    // score += control * 1; // very small weight

    score
}

#[inline(always)]
fn pst_value(table: &[i16; 64], sq: usize, color: usize) -> i32 {
    if color == 0 {
        table[sq] as i32
    } else {
        table[constlib::mirror_sq(sq)] as i32
    }
}
