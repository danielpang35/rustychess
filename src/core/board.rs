use crate::core::piece;
mod piece;
pub struct Board {
    pieces: u64,
}
impl Board
    {
    //constructor
    pub fn new() -> Self
    {
        Self{pieces: 3}
    }
    
    pub fn piece_at(self, rank: i32, file: i32) -> Piece {
        //given rank and file
        let bitmask: u64 = 1 << (rank * 8 + file);
        if self.pieces << (rank*8 + file) == 1{
            return true;
        } else {
            return false;
        }
    }
    pub fn toStr(self) -> String
    {
        let mut s = String::from("+---+---+---+---+---+---+---+---+\n");
        for r in 7..0
        {
            for f in 0..7
            { 
                s.insert_str(f,ifself.piece_at(r,f){"1"}else{" "});
            }
            s.push_str("\n");
            


        }
        s.push_str( "  a   b   c   d   e   f   g   h\n");

        return s;
    }
}
