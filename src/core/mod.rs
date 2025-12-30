pub mod castling;
pub mod movegen;
pub mod piece;
pub mod constlib;
pub mod r#move;
pub mod state;
pub mod cli;
pub mod zobrist;
#[cfg(test)]
pub mod tests;

pub use piece::Piece;
pub use piece::PieceLocations;
pub use piece::PieceType;
pub use piece::PieceIndex;
pub use r#move::Move;
pub use castling::CastlingRights;
pub use state::BoardState;
pub use std::sync::Arc;

use crate::core::zobrist::{Z_PIECE_SQ, Z_SIDE, Z_CASTLING, Z_EP_FILE};


//a struct defining the physical aspects of the board
// #[derive(Clone)]
pub struct Board {
    pub occupied: u64,
    pub pieces: [u64; 12], //a bitboard for each piece
    pub playerpieces: [u64; 2],
    pub turn: u8,
    pub piecelocs: PieceLocations,
    pub state: Arc<BoardState>,
    pub ply: u16,
}
impl Board {
    //constructor
    pub fn new() -> Self {
        Self {
            occupied: 0,
            pieces: [0; 12],
            playerpieces: [0; 2],
            turn: 0,
            piecelocs: PieceLocations::new(),
            state: Arc::new(BoardState::new()),
            ply: 0,
        }
    }
    pub fn clone(&self) -> Board {
        Board {
            occupied: self.occupied,
            pieces: self.pieces,
            playerpieces: self.playerpieces,
            turn: self.turn,
            piecelocs: self.piecelocs,
            state: Arc::clone(&self.state),
            ply: self.ply,
        }
    }

