pub struct Move;

use crate::core::Board as Board;

use super::piece::Piece as Piece;

//a struct board bookkeeping
pub struct Position 
{
  //state of board
  //keeps info about castling,turn,pins,attacks
  turn: bool,
  board: [u8;64],
  
}

impl Position {
    pub fn new() -> Self {
      Self{board: [0;64],
          turn: true}
    }
  
    pub fn parseFen(&mut self, fen: String) {
      let mut rank: usize = 8; let mut file: usize = 0;
      
      for c in fen.chars() {
        if c.is_numeric()
        {
           //skip this number of squares
          file += c.to_digit(10).unwrap() as usize;
        } else {
          self.put_piece(c, rank,file)
        }
      }
    }
    pub fn put_piece(&self, piece: char, rank: usize, file: usize)
    {
      // let color = piece.isuppercase()
      // self.board[rank * 8+ file] = 
    }
    fn push(&self){
        //TODO: push move to move stack
    }

    fn pop(&self){
        //TODO: pop move from move stack
    }

    fn gen_moves(&self)
    {
      
    }
}