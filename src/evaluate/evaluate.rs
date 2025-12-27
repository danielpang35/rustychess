use crate::core::{Board, PieceIndex};

pub fn evaluate(board: &Board) -> i32 {
    let mut score: i32 = 0;

    for color in 0usize..2usize {
        let factor: i32 = if color as u8 == board.turn { 1 } else { -1 };

        let base = 6 * color;

        let qbb = board.pieces[base + PieceIndex::Q.index() as usize];
        let rbb = board.pieces[base + PieceIndex::R.index() as usize];
        let bbb = board.pieces[base + PieceIndex::B.index() as usize];
        let nbb = board.pieces[base + PieceIndex::N.index() as usize];
        let pbb = board.pieces[base + PieceIndex::P.index() as usize];

        let q = qbb.count_ones() as i32;
        let r = rbb.count_ones() as i32;
        let b = bbb.count_ones() as i32;
        let n = nbb.count_ones() as i32;
        let p = pbb.count_ones() as i32;

        score += factor * (100 * p + 300 * n + 330 * b + 500 * r + 900 * q);
    }

    score
}
