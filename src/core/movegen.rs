use super::*;
use crate::core::constlib;
use crate::core::r#move::*;
pub enum PieceIndex {
  P,
  N,
  B,
  R,
  Q,
  K,
  p,
  n,
  b,
  r,
  q,
  k,
}
impl PieceIndex {
  pub fn index(self)->usize {
    unsafe {self as usize}
  }
}

pub struct MoveGenerator {
    //precomputed attack bitboard for each square of the board
    pub king: [u64; 64],
    pub knight: [u64; 64],
    pub pawnattacks: [[u64; 64];2], //array of length two of array of 64 bitboards
    pub pawnmoves: [[u64; 64];2], 
    pub bishop: [u64; 64],
    pub rook: [u64; 64],
}

impl MoveGenerator {
    pub fn new() -> Self {
      //creates a new movegenerator, initializes precalculated information
        let mut moveg = Self {
            king: [0; 64],
            knight: [0; 64],
            pawnattacks: [[0; 64];2],
            pawnmoves: [[0;64];2],
            bishop: [0; 64],
            rook: [0; 64],
        };
        moveg.init_king();
        moveg.init_knight();
        moveg.init_pawnattacks();
        moveg.init_pawnmoves();
        moveg
    }
  
    pub fn generate(&self, board: &Board)->Vec<Move>
      {
        let mut moves = Vec::new();
        self.generatepawnmoves(board,&mut moves);
        self.generateknightmoves(board,&mut moves);
        self.generatebishopmoves(board,&mut moves);
        self.generaterookmoves(board,&mut moves);
        self.generatequeenmoves(board,&mut moves);
        self.generatekingmoves(board,&mut moves);
        moves
      }
    
