use std::fs::File;
use std::io::Read;

use crate::core::{piece::PieceType, Board};

#[derive(Clone, Debug)]
pub struct Nnue {
    pub b1: [i32; 256],
}

impl Default for Nnue {
    fn default() -> Self {
        Self { b1: [0; 256] }
    }
}

impl Nnue {
    pub fn load(path: &str) -> std::io::Result<Self> {
        // Try to load an optional bias vector from disk so that existing files
        // still round-trip, but fall back to zeros when the file is absent.
        let mut nnue = Self::default();
        if let Ok(mut file) = File::open(path) {
            let mut buf = vec![];
            if file.read_to_end(&mut buf).is_ok() {
                for (i, chunk) in buf.chunks_exact(4).take(256).enumerate() {
                    nnue.b1[i] = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                }
            }
        }
        Ok(nnue)
    }

    #[inline(always)]
    pub fn eval_cp_like(&self, board: &Board) -> i32 {
        material_eval(board)
    }
}

#[inline(always)]
pub fn nnue_add_piece(
    _nnue: &Nnue,
    acc_w: &mut [i32; 256],
    acc_b: &mut [i32; 256],
    _wk_sq: usize,
    _bk_sq: usize,
    piece_idx: usize,
    sq: usize,
) {
    let val = piece_value_idx(piece_idx);
    if piece_idx < 6 {
        acc_w[sq % acc_w.len()] += val;
    } else {
        acc_b[sq % acc_b.len()] += val;
    }
}

#[inline(always)]
pub fn nnue_sub_piece(
    _nnue: &Nnue,
    acc_w: &mut [i32; 256],
    acc_b: &mut [i32; 256],
    _wk_sq: usize,
    _bk_sq: usize,
    piece_idx: usize,
    sq: usize,
) {
    let val = piece_value_idx(piece_idx);
    if piece_idx < 6 {
        acc_w[sq % acc_w.len()] -= val;
    } else {
        acc_b[sq % acc_b.len()] -= val;
    }
}

#[inline(always)]
fn material_eval(board: &Board) -> i32 {
    let mut score = 0;
    // Piece order matches PieceIndex variants.
    let values = [
        100, 320, 330, 500, 900, 0, // white
        -100, -320, -330, -500, -900, 0, // black
    ];

    for (idx, &bb) in board.pieces.iter().enumerate() {
        let count = bb.count_ones() as i32;
        score += values[idx] * count;
    }

    if board.turn == 0 {
        score
    } else {
        -score
    }
}

#[inline(always)]
fn piece_cp(pt: PieceType) -> i32 {
    match pt {
        PieceType::P => 100,
        PieceType::N => 320,
        PieceType::B => 330,
        PieceType::R => 500,
        PieceType::Q => 900,
        _ => 0,
    }
}

#[inline(always)]
fn piece_value_idx(idx: usize) -> i32 {
    match idx % 6 {
        0 => 100,
        1 => 320,
        2 => 330,
        3 => 500,
        4 => 900,
        _ => 0,
    }
}
