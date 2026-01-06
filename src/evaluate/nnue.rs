use crate::core::{Board, PieceIndex};
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const MAGIC: &[u8; 4] = b"NNUE";
const VERSION: u32 = 2;
#[derive(Clone)]
pub struct Nnue {
    pub num_feat: usize,
    pub hidden: usize,
    pub h1: usize,
    pub h2: usize,

    pub scale_emb: i32,
    pub scale_fc1: i32,
    pub scale_fc2: i32,
    pub scale_out: i32,
    pub scale_fast_out: i32,

    pub emb: Vec<i16>, // [num_feat * hidden]
    pub b1: Vec<i32>,  // [hidden]

    pub fc1_w: Vec<i16>, // [h1 * (2*hidden)]
    pub fc1_b: Vec<i32>, // [h1]

    pub fc2_w: Vec<i16>, // [h2 * h1]
    pub fc2_b: Vec<i32>, // [h2]

    pub out_w: Vec<i16>, // [h2]
    pub out_b: i32,

    pub fast_out_w: Vec<i16>, // [2 * hidden]
    pub fast_out_b: i32,
}

impl Nnue {
    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;

        let mut off = 0usize;

        let magic = &buf[off..off + 4];
        off += 4;
        if magic != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad NNUE magic",
            ));
        }

        let ver = read_u32(&buf, &mut off);
        if ver != VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad NNUE version",
            ));
        }

        let num_feat = read_u32(&buf, &mut off) as usize;
        let hidden = read_u32(&buf, &mut off) as usize;
        let h1 = read_u32(&buf, &mut off) as usize;
        let h2 = read_u32(&buf, &mut off) as usize;

        if hidden != 256 || h1 != 32 || h2 != 32 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unexpected NNUE dimensions",
            ));
        }

        let scale_emb = read_i32(&buf, &mut off);
        let scale_fc1 = read_i32(&buf, &mut off);
        let scale_fc2 = read_i32(&buf, &mut off);
        let scale_out = read_i32(&buf, &mut off);
        let mut scale_fast_out = scale_out;

        let mut emb = vec![0i16; num_feat * hidden];
        read_i16_slice(&buf, &mut off, &mut emb);

        let mut b1 = vec![0i32; hidden];
        read_i32_slice(&buf, &mut off, &mut b1);

        let mut fc1_w = vec![0i16; h1 * (2 * hidden)];
        read_i16_slice(&buf, &mut off, &mut fc1_w);

        let mut fc1_b = vec![0i32; h1];
        read_i32_slice(&buf, &mut off, &mut fc1_b);

        let mut fc2_w = vec![0i16; h2 * h1];
        read_i16_slice(&buf, &mut off, &mut fc2_w);

        let mut fc2_b = vec![0i32; h2];
        read_i32_slice(&buf, &mut off, &mut fc2_b);

        let mut out_w = vec![0i16; h2];
        read_i16_slice(&buf, &mut off, &mut out_w);

        let out_b = read_i32(&buf, &mut off);

        let mut fast_out_w = Vec::new();
        let mut fast_out_b = 0;

        let needed_fast = 4 + (2 * 2 * hidden) + 4; // scale + weights + bias
        if buf.len() >= off + needed_fast {
            scale_fast_out = read_i32(&buf, &mut off);

            fast_out_w = vec![0i16; 2 * hidden];
            read_i16_slice(&buf, &mut off, &mut fast_out_w);

            fast_out_b = read_i32(&buf, &mut off);
        }

        Ok(Self {
            num_feat,
            hidden,
            h1,
            h2,
            scale_emb,
            scale_fc1,
            scale_fc2,
            scale_out,
            scale_fast_out,
            emb,
            b1,
            fc1_w,
            fc1_b,
            fc2_w,
            fc2_b,
            out_w,
            out_b,
            fast_out_w,
            fast_out_b,
        })
    }

    #[inline(always)]
    fn emb_row(&self, feat: usize) -> &[i16] {
        let start = feat * self.hidden;
        &self.emb[start..start + self.hidden]
    }

    #[inline(always)]
    fn has_fast_head(&self) -> bool {
        self.fast_out_w.len() == 2 * self.hidden
    }

    pub fn eval_fast_cp_like(&self, board: &Board) -> i32 {
        if !self.has_fast_head() {
            return self.eval_cp_like(board);
        }

        debug_assert!(board.nnue_inited);

        let (stm, nstm) = if board.turn == 0 {
            (&board.nnue_acc_w, &board.nnue_acc_b)
        } else {
            (&board.nnue_acc_b, &board.nnue_acc_w)
        };

        let clamp_hi = 127 * self.scale_emb;
        let mut sum: i64 = self.fast_out_b as i64;
        for i in 0..self.hidden {
            let mut x = stm[i];
            if x < 0 {
                x = 0;
            }
            if x > clamp_hi {
                x = clamp_hi;
            }
            sum += (x as i64) * (self.fast_out_w[i] as i64);
        }
        for i in 0..self.hidden {
            let mut x = nstm[i];
            if x < 0 {
                x = 0;
            }
            if x > clamp_hi {
                x = clamp_hi;
            }
            sum += (x as i64) * (self.fast_out_w[self.hidden + i] as i64);
        }

        let denom = (self.scale_emb as i64) * (self.scale_fast_out as i64);

        let num = sum * 1200;

        let cp = if num >= 0 {
            (num + denom / 2) / denom
        } else {
            (num - denom / 2) / denom
        };
        cp as i32
    }
    pub fn eval_cp_like(&self, board: &Board) -> i32 {
        debug_assert!(board.nnue_inited);

        let (stm, nstm) = if board.turn == 0 {
            (&board.nnue_acc_w, &board.nnue_acc_b)
        } else {
            (&board.nnue_acc_b, &board.nnue_acc_w)
        };

        let clamp_hi = 127 * self.scale_emb;

        // FC1 output
        let mut h1 = [0i32; 32];

        for j in 0..32 {
            let mut sum: i64 = self.fc1_b[j] as i64;
            let row = &self.fc1_w[j * 512..(j + 1) * 512];

            for i in 0..256 {
                let mut x = stm[i];
                if x < 0 {
                    x = 0;
                }
                if x > clamp_hi {
                    x = clamp_hi;
                }
                sum += (x as i64) * (row[i] as i64);
            }
            for i in 0..256 {
                let mut x = nstm[i];
                if x < 0 {
                    x = 0;
                }
                if x > clamp_hi {
                    x = clamp_hi;
                }
                sum += (x as i64) * (row[256 + i] as i64);
            }

            let mut v = (sum / self.scale_fc1 as i64) as i32;
            if v < 0 {
                v = 0;
            }
            if v > clamp_hi {
                v = clamp_hi;
            }
            h1[j] = v;
        }

        // FC2
        let mut h2 = [0i32; 32];

        for j in 0..32 {
            let mut sum: i64 = self.fc2_b[j] as i64;
            let row = &self.fc2_w[j * 32..(j + 1) * 32];

            for i in 0..32 {
                sum += (h1[i] as i64) * (row[i] as i64);
            }

            let mut v = (sum / self.scale_fc2 as i64) as i32;
            if v < 0 {
                v = 0;
            }
            if v > clamp_hi {
                v = clamp_hi;
            }
            h2[j] = v;
        }

        // OUT
        let mut sum: i64 = self.out_b as i64;
        for i in 0..32 {
            sum += (h2[i] as i64) * (self.out_w[i] as i64);
        }

        // OUT sum is in scale (scale_emb * scale_out)
        let denom = (self.scale_emb as i64) * (self.scale_out as i64);

        // Convert to CENTIPAWNS with rounding.
        // Multiply by 100 because 1.00 = 1 pawn.
        let num = sum * 1200;

        // round-to-nearest for i64 division (works for negative too)
        let cp = if num >= 0 {
            (num + denom / 2) / denom
        } else {
            (num - denom / 2) / denom
        };
        cp as i32
    }
}

