use std::f32::consts::E;

use super::*;
use crate::core::constlib;
use crate::core::r#move::*;
use crate::core::castling::*;
use crate::core::piece::PieceIndex;

pub struct MoveGenerator {
    //precomputed attack bitboard for each square of the board
    pub king: [u64; 64],
    pub knight: [u64; 64],
    pub pawnattacks: [[u64; 64];2], //array of length two of array of 64 bitboards
    pub pawnmoves: [[u64; 64];2], 
    pub bishop: [u64; 64],
    pub rook: [u64; 64],
    pub line_between:[[u64;64];64],
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
            line_between:[[0;64];64],
        };
        moveg.init_king();
        moveg.init_knight();
        moveg.init_pawnattacks();
        moveg.init_pawnmoves();
        moveg.gen_line_bbs();
        moveg
    }
    #[inline(always)]
    pub fn generate(& self, board: &mut Board)->Vec<Move>
      {
        let mut moves = Vec::new();
        
        let us = board.turn;
        let them = us ^ 1;
        // -------------- COMPUTE PINNED AND PINNERS -----------
        let (pinned, pinners) = self.getpinned(board);
        board.pinned = pinned;
        board.pinners = pinners;

        



        let mut checkers = self.getcheckers(board, board.occupied);
        let pininfo = (board.pinned,board.pinners);
        
        
        let kingidx = if board.turn == 0 {
                      PieceIndex::K.index()
                            } else {
                      PieceIndex::k.index()
            };
        let mut king_bb = board.pieces[kingidx];
        let king_sq = constlib::poplsb(&mut king_bb) as i8;
          
        let king_bb = board.pieces[kingidx];

        let occ_wo_king = board.occupied & !king_bb;
        let enemy_attacks_wo_king = self.makeattackedmask(board, them, occ_wo_king);
        if checkers != 0
        {
          //if in check, generate evasions.
          self.genevasions(board, &mut moves,&mut checkers,pininfo,king_sq, enemy_attacks_wo_king);
          return moves;
        }
        self.generatepawnmoves(board,&mut moves,false,(0,0),pininfo,king_sq);
        self.generateknightmoves(board,&mut moves,false,(0,0),pininfo);
        self.generatebishopmoves(board,&mut moves,false,(0,0),pininfo,king_sq);
        self.generaterookmoves(board,&mut moves,false,(0,0),pininfo,king_sq);
        self.generatequeenmoves(board,&mut moves,false,(0,0),pininfo,king_sq);


        

        self.generatekingmoves(board,&mut moves,king_sq, enemy_attacks_wo_king);
        moves
      }
    #[inline(always)]
    pub fn generatekingmoves(&self,board:&Board,movelist:&mut Vec<Move>, kingsq:i8, enemy_attacks: u64) {
      let color = board.turn;
      let them = if color == 0 {1} else {0};
      
      let ind = kingsq;
      let mut attacks = self.king[ind as usize];
      //access cached attacked array
      let kingdanger = enemy_attacks;
      // if color == 1 {
      //   constlib::print_bitboard(kingdanger);
      // }
      attacks &= !kingdanger;
      attacks &= !board.playerpieces[color as usize];
      while attacks != 0 {
        let dst = constlib::poplsb(&mut attacks);

        if (1u64 << dst) & board.playerpieces[them as usize] != 0 {
          movelist.push(Move::makeCapture(ind as u8,dst));
        } else {movelist.push(Move::makeQuiet(ind as u8,dst));}
      }
      
      self.generatecastling(board, movelist, enemy_attacks);
    }
    #[inline(always)]
    pub fn generatequeenmoves(&self, board:&Board, movelist:&mut Vec<Move>,evasions:bool,target:(u64,u64),pininfo:(u64,u64), kingsq:i8) {
      let color = board.turn;
      let mut qbb = if (color == 0) {board.pieces[PieceIndex::Q.index()]} else {board.pieces[PieceIndex::q.index()]};
      let them = if color == 0 {1} else {0};
      let pinned = pininfo.0;
      let pinners = pininfo.1;
      while qbb != 0 {
        let ind = constlib::poplsb(&mut qbb);
        let mut attacks = constlib::compute_rook(ind as i8,board.occupied) | constlib::compute_bishop(ind as i8, board.occupied);
        //returned attacks allow friendly pieces to be captured.
        attacks &= !board.playerpieces[color as usize];
        if pinned & (1u64 << ind) != 0 {
          //if queen is pinned, get the line between the king and the queen as the queen's only moves.
          attacks &= self.line_between[ind as usize][kingsq as usize];
          
        }
        //if evasion is turned on, then only generate attacks which block or take opposing checker
        if evasions {
          attacks &= target.0 | target.1;
        }
        while attacks != 0 {
          let dst = constlib::poplsb(&mut attacks);
          if (1u64 << dst) & board.playerpieces[them as usize] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
    }
    #[inline(always)]
    pub fn generaterookmoves(&self, board:&Board, movelist:&mut Vec<Move>,evasions:bool,target:(u64,u64),pininfo:(u64,u64), kingsq:i8) {
      let color = board.turn;
      let mut rbb = if (color == 0) {board.pieces[PieceIndex::R.index()]} else {board.pieces[PieceIndex::r.index()]};
      let them = if color == 0 {1} else {0};
      let pinned = pininfo.0;
      let pinners = pininfo.1;

      while rbb != 0 {
        let ind = constlib::poplsb(&mut rbb);
        let mut attacks = constlib::compute_rook(ind as i8,board.occupied);
        // returned attacks allow friendly pieces to be captured.
        attacks &= !board.playerpieces[color as usize];

        if pinned & (1u64 << ind) != 0 {
          attacks &= self.line_between[ind as usize][kingsq as usize];
        }
        //if evasion is turned on, then only generate attacks which block or take opposing checker
        if evasions {
          attacks &= target.0 | target.1;
        }
        while attacks != 0 {
          let dst = constlib::poplsb(&mut attacks);
          if (1u64 << dst) & board.playerpieces[them] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
    }
    #[inline(always)]
    pub fn generatebishopmoves(&self, board:&Board, movelist:&mut Vec<Move>,evasions:bool,target:(u64,u64),pininfo:(u64,u64), kingsq:i8) {
      let color = board.turn;
      let mut bbb = if (color == 0) {board.pieces[PieceIndex::B.index()]} else {board.pieces[PieceIndex::b.index()]};
      let them = if color == 0 {1} else {0};
      let pinned = pininfo.0;
      let pinners = pininfo.1;
      
      while bbb != 0 {
        let ind = constlib::poplsb(&mut bbb);
        let mut attacks = constlib::compute_bishop(ind as i8,board.occupied);
        // returned attacks allow friendly pieces to be captured.
        attacks &= !board.playerpieces[color as usize];
        if pinned & (1u64 << ind) != 0 {
          attacks &= self.line_between[ind as usize][kingsq as usize];
        }
        //if evasion is turned on, then only generate attacks which block or take opposing checker
        if evasions {
          attacks &= target.0 | target.1;
        }
        while attacks != 0 {
          let dst = constlib::poplsb(&mut attacks);
          if (1u64 << dst) & board.playerpieces[them as usize] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
    }
    #[inline(always)]
    pub fn generateknightmoves(&self,board: &Board, movelist: &mut Vec<Move>,evasions:bool,target:(u64,u64),pininfo:(u64,u64)) {
      let color = board.turn;
      let enemy = if color == 0 {1} else {0};
      let mut kbb = if color == 0 {board.pieces[PieceIndex::N.index()]} else {board.pieces[PieceIndex::n.index()]};
      let pinned = pininfo.0;      
      while kbb != 0 {
        let ind = constlib::poplsb(&mut kbb);
        if pinned & (1u64 << ind) != 0 {
          //knight is pinned, just return
          continue;
        }
        let mut attacks = self.knight[ind as usize];

        attacks &= !board.playerpieces[color as usize];
        //if evasion is turned on, then only generate attacks which block or take opposing checker
        if evasions {
          attacks &= target.0 | target.1;
        }
        while attacks != 0 {
          let dst = constlib::poplsb(&mut attacks);
          if (1u64 << dst) & board.playerpieces[enemy as usize] != 0 {
            movelist.push(Move::makeCapture(ind,dst));
          } else {movelist.push(Move::makeQuiet(ind,dst));}
        }
      }
      
    }
    #[inline(always)]
    pub fn generatepawnmoves(&self,board: &Board, movelist: &mut Vec<Move>,evasions:bool,target:(u64,u64),pininfo:(u64,u64),kingsq:i8) {
      //looks up pawn attacks and moves, finds out if they're possible
      //creates a bitmove for each possible move
      //mutates given array
      let color = board.turn;
      // constlib::print_bitboard(board.getpinned()[color as usize]);
      // board.print();
      let mut pbb = if color == 0 {board.pieces[PieceIndex::P.index()]} else {board.pieces[PieceIndex::p.index()]};
      //println!("making pawn moves for {}",if color == 0 {"WHITE"} else {"BLACK"});
      let pinners = pininfo.1;
      let pinned = pininfo.0;
      
      while pbb != 0 {
        let ind = constlib::poplsb(&mut pbb);
        //get pawn moves and attacks at square
        
        //moves are legal if square is not occupied
        // base one-square push legality
        let mut one_push = self.pawnmoves[color as usize][ind as usize];
        one_push &= !board.occupied;
        
        if pinned & (1u64 << ind) != 0 {
          // For pinned pawns, filter pushes to the ray between the king and the pawn
            let pin_line = self.line_between[ind as usize][kingsq as usize];
            one_push &= pin_line;
        }

        // single pushes (final square must block check)
        let mut single_pushes = one_push;
        if evasions {
            single_pushes &= target.1;
        }

        let mut moves = single_pushes;

        while moves != 0
        {
          //loops through pawnpushes
          let dst = constlib::poplsb(&mut moves) as u8;
          if(dst /8 == 7 || dst /8 == 0)
          {
            //this is a promotion
            movelist.push(Move::makeProm(ind as u8, dst, PieceType::B));
            movelist.push(Move::makeProm(ind as u8, dst, PieceType::N));
            movelist.push(Move::makeProm(ind as u8, dst, PieceType::R));
            movelist.push(Move::makeProm(ind as u8, dst, PieceType::Q));
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
        if evasions {
          push_two_moves &= target.1;
        }
        //now create a vec of double pawn pushes:
        while push_two_moves != 0 
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
        if pinned & (1u64 << ind) != 0 {
          
          // For pinned pawns, filter attacks to the intersection of pawn attacks and pin lines
          let pin_line = self.line_between[ind as usize][kingsq as usize];
          attacks &= pin_line;
        }
        if evasions {
          attacks &= target.0 | target.1;
        }
        
       //generate ep moves
        let ep = board.getep();

        // En-passant is a special case for legality because removing the pawn from its
        // original file can expose a discovered check on our own king (classic EP pin).
        // Therefore we must apply *pin constraints* and a *self-check test*.
        if (ep < 64)
            && (self.pawnattacks[color as usize][ind as usize] & (1u64 << ep) != 0)
            && (constlib::get_rank(ind as u8) != 2)
            && (constlib::get_rank(ind as u8) != 7)
        {
          // If this pawn is pinned, EP is only legal if the destination lies on the pin line.
          if pinned & (1u64 << ind) != 0 {
            let pin_line = self.line_between[ind as usize][kingsq as usize];
            if (pin_line & (1u64 << ep)) == 0 {
              // EP would move off the pin line -> illegal (would expose self-check).
              continue;
            }
          }

          if evasions {
            let offset = if color == 0 {-8} else {8};
            if (1u64 << ep) & target.1 != 0{
              movelist.push(Move::makeEP(ind as u8, ep));
            } else if (1u64 << (ep as i8 + offset)) & target.0 != 0{
              movelist.push(Move::makeEP(ind as u8, ep));
            }
          } else {
            let enemypawn = if color == 0 {ep - 8} else {ep + 8};
            // Simulate EP occupancy for check detection:
            // - remove the moving pawn from its source square
            // - remove the captured pawn behind the EP square
            // - add the pawn on the EP destination square
            let blockers = (board.occupied & !((1u64 << ind) | (1u64 << enemypawn))) | (1u64 << ep);
            let checkers = self.getcheckers(board, blockers);
            if checkers == 0 {
              movelist.push(Move::makeEP(ind as u8, ep));
            }
          }
          
        }
        while attacks != 0 
        {
          let dst = constlib::poplsb(&mut attacks) as u8;
          if(dst / 8 == 7 || dst / 8  == 0)
          {
            movelist.push(Move::makePromCap(ind as u8, dst, PieceType::B));
            movelist.push(Move::makePromCap(ind as u8, dst, PieceType::N));
            movelist.push(Move::makePromCap(ind as u8, dst, PieceType::R));
            movelist.push(Move::makePromCap(ind as u8, dst, PieceType::Q));
          } else {movelist.push(Move::makeCapture(ind as u8, dst))
          }
        }
        //create capture bitmoves
        //save legalmoves for this square
        }
      }

    pub fn init_pawnmoves(&mut self) {
      for i in 8..56 {
        let mut wmoves = 0;
        let mut bmoves = 0;
        let sq = 1u64 << i;
        wmoves |= constlib::genShift(constlib::north, sq);
        bmoves |= constlib::genShift(constlib::south, sq);
        
        
        self.pawnmoves[0][i] = wmoves;
        self.pawnmoves[1][i] = bmoves;

      }
    }
    pub fn init_pawnattacks(&mut self) { 
      for i in 0..64 {
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
            let mut kingboard = 1u64 << i;
            //get left and right of king
            attacks |= constlib::dirShift(constlib::west, kingboard)
                | constlib::dirShift(constlib::east, kingboard);
            //move left, center, and right of king up and down to get attacks
            kingboard |= attacks;
            attacks |= constlib::dirShift(constlib::north, kingboard)
                | constlib::dirShift(constlib::south, kingboard);
            if i%8==7 
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
          let knightpos = 1u64 << i;
          let left = constlib::dirShift(constlib::east,knightpos);
          let right = constlib::dirShift(constlib::west,knightpos);
          attacks |= (left | right)<< 16;//left and right by one, up by two
          attacks |= (left | right) >> 16;
          let east = constlib::dirShift(constlib::east, left);
          let west = constlib::dirShift(constlib::west, right);
          attacks |= (east | west) << 8;
          attacks |= (east | west) >> 8;
          if i % 8 >= 6 
          {
            //on the g or h file
            attacks &= constlib::notAFile;
            attacks &=!0x0202020202020202;
          } else if i % 8 <= 1 
          {
            //on the a or bfile
            attacks &= constlib::notHFile;
            attacks &= !0x4040404040404040;
          }
          self.knight[i] = attacks;
        }
      }
    
    
    #[inline(always)]
    pub fn generatecastling(&self, board: &Board, movelist: &mut Vec<Move>, enemy_attacks: u64) {
    let color = board.turn;
    let rights = board.castling_rights;
    if oppressed(rights) { return; }

    // Squares attacked by the opponent (reliable for legality)
    let kingdanger = enemy_attacks;

    if color == 0 {
        let kingsq: u8 = constlib::square_from_string("e1");

        // White king-side: empty f1,g1; and e1,f1,g1 not attacked
        if wkingside(rights) {
            let empty_between =
                constlib::squarebb_from_string("f1") |
                constlib::squarebb_from_string("g1");
            let king_path =
                constlib::squarebb_from_string("e1") |
                constlib::squarebb_from_string("f1") |
                constlib::squarebb_from_string("g1");

            if (board.occupied & empty_between) == 0 && (kingdanger & king_path) == 0 {
                movelist.push(Move::makeKingCastle(kingsq, 7));
            }
        }

        // White queen-side: empty b1,c1,d1; and e1,d1,c1 not attacked
        if wqueenside(rights) {
            let empty_between =
                constlib::squarebb_from_string("b1") |
                constlib::squarebb_from_string("c1") |
                constlib::squarebb_from_string("d1");
            let king_path =
                constlib::squarebb_from_string("e1") |
                constlib::squarebb_from_string("d1") |
                constlib::squarebb_from_string("c1");

            if (board.occupied & empty_between) == 0 && (kingdanger & king_path) == 0 {
                movelist.push(Move::makeQueenCastle(kingsq, 0));
            }
        }
    } else {
        let kingsq: u8 = constlib::square_from_string("e8");

        // Black king-side: empty f8,g8; and e8,f8,g8 not attacked
        if bkingside(rights) {
            let empty_between =
                constlib::squarebb_from_string("f8") |
                constlib::squarebb_from_string("g8");
            let king_path =
                constlib::squarebb_from_string("e8") |
                constlib::squarebb_from_string("f8") |
                constlib::squarebb_from_string("g8");

            if (board.occupied & empty_between) == 0 && (kingdanger & king_path) == 0 {
                movelist.push(Move::makeKingCastle(kingsq, 63));
            }
        }

        // Black queen-side: empty b8,c8,d8; and e8,d8,c8 not attacked
        if bqueenside(rights) {
            let empty_between =
                constlib::squarebb_from_string("b8") |
                constlib::squarebb_from_string("c8") |
                constlib::squarebb_from_string("d8");
            let king_path =
                constlib::squarebb_from_string("e8") |
                constlib::squarebb_from_string("d8") |
                constlib::squarebb_from_string("c8");

            if (board.occupied & empty_between) == 0 && (kingdanger & king_path) == 0 {
                movelist.push(Move::makeQueenCastle(kingsq, 56));
            }
        }
    }
}
pub fn makeattackedmask_for_color(&self, board: &Board, color: u8, blockers: u64) -> u64
{
      let enemy = if color == 0 {1} else {0};
      let mut attacks = 0;
      let mut pbb = board.pieces[(6*enemy+PieceIndex::P.index())];
      let mut nbb = board.pieces[(6*enemy+PieceIndex::N.index())];
      let mut bbb = board.pieces[(6*enemy+PieceIndex::B.index())];
      let mut rbb = board.pieces[(6*enemy+PieceIndex::R.index())];
      let mut qbb = board.pieces[(6*enemy+PieceIndex::Q.index())];
      let mut kbb = board.pieces[(6*enemy+PieceIndex::K.index())];
      while pbb != 0 {
        let ind = constlib::poplsb(&mut pbb);
        attacks |= self.pawnattacks[enemy][ind as usize];
      }
      while kbb != 0 {
        let ind = constlib::poplsb(&mut kbb);
        attacks |= self.king[ind as usize];
      }
      while nbb != 0 {
        let ind = constlib::poplsb(&mut nbb);
        attacks |= self.knight[ind as usize];
      }
      while bbb != 0 {
        let ind = constlib::poplsb(&mut bbb);
        attacks |= constlib::compute_bishop(ind as i8, blockers);
      }
      while rbb != 0 {
        let ind = constlib::poplsb(&mut rbb);
        attacks |= constlib::compute_rook(ind as i8, blockers);
      }
      while qbb != 0 {
        let ind = constlib::poplsb(&mut qbb);
        let qatt = constlib::compute_bishop(ind as i8, blockers) | constlib::compute_rook(ind as i8, blockers);
        attacks |= qatt
      }
      
      attacks
}
    #[inline(always)]
    pub fn makeattackedmask(&self, board:&Board,color: u8, blockers:u64) -> u64 {
      let mut attacks = 0u64;

      let base = 6 * color as usize;
      let mut pbb = board.pieces[base + PieceIndex::P.index()];
      let mut nbb = board.pieces[base + PieceIndex::N.index()];
      let mut bbb = board.pieces[base + PieceIndex::B.index()];
      let mut rbb = board.pieces[base + PieceIndex::R.index()];
      let mut qbb = board.pieces[base + PieceIndex::Q.index()];
      let mut kbb = board.pieces[base + PieceIndex::K.index()];

      while pbb != 0 {
          let sq = constlib::poplsb(&mut pbb);
          attacks |= self.pawnattacks[color as usize][sq as usize];
      }
      while kbb != 0 {
          let sq = constlib::poplsb(&mut kbb);
          attacks |= self.king[sq as usize];
      }
      while nbb != 0 {
          let sq = constlib::poplsb(&mut nbb);
          attacks |= self.knight[sq as usize];
      }
      while bbb != 0 {
          let sq = constlib::poplsb(&mut bbb);
          attacks |= constlib::compute_bishop(sq as i8, blockers);
      }
      while rbb != 0 {
          let sq = constlib::poplsb(&mut rbb);
          attacks |= constlib::compute_rook(sq as i8, blockers);
      }
      while qbb != 0 {
          let sq = constlib::poplsb(&mut qbb);
          attacks |= constlib::compute_bishop(sq as i8, blockers) | constlib::compute_rook(sq as i8, blockers);
      }
      attacks
    }
    #[inline(always)]
    pub fn getkingdangermask(&self, board:&Board) -> u64 {
      board.attacked[board.turn as usize]
    }
    

    #[inline(always)]
    pub fn getcheckers(&self, board:&Board, blockers: u64) -> u64 {
      let color = board.turn;
      let enemy = if color == 0 {1} else {0};
      let kingidx = if color == 0 {PieceIndex::K.index()} else {PieceIndex::k.index()};
      let mut attackers = 0;
      let mut king = board.pieces[kingidx];
      debug_assert!(king != 0, "King bitboard is 0 in getcheckers()");
      
      // let kingsq = constlib::poplsb(&mut king) as i8;
      let kingsq = king.trailing_zeros() as i8;
      let pbb = board.pieces[(6*enemy+PieceIndex::P.index())];
      let nbb = board.pieces[(6*enemy+PieceIndex::N.index())];
      let bbb = board.pieces[(6*enemy+PieceIndex::B.index())];
      let rbb = board.pieces[(6*enemy+PieceIndex::R.index())];
      let qbb = board.pieces[(6*enemy+PieceIndex::Q.index())];
      let batt = constlib::compute_bishop(kingsq, blockers);
      let ratt = constlib::compute_rook(kingsq, blockers);
      

      attackers |= (self.pawnattacks[color as usize][kingsq as usize] & pbb)
                |  (self.knight[kingsq as usize] & nbb) 
                |  (batt & bbb)
                |  (ratt & rbb)
                |  ((batt | ratt) & qbb);
      attackers
    }
    #[inline(always)]
    pub fn in_check(&self, board: &Board) -> bool {
        self.getcheckers(board, board.occupied) != 0
    }
    #[inline(always)]
    pub fn getpinned(&self, board:&Board) -> (u64,u64){
      let color = board.turn;
      let enemy = if color == 0 {1} else {0};
      let kingidx = if color == 0 {PieceIndex::K.index()} else {PieceIndex::k.index()};
      let mut king = board.pieces[kingidx];
      let kingsq = constlib::poplsb(&mut king) as i8;
      
      let blockers = board.occupied;
      
      
      //get all sliding attacks of enemy.
      let mut bbb = board.pieces[(6*enemy+PieceIndex::B.index())];
      let mut rbb = board.pieces[(6*enemy+PieceIndex::R.index())];
      let mut qbb = board.pieces[(6*enemy+PieceIndex::Q.index())];
      

      //chat gpt mode activated...
      //lets get all diagonal sliders...
      let mut sliders = bbb | qbb;
      let mut pinners = 0;
      let mut pinned = 0;
      //lets iterate through all diagonal sliders...
      while sliders != 0 {
        let ind = constlib::poplsb(&mut sliders);

        //lets check that the king is indeed on a diagonal from the sliding piece
        if constlib::compute_bishop(kingsq, 0) & (1<<ind) ==0 {
          //check if the king is on a diagonal from the given slider
          continue;
        }

        //now, get the ray from the king to the slider
        //then, get the ray from the slider to the king

        let ray_king_to_slider = Self::get_ray_mask_blockers(kingsq, ind as i8, board.occupied);
        let ray_slider_to_king = Self::get_ray_mask_blockers(ind as i8, kingsq, board.occupied);
        
        //now, get the intersection of the two rays
        let intersection = ray_king_to_slider & ray_slider_to_king;
        
        if intersection.count_ones() != 1 {
          //if the intersection is not exactly one square, then the piece is not pinned
          continue;
        }

        if (ray_slider_to_king & board.occupied) != intersection{
          //this means that along the ray from the slider to the king, there is another piece other than just the exisiting intersection
          continue;
        }
        
        pinned |= intersection;
        pinners |= 1<< ind;
        
      }


      //now, get all orthogonal sliders...
      let mut sliders = rbb | qbb;
      while sliders != 0 {
        let ind = constlib::poplsb(&mut sliders);
        //lets check that the king is indeed on a orthogonal line from the sliding piece
        if constlib::compute_rook(kingsq, 0) & (1<<ind) ==0 {
          //check if the king is on a diagonal from the given slider
          continue;
        }

        //now, get the ray from the king to the slider
        //then, get the ray from the slider to the king

        let ray_king_to_slider = Self::get_ray_mask_blockers(kingsq, ind as i8, board.occupied);
        let ray_slider_to_king = Self::get_ray_mask_blockers(ind as i8, kingsq, board.occupied);
        
        //now, get the intersection of the two rays
        let intersection = ray_king_to_slider & ray_slider_to_king;
        if intersection.count_ones() != 1 {
          //if the intersection is not exactly one square, then the piece is not pinned
          continue;
        }

        if (ray_slider_to_king & board.occupied) != intersection{
          //this means that along the ray from the slider to the king, there is another piece other than just the exisiting intersection
          continue;
        }
        
        pinned |= intersection;
        pinners |= 1<< ind;
      }
     
      
      // while bbb != 0 {
      //   let ind = constlib::poplsb(&mut bbb);
      //   let rayfromking = Self::get_ray_mask_blockers(kingsq, ind as i8,board.occupied);
      //   let batt = constlib::compute_bishop(ind as i8, board.occupied);
        
        
      //   let mut potentiallypinned = batt & rayfromking & pinnables;
      //   if potentiallypinned != 0{
      //     constlib::poplsb(&mut potentiallypinned);
      //     //if the bishop attack intersects the pinnablemask and friendly pieces, bishop is a pinner
      //     pinners |= 1u64 << ind;
      //   }
      //   sliders |= batt;
      // }
      // while qbb != 0 {
      //   let ind = constlib::poplsb(&mut qbb);
      //   let rayfromking = Self::get_ray_mask_blockers(kingsq, ind as i8,board.occupied);
      //   let qatt = Self::get_ray_mask_blockers(ind as i8, kingsq, board.occupied);
      //   let mut potentiallypinned = qatt & rayfromking & pinnables;
      //   if potentiallypinned != 0{
      //     constlib::poplsb(&mut potentiallypinned);
      //     //if the bishop attack intersects the pinnablemask and friendly pieces, bishop is a pinner
      //     // println!("queen is pinning");
      //     // constlib::print_bitboard(qatt);
      //     // constlib::print_bitboard(rayfromking);
      //     // constlib::print_bitboard(pinnables);}
      //     pinners |= 1u64 << ind;
          

      //   }
      //   sliders |= qatt;
      // }
      // while rbb != 0{ 
      //   let ind = constlib::poplsb(&mut rbb);
      //   let ratt = constlib::compute_rook(ind as i8,board.occupied);
      //   let mut potentiallypinned = ratt & pinnablemask & pinnables;
      //   if potentiallypinned != 0{
      //     constlib::poplsb(&mut potentiallypinned);
      //     //if the bishop attack intersects the pinnablemask and friendly pieces, bishop is a pinner
      //     println!("rook is pinning");
      //     pinners |= 1u64 << ind;
      //   }
        
      //   sliders |= ratt;
      // }
      // let pinned = pinnablemask & sliders & pinnables;

      (pinned, pinners)
      
    }
    // pub fn getlegalpinnedmoves(pinners: &mut u64) -> u64 {
    //   let color = board.turn;
    //   let color = board.turn;
    //   let enemy = if color == 0 {1} else {0};
    //   let kingidx = if color == 0 {PieceIndex::K.index()} else {PieceIndex::k.index()};
    //   let mut king = board.pieces[kingidx];
    //   let kingsq = constlib::poplsb(&mut king) as i8;
    //   while pinners != 0 {
    //     let ind = constlib::poplsb(&mut pinners);

    //   }
    // }

    #[inline(always)]
    pub fn enemy_attacks(&self, board: &Board) -> u64 {
        // Compute attacks by the side NOT to move (enemy of board.turn)
        // without mutating the board.
        // This requires makeattackedmask-like logic that takes an explicit color.
        self.makeattackedmask_for_color(board, board.turn ^ 1, board.occupied)
    }
    #[inline(always)]
    pub fn genevasions(&self, board:&Board, movelist: &mut Vec<Move>, checkers: &mut u64, pininfo: (u64, u64), kingsq: i8, enemy_attacks: u64) {
      
      let mut ct = 0;
      let mut checkerscopy = *checkers;
      while *checkers != 0 {
        constlib::poplsb(checkers);
        ct+=1;
      }
      self.generatekingmoves(board, movelist,kingsq, enemy_attacks);
      if ct > 1 {
        //double check at least.
        return
      } else {
        //either move king out of check
        //capture checking piece
        //block checking piece
        
        let capture_mask = checkerscopy;
        let checkersq = constlib::poplsb(&mut checkerscopy);
        let piecetype = board.piecelocs.piece_at(checkersq as u8).get_piece_type();
        let block_mask = match piecetype {
          //get the squares that a piece can move to to block check
          PieceType::R | PieceType::Q | PieceType::B => Self::get_ray_mask(checkersq as i8,kingsq as i8),
          _ => 0,
        };
        //capture_mask is mask of capturable pieces
        //block_mask is mask of squares friendly pieces can move to
        self.generatepawnmoves(board,movelist, true, (capture_mask,block_mask), pininfo,kingsq);
        self.generateknightmoves(board,movelist, true, (capture_mask,block_mask), pininfo);
        self.generatebishopmoves(board,movelist, true, (capture_mask,block_mask), pininfo,kingsq);
        self.generaterookmoves(board,movelist, true, (capture_mask,block_mask), pininfo,kingsq);
        self.generatequeenmoves(board,movelist, true, (capture_mask,block_mask), pininfo,kingsq);
      }
    }
    #[inline(always)]
    fn get_ray_mask(source_square: i8, target_square: i8) -> u64 {
      let mut ray_mask = 0;
      let mut square = source_square as i8;
      loop {
          let file_diff = (target_square % 8) as isize - (square % 8) as isize;
          let rank_diff = (target_square / 8) as isize - (square / 8) as isize;
  
          if file_diff == 0 {
              // Vertical ray
              square += if rank_diff > 0 { 8 } else { -8 };
          } else if rank_diff == 0 {
              // Horizontal ray
              square += if file_diff > 0 { 1 } else { -1 };
          } else if file_diff.abs() == rank_diff.abs() {
              // Diagonal ray
              let file_step = if file_diff > 0 { 1 } else { -1 };
              let rank_step = if rank_diff > 0 { 8 } else { -8 };
              square += file_step + rank_step;
          } else {
              // Invalid ray, should not happen
              break;
          }
          if square == target_square as i8{
            break;
          }
          // Skip the source square and the target square
          
          ray_mask |= 1u64 << square;
      
        }
      ray_mask
      }
  #[inline(always)]
fn get_ray_mask_blockers(source_square: i8, target_square: i8, blockers: u64) -> u64 {
    let mut ray_mask: u64 = 0;
    let mut square: i16 = source_square as i16;
    let target: i16 = target_square as i16;

    loop {
        let file_diff = (target % 8) - (square % 8);
        let rank_diff = (target / 8) - (square / 8);

        if file_diff == 0 {
            square += if rank_diff > 0 { 8 } else { -8 };
        } else if rank_diff == 0 {
            square += if file_diff > 0 { 1 } else { -1 };
        } else if file_diff.abs() == rank_diff.abs() {
            let file_step = if file_diff > 0 { 1 } else { -1 };
            let rank_step = if rank_diff > 0 { 8 } else { -8 };
            square += file_step + rank_step;
        } else {
            break; // not aligned
        }

        // stop if we ran off-board (prevents shift overflow)
        if square < 0 || square >= 64 {
            break;
        }

        let sq_mask = 1u64 << (square as u8);
        ray_mask |= sq_mask;

        if square == target || (sq_mask & blockers) != 0 {
            break;
        }
    }

    ray_mask
}
#[inline(always)]
pub fn generate_qcaptures(&self, board: &mut Board)-> Vec<Move>{

    let mut out = Vec::<Move>::new();
    let us = board.turn;
    let them = us ^ 1;

    // If in check, qsearch needs evasions, not "captures only".
    let mut checkers = self.getcheckers(board, board.occupied);
    if checkers != 0 {
        // Reuse your existing evasion generator path.
        // Keep this identical to generate()’s in-check behavior.
        let (pinned, pinners) = self.getpinned(board);
        board.pinned = pinned;
        board.pinners = pinners;
        let pininfo = (pinned, pinners);
        let kingidx = if us == 0 { PieceIndex::K.index() } else { PieceIndex::k.index() };
        let king_bb = board.pieces[kingidx];
        debug_assert!(king_bb != 0);
        let king_sq_u8 = king_bb.trailing_zeros() as u8;
        let king_sq = king_sq_u8 as i8;

        let occ_wo_king = board.occupied & !(1u64 << king_sq_u8);
        let enemy_attacks_wo_king = self.makeattackedmask(board, them, occ_wo_king);

        self.genevasions(board,  &mut out, &mut checkers, pininfo, king_sq, enemy_attacks_wo_king);
        return out;
    }

    // --- pinned info (must be current for legality on pinned sliders/pawns)
    // Use whichever is your canonical source:
    // let pininfo = board.state.getpinned(board);
    let (pinned, pinners) = self.getpinned(board);
    board.pinned = pinned;
    board.pinners = pinners;
    let pininfo = (pinned, pinners);

    // --- king square + enemy attacks without king (for king captures legality)
    let kingidx = if us == 0 { PieceIndex::K.index() } else { PieceIndex::k.index() };
    let king_bb = board.pieces[kingidx];
    debug_assert!(king_bb != 0);
    let king_sq_u8 = king_bb.trailing_zeros() as u8;
    let king_sq = king_sq_u8 as i8;

    let occ_wo_king = board.occupied & !(1u64 << king_sq_u8);
    let enemy_attacks_wo_king = self.makeattackedmask(board, them, occ_wo_king);

    let enemy_king_idx = if them == 0 { PieceIndex::K.index() } else { PieceIndex::k.index() };
    let enemy_king_bb = board.pieces[enemy_king_idx];

    // Never generate captures of the king.
    let enemy_occ = board.playerpieces[them as usize] & !enemy_king_bb;

    // ===========================
    // Pawn captures + promotions
    // ===========================
    let mut pbb = if us == 0 { board.pieces[PieceIndex::P.index()] } else { board.pieces[PieceIndex::p.index()] };
    while pbb != 0 {
        let from = constlib::poplsb(&mut pbb) as u8;

        // If pinned, pawn may only move along pin line.
        let pin_line = if (pinned & (1u64 << from)) != 0 {
            self.line_between[from as usize][king_sq as usize]
        } else {
            u64::MAX
        };

        // Normal captures (diagonals)
        let mut caps = self.pawnattacks[us as usize][from as usize] & enemy_occ & pin_line;
        while caps != 0 {
            let dst = constlib::poplsb(&mut caps) as u8;

            // Promotion capture?
            if (us == 0 && dst >= 56) || (us == 1 && dst < 8) {
                out.push(Move::makePromCap(from, dst, PieceType::N));
                out.push(Move::makePromCap(from, dst, PieceType::B));
                out.push(Move::makePromCap(from, dst, PieceType::R));
                out.push(Move::makePromCap(from, dst, PieceType::Q));
            } else {
                out.push(Move::makeCapture(from, dst));
            }
        }

        // En passant capture (tactical)
        if board.ep_square != 64 {
            let epsq = board.ep_square;
            let ep_mask = self.pawnattacks[us as usize][from as usize] & (1u64 << epsq) & pin_line;
            if ep_mask != 0 {
                out.push(Move::makeEP(from, epsq));
            }
        }

        // Quiet promotions (also tactical in qsearch)
        if us == 0 {
            // from on rank 7 => 48..55
            if from >= 48 && from <= 55 {
                let dst = from + 8;
                if (board.occupied & (1u64 << dst)) == 0 {
                    if (pinned & (1u64 << from)) == 0 || (pin_line & (1u64 << dst)) != 0 {
                        out.push(Move::makeProm(from, dst, PieceType::N));
                        out.push(Move::makeProm(from, dst, PieceType::B));
                        out.push(Move::makeProm(from, dst, PieceType::R));
                        out.push(Move::makeProm(from, dst, PieceType::Q));
                    }
                }
            }
        } else {
            // black from on rank 2 => 8..15
            if from >= 8 && from <= 15 {
                let dst = from - 8;
                if (board.occupied & (1u64 << dst)) == 0 {
                    if (pinned & (1u64 << from)) == 0 || (pin_line & (1u64 << dst)) != 0 {
                        out.push(Move::makeProm(from, dst, PieceType::N));
                        out.push(Move::makeProm(from, dst, PieceType::B));
                        out.push(Move::makeProm(from, dst, PieceType::R));
                        out.push(Move::makeProm(from, dst, PieceType::Q));
                    }
                }
            }
        }
    }

    // =================
    // Knight captures
    // =================
    let mut nbb = if us == 0 { board.pieces[PieceIndex::N.index()] } else { board.pieces[PieceIndex::n.index()] };
    while nbb != 0 {
        let from = constlib::poplsb(&mut nbb) as u8;
        if (pinned & (1u64 << from)) != 0 {
            continue; // pinned knights cannot move
        }
        let mut caps = self.knight[from as usize] & enemy_occ;
        while caps != 0 {
            let dst = constlib::poplsb(&mut caps) as u8;
            out.push(Move::makeCapture(from, dst));
        }
    }

    // =================
    // Bishop captures
    // =================
    let mut bbb = if us == 0 { board.pieces[PieceIndex::B.index()] } else { board.pieces[PieceIndex::b.index()] };
    while bbb != 0 {
        let from = constlib::poplsb(&mut bbb) as u8;
        let mut caps = constlib::compute_bishop(from as i8, board.occupied) & enemy_occ;
        if (pinned & (1u64 << from)) != 0 {
            caps &= self.line_between[from as usize][king_sq as usize];
        }
        while caps != 0 {
            let dst = constlib::poplsb(&mut caps) as u8;
            out.push(Move::makeCapture(from, dst));
        }
    }

    // ===============
    // Rook captures
    // ===============
    let mut rbb = if us == 0 { board.pieces[PieceIndex::R.index()] } else { board.pieces[PieceIndex::r.index()] };
    while rbb != 0 {
        let from = constlib::poplsb(&mut rbb) as u8;
        let mut caps = constlib::compute_rook(from as i8, board.occupied) & enemy_occ;
        if (pinned & (1u64 << from)) != 0 {
            caps &= self.line_between[from as usize][king_sq as usize];
        }
        while caps != 0 {
            let dst = constlib::poplsb(&mut caps) as u8;
            out.push(Move::makeCapture(from, dst));
        }
    }

    // ===============
    // Queen captures
    // ===============
    let mut qbb = if us == 0 { board.pieces[PieceIndex::Q.index()] } else { board.pieces[PieceIndex::q.index()] };
    while qbb != 0 {
        let from = constlib::poplsb(&mut qbb) as u8;
        let mut caps = (constlib::compute_bishop(from as i8, board.occupied) | constlib::compute_rook(from as i8, board.occupied)) & enemy_occ;
        if (pinned & (1u64 << from)) != 0 {
            caps &= self.line_between[from as usize][king_sq as usize];
        }
        while caps != 0 {
            let dst = constlib::poplsb(&mut caps) as u8;
            out.push(Move::makeCapture(from, dst));
        }
    }

    // =============
    // King captures
    // =============
    {
        let from = king_sq_u8;
        let mut caps = self.king[from as usize] & enemy_occ;
        caps &= !enemy_attacks_wo_king; // cannot capture into check
        while caps != 0 {
            let dst = constlib::poplsb(&mut caps) as u8;
            out.push(Move::makeCapture(from, dst));
        }
    }
    out

    
}
  #[inline(always)]
    pub fn is_square_attacked_by(&self, board: &Board, occ: u64, by_side: u8, sq: u8) -> bool {
        let sq_bb = 1u64 << (sq as u64);

        // --- Pawns ---
        // by_side == 0 => white pawns attack NE/NW (from their perspective).
        // by_side == 1 => black pawns attack SE/SW.
        //
        // We want: "is sq attacked by pawns of by_side?"
        // Equivalent: "is there a pawn of by_side on a square that attacks sq?"
        let pawns = board.pieces[if by_side == 0 { PieceIndex::P.index() } else { PieceIndex::p.index() }];

        if by_side == 0 {
            // White pawn attacks: from pawn square -> +7/+9.
            // So attackers to sq are at sq-7 and sq-9.
            // Use shifts on sq_bb to find those pawn squares.
            let from_se = (sq_bb >> 7) & !constlib::filesmasks[7]; // not from h-file
            let from_sw = (sq_bb >> 9) & !constlib::filesmasks[0]; // not from a-file
            if (pawns & (from_se | from_sw)) != 0 { return true; }
        } else {
            // Black pawn attacks: from pawn square -> -7/-9.
            // Attackers to sq are at sq+7 and sq+9.
            let from_ne = (sq_bb << 7) & !constlib::filesmasks[7]; // mask h-file
            let from_nw = (sq_bb << 9) & !constlib::filesmasks[0]; // mask a-file
            if (pawns & (from_ne | from_nw)) != 0 { return true; }
        }

        // --- Knights ---
        let knights = board.pieces[if by_side == 0 { PieceIndex::N.index() } else { PieceIndex::n.index() }];
        if (knights & self.knight[sq as usize]) != 0 { return true; }

        // --- King ---
        let king = board.pieces[if by_side == 0 { PieceIndex::K.index() } else { PieceIndex::k.index() }];
        if (king & self.king[sq as usize]) != 0 { return true; }

        // --- Bishops / Queens (diagonals) ---
        let bishops = board.pieces[if by_side == 0 { PieceIndex::B.index() } else { PieceIndex::b.index() }];
        let queens  = board.pieces[if by_side == 0 { PieceIndex::Q.index() } else { PieceIndex::q.index() }];
        let diag_attackers = bishops | queens;

        // Use your existing diagonal slider attack for a single square:
        // IMPORTANT: replace `self.bishop_attacks(sq, occ)` with your actual API.
        let bishop_rays = constlib::compute_bishop(sq as i8, occ);
        if (diag_attackers & bishop_rays) != 0 { return true; }

        // --- Rooks / Queens (orthogonals) ---
        let rooks  = board.pieces[if by_side == 0 { PieceIndex::R.index() } else { PieceIndex::r.index() }];
        let ortho_attackers = rooks | queens;

        // Replace `self.rook_attacks(sq, occ)` with your actual API.
        let rook_rays = constlib::compute_rook(sq as i8, occ);
        if (ortho_attackers & rook_rays) != 0 { return true; }

        false
    }
    #[inline(always)]
    pub fn gen_line_bbs(&mut self) {
      for i in 0..64_i8 {
          for j in 0..64_i8 {
              let i_bb: u64 = 1_u64 << i;
              let j_bb: u64 = 1_u64 << j;
              if constlib::compute_rook(i, 0) & j_bb != 0 {
                  self.line_between[i as usize][j as usize] |=
                      (constlib::compute_rook(j,0) & constlib::compute_rook(i,0)) | i_bb | j_bb;
              } else if constlib::compute_bishop(i, 0) & j_bb != 0 {
                  self.line_between[i as usize][j as usize] |=
                      (constlib::compute_bishop(j,0) & constlib::compute_bishop(i,0)) | i_bb | j_bb;
              } else {
                  self.line_between[i as usize][j as usize] = 0;
              }
          }
      }
    }

  }
    

