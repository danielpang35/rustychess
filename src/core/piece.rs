use std::mem;
enum PieceType {
    NONE = 0,
    P,
    N,
    B,
    R,
    Q,
    K,
}
impl PieceType {
    pub fn get_piece_value(self) -> u8 {
        match self {
            PieceType::P => 1,
            PieceType::N | PieceType::B => 3,
            PieceType::R => 5,
            PieceType::Q => 9,
            PieceType::K => 0,
            PieceType::NONE => 0,
        }
    }
    pub fn get_piece_type(self) -> char {
        match self {
            PieceType::P => 'p',
            PieceType::N => 'n',
            PieceType::B => 'b',
            PieceType::R => 'r',
            PieceType::Q => 'q',
            PieceType::K => 'k',
            _ => panic!(),
        }
    }
}

pub enum Piece {
    None = 0b0000,
    WP = 0b0001,
    WN = 0b0010,
    WB = 0b0011,
    WR = 0b0100,
    WQ = 0b0101,
    WK = 0b0110,

    BP = 0b1001,
    BN = 0b1010,
    BB = 0b1011,
    BR = 0b1100,
    BQ = 0b1101,
    BK = 0b1110,
}

impl Piece {
    pub fn get_color(self) -> bool {
      return unsafe { mem::transmute((self as u8 >> 3) & 0b1) };
      
        if (self as u8) << 3 & 1 == 1 {
            return true;
        } else {
            return false;
        }
    }
  
    pub fn get_piece_type(self) -> u8 {
      return unsafe {mem::transmute((self as u8 ) * 0b0111)};
    }
}


