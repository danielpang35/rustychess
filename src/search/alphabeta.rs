use crate::core::{Board, Move, movegen::MoveGenerator, PieceIndex};
use crate::evaluate::evaluate;
use crate::search::Search;
use crate::search::tt::{TT_EMPTY, TT_EXACT, TT_LOWER, TT_UPPER};

const Mate: i32
    = 99999;

const MATE_WINDOW: i32
    = 90000; // centipawns
pub fn alphabeta(search: &mut Search, board: &mut Board, depth: u8, generator: &MoveGenerator, mut alpha: i32, beta: i32) -> i32 {
    //update search
    search.nodes += 1;
    let key = board.state.hash;
    let ply = board.ply;
    let alpha0 = alpha;
    let mut best_move = Move::new();

    search.tt_probes += 1;
    let entry = search.tt.probe(key);
    if entry.flag != TT_EMPTY && entry.key == key && entry.depth >= depth {
        search.tt_hits += 1;
        let tt_score = score_from_tt(entry.score, ply);
        match entry.flag {
            TT_EXACT => {   
                            search.tt_exact += 1;
                            return tt_score},
            TT_LOWER => {
                if tt_score >= beta {                     
                            search.tt_cut_lower += 1;
                            return tt_score; }
                if tt_score > alpha { alpha = tt_score; }
            }
            TT_UPPER => {
                if tt_score <= alpha {                     
                            search.tt_cut_upper += 1;
                            return tt_score; }
            }
            _ => {}
        }
        if alpha >= beta {
            return tt_score;
        }
    }
    if depth == 0 {
        return qsearch(search, board, generator, alpha, beta, 0);
    }   
    
    let mut moves = generator.generate(board);
    if moves.is_empty() {
        // check for checkmate or stalemate
        if generator.in_check(board) {
            return -99999 + board.ply as i32; // checkmate score
        } else {
            return 0; // stalemate
        }
    }
    
    let node_ply = board.ply as usize;
    let in_check = generator.in_check(board);
    let mut depth = depth;
    if in_check && depth < 15 {
        depth += 1; // check extension
    }

    if entry.flag != TT_EMPTY && entry.key == key && entry.best != 0 {
        let bm = Move::from_u16(entry.best);
        if let Some(pos) = moves.iter().position(|&m| m == bm) {
            moves.swap(0, pos);
            search.tt_move_used += 1;
        }
    }
    search.order_moves_range(&mut moves, board, node_ply);
    for (i, m) in moves.iter().copied().enumerate() {
        board.push(m, &generator);
        search.debug_after_push(board, generator, m);

        let mut score: i32;
        

        // --- LMR: late quiet moves searched at reduced depth first ---
        if depth >= 4 && !in_check && m.isquiet() && i>=4{
            
            // "Gives check?" using cached attacked (essentially free).
            let kingidx = if board.turn == 0 { PieceIndex::K.index() } else { PieceIndex::k.index() };
            let gives_check = (board.state.attacked[board.turn as usize] & board.pieces[kingidx]) != 0;

            if gives_check {
                // Never reduce checking moves.
                score = -alphabeta(search, board, depth - 1, generator, -beta, -alpha);
            } else {
                // Reduced-depth search
                score = -alphabeta(search, board, depth - 2, generator, -beta, -alpha);
                search.lmr_reductions += 1;
                // Re-search if it looks interesting
                if score > alpha - 25 {
                    score = -alphabeta(search, board, depth - 1, generator, -beta, -alpha);
                    search.lmr_researches += 1;
                }
            }
        } else {
            score = -alphabeta(search, board, depth - 1, generator, -beta, -alpha);
        }


        board.pop();

        if score >= beta {
            search.store_killer(node_ply, m);
            search.store_history_cutoff(m, depth);
            search.tt.store(
                key,
                depth,
                TT_LOWER,
                score_to_tt(score, ply),
                m.as_u16(),
            );
            return score;
        }
        if score > alpha {
            alpha = score;
            best_move = m;
        }
    }

    let flag = if alpha <= alpha0 { TT_UPPER } else { TT_EXACT };
        search.tt.store(
            key,
            depth,
            flag,
            score_to_tt(alpha, ply),
            best_move.as_u16(),
        );

    alpha

}

#[inline(always)]
fn qsearch(
    search: &mut Search,
    board: &mut Board,
    generator: &MoveGenerator,
    mut alpha: i32,
    beta: i32,
    qply: u8,
) -> i32 {
    search.qnodes += 1;
    const QPLY_MAX: u8 = 8;
    if qply >= QPLY_MAX {
        return evaluate(board, generator);
    }

    // If we're in check, we must search evasions; stand-pat is illegal.
    let in_check = generator.in_check(board);
    if !in_check {
        let stand_pat = evaluate(board, generator);
        // If we are so far below alpha that even winning a queen can't help, prune.
        const DELTA: i32 = 600; // queen value in your eval units (centipawns)
        if stand_pat + DELTA < alpha {
            return alpha;
        }
        if stand_pat >= beta {
            return stand_pat; // fail-soft
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
    }

    // Generate moves:
    // - if not in check: captures only (plus optionally promotions)
    // - if in check: all legal moves (evasions)
    let mut moves = generator.generate(board);

    if moves.is_empty() {
        // No legal moves: mate/stalemate
        if in_check {
            return -99999 + board.ply as i32;
        } else {
            return 0;
        }
    }

    if !in_check {

        // Filter down to tactical moves only.
        // Captures are mandatory; including promotions is recommended (cheap and important).
        moves.retain(|m| m.iscapture() || m.isprom());
        if moves.is_empty() {
            // Quiet position: stand_pat already returned.
            return alpha;
        }
    }

    // Order tacticals for qsearch (reuse your existing ordering logic)
    // NOTE: ply is still board.ply at this node.
    search.order_moves(&mut moves, board, board.ply as usize);

    for m in moves {
        board.push(m, generator);

        let score = -qsearch(search, board, generator, -beta, -alpha, qply + 1);

        board.pop();

        if score >= beta {
            return score; // fail-soft
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}



#[inline(always)]
fn score_to_tt(score: i32, ply: u16) -> i32 {
    if score > MATE_WINDOW {
        score + ply as i32
    } else if score < -MATE_WINDOW {
        score - ply as i32
    } else {
        score
    }
}

#[inline(always)]
fn score_from_tt(score: i32, ply: u16) -> i32 {
    if score > MATE_WINDOW {
        score - ply as i32
    } else if score < -MATE_WINDOW {
        score + ply as i32
    } else {
        score
    }
}