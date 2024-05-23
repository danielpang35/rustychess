pub enum CastlingRights {
    NoCastling = 0b0000,
    WKingside = 0b0001,
    WQueenside = 0b0010,
    BKingside = 0b0100,
    BQueenside = 0b1000,
}
impl CastlingRights {
}
pub fn wkingside(rights: u8) -> bool {
    println!("RIGHTS:{}",rights);
    unsafe {
        if (rights & CastlingRights::WKingside as u8) != 0 {
            return true
        } else {
            return false
        }
    }
}
pub fn wqueenside(rights: u8) -> bool {
    println!("RIGHTS:{}",rights);
    unsafe {
        if (rights & CastlingRights::WQueenside as u8) != 0 {
            return true
        } else {
            return false
        }
    }
}
pub fn bkingside(rights: u8) -> bool {
    println!("RIGHTS:{}",rights);
    unsafe {
        if (rights & CastlingRights::BKingside as u8) != 0 {
            return true
        } else {
            return false
        }
    }
}
pub fn bqueenside(rights: u8) -> bool {
    println!("RIGHTS:{}",rights);
    unsafe {
        if (rights & CastlingRights::BQueenside as u8) != 0 {
            return true
        } else {
            return false
        }
    }
}
pub fn get_castling_mask(string: &str) -> u8 {
    let mut mask = 0;
    for c in string.chars() {
        let right = match c {
            'K' => CastlingRights::WKingside,
            'Q' => CastlingRights::WQueenside,
            'k' => CastlingRights::BKingside,
            'q' => CastlingRights::BQueenside,
            '-' => CastlingRights::NoCastling,
            _ => panic!("BAD CASTLING"),
        };
        mask |= right as u8;
    }
    return mask;
}

