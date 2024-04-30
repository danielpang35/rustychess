use std::collections::HashMap;

enum PieceType {
    Pawn:1,
    Knight:2,
    Bishop:3,
    Rook:4,
    Queen:5,
    King:6,
}

pub struct Piece {
    type: u8,
}
impl Piece
{
    pub fn new(name, color)->Piece
    {
        Self{type: PieceType(name) + (color << 3) }
    }
}