    pub fn push(&mut self, bm:Move, movegen: &movegen::MoveGenerator) {
        
        let color = self.turn;
        let enemy = if color == 0 {1} else {0};
        //push a move to the board
        // println!("{}",if self.turn == 0 {"WHITE"} else {"BLACK"});
        //unpack move
        let quiet = bm.isquiet();
        let ep = bm.isep();
        let castle = bm.iscastle();
        let capture = bm.iscapture();
        let prom = bm.isprom();
        let dbpush = bm.isdoublepawn();
        let from = bm.getSrc();
        let to = bm.getDst();
        let piece = self.piecelocs.piece_at(from);
        
        assert!(piece != Piece::None);
        //TODO: refactor this: just pass around a board state reference
        #[inline(always)]
        fn file_of(sq: u8) -> usize { (sq & 7) as usize }

        let old_castle = self.state.castling_rights;
        let old_ep = self.state.ep_square;
        let mut h = self.state.hash;

        // remove old EP (if any)
        if old_ep != 64 {
            h ^= Z_EP_FILE[file_of(old_ep)];
        }
        // remove old castling
        h ^= Z_CASTLING[(old_castle & 0x0F) as usize];

        let mut newstate = BoardState::new();
        self.ply += 1;
        if castle {
            //get side of castling
            self.apply_castling(from as i8, to as i8);
                let ksrc = from as i8;
                let rsrc = to as i8;
                let kingside = ksrc < rsrc;
                let (kdst, rdst) = if kingside { (ksrc + 2, ksrc + 1) } else { (ksrc - 2, ksrc - 1) };

                let king_idx = (6 * color as usize) + PieceIndex::K.index();
                let rook_idx = (6 * color as usize) + PieceIndex::R.index();

                h ^= Z_PIECE_SQ[king_idx][ksrc as usize];
                h ^= Z_PIECE_SQ[king_idx][kdst as usize];
                h ^= Z_PIECE_SQ[rook_idx][rsrc as usize];
                h ^= Z_PIECE_SQ[rook_idx][rdst as usize];
        } else {
            //update occupied bitboard
            
            self.occupied ^= (1 << from);
            self.occupied |= (1<<to);
            //update piece bitboards
            let pieceidx = piece.getidx();
            
            self.pieces[pieceidx] ^= (1<<from);
            self.pieces[pieceidx] |= (1<<to);
            
            if capture {
                
                let capturedpiece = self.piecelocs.piece_at(to);
                let capturedidx = if ep {6*enemy+PieceIndex::P.index()}
                    else {capturedpiece.getidx()};
                let mut capsq = to;
                if ep {
                    capsq = if color == 0 {to - 8} else {to + 8};
                    self.occupied ^= (1<<capsq);
                    self.piecelocs.remove(capsq);
                    
                }
                
                //if a piece was captured, toggle the bit that it was on on its bitboard
                self.pieces[capturedidx] ^= (1<<capsq);
                
                //also remove enemy piece from enemy playerpiece bitboard
                self.playerpieces[enemy as usize] ^= (1<<capsq);
                
                //if a piece was captured, also toggle the to square for occupied bitboard
                newstate.capturedpiece = if ep { PieceType::P } else { capturedpiece.get_piece_type() };

                h ^= Z_PIECE_SQ[capturedidx][capsq as usize]; //ZOBRIST UPDATE FOR CAPTURE
            }
            
            //toggle our playerpieces bitboard
            self.playerpieces[color as usize] ^= (1<<from) | (1<<to); 

            //below code is for printing after specific moves
            // if (from == constlib::square_from_string("d1"))
            // {println!("We are pushing the move b7b5. here are playerpieces after toggling. playerpieces from, to playerpieces");
            // constlib::print_bitboard(1<<from);
            // constlib::print_bitboard(1<<to);
            // constlib::print_bitboard(self.playerpieces[color as usize]);

            // constlib::print_bitboard(self.playerpieces[((color+1)%2) as usize]);}

            //update piecelocations
            self.piecelocs.place(to, piece);
            self.piecelocs.remove(from);
            //update ep
            if piece.get_piece_type() == PieceType::P
            {           
                if dbpush {
                    newstate.ep_square = if self.turn == 0 {to - 8} else {to + 8};
                    // constlib::print_bitboard(constlib::squaretobb(self.ep_square));
                } else {
                    newstate.ep_square = 64;
                }
                if prom {
                    let prompiece = bm.prompiece();
                    let prompiece = prompiece.to_piece(color);
                    let promidx = prompiece.getidx();
                    self.pieces[promidx] |= (1<<to);
                    //also remove the pawn from the to square
                    self.pieces[pieceidx] ^= (1<<to);
                    self.piecelocs.place(to,prompiece );
                    h ^= Z_PIECE_SQ[pieceidx][to as usize];
                    // add promoted piece at to
                    h ^= Z_PIECE_SQ[promidx][to as usize];
                }
            } 
            // constlib::print_bitboard(self.pieces[pieceidx]);
            // constlib::print_bitboard(self.occupied);
            // self.print();
        } 
        
        //stupidity check
        assert!(from != to);
        //update occupied, pieces, playerpieces, change turn, update castling rights, update ep square, update piecelocs

        //if a rook or king's starting square is the starting square or ending square of a move,
        //remove that side from the castling rights
        let updatecastlemask = !(castling::get_rights(to) | castling::get_rights(from));
        // println!("queenside castle was:{}", castling::wqueenside(self.castling_rights));

        // println!("Updating castle mask with : {}", format!("{updatecastlemask:b}"));
        newstate.castling_rights = self.state.castling_rights & updatecastlemask;

        //update pininfo and attacked squares
        //also yea... you need to refactor this ugly ass code...
        // Store pin info passed from generate() instead of recalculating
        
        //now that board state has been updated
        // let's switch the player's turn,
        // and let's calculate pins for next turn:
        
        if !castle {
            let moved_idx = piece.getidx();
            h ^= Z_PIECE_SQ[moved_idx][from as usize];
            h ^= Z_PIECE_SQ[moved_idx][to as usize];
        }
        self.turn = enemy as u8;
        h ^= Z_SIDE;
        // add new castling
        h ^= Z_CASTLING[(newstate.castling_rights & 0x0F) as usize];

        // add new EP (if any)
        if newstate.ep_square != 64 {
            h ^= Z_EP_FILE[file_of(newstate.ep_square)];
        }

        newstate.hash = h;

        let pin = movegen.getpinned(self);
        newstate.pinned = pin.0;
        newstate.pinners = pin.1;
        

        // newstate.attacked[white] = all squares which white attacks after the move has been pushed
        let kingidx = if self.turn == 0 { PieceIndex::K.index() } else { PieceIndex::k.index() };
        let blockers = self.occupied & !self.pieces[kingidx];
        newstate.attacked[self.turn as usize] = movegen.makeattackedmask(self, blockers);

        

        newstate.prev = Some(Arc::clone(&self.state));
        newstate.prev_move = bm;

        self.state = Arc::from(newstate);
    
        assert!(!(self.state == *self.state.prev.as_ref().unwrap()));
        debug_assert_eq!(self.state.hash, Self::compute_hash(self, &self.state));
    }

