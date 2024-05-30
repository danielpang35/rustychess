
pub use crate::core::movegen::MoveGenerator;
pub use crate::core::Board;
/// Direction of going north on a chessboard.
pub const north: i8 = 8;
/// Direction of going south on a chessboard.
pub const south: i8 = -8;
/// Direction of going west on a chessboard.
pub const west: i8 = -1;
/// Direction of going east on a chessboard.
pub const east: i8 = 1;

/// Direction of going northeast on a chessboard.
pub const northeast: i8 = 9;
/// Direction of going northwest on a chessboard.
pub const northwest: i8 = 7;
/// Direction of going southeast on a chessboard.
pub const southeast: i8 = -7;
/// Direction of going southwest on a chessboard.
pub const southwest: i8 = -9;


pub const fn initRankMaskLookup() -> [u64; 8] {
    let rank1 = 0xFF;
    let mut arr = [0; 8];
    let mut i = 0;
    while i < 8 {
        arr[i] = rank1 << (8 * i);
        i += 1;
    }

    arr
}

pub const fn initFileMaskLookup() -> [u64; 8] {
    let fileA = 0b0101010101010101;
    let mut arr = [0; 8];
    let mut i = 0;
    while i < 8 {
        arr[i] = fileA << (i);
        i+=1
    }
    arr
}

pub const rankmasks: [u64; 8] = initRankMaskLookup();
pub const filesmasks: [u64; 8] = initFileMaskLookup();

pub const notAFile:u64 = 0xfefefefefefefefe; // ~0x0101010101010101
pub const notHFile:u64 = 0x7f7f7f7f7f7f7f7f; // ~0x8080808080808080


pub fn perft(board: &mut Board, depth: u8, mg: &MoveGenerator) -> u64{
  let mut ct = 0;
  if depth == 0 {
      return 1
  }
  let ml = mg.generate(board);
  for bm in ml {
      board.push(bm);
      ct += perft(board, depth - 1, mg);
      board.pop();
  }
  ct
}

pub fn get_rank(square: u8) -> u8 {
  // For a square in the range 0-63, divide by 8 and add 1 to get the rank (1-8)
  (square / 8) + 1
}
fn get_file(square: u8) -> u8 {
  // For a square in the range 0-63, divide by 8 and add 1 to get the rank (1-8)
  (square % 8) + 1
}

pub fn dirShift(direction: i8, bitboard: u64) -> u64 {
  //perform a general bit shift in the specified direction
    return genShift(direction,bitboard);
}
pub fn genShift(shift: i8, bitboard: u64) -> u64 {
    if shift > 0 {
        bitboard.wrapping_shl(shift as u32)
    } else {
        bitboard.wrapping_shr((-shift) as u32)
    }
}

pub fn poplsb(bitboard: &mut u64) -> u8
  {
    //returns number of zeros to lsb
    //sets lsb to 0
    let zeros = bitboard.trailing_zeros();
    if(*bitboard != 0) {
      *bitboard &= *bitboard - 1;
    }
    zeros as u8
  }

pub fn print_bitboard(bitboard: u64) {
  println!("----------------------------\n");
    for row in (0..8).rev() {
        for col in 0..8 {
            let square = row * 8 + col;
            let mask = 1 << square;
            let value = if (bitboard & mask) != 0 { "1" } else { "0" };
            print!("{} ", value);
        }
        println!();
    }
  println!("----------------------------\n");

}
pub fn square_from_string(square_str:&str) -> u8 {
  let mut square_value: u8 = 64; // Initialize with an invalid value
  if square_str.len() == 2 {
      let file = square_str.chars().next().unwrap();
      let rank = square_str.chars().nth(1).unwrap();

      let file_value = match file {
          'a' => 0,
          'b' => 1,
          'c' => 2,
          'd' => 3,
          'e' => 4,
          'f' => 5,
          'g' => 6,
          'h' => 7,
          _ => return 64, // Return the invalid value if the file is invalid
      };

      let rank_value = match rank {
          '1' => 0,
          '2' => 1,
          '3' => 2,
          '4' => 3,
          '5' => 4,
          '6' => 5,
          '7' => 6,
          '8' => 7,
          _ => return 64, // Return the invalid value if the rank is invalid
      };

      square_value = (rank_value * 8 + file_value) as u8;
  }

  square_value
}
pub fn squarebb_from_string(square_str: &str) -> u64 {
  let mut square_value: u8 = 64; // Initialize with an invalid value

  if square_str.len() == 2 {
      let file = square_str.chars().next().unwrap();
      let rank = square_str.chars().nth(1).unwrap();

      let file_value = match file {
          'a' => 0,
          'b' => 1,
          'c' => 2,
          'd' => 3,
          'e' => 4,
          'f' => 5,
          'g' => 6,
          'h' => 7,
          _ => return 64, // Return the invalid value if the file is invalid
      };

      let rank_value = match rank {
          '1' => 0,
          '2' => 1,
          '3' => 2,
          '4' => 3,
          '5' => 4,
          '6' => 5,
          '7' => 6,
          '8' => 7,
          _ => return 64, // Return the invalid value if the rank is invalid
      };

      square_value = (rank_value * 8 + file_value) as u8;
  }

  squaretobb(square_value)
}
pub fn squaretobb(square:u8) -> u64 {
  //helper to generate a one square bitboard from given square
  1 << square
}
pub fn squaretouci(square: u8) -> String {
  if square >= 64 {
      panic!("Square number must be between 0 and 63");
  }
  let file = (square % 8) + b'a';
  let rank = (square / 8) + 1 + b'0';
  format!("{}{}", file as char, rank as char)
}


pub fn compute_bishop(sq:i8,blockers:u64) -> u64{
      
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
        0=> {currpos += northeast;
            if currpos >= 64 || currpos % 8 <= 0 {break;}},
        //for up left, check if wrapped around to h file. if so break
        1=> {currpos += northwest;
          if currpos >= 64 || currpos % 8 >= 7 {break;}},
        2=> {currpos += southwest;
          if currpos < 0 || currpos % 8 >= 7 {break;}},
        3=> {currpos += southeast;
          if currpos < 0 || currpos % 8 <=0 {break;}},
        _ => panic!(),
      }
      attacks |= genShift(currpos, 1 as u64);
      if genShift(currpos, 1 as u64) & blockers != 0 {
        break;
      }
      //set the current position as a potential attack
      
      
    }
  }
  return attacks
}

#[inline(always)]
pub fn compute_rook(sq:i8, blockers:u64)->u64{
  let mut attacks = 0;
  let mut currpos: i8;
  for ortho in 0..4 {
    currpos = sq;
    loop {
      match ortho {
        //for up, check if greater than 64. if so, break
        0=> {currpos += north;
          if currpos >= 64 {break;}},
        //for left, check if wrapped around to h file. if so break
        1=> {currpos += west;
          if currpos < 0 || currpos % 8 >= 7 {break;}},
        2=> {currpos += south;
          if currpos < 0 {break;}},
        3=> {currpos += east;
          if currpos % 8 <=0 {break;}},
        _ => panic!(),
      }
      attacks |= genShift(currpos, 1 as u64);
      if genShift(currpos, 1 as u64) & blockers != 0 {break;}
    }
  }
  attacks
    
}