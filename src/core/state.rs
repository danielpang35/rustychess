use crate::core::piece::Piece;
use crate::core::r#move::Move;

/// Compact, allocation-free undo record for `Board::push()` / `Board::pop()`.
///
/// Stores only the previous values that cannot be derived when unmaking a move,
/// plus captured-piece information (including EP capture square).
#[derive(Copy, Clone, Debug)]
pub struct Undo {
    /// The move that was made and will be undone.
    pub mv: Move,

    /// Previous castling rights.
    pub castling_rights: u8,

    /// Previous EP square (64 means none).
    pub ep_square: u8,

    /// Previous zobrist hash.
    pub hash: u64,

    /// Captured piece identity (Piece::None if no capture).
    pub captured_piece: Piece,

    /// Square the captured piece came from (dst for normal captures, pawn square for EP).
    /// 64 means no capture.
    pub captured_sq: u8,
}

impl Undo {
    #[inline(always)]
    pub fn new(mv: Move, castling_rights: u8, ep_square: u8, hash: u64) -> Self {
        Self {
            mv,
            castling_rights,
            ep_square,
            hash,
            captured_piece: Piece::None,
            captured_sq: 64,
        }
    }
}