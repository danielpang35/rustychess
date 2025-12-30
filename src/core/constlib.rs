
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

pub const PAWN_PST: [i16; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      10,  10,  10,  10,  10,  10,  10,   10,
      5,   5,   15,  20,   20,   10,  5,   5,
      10,   5,   15,  40,  40,   0,   0,   0,
      10,   5,  15,  40,  40,  5,   5,   5,
     10,  10,  20,  20,  20,  10,  10,  10,
      0,  0,   0,   0,   0,   10,   10,   10,
      0,   0,   0,   0,   0,   0,   0,   0,
];

pub const KNIGHT_PST: [i16; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];


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


pub fn perft(board: &mut Board, depth: u8, mg: &MoveGenerator) -> u64 {
  let mut ct = 0;
  if depth == 0 {
      return 1
  }
  let ml = mg.generate(board);
  for bm in ml {
      board.push(bm,mg);
      let moves = perft(board, depth - 1, mg);
      ct += moves;
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

//FLIP A SQUARE OVER THE MIDDLE RANK
#[inline(always)]
pub fn mirror_sq(sq: usize) -> usize {
    sq ^ 56
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
      // Calculate next position and check for wraparound BEFORE adding to attacks
      let nextpos: i8 = match diag {
        0 => { // northeast (up + right)
          let file = currpos % 8;
          if file == 7 { break; } // Already on h-file, can't go further right
          currpos + northeast
        },
        1 => { // northwest (up + left)
          let file = currpos % 8;
          if file == 0 { break; } // Already on a-file, can't go further left
          currpos + northwest
        },
        2 => { // southwest (down + left)
          let file = currpos % 8;
          if file == 0 { break; } // Already on a-file, can't go further left
          currpos + southwest
        },
        3 => { // southeast (down + right)
          let file = currpos % 8;
          if file == 7 { break; } // Already on h-file, can't go further right
          currpos + southeast
        },
        _ => unreachable!(),
      };
      
      // Check if next position is out of bounds
      if nextpos < 0 || nextpos >= 64 { break; }
      
      // Check for file wraparound by comparing old/new file positions
      let old_file = currpos % 8;
      let new_file = nextpos % 8;
      if diag == 0 || diag == 3 { // northeast, southeast - moving right
        if new_file < old_file { break; } // wrapped to left
      } else { // northwest, southwest - moving left
        if new_file > old_file { break; } // wrapped to right
      }
      
      currpos = nextpos;
      let step_bb = genShift(currpos, 1u64);
      
      // If this square has a blocker, include it (capture square) then stop
      if step_bb & blockers != 0 {
        attacks |= step_bb; // attacker can capture the blocker
        break;
      }
      // Add this square to attacks (no blocker present)
      attacks |= step_bb;
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
      // Calculate next position and check for wraparound
      let nextpos: i8 = match ortho {
        0 => { // north (up): moving through ranks towards rank 8
          let rank = currpos / 8;
          if rank == 7 { break; } // Already on rank 8, can't go higher
          currpos + north
        },
        1 => { // west (left): moving through files towards a-file
          let file = currpos % 8;
          if file == 0 { break; } // Already on a-file, can't go further left
          currpos + west
        },
        2 => { // south (down): moving through ranks towards rank 1
          let rank = currpos / 8;
          if rank == 0 { break; } // Already on rank 1, can't go lower
          currpos + south
        },
        3 => { // east (right): moving through files towards h-file
          let file = currpos % 8;
          if file == 7 { break; } // Already on h-file, can't go further right
          currpos + east
        },
        _ => unreachable!(),
      };
      
      // Check if next position is out of bounds
      if nextpos < 0 || nextpos >= 64 { break; }
      
      // Check for file wraparound on horizontal moves
      if ortho == 1 || ortho == 3 { // west or east
        let old_file = currpos % 8;
        let new_file = nextpos % 8;
        if ortho == 3 && new_file < old_file { break; } // east but wrapped left
        if ortho == 1 && new_file > old_file { break; } // west but wrapped right
      }
      
      currpos = nextpos;
      let step_bb = genShift(currpos, 1u64);
      
      // If this square has a blocker, include it (capture square) then stop
      if step_bb & blockers != 0 {
        attacks |= step_bb; // attacker can capture the blocker
        break;
      }
      // Add this square to attacks (no blocker present)
      attacks |= step_bb;
    }
  }
  attacks
}