#[inline(always)]
fn read_u32(buf: &[u8], off: &mut usize) -> u32 {
    let v = u32::from_le_bytes(buf[*off..*off + 4].try_into().unwrap());
    *off += 4;
    v
}
#[inline(always)]
fn read_i32(buf: &[u8], off: &mut usize) -> i32 {
    let v = i32::from_le_bytes(buf[*off..*off + 4].try_into().unwrap());
    *off += 4;
    v
}
#[inline(always)]
fn read_i16_slice(buf: &[u8], off: &mut usize, out: &mut [i16]) {
    let n = out.len() * 2;
    let bytes = &buf[*off..*off + n];
    *off += n;
    for (i, chunk) in bytes.chunks_exact(2).enumerate() {
        out[i] = i16::from_le_bytes([chunk[0], chunk[1]]);
    }
}
#[inline(always)]
fn read_i32_slice(buf: &[u8], off: &mut usize, out: &mut [i32]) {
    let n = out.len() * 4;
    let bytes = &buf[*off..*off + n];
    *off += n;
    for (i, chunk) in bytes.chunks_exact(4).enumerate() {
        out[i] = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
}

#[inline(always)]
fn feat_index(king_sq: usize, piece_idx: usize, piece_sq: usize) -> usize {
    ((king_sq * 12 + piece_idx) * 64 + piece_sq) + 1 // +1 for PAD row
}

#[inline(always)]
fn apply_delta(acc: &mut [i32; 256], row: &[i16], sign: i32) {
    // sign is +1 (add) or -1 (remove)
    for i in 0..256 {
        acc[i] += sign * (row[i] as i32);
    }
}
#[inline(always)]
pub fn add_row(acc: &mut [i32; 256], row: &[i16]) {
    for i in 0..256 {
        acc[i] += row[i] as i32;
    }
}

#[inline(always)]
pub fn sub_row(acc: &mut [i32; 256], row: &[i16]) {
    for i in 0..256 {
        acc[i] -= row[i] as i32;
    }
}

#[inline(always)]
pub fn nnue_add_piece(
    nnue: &Nnue,
    acc_w: &mut [i32; 256],
    acc_b: &mut [i32; 256],
    wk_sq: usize,
    bk_sq: usize,
    piece_idx: usize,
    sq: usize,
) {
    let fw = feat_index(wk_sq, piece_idx, sq);
    let fb = feat_index(bk_sq, piece_idx, sq);
    add_row(acc_w, nnue.emb_row(fw));
    add_row(acc_b, nnue.emb_row(fb));
}

#[inline(always)]
pub fn nnue_sub_piece(
    nnue: &Nnue,
    acc_w: &mut [i32; 256],
    acc_b: &mut [i32; 256],
    wk_sq: usize,
    bk_sq: usize,
    piece_idx: usize,
    sq: usize,
) {
    let fw = feat_index(wk_sq, piece_idx, sq);
    let fb = feat_index(bk_sq, piece_idx, sq);
    sub_row(acc_w, nnue.emb_row(fw));
    sub_row(acc_b, nnue.emb_row(fb));
}
#[inline(always)]
fn update_piece_move(nnue: &Nnue, board: &mut Board, piece_idx: usize, from: usize, to: usize) {
    let wk_sq = board.pieces[PieceIndex::K.index()].trailing_zeros() as usize;
    let bk_sq = board.pieces[6 + PieceIndex::K.index()].trailing_zeros() as usize;

    // Update white-king perspective accumulator
    let f_w = feat_index(wk_sq, piece_idx, from);
    let t_w = feat_index(wk_sq, piece_idx, to);
    apply_delta(&mut board.nnue_acc_w, nnue.emb_row(f_w), -1);
    apply_delta(&mut board.nnue_acc_w, nnue.emb_row(t_w), 1);

    // Update black-king perspective accumulator
    let f_b = feat_index(bk_sq, piece_idx, from);
    let t_b = feat_index(bk_sq, piece_idx, to);
    apply_delta(&mut board.nnue_acc_b, nnue.emb_row(f_b), -1);
    apply_delta(&mut board.nnue_acc_b, nnue.emb_row(t_b), 1);
}

#[inline(always)]
pub fn nnue_move_piece(
    nnue: &Nnue,
    acc_w: &mut [i32; 256],
    acc_b: &mut [i32; 256],
    wk_sq: usize,
    bk_sq: usize,
    piece_idx: usize,
    from: usize,
    to: usize,
) {
    let fw_from = feat_index(wk_sq, piece_idx, from);
    let fw_to = feat_index(wk_sq, piece_idx, to);
    let fb_from = feat_index(bk_sq, piece_idx, from);
    let fb_to = feat_index(bk_sq, piece_idx, to);

    let w_from = nnue.emb_row(fw_from);
    let w_to = nnue.emb_row(fw_to);
    let b_from = nnue.emb_row(fb_from);
    let b_to = nnue.emb_row(fb_to);

    for i in 0..256 {
        acc_w[i] += (w_to[i] as i32) - (w_from[i] as i32);
        acc_b[i] += (b_to[i] as i32) - (b_from[i] as i32);
    }
}
