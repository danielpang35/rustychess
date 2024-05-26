use crate::core::constlib;
pub enum CastlingRights {
    NoCastling = 0b0000,
    WKingside = 0b0001,
    WQueenside = 0b0010,
    BKingside = 0b0100,
    BQueenside = 0b1000,
}
impl CastlingRights {
}

pub fn get_rights(sq:u8) -> u8 {
    //this function takes a square and returns the appropriate castling right mask for that square
    //if any square which the rook or the king starts on is interacted with, those rights must be updated
    unsafe {
        match sq {
        //starting white rook squares
        sq if sq == constlib::square_from_string("a1") => CastlingRights::WQueenside as u8,
        sq if sq == constlib::square_from_string("h1") => CastlingRights::WKingside as u8,
        
        //starting black rook squares
        sq if sq == constlib::square_from_string("a8") => CastlingRights::BQueenside as u8,
        sq if sq == constlib::square_from_string("h8") => CastlingRights::BKingside as u8,
        
        //starting king squares
        sq if sq == constlib::square_from_string("e1") => CastlingRights::WKingside as u8 | CastlingRights::WQueenside as u8,
        sq if sq == constlib::square_from_string("e8") => CastlingRights::BKingside as u8 | CastlingRights::BQueenside as u8,

        _ => CastlingRights::NoCastling as u8,
    }
    }
}
pub fn oppressed(rights: u8) -> bool {
    //if there are no rights, the player is oppressed
    rights == 0
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

