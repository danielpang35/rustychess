pub mod castling;
pub mod movegen;
pub mod piece;
pub mod constlib;
pub mod r#move;

pub use piece::Piece;
pub use piece::PieceLocations;
pub use piece::PieceType;
pub use r#move::Move;

//a struct defining the physical aspects of the board
pub struct Board {
    occupied: u64,
    pieces: [u64; 12], //a bitboard for each piece
    playerpieces: [u64; 2],
    turn: u8,
    castling_rights: u8,
    ep_square: u8,
    piecelocs: PieceLocations,
    pinned: [u64; 2], //friendly pieces
    pinners: [u64; 2], //enemy pieces
}
impl Board {
    //constructor
    pub fn new() -> Self {
        Self {
            occupied: 0,
            pieces: [0; 12],
            playerpieces: [0; 2],
            turn: 0,
            castling_rights: 15,
            ep_square: 0,
            piecelocs: PieceLocations::new(),
            pinned: [0; 2],
            pinners: [0; 2],
        }
    }
    pub fn push(&self, bm:Move) {

    }

    pub fn setPinned(&mut self, pinned:u64) {
        let color = self.turn;
        self.pinned[color as usize] = pinned;
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
        let mask = castling::get_castling_mask(castling_rights);
        self.castling_rights = mask;

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
