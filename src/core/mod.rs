pub mod castling;
pub mod movegen;
pub mod piece;
pub mod constlib;
pub mod r#move;

pub use piece::Piece;
pub use piece::PieceLocations;
pub use piece::PieceType;
pub use r#move::Move;
pub use castling::CastlingRights;

pub use std::rc::Rc;
//a struct defining the physical aspects of the board
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
        }
    }
    pub fn push(&mut self, bm:Move) {
        println!("Pushing move to board");
        bm.print();
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

        //update occupied bitboard
        self.occupied ^= (1 << from);
        self.occupied |= (1 <<to);
        constlib::print_bitboard(self.occupied);

        //update piece bitboards
        let pieceidx = piece.getidx();
        self.pieces[pieceidx] ^= (1<<from) | (1<<to);
        println!("Moved piece {} from square {}",pieceidx, from);
        if capture {
            //if a piece was captured, toggle the bit that it was on on its bitboard
            let capturedidx = self.piecelocs.piece_at(to).getidx();
            self.pieces[capturedidx] ^= (1<<to);
            constlib::print_bitboard(self.pieces[capturedidx]);
            println!("Removed piece {} from square {}",capturedidx, to);

            //also remove enemy piece from enemy playerpiece bitboard
            self.playerpieces[enemy as usize] ^= (1<<to);
        }

        //toggle our playerpieces bitboard
        self.playerpieces[color as usize] ^= (1<<from) | (1<<to);
        println!("Friendly pieces bitboard:");
        constlib::print_bitboard(self.playerpieces[color as usize]);
        println!("Enemy pieces bitboard:");
        constlib::print_bitboard(self.playerpieces[enemy as usize]);

        //update piecelocations
        self.piecelocs.place(to, piece);
        self.piecelocs.remove(from);
        println!("piece {}",piece.get_piece_type().get_piece_type());

        //update ep
        if dbpush {
            self.ep_square = if self.turn == 0 {to + 8} else {to - 8};
            constlib::print_bitboard(constlib::squaretobb(self.ep_square));
            println!("Updated EP");
        } else {
            self.ep_square = 0;
        }
        if !castling::oppressed(self.castling_rights) {
            //if there are still castling rights
            //if a rook or king's starting square is the starting square or ending square of a move,
            //remove that side from the castling rights
            let updatecastlemask = !(castling::get_rights(to) | castling::get_rights(from));
            // println!("queenside castle was:{}", castling::wqueenside(self.castling_rights));

            // println!("Updating castle mask with : {}", format!("{updatecastlemask:b}"));
            self.castling_rights &= updatecastlemask;
        }
        //stupidity check
        assert!(from != to);
        //update occupied, pieces, playerpieces, change turn, update castling rights, update ep square, update piecelocs

        //update pininfo and attacked squares
        //also yea... you need to refactor this ugly ass code...
        let pininfo = movegen::MoveGenerator::getpinned(&mut movegen::MoveGenerator::new(),self);
        self.pinned[color as usize] = pininfo.0;
        self.pinners[color as usize] = pininfo.1;
        self.attacked[self.turn as usize] = movegen::MoveGenerator::makeattackedmask(&mut movegen::MoveGenerator::new(),self,self.occupied);

        //update turn
        self.turn = enemy;
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
    pub fn toStr(&self, mut input: String) -> String {
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
        input.push_str(s.as_str());
        return input;
    }
  
  //returns string of bitboard
  pub fn bbtostr(&self, mut input: String) -> String {
      constlib::print_bitboard(self.occupied);
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
