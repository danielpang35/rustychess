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
    *bitboard &= *bitboard - 1;
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

pub fn squaretouci(square: u8) -> String {
  if square >= 64 {
      panic!("Square number must be between 0 and 63");
  }
  let file = (square % 8) + b'a';
  let rank = (square / 8) + 1 + b'0';
  format!("{}{}", file as char, rank as char)
}

