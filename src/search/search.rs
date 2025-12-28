use crate::core::{Board, Move, movegen::MoveGenerator};
use crate::search::alphabeta::alphabeta;

pub struct Search {
    pub nodes: u8,
}

impl Search {
    pub fn new() -> Self {
        Self { nodes: 0}
    }

    pub fn search_root(&mut self, board: &mut Board, depth: u8, mg: &MoveGenerator) -> Move {
        let moves = mg.generate(board);
        let mut best_score = -10000;
        let mut best_move = Move::new();
        for m in moves {
            board.push(m, &mg);
            let score = -alphabeta(self, board,depth - 1, mg,-10000, 10000);
            board.pop();
            if score > best_score {
                best_move = m;
                best_score = score;
            }
        }
        println!("Best move found with score {}", best_score);
        best_move.print();
        best_move
        
    }

}