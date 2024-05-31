pub use std::rc::Rc;
pub use crate::core::r#move::Move;
pub use super::piece::PieceType;
#[derive(Clone, PartialEq)]

pub struct BoardState {    
    pub castling_rights: u8,
    pub ep_square: u8,
    pub capturedpiece: PieceType,
    pub pinned: [u64; 2], //friendly pieces
    pub pinners: [u64; 2], //enemy pieces
    pub attacked: [u64; 2],
    pub prev: Option<Rc<BoardState>>,
    pub prev_move: Move,
}

impl BoardState {
    pub fn new() -> Self {
        Self {    
            castling_rights: 0,
            ep_square: 0,
            capturedpiece: PieceType::NONE,
            pinned: [0; 2], //friendly pieces
            pinners: [0; 2], //enemy pieces
            attacked: [0; 2],
            prev: None,
            prev_move: Move::new(),  
        }
    }
    pub fn setpinned(&mut self, pinned: u64, turn: u8) {
        self.pinned[turn as usize] = pinned;
    }

}