    pub fn pop(&mut self) {

        let statecopy = &self.state;         //save copy of current state
        let previous = match self.state.prev.as_ref().cloned() { //get the previous state
            Some(p) => p,
            None => {
                eprintln!("Board::pop called but no previous state available; ignoring pop");
                return;
            }
        };

        //for the purposes of this function, treat the player who's move is being undone as the friendly side
        let color = if self.turn == 0 {1} else {0};
        let enemy = self.turn;
        

        let bm = self.state.prev_move;
        // println!("Undoing");
        // bm.print();
        //disect previous move

        let quiet = bm.isquiet();
        let ep = bm.isep();
        let castle = bm.iscastle();
        let capture = bm.iscapture();
        let prom = bm.isprom();
        let dbpush = bm.isdoublepawn();

        let from = bm.getSrc();
        let to = bm.getDst();

        let piece = self.piecelocs.piece_at(to);
        

        if castle {
            self.undo_castling(from as i8, to as i8);
        } else {
            assert!(piece.get_color() == color);
            if prom {
                let prom_piece = self.piecelocs.piece_at(to);
                let promidx = prom_piece.getidx();

                // 1) remove promoted piece from bitboards and piecelocs
                self.pieces[promidx] ^= 1<<to;
                self.piecelocs.remove(to);

                // 2) restore pawn at from
                let pawn = Piece::make(color, PieceType::P);
                let pawnidx = pawn.getidx();
                self.pieces[pawnidx] |= 1<<from;
                self.piecelocs.place(from, pawn);

                // 3) occupied and playerpieces restoration should treat this as a normal move from->to
                //    but do NOT toggle prom piece to from.
                self.occupied ^= (1<<from) | (1<<to);
                self.playerpieces[color as usize] ^= (1<<from) | (1<<to);

                // then continue with capture restoration (if capture), but skip the generic piece toggles
                
                // IMPORTANT: return early from the non-castle branch OR guard the generic logic with `if !prom`.
            // 4) restore captured piece if this was a capture (including prom-capture and prom-ep impossible)
        if capture {
            let mut capturedpiece = statecopy.capturedpiece.to_piece(enemy);
            let mut capsq = to;

            

            let capturedidx = capturedpiece.getidx();
            self.pieces[capturedidx] ^= 1<<capsq;
            self.playerpieces[enemy as usize] ^= 1<<capsq;
            self.piecelocs.place(capsq, capturedpiece);
            self.occupied ^= 1<<capsq;
        }

        // promotion handled fully; skip generic undo
        self.state = previous;
        self.turn = color;
        self.ply -= 1;
        return;
    }

    // -------- generic non-promotion undo below --------

    self.occupied ^= (1<<from) | (1<<to);

    self.piecelocs.remove(to);
    self.piecelocs.place(from, piece);

    let pieceidx = piece.getidx();
    self.pieces[pieceidx] ^= (1<<to) | (1<<from);

    if capture {
        let mut capturedpiece = statecopy.capturedpiece.to_piece(enemy);
        let mut capsq = to;
        if ep {
            capturedpiece = if color == 0 {Piece::BP} else {Piece::WP};
            capsq = if color == 0 {to - 8} else {to + 8};
        }
        let capturedidx = capturedpiece.getidx();
        self.pieces[capturedidx] ^= (1<<capsq);
        self.playerpieces[enemy as usize] ^= (1<<capsq);
        self.piecelocs.place(capsq, capturedpiece);
        self.occupied ^= (1<<capsq);
    }

    self.playerpieces[color as usize] ^= (1<<from) | (1<<to);
}

    // common tail
    self.state = previous;
    self.turn = color;
    self.ply -= 1;
    }

