pub mod castling;
pub mod movegen;
pub mod piece;
pub mod constlib;
pub mod r#move;
pub mod state;

pub use piece::Piece;
pub use piece::PieceLocations;
pub use piece::PieceType;
pub use piece::PieceIndex;
pub use r#move::Move;
pub use castling::CastlingRights;
pub use state::BoardState;
pub use std::rc::Rc;



//a struct defining the physical aspects of the board
// #[derive(Clone)]
pub struct Board {
    pub occupied: u64,
    pub pieces: [u64; 12], //a bitboard for each piece
    pub playerpieces: [u64; 2],
    pub turn: u8,
    pub castling_rights: u8,
    pub ep_square: u8,
    pub piecelocs: PieceLocations,
    pub pinned: [u64; 2], //friendly pieces
    pub pinners: [u64; 2], //enemy pieces
    pub attacked: [u64; 2],
    pub prev: Option<Rc<Board>>,
    pub prev_move: Move,

    //TODO: Remove other stuff
    pub state: Rc<BoardState>,
}
impl Board {
    //constructor
    pub fn new() -> Self {
        Self {
            occupied: 0,
            pieces: [0; 12],
            playerpieces: [0; 2],
            turn: 0,
            castling_rights: 0,
            ep_square: 0,
            piecelocs: PieceLocations::new(),
            pinned: [0; 2],
            pinners: [0; 2],
            attacked: [0; 2],
            prev: None,
            prev_move: Move::new(),
            state: Rc::new(BoardState::new()),

        }
    }
    pub fn clone(&self) -> Board {
        Board {
            occupied: self.occupied,
            pieces: self.pieces,
            playerpieces: self.playerpieces,
            turn: self.turn,
            castling_rights: self.castling_rights,
            ep_square: self.ep_square,
            piecelocs: self.piecelocs,
            pinned: self.pinned,
            pinners: self.pinned,
            attacked: self.attacked,
            prev: self.prev.as_ref().cloned(),  //i don't think I should clone this, I need to have a clone point to the same previous board state
            prev_move: self.prev_move,
            state: Rc::clone(&self.state),
        }
    }

    pub fn push(&mut self, bm:Move) {
        let color = self.turn;
        let enemy = if color == 0 {1} else {0};
        //push a move to the board

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
        let boardcopy = self.clone();
        if castle {
            //get side of castling
            self.apply_castling(from as i8, to as i8);
        } else {
            //update occupied bitboard
            self.occupied ^= (1 << from) | (1<<to);
            //update piece bitboards
            let pieceidx = piece.getidx();
            
            self.pieces[pieceidx] ^= (1<<from);
            self.pieces[pieceidx] |= (1<<to);
            if capture {
                let capturedidx = if ep {6*enemy+PieceIndex::P.index()}
                    else {self.piecelocs.piece_at(to).getidx()};
                let mut capsq = to;
                if ep {
                    capsq = if color == 0 {to - 8} else {to + 8};
                    self.occupied ^= (1<<capsq);
                    self.piecelocs.remove(capsq);
                }
                //if a piece was captured, toggle the bit that it was on on its bitboard
                self.pieces[capturedidx] ^= (1<<capsq);
                // constlib::print_bitboard(self.pieces[capturedidx]);
                //also remove enemy piece from enemy playerpiece bitboard
                self.playerpieces[enemy as usize] ^= (1<<capsq);
                
                //if a piece was captured, also toggle the to square for occupied bitboard
                self.occupied ^= (1<<to);
            }
            
            //toggle our playerpieces bitboard
            self.playerpieces[color as usize] ^= (1<<from) | (1<<to); 
           

            //update piecelocations
            self.piecelocs.place(to, piece);
            self.piecelocs.remove(from);
            //update ep
            if piece.get_piece_type() == PieceType::P
            {           
                if dbpush {
                    self.ep_square = if self.turn == 0 {to - 8} else {to + 8};
                    // constlib::print_bitboard(constlib::squaretobb(self.ep_square));
                } else {
                    self.ep_square = 0;
                }
                if prom {
                    let prompiece = bm.prompiece();
                    let prompiece = prompiece.to_piece(color);
                    let promidx = prompiece.getidx();
                    self.pieces[promidx] ^= (1<<to);
                    self.piecelocs.place(to,prompiece );
                }
            } 

           
        } 
        
        //stupidity check
        assert!(from != to);
        //update occupied, pieces, playerpieces, change turn, update castling rights, update ep square, update piecelocs

        if !castling::oppressed(self.castling_rights) {
            //if there are still castling rights
            //if a rook or king's starting square is the starting square or ending square of a move,
            //remove that side from the castling rights
            let updatecastlemask = !(castling::get_rights(to) | castling::get_rights(from));
            // println!("queenside castle was:{}", castling::wqueenside(self.castling_rights));

            // println!("Updating castle mask with : {}", format!("{updatecastlemask:b}"));
            self.castling_rights &= updatecastlemask;
        }
        //update pininfo and attacked squares
        //also yea... you need to refactor this ugly ass code...
        let pininfo = movegen::MoveGenerator::getpinned(&mut movegen::MoveGenerator::new(),self);
        self.pinned[color as usize] = pininfo.0;
        self.pinners[color as usize] = pininfo.1;
        self.attacked[self.turn as usize] = movegen::MoveGenerator::makeattackedmask(&mut movegen::MoveGenerator::new(),self,self.occupied);

        //update turn
        self.turn = enemy as u8;
        self.prev = Some(Rc::from(boardcopy));
        self.prev_move = bm;
        
    }

