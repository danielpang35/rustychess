pub enum CastlingRights {
    NoCastling = 0b0000,
    WKingside = 0b0001,
    WQueenside = 0b0010,
    BKingside = 0b0100,
    BQueenside = 0b1000,
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

