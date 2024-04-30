use super::*;
pub struct Board {
    occupied: u64,
    pawnpieces: u64,
    knightpieces: u64,
    bishoppieces: u64,
    rookpieces: u64,
    queenpieces: u64,
    kingpieces: u64,
}
impl Board {
    //constructor
    pub fn new() -> Self {
        Self {occupied:7,
             pawnpieces:0,
             knightpieces: 0,
             bishoppieces: 0,
             rookpieces: 0,
             queenpieces: 0,
             kingpieces: 0,}
    }
    
    pub fn piece_exists_at(&self, rank: usize, file: usize) -> bool {
        //given rank and file
        let result = self.occupied >> (rank * 8 + file);
        return if result & 1 == 1 {true}else{false}
    }


    
  pub fn toStr(&self, mut input: String) -> String
  {
      let mut s = String::from("+---+---+---+---+---+---+---+---+");
      for r in (0..=7).rev()
      {
        let mut row = String::from("\n|");
          for f in 0..=7
          { 
            let mut sq = if self.piece_exists_at(r,f){"  1"}else{" "};
            row.push_str(sq);
          }
          s.push_str(&row)


      }
      s.push_str( "\n  a   b   c   d   e   f   g   h\n");
      input.push_str(s.as_str());
      return input;
  }
}