    pub fn pop(&mut self) {
        let previous = self.prev.as_ref().unwrap().clone();
        let penult = previous.prev.clone();
        let color = previous.turn;
        let enemy = if color == 0 {1} else {0};
        let bm = self.prev_move;
        //disect previous move
        let quiet = bm.isquiet();
        let ep = bm.isep();
        let castle = bm.iscastle();
        let capture = bm.iscapture();
        let prom = bm.isprom();
        let dbpush = bm.isdoublepawn();
        let from = bm.getSrc();
        let to = bm.getDst();
        let piece = previous.piecelocs.piece_at(from);
        assert!(piece.get_color() == color);
        if castle {
            self.undo_castling(from as i8, to as i8);
        } else {
            if prom {
                let promidx = self.piecelocs.piece_at(to).getidx();
                self.pieces[promidx] ^= (1<<to);
            }
            //toggle the occupied bitboard, putting the piece back where it was
            self.occupied ^= (1<<from) | (1<<to);

            //remove from dest square, place back at from square
            self.piecelocs.remove(to);
            self.piecelocs.place(from, piece);
            

            //restore moved piece bitboard by toggling from and to squares
            let pieceidx = piece.getidx();
            self.pieces[pieceidx] |= (1<<from);
            self.pieces[pieceidx] ^= (1<<to);
            assert!(piece.get_color() == color);
            if capture {
                //get captured piece and put it back on the square it was captured on
                let mut capturedpiece = previous.piecelocs.piece_at(to);
                let mut capsq = to;
                if ep {
                    capturedpiece = if color == 0 {Piece::BP} else {Piece::WP};
                    capsq = if color == 0 {to - 8} else {to + 8};
                    self.occupied ^= (1<<capsq);
                }
                let capturedidx = capturedpiece.getidx();
                self.pieces[capturedidx] ^= (1<<capsq);

                //restore occupied bitboard
                
                //put captured piece back 
                self.playerpieces[enemy as usize] ^= (1<<capsq);
                
                //place the capturedpiece back where it was before being captured
                self.piecelocs.place(capsq, capturedpiece);
                self.occupied ^= (1<<to);
            }
            
            //put moved piece back
            self.playerpieces[color as usize] ^= (1<<from) | (1<<to); 
            // println!("Final friendly playerpieces after unmaking move");
            // constlib::print_bitboard(self.playerpieces[color as usize]);
            // println!("Final enemy playerpieces after unmaking move");
            // constlib::print_bitboard(self.playerpieces[enemy as usize]);
            // println!("Final moved piece bitboard after unmaking move");
            // constlib::print_bitboard(self.pieces[pieceidx]);
            // constlib::print_bitboard(self.pieces[PieceIndex::p.index()]);
        }
        //update castling rights
        // self.pieces = previous.pieces;
        // self.playerpieces = previous.playerpieces;
        // self.occupied = previous.occupied;
        // self.piecelocs = previous.piecelocs;
        self.ep_square = previous.ep_square;
        self.castling_rights = previous.castling_rights;
        self.pinned = previous.pinned;
        self.attacked = previous.attacked;
        self.turn = color;
        self.prev = penult;
        self.prev_move = previous.prev_move;
        //TODO: undo castling if any, unset ep square, restore pinned info from previous, restore attacked squares from previous
        
        //need to check if the squares interacted with last turn are a king square or rook square
        //TODO: update turn, 

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
        println!("Applied castling!");
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
        println!("Undid castling!");
        
    }
    pub fn piece_exists_at(&self, rank: usize, file: usize) -> bool {
        //given rank and file
        let result = self.occupied >> (rank * 8 + file);
        return if result & 1 == 1 { true } else { false };
    }

    //creates board from fen string
    pub fn from_fen(&mut self, fen: String) {
        let mut fields = fen.split(" ");
        let pieces = fields.next().unwrap().chars();
        let mut rank: usize = 7;
        let mut file: usize = 0;
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
        if color.chars().nth(0).unwrap() == 'w' {
            self.turn = 0;
        } else {
            self.turn = 1;
        }

        let castling_rights = fields.next().unwrap();
        self.castling_rights = castling::get_castling_mask(castling_rights);
        let mut ep_sq = 0;
        let ep = fields.next().unwrap();
        for (i, ch) in ep.chars().enumerate() {
            if i == 0 {
                //parse file
                match ch {
                    'a' => ep_sq += 0,
                    'b' => ep_sq += 1,
                    'c' => ep_sq += 2,
                    'd' => ep_sq += 3,
                    'e' => ep_sq += 4,
                    'f' => ep_sq += 5,
                    'g' => ep_sq += 6,
                    'h' => ep_sq += 7,
                    '-' => {}
                    _ => panic!("Invalid ep square"),
                };
            } else {
                if ch.to_digit(10).unwrap() as u8 == 3 {
                    ep_sq += 16;
                } else {
                    ep_sq += 40;
                }
            }
        }
        self.ep_square = ep_sq;
        //get pininfo (eventually, think about refactoring this. look how ugly that is. Brother ewww)
        let pininfo = movegen::MoveGenerator::getpinned(&mut movegen::MoveGenerator::new(),self);
        self.pinned[self.turn as usize] = pininfo.0;
        self.pinners[self.turn as usize] = pininfo.1;
        self.attacked[self.turn as usize] = movegen::MoveGenerator::makeattackedmask(&mut movegen::MoveGenerator::new(),self,self.occupied)

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
    pub fn toStr(&self) -> String {
        let mut s = String::from("+---+---+---+---+---+---+---+---+");
        for r in (0..=7).rev() {
            let mut row = String::from("\n|");
            for f in 0..=7 {
                let p = self.piecelocs.piece_at(r*8+ f).get_piece_type().get_piece_type().to_string();
                row.push_str(&p);
            }
            s.push_str(&row)
        }
        s.push_str("\n   a  b  c  d  e  f  g  h\n");
        s
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
}
