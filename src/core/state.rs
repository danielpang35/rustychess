pub use std::sync::Arc;
pub use crate::core::r#move::Move;
pub use super::piece::PieceType;
#[derive(Clone, PartialEq)]

pub struct BoardState {    
    pub castling_rights: u8,
    pub ep_square: u8,
    pub capturedpiece: PieceType,
    pub pinned: u64, //friendly pieces
    pub pinners: u64, //enemy pieces
    pub attacked: [u64; 2], //  attacked[board.turn] == squares attacked by ENEMY
    pub prev: Option<Arc<BoardState>>,
    pub prev_move: Move,
}

impl BoardState {
    pub fn new() -> Self {
        Self {    
            castling_rights: 0,
            ep_square: 64,
            capturedpiece: PieceType::NONE,
            pinned: 0, //friendly pieces
            pinners: 0, //enemy pieces
            attacked: [0; 2],
            prev: None,
            prev_move: Move::new(),  
        }
    }
    pub fn getpinned(&self, board: &crate::core::Board) -> (u64,u64) {
        (self.pinned, self.pinners)
    }

}

