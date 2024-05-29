//a move is represented as a u16.
//first 6 bits are the from square, next 6 bits are to square
//the next 4 bits are flags:
//the next two bits represent promotion piece (N-0, Q-4)
//! the next two bits are a special move flag: promotion (1), en passant (2), castling (3)
use crate::core::piece::PieceType;
use crate::core::constlib;

//CODES:
// 0000  ===> Quiet move
// 0001  ===> Double Pawn Push
// 0010  ===> King Castle
// 0011  ===> Queen Castle
// 0100  ===> Capture
// 0101  ===> EP Capture
// 0110  ===>
// 0111  ===>
// 1000  ===> Knight Promotion
// 1001  ===> Bishop Promo
// 1010  ===> Rook   Promo
// 1011  ===> Queen  Capture  Promo
// 1100  ===> Knight Capture  Promotion
// 1101  ===> Bishop Capture  Promo
// 1110  ===> Rook   Capture  Promo
// 1111  ===> Queen  Capture  Promo

const SRC_MASK: u16 = 0b0000_000000_111111;
const DST_MASK: u16 = 0b0000_111111_000000;
const FROM_TO_MASK: u16 = 0b0000_111111_111111;
const PR_MASK: u16 = 0b1000_000000_000000; //encodes a promotion
const CP_MASK: u16 = 0b0100_000000_000000; //encodes a capture
const FLAG_MASK: u16 = 0b1111_000000_000000;
const SP_MASK: u16 = 0b0011_000000_000000; //encodes which piece to promote to. If not promotion, then this encodes castling or ep

#[derive(Copy, Clone)]
pub struct Move {
    data: u16,
}
impl Move {
  pub const FLAG_QUIET: u16 = 0b0000;
  pub const FLAG_DOUBLE_PAWN: u16 = 0b0001;
  pub const FLAG_KING_CASTLE: u16 = 0b0010;
  pub const FLAG_QUEEN_CASTLE: u16 = 0b0011;
  pub const FLAG_CAPTURE: u16 = 0b0100;
  pub const FLAG_EP: u16 = 0b0101;
  pub const ILLEGAL_FLAG_1: u16 = 0b0110;
  pub const ILLEGAL_FLAG_2: u16 = 0b0111;
  pub const FLAG_PROMO_N: u16 = 0b1000;
  pub const FLAG_PROMO_B: u16 = 0b1001;
  pub const FLAG_PROMO_R: u16 = 0b1010;
  pub const FLAG_PROMO_Q: u16 = 0b1011;
  pub const FLAG_PROMO_CAP_N: u16 = 0b1100;
  pub const FLAG_PROMO_CAP_B: u16 = 0b1101;
  pub const FLAG_PROMO_CAP_R: u16 = 0b1110;
  pub const FLAG_PROMO_CAP_Q: u16 = 0b1111;
  
    pub fn new() -> Self {
        Self { data: 0, }
    }
    pub fn print(self) {
      let src = constlib::squaretouci(self.getSrc());
      let dst = constlib::squaretouci(self.getDst());
      println!("Move:\n{}{}",src, dst)
    }
    pub fn movemasktoBitMoves(src: u8, movemask: &mut u64)-> Vec<Move>
      {
        let mut vec = Vec::new();
        while *movemask != 0 {
          let dst = constlib::poplsb(movemask) as u8;
          let bitm = Move::make(src,dst, 0);
          vec.push(bitm);
        }
        return vec;
      }
    pub fn is_null(self) -> bool {
      self.data == 0
    }
    pub fn make(src: u8, dst:u8, flag:u16) -> Move
    //make a bitmove given src, destination and flag
    { 
      Move {
        data: (src as u16) | ((dst as u16) << 6) | (flag << 12)
      }
    }
    pub fn makeQuiet(src:u8, dst:u8) -> Move{
      Move::make(src, dst, 0)
    }
    pub fn makeDBPawnPush(src:u8, dst:u8) -> Move{
      Move::make(src, dst, Move::FLAG_DOUBLE_PAWN)
    }
    pub fn makeCapture(src:u8,dst:u8) -> Move {
      Move::make(src,dst, Move::FLAG_CAPTURE)
    }
    pub fn makePromCap(src:u8, dst:u8, piece:PieceType) -> Move {
      let flag = match piece {
        PieceType::N => Move::FLAG_PROMO_CAP_N,
        PieceType::B => Move::FLAG_PROMO_CAP_B,
        PieceType::R => Move::FLAG_PROMO_CAP_R,
        PieceType::Q => Move::FLAG_PROMO_CAP_Q,
        _ => panic!("Invalid promotion type")
      };
      Move::make(src, dst,flag)
    }
    pub fn makeProm(src:u8, dst:u8, piece: PieceType) -> Move {
      let flag = match piece {
        PieceType::N => Move::FLAG_PROMO_N,
        PieceType::B => Move::FLAG_PROMO_B,
        PieceType::R => Move::FLAG_PROMO_R,
        PieceType::Q => Move::FLAG_PROMO_Q,
        _ => panic!("Invalid promotion type")
      };
      Move::make(src, dst,flag)
    }
    pub fn makeEP(src:u8, dst:u8) -> Move {
      Move::make(src,dst,Move::FLAG_EP)
    }
    pub fn makeKingCastle(src:u8, dst:u8) -> Move {
      Move::make(src, dst, Move::FLAG_KING_CASTLE)
    }
    pub fn makeQueenCastle(src:u8, dst:u8) -> Move {
      Move::make(src, dst, Move::FLAG_QUEEN_CASTLE)
    }
    pub fn data(self) -> u16 {
      return self.data;
    }
    pub fn flag(self) -> u16 {
      //return flag bits of self
      return (self.data & FLAG_MASK) >> 12
    }
    pub fn getSrc(self) ->u8{
        return (self.data & SRC_MASK) as u8;
    }
    pub fn getDst(self) ->u8 {
        return ((self.data & DST_MASK) >> 6) as u8;
    }
    pub fn isprom(self) ->bool{
        return (self.data & PR_MASK) != 0;
    }
    pub fn iscapture(self) ->bool{
      return (self.data & CP_MASK) != 0;
    }
    pub fn iscastle(self)->bool{

      (self.data >> 13 ) == 1
    }
    pub fn iskingcastle(self)->bool{
      self.flag() == Move::FLAG_KING_CASTLE
    }
    pub fn isqueencastle(self)->bool{
      self.flag() == Move::FLAG_QUEEN_CASTLE
  }
    pub fn isep(self)->bool {
      self.flag() == Move::FLAG_EP
    }
    pub fn isdoublepawn(self)->bool {
      self.flag() == Move::FLAG_DOUBLE_PAWN
    }
    pub fn isquiet(self)->bool {
      self.flag() == 0
    }
    pub fn prompiece(&self) -> PieceType {
        let bits = (self.flag() & 0b0011);
        match bits {
            0 => PieceType::N,
            1 => PieceType::B,
            2 => PieceType::R,
            3 | _ => PieceType::Q,
        }
    }
}

