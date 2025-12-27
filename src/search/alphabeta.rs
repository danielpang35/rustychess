use crate::core::{Board, Move, movegen::MoveGenerator};
use crate::evaluate::evaluate;
use crate::search::Search;
pub fn alphabeta(search: &mut Search, board: &mut Board, depth: u8, generator: &MoveGenerator, mut alpha: i32, beta: i32) -> i32 {
    //update search
    search.nodes += 1;
    if depth == 0 {
        return evaluate::evaluate(board);
    }   
    
    let moves = generator.generate(board);
    if moves.is_empty() {
        // check for checkmate or stalemate
        if generator.in_check(board) {
            return -99999; // checkmate score
        } else {
            return 0; // stalemate
        }
    }
    for m in moves {
        board.push(m, &generator);
        let score = -alphabeta(search, board, depth - 1, generator, -beta, -alpha);
        board.pop();

        if score >= beta {
            return beta; // Beta cutoff
        }
        alpha = alpha.max(score);
    }
    alpha

}

