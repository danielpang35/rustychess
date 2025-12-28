use crate::core::{Board, Move, movegen::MoveGenerator, PieceIndex};
use crate::search::alphabeta::alphabeta;

pub struct Search {
    pub nodes: u64,
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
            self.debug_after_push(board, mg, m);
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

    pub fn search_iterative(
        &mut self,
        board: &mut Board,
        max_depth: u8,
        mg: &MoveGenerator,
    ) -> Move {
        const INF: i32 = 30_000;

        let mut pv: Option<Move> = None;          // previous iteration best move

        for depth in 1..=max_depth {
            self.nodes = 0;
            let mut moves = mg.generate(board);
            let mut best_score = -10000;
            let mut best_move = Move::new();    

            if let Some(prev) = pv {
                Self::pv_first(&mut moves, &prev);
            }   

            for m in moves {
                board.push(m, &mg);
                self.debug_after_push(board, mg, m);
                let score = -alphabeta(self, board,depth - 1, mg,-10000, 10000);
                board.pop();
                if score > best_score {
                    best_move = m;
                    best_score = score;
                }
            }
            pv = Some(best_move);
        }

        match pv {
            Some(x) => {return x}

            None => {
                return Move::new();
            }
        }
        


        }

    


    #[cfg(debug_assertions)]
    #[inline(always)]
    pub(crate) fn debug_after_push(&self, board: &mut Board, mg: &MoveGenerator, m: Move) {
        // 1) King presence sanity: exactly one white king and one black king
        debug_assert_eq!(
            board.pieces[PieceIndex::K.index()].count_ones(),
            1,
            "White king count != 1 after move {}->{}",
            m.getSrc(),
            m.getDst()
        );
        debug_assert_eq!(
            board.pieces[PieceIndex::k.index()].count_ones(),
            1,
            "Black king count != 1 after move {}->{}",
            m.getSrc(),
            m.getDst()
        );

        // 2) Move legality sanity: mover did NOT leave their own king in check
        //
        // After push(), board.turn has already flipped to the opponent.
        // To test whether the mover's king is in check, temporarily flip turn
        // back to the mover and reuse mg.in_check().
        let saved_turn = board.turn;
        board.turn = if saved_turn == 0 { 1 } else { 0 };

        let mover_in_check = mg.in_check(board);

        board.turn = saved_turn;

        debug_assert!(
            !mover_in_check,
            "Illegal move (left mover king in check): {}->{}",
            m.getSrc(),
            m.getDst()
        );
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub(crate) fn debug_after_push(&self, _board: &mut Board, _mg: &MoveGenerator, _m: Move) {
        // compiled out in release
    }
    //helper function which swaps the best move to the front if it exists
    fn pv_first<T: PartialEq>(moves: &mut [T], pv: &T) {
    if let Some(i) = moves.iter().position(|m| m == pv) {
        moves.swap(0, i);
    }
}

}