    pub fn apply_castling(&mut self, ksrc: i8, rsrc: i8) {
        let kingside = ksrc < rsrc;
        let us = self.turn;
        let mut kdst: i8 = 0;
        let mut rdst: i8 = 0;
        if kingside {
            kdst = ksrc + 2;
            rdst = ksrc + 1;
        } else {
            kdst = ksrc as i8 - 2;
            rdst = ksrc as i8 - 1;
        }
        let kingidx = 6*us as usize +PieceIndex::K.index();
        let rookidx = 6*us as usize+PieceIndex::R.index();
        
        //update pieces bitboards
        self.pieces[kingidx] ^= (1<<ksrc) | (1<<kdst);
        self.pieces[rookidx] ^= (1<<rsrc) | (1<<rdst);

        //update occupied bitboard
        self.occupied ^= (1<<ksrc) | (1<<kdst) | (1<<rsrc) | (1<<rdst);

        //update piecelocs
        let king = self.piecelocs.piece_at(ksrc as u8);
        let rook = self.piecelocs.piece_at(rsrc as u8); 
        self.piecelocs.remove(ksrc as u8);
        self.piecelocs.place(kdst as u8,king);

        self.piecelocs.remove(rsrc as u8);
        self.piecelocs.place(rdst as u8, rook);

        //update playerpieces
        self.playerpieces[us as usize] ^= (1<<ksrc) | (1<<kdst) | (1<<rsrc) | (1<<rdst);
        
    }
    pub fn undo_castling(&mut self, ksrc: i8, rsrc: i8) {
        //if kingside castle, then put rook and king back on their starting squares
        let kingside = ksrc < rsrc;
        let us = self.turn;
        let enemy = if us == 0 {1} else {0};
        let mut kdst = 0;
        let mut rdst = 0;
        if kingside {
            kdst = ksrc + 2;
            rdst = ksrc + 1;
        } else {
            kdst = ksrc as i8 - 2;
            rdst = ksrc as i8 - 1;
        }
        let kingidx = 6*enemy as usize +PieceIndex::K.index();
        let rookidx = 6*enemy as usize+PieceIndex::R.index();
        self.pieces[kingidx] ^= (1<<ksrc) | (1<<kdst);
        self.pieces[rookidx] ^= (1<<rsrc) | (1<<rdst);

        //update occupied bitboard
        self.occupied ^= (1<<ksrc) | (1<<kdst) | (1<<rsrc) | (1<<rdst);

        //update piecelocs
        let king = self.piecelocs.piece_at(kdst as u8);
        let rook = self.piecelocs.piece_at(rdst as u8); 
        self.piecelocs.remove(kdst as u8);
        self.piecelocs.place(ksrc as u8,king);

        self.piecelocs.remove(rdst as u8);
        self.piecelocs.place(rsrc as u8, rook);

        //update playerpieces
        self.playerpieces[enemy as usize] ^= (1<<ksrc) | (1<<kdst) | (1<<rsrc) | (1<<rdst);
        
    }
    pub fn piece_exists_at(&self, rank: usize, file: usize) -> bool {
        //given rank and file
        let result = self.occupied >> (rank * 8 + file);
        return if result & 1 == 1 { true } else { false };
    }

    
    pub fn getpinned(&self) -> u64 {
        self.state.pinned
    }
    pub fn getattacked(&self) -> [u64;2] {
        self.state.attacked
    }
    pub fn getep(&self) -> u8 {
        self.state.ep_square
    }
    //creates board from fen string
    pub fn from_fen(&mut self, fen: String) {
        let mut fields = fen.split(" ");
        let pieces = fields.next().unwrap().chars();
        let mut rank: usize = 7;
        let mut file: usize = 0;
        self.ply = 0;
        
        let mut state = BoardState {    
            castling_rights: 0,
            ep_square: 64,
            capturedpiece: PieceType::NONE,
            pinned: 0, //friendly pieces
            pinners: 0, //enemy pieces
            attacked: [0; 2],
            prev: None,
            prev_move: Move::new(),  
            hash: 0,
        };

        for c in pieces {
            if c.is_numeric() {
                //skip this number of squares
                file += c.to_digit(10).unwrap() as usize;
            } else if c.is_alphabetic() {
                self.put_piece(c, rank, file);
                file += 1;
            } else if c == '/' {
                rank -= 1;
                file = 0;
            }
        }
        let color = fields.next().unwrap();
        

        let castling_rights = fields.next().unwrap();
        let ep = fields.next().unwrap();
        state.ep_square = if ep == "-" { 64 } else { constlib::square_from_string(ep) };

        if color.chars().nth(0).unwrap() == 'w' {
            self.turn = 0;
        } else {
            self.turn = 1;
        }
            //state version

        let pininfo = movegen::MoveGenerator::getpinned(&mut movegen::MoveGenerator::new(), self);
        state.pinned = pininfo.0;   // ✅ Just assign directly
        state.pinners = pininfo.1;

        state.castling_rights = castling::get_castling_mask(castling_rights);
        
        state.attacked[self.turn as usize] = movegen::MoveGenerator::makeattackedmask(&mut movegen::MoveGenerator::new(),self,self.occupied);

        state.hash = Self::compute_hash(self, &state);

        self.state = Arc::new(state);
    }
    #[inline(always)]
    fn file_of(sq: u8) -> usize { (sq & 7) as usize }

    pub fn compute_hash(board: &crate::core::Board, state: &crate::core::BoardState) -> u64 {
        use crate::core::zobrist::{Z_PIECE_SQ, Z_SIDE, Z_CASTLING, Z_EP_FILE};
        use crate::core::constlib;

        let mut h: u64 = 0;

        for p in 0..12usize {
            let mut bb = board.pieces[p];
            while bb != 0 {
                let sq = constlib::poplsb(&mut bb) as usize;
                h ^= Z_PIECE_SQ[p][sq];
            }
        }

        if board.turn == 1 { h ^= Z_SIDE; }
        h ^= Z_CASTLING[(state.castling_rights & 0x0F) as usize];

        if state.ep_square != 64 {
            h ^= Z_EP_FILE[Self::file_of(state.ep_square)];
        }
        h
    }
    pub fn set_startpos(&mut self) {
        self.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    }
    //updates the board state by placing a piece at a location
    //helper function to create board from fen
    pub fn put_piece(&mut self, ch: char, rank: usize, file: usize) {
        let ind = rank * 8 + file;
        let color = if ch.is_uppercase() { 0 } else { 1 };
        let piece = match ch {
            'p' | 'P' => PieceType::P,
            'n' | 'N' => PieceType::N,
            'b' | 'B' => PieceType::B,
            'r' | 'R' => PieceType::R,
            'q' | 'Q' => PieceType::Q,
            'k' | 'K' => PieceType::K,
            _ => PieceType::NONE,
        };
        self.playerpieces[color] |= 1 << ind;
        // println!("Index into pieces; {}", piece as usize +(color*6));
        self.pieces[piece as usize + (color*6 ) - 1] |= 1 << ind;
        self.occupied |= 1 << ind;
        self.piecelocs.place(ind as u8, Piece::make(color as u8, piece));
      
    }
    
    //returns string of board
    pub fn print(&self) {
        let mut s = String::from("+---+---+---+---+---+---+---+---+");
        for r in (0..=7).rev() {
            let mut row = String::from("\n|");
            for f in 0..=7 {
                let p = self.piecelocs.piece_at(r*8+ f);
                let mut ch = self.piecelocs.piece_at(r*8+ f).get_piece_type().get_piece_type().to_string();

                if p != Piece::None && p.get_color() == 0 {
                    ch = ch.to_ascii_uppercase();
                }

                row.push_str(&ch);
            }
            s.push_str(&row)
        }
        s.push_str("\n   a  b  c  d  e  f  g  h\n");
        println!("{}",s);
    }
  
  
  //returns string of bitboard
  pub fn bbtostr(&self, mut input: String) -> String {
      let mut s = String::from("+---+---+---+---+---+---+---+---+");
      let mg = movegen::MoveGenerator::new();
      let precomputedpawns = mg.pawnattacks[0][1];
      for r in (0..=7).rev() {
          let mut row = String::from("\n|");
          for f in 0..=7 {
              let p = if (precomputedpawns >> (r*8 +f) & 1) == 1{"1"} else {"="};
              
              row.push_str(&p);
          }
          s.push_str(&row)
      }
      s.push_str("\n   a  b  c  d  e  f  g  h\n");
      input.push_str(s.as_str());
      return input;
  }

  pub fn board_to_chars(&self) -> Vec<String> {
    let mut out = Vec::with_capacity(64);

    for sq in 0u8..64 {
        let p = self.piecelocs.piece_at(sq);
        let c = if p == Piece::None {
            '.'
        } else {
            let mut ch = p.get_piece_type().get_piece_type(); // 'p','n','b','r','q','k'
            if p.get_color() == 0 {
                ch = ch.to_ascii_uppercase();
            }
            ch
        };
        out.push(c.to_string());
    }

    debug_assert_eq!(out.len(), 64);
    debug_assert!(out.iter().all(|s| s.chars().count() == 1));
    out
}
}