    pub fn generatekingmoves(&self,board:&Board,movelist:&mut Vec<Move>) {
      let color = board.turn;
      let mut kbb = if (color == 0) {board.pieces[PieceIndex::K.index()]} else {board.pieces[PieceIndex::k.index()]};
      let them = if color == 0 {1} else {0};
      while kbb != 0 {
        let ind = constlib::poplsb(&mut kbb);
        let mut attacks = self.king[ind as usize];
        //returned attacks allow friendly pieces to be captured.
        attacks &= !board.playerpieces[color as usize];
        constlib::print_bitboard(attacks);
        println!("attacks for square {}",constlib::squaretouci(ind));
        while(attacks != 0){
          let dst = constlib::poplsb(&mut attacks);
          if (1 << dst) & board.playerpieces[them as usize] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
    }
    pub fn generatequeenmoves(&self, board:&Board, movelist:&mut Vec<Move>) {
      let color = board.turn;
      let mut qbb = if (color == 0) {board.pieces[PieceIndex::Q.index()]} else {board.pieces[PieceIndex::q.index()]};
      let them = if color == 0 {1} else {0};
      while qbb != 0 {
        let ind = constlib::poplsb(&mut qbb);
        let mut attacks = self.compute_rook(ind as i8,board.occupied) | self.compute_bishop(ind as i8, board.occupied);
        //returned attacks allow friendly pieces to be captured.
        attacks &= !board.playerpieces[color as usize];
        while(attacks != 0){
          let dst = constlib::poplsb(&mut attacks);
          if (1 << dst) & board.playerpieces[them as usize] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
    }
    pub fn generaterookmoves(&self, board:&Board, movelist:&mut Vec<Move>) {
      let color = board.turn;
      let mut rbb = if (color == 0) {board.pieces[PieceIndex::R.index()]} else {board.pieces[PieceIndex::r.index()]};
      let them = if color == 0 {1} else {0};
      while rbb != 0 {
        let ind = constlib::poplsb(&mut rbb);
        let mut attacks = self.compute_rook(ind as i8,board.occupied);
        //returned attacks allow friendly pieces to be captured.
        attacks &= !board.playerpieces[color as usize];
        while(attacks != 0){
          let dst = constlib::poplsb(&mut attacks);
          if (1 << dst) & board.playerpieces[them] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
    }
    pub fn generatebishopmoves(&self, board:&Board, movelist:&mut Vec<Move>) {
      let color = board.turn;
      let mut bbb = if (color == 0) {board.pieces[PieceIndex::B.index()]} else {board.pieces[PieceIndex::b.index()]};
      let them = if color == 0 {1} else {0};
      while bbb != 0 {
        let ind = constlib::poplsb(&mut bbb);
        let mut attacks = self.compute_bishop(ind as i8,board.occupied);
        //returned attacks allow friendly pieces to be captured.
        attacks &= !board.playerpieces[color as usize];
        while(attacks != 0){
          let dst = constlib::poplsb(&mut attacks);
          if (1 << dst) & board.playerpieces[them as usize] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
    }
    pub fn generateknightmoves(&self,board: &Board, movelist: &mut Vec<Move>) {
      let color = board.turn;
      let enemy = if color == 0 {1} else {0};
      let mut kbb = if color == 0 {board.pieces[PieceIndex::N.index()]} else {board.pieces[PieceIndex::n.index()]};
      while kbb != 0 {
        let ind = constlib::poplsb(&mut kbb);
        let mut attacks = self.knight[ind as usize];
        attacks &= !board.playerpieces[color as usize];
        while(attacks != 0){
          let dst = constlib::poplsb(&mut attacks);
          if (1 << dst) & board.playerpieces[enemy as usize] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
      
    }
    pub fn generatepawnmoves(&self,board: &Board, movelist: &mut Vec<Move>) {
      //looks up pawn attacks and moves, finds out if they're possible
      //creates a bitmove for each possible move
      //mutates given array
      let color = board.turn;
      let mut pbb = if color == 0 {board.pieces[PieceIndex::P.index()]} else {board.pieces[PieceIndex::p.index()]};

      println!("making pawn moves for {}",if color == 0 {"WHITE"} else {"BLACK"});
      while pbb != 0 {
        let ind = pbb.trailing_zeros();
        //get pawn moves and attacks at square
        
        //moves are legal if square is not occupied
        let mut moves = self.pawnmoves[color as usize][ind as usize];
        moves &= !board.occupied;
        let one_push = moves;
        while(moves != 0)
        {
          //loops through pawnpushes
          let dst = constlib::poplsb(&mut moves) as u8;
          if(dst /8 == 7 || dst /8 == 0)
          {
            //this is a promotion
            movelist.push(Move::makeProm(ind as u8, dst))
          } else {
            //this is a quiet pawnpush
            movelist.push(Move::makeQuiet(ind as u8, dst))
          }
        }
        
        //move one up if board spot is not occupied^
        //for each legal one move pawn push, check if can push another.
        let shift = if color == 0 {constlib::north} else {constlib::south};
        let rankmask = if color == 0 {constlib::rankmasks[2]} else {constlib::rankmasks[5]};
        let mut push_two_moves = constlib::genShift(shift, (one_push&rankmask));
        push_two_moves &= !board.occupied;
        //now create a vec of double pawn pushes:
        while(push_two_moves != 0)
        {
          let dst = constlib::poplsb(&mut push_two_moves) as u8;
          let doublepushbitm = Move::makeDBPawnPush(ind as u8, dst);
          movelist.push(doublepushbitm);
        }
        let mut attacks = self.pawnattacks[color as usize][ind as usize];
        //generate legal moves
        //attacks are legal if enemy occupies square
        //playerpieces is list of piece positions by color
        let enemypieces = if color == 0 {board.playerpieces[1]} else {board.playerpieces[0]};
        attacks &= enemypieces;
        if(self.pawnattacks[color as usize][ind as usize] & (1<<board.ep_square) != 0)
        {
          movelist.push(Move::makeEP(ind as u8, board.ep_square));
        }
        while(attacks != 0)
        {
          let dst = constlib::poplsb(&mut attacks) as u8;
          if(dst / 8 == 7 || dst / 8 == 0)
          {
            movelist.push(Move::makePromCap(ind as u8, dst))
          } else {movelist.push(Move::makeCapture(ind as u8, dst))
          }
        }
        //create capture bitmoves
        //set last bit of pawn bb to 0
        pbb = pbb & (pbb-1);
        //save legalmoves for this square
        }
      }

    pub fn init_pawnmoves(&mut self) {
      for i in 8..56 {
        let mut wmoves = 0;
        let mut bmoves = 0;
        let sq = 1 << i;
        wmoves |= constlib::genShift(constlib::north, sq);
        bmoves |= constlib::genShift(constlib::south, sq);
        
        
        self.pawnmoves[0][i] = wmoves;
        self.pawnmoves[1][i] = bmoves;

      }
    }
    pub fn init_pawnattacks(&mut self) { 
      for i in 8..56 {
        //compute pawn attacks for white and black for each square
        
        let mut wattacks = 0;
        let mut battacks = 0;
        let pawn = 1<<i;
        let ne = constlib::genShift(constlib::northeast, pawn) & constlib::notAFile;
        let nw = constlib::genShift(constlib::northwest, pawn) & constlib::notHFile;
        let se = constlib::genShift(constlib::southeast, pawn) & constlib::notAFile;
        let sw = constlib::genShift(constlib::southwest, pawn) & constlib::notHFile;
        wattacks |= ne | nw;
        battacks |= se | sw;
        self.pawnattacks[0][i] = wattacks;
        self.pawnattacks[1][i] = battacks;
        //println!("wattacks mask:{}, at ind {}",wattacks,i);
        //constlib::print_bitboard(wattacks);
      }
    }
    pub fn init_king(&mut self) {
        for i in 0..64 {
            let mut attacks = 0;
            let mut kingboard = 1 << i;
            //get left and right of king
            attacks |= constlib::dirShift(constlib::west, kingboard)
                | constlib::dirShift(constlib::east, kingboard);
            //move left, center, and right of king up and down to get attacks
            kingboard |= attacks;
            attacks |= constlib::dirShift(constlib::north, kingboard)
                | constlib::dirShift(constlib::south, kingboard);
            if(i%8==7)
            {
              attacks &= constlib::notAFile;
            } else if(i % 8 ==0) {
              attacks &= constlib::notHFile;
            }
            self.king[i] = attacks;}
    }
    pub fn init_knight(&mut self)
      {
        for i in 0..64 {          
          let mut attacks = 0;
          let knightpos = 1 << i;
          let left = constlib::dirShift(constlib::east,knightpos);
          let right = constlib::dirShift(constlib::west,knightpos);
          attacks |= (left | right)<< 16;//left and right by one, up by two
          attacks |= (left | right) >> 16;
          let east = constlib::dirShift(constlib::east, left);
          let west = constlib::dirShift(constlib::west, right);
          attacks |= (east | west) << 8;
          attacks |= (east | west) >> 8;
          if(i % 8 >= 6)
          {
            //on the g or h file
            attacks &= constlib::notAFile;
            attacks &=!0x0202020202020202;
          } else if(i % 8 <= 1)
          {
            //on the a or bfile
            attacks &= constlib::notHFile;
            attacks &= !0x4040404040404040;
          }
          self.knight[i] = attacks;
        }
      }
    pub fn compute_bishop(&self,sq:i8,blockers:u64) -> u64{
      //index 0-3->diagonals
      //index 4-7->orthogonals
      
      let mut attacks = 0;
      //current square
      let mut currpos: i8;
      for diag in 0..4{
        //start pos
        currpos = sq;
        loop {
          //get diagonal direction.
          match diag {
            //for up right, check if wrapped around to a file. if so, break
            0=> {currpos += constlib::northeast;
                if currpos >= 64 || currpos % 8 <= 0 {break;}},
            //for up left, check if wrapped around to h file. if so break
            1=> {currpos += constlib::northwest;
              if currpos >= 64 || currpos % 8 >= 7 {break;}},
            2=> {currpos += constlib::southwest;
              if currpos < 0 || currpos % 8 >= 7 {break;}},
            3=> {currpos += constlib::southeast;
              if currpos < 0 || currpos % 8 <=0 {break;}},
            _ => panic!(),
          }
          attacks |= constlib::genShift(currpos, 1 as u64);
          if constlib::genShift(currpos, 1 as u64) & blockers != 0 {
            break;
          }
          //set the current position as a potential attack
          
          
        }
      }
      return attacks
    }

    //UNCOMMENT THIS IF YOU WANT PRECOMPUTED BISHOP TABLES, HOWEVER THIS SUCKS.
    // pub fn init_bishop(&mut self) {
    //     //index 0-3->diagonals
    //     //index 4-7->orthogonals
    //     for i in 0..64 {
    //       let mut attacks = 0;
    //       //current square
    //       let pos = i;
    //       let mut currpos: i8;
    //       for diag in 0..4{
    //         //start pos
    //         currpos = pos;
    //         loop {
    //           //get diagonal direction.
    //           match diag {
    //             //for up right, check if wrapped around to a file. if so, break
    //             0=> {currpos += constlib::northeast;
    //                 if currpos >= 64 || currpos % 8 <= 0 {break;}},
    //             //for up left, check if wrapped around to h file. if so break
    //             1=> {currpos += constlib::northwest;
    //               if currpos >= 64 || currpos % 8 >= 7 {break;}},
    //             2=> {currpos += constlib::southwest;
    //               if currpos < 0 || currpos % 8 >= 7 {break;}},
    //             3=> {currpos += constlib::southeast;
    //               if currpos < 0 || currpos % 8 <=0 {break;}},
    //             _ => panic!(),
    //           }
              
    //           //set the current position as a potential attack
    //           attacks |= constlib::genShift(currpos, 1 as u64);
              
    //         }
    //       }
    //       self.bishop[i as usize] = attacks;
    //     }
    // }
    pub fn compute_rook(&self, sq:i8, blockers:u64)->u64{
      let mut attacks = 0;
      let mut currpos: i8;
      for ortho in 0..4 {
        currpos = sq;
        loop {
          match ortho {
            //for up, check if greater than 64. if so, break
            0=> {currpos += constlib::north;
              if currpos >= 64 {break;}},
            //for left, check if wrapped around to h file. if so break
            1=> {currpos += constlib::west;
              if currpos < 0 || currpos % 8 >= 7 {break;}},
            2=> {currpos += constlib::south;
              if currpos < 0 {break;}},
            3=> {currpos += constlib::east;
              if currpos % 8 <=0 {break;}},
            _ => panic!(),
          }
          attacks |= constlib::genShift(currpos, 1 as u64);
          if constlib::genShift(currpos, 1 as u64) & blockers != 0 {break;}
        }
      }
      attacks
        
    }
    // pub fn init_rook(&mut self){
    //   for i in 0..64 {
    //     let mut attacks = 0;
    //     let pos = i;
    //     let mut currpos: i8;
    //     for ortho in 0..4 {
    //       currpos = pos;
    //       loop {
    //         match ortho {
    //           //for up, check if greater than 64. if so, break
    //           0=> {currpos += constlib::north;
    //             if currpos >= 64 {break;}},
    //           //for left, check if wrapped around to h file. if so break
    //           1=> {currpos += constlib::west;
    //             if currpos < 0 || currpos % 8 >= 7 {break;}},
    //           2=> {currpos += constlib::south;
    //             if currpos < 0 {break;}},
    //           3=> {currpos += constlib::east;
    //             if currpos % 8 <=0 {break;}},
    //           _ => panic!(),
    //         }
    //         attacks |= constlib::genShift(currpos, 1 as u64);
    //       }
    //     }
    //     self.rook[i as usize] = attacks;
        
    //   }
    // }
    

    
}
