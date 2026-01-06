use crate::core::{Board, Move, movegen::MoveGenerator, PieceIndex, constlib};
use crate::evaluate::{evaluate,evaluate_neural};
use crate::search::Search;
use crate::search::tt::{TT_EMPTY, TT_EXACT, TT_LOWER, TT_UPPER};


const Mate: i32
    = 99999;

const MATE_WINDOW: i32
    = 90000; // centipawns
pub fn alphabeta(search: &mut Search, board: &mut Board, depth: u8, generator: &MoveGenerator, mut alpha: i32, beta: i32) -> i32 {
    //update search
    search.nodes += 1;
    let key = board.hash;
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
            search.tt_cutoffs += 1;
            return tt_score;
        }
    }
    if depth == 0 {
        return qsearch(search, board, generator, alpha, beta, 0);
    }   
    
    let mut moves = generator.generate(board);
    let in_check = generator.in_check(board);

    if moves.is_empty() {
        // check for checkmate or stalemate
        if in_check {
            return -99999 + board.ply as i32; // checkmate score
        } else {
            return 0; // stalemate
        }
    }
    
    let node_ply = board.ply as usize;
    let mut depth = depth;
    if in_check && depth < 15 {
        depth += 1; // check extension
    }
    //get the transposition table move regardless of depth and put it first.
    let tt_move = if entry.flag != TT_EMPTY { entry.best } else { 0 };
    if tt_move != 0 {
        Search::pv_first(&mut moves, &Move::from_u16(tt_move));
    }
    if entry.flag != TT_EMPTY && entry.key == key && entry.best != 0 {
        search.tt_key_hits += 1;
        let bm = Move::from_u16(entry.best);
        if let Some(pos) = moves.iter().position(|&m| m == bm) {
            moves.swap(0, pos);
            search.tt_move_used += 1;
        }
    }
    search.order_moves_range(&mut moves, board, node_ply);
    for (i, m) in moves.iter().copied().enumerate() {
        let enemy = if board.turn == 0 { 1 } else { 0 };
        let enemy_king_idx = if enemy == 0 { PieceIndex::K.index() } else { PieceIndex::k.index() };
        let mut kingbb = board.pieces[enemy_king_idx];
        let enemy_king_sq = constlib::poplsb(&mut kingbb);

        if m.getDst() == enemy_king_sq as u8 {
            eprintln!("ABOUT TO PUSH KING-CAPTURE MOVE: from={} to={}", m.getSrc(), m.getDst());
            // board.print();
        }
        board.push(m, &generator, &search.nnue);
        search.debug_after_push(board, generator, m);

        let mut score: i32;
        

        // --- LMR: late quiet moves searched at reduced depth first ---

                // --- LMR: late quiet moves searched at reduced depth first ---
        if depth >= 4 && !in_check && m.isquiet() && i >= 4 {

            // Robust "gives check" (see Patch 3): compute attacks by the mover.
            // After push(), board.turn is the side-to-move (the opponent).
            // let kingidx = if board.turn == 0 { PieceIndex::K.index() } else { PieceIndex::k.index() };
            // let gives_check = (board.attacked[board.turn as usize] & board.pieces[kingidx]) != 0;
            let gives_check = generator.in_check(board);
            if gives_check {
                // Never reduce checking moves.
                score = -alphabeta(search, board, depth - 1, generator, -beta, -alpha);
            } else {
                // Reduced-depth NULL-WINDOW search (critical fix)
                // depth>=4 guarantees depth-2 is valid.
                score = -alphabeta(search, board, depth - 2, generator, -alpha - 1, -alpha);
                search.lmr_reductions += 1;

                // Re-search ONLY on fail-high (critical fix)
                if score > alpha {
                    score = -alphabeta(search, board, depth - 1, generator, -beta, -alpha);
                    search.lmr_researches += 1;
                }
            }
        } else {
            score = -alphabeta(search, board, depth - 1, generator, -beta, -alpha);
        }



        board.pop(generator, &search.nnue);

        if score >= beta {
            search.store_killer(node_ply, m);
            search.store_history_cutoff(m, depth);
            search.tt.store(
                key,
                depth,
                TT_LOWER,
                score_to_tt(beta, ply),
                m.as_u16(),
            );
            return beta;
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
        // return evaluate(board, generator);
        return evaluate_neural(board, &search.nnue);
    }

    // If we're in check, we must search evasions; stand-pat is illegal.
    let in_check = generator.in_check(board);
    let mut stand_pat_opt: Option<i32> = None;
    if !in_check {
        let stand_pat = evaluate_neural(board, &search.nnue);
        stand_pat_opt = Some(stand_pat);

        // If we are so far below alpha that even winning a queen can't help, prune.
        const DELTA: i32 = 900; // queen value in your eval units (centipawns)
        if stand_pat + DELTA < alpha {
            return alpha;
        }
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
    }

    // Generate moves:
    // - if not in check: captures only (plus optionally promotions)
    // - if in check: all legal moves (evasions)
    let mut moves = if in_check {
        generator.generate(board)          // evasions
    } else {
        generator.generate_qcaptures(board) // tactical only
    };

    if moves.is_empty() {
        if in_check {
            return -99999 + board.ply as i32;
        } else {
            return alpha; // no captures/promotions; stand_pat already handled alpha
        }
    }
    #[cfg(debug_assertions)]
    if !in_check {
        debug_assert!(moves.iter().all(|m| m.iscapture() || m.isprom()),
            "generate_qcaptures produced a non-tactical move");
    }
    // Order tacticals for qsearch (reuse your existing ordering logic)
    // NOTE: ply is still board.ply at this node.
    search.order_moves(&mut moves, board, board.ply as usize);

    for m in moves {
        // Per-move delta pruning (only when not in check).
        if let Some(stand_pat) = stand_pat_opt {
            let mut gain = 0;

            if m.iscapture() {
                if m.isep() {
                    gain += 100;
                } else {
                    let cap = board.piecelocs.piece_at(m.getDst());
                    if cap != crate::core::piece::Piece::None {
                        gain += piece_cp(cap.get_piece_type());
                    }
                }
            }

            if m.isprom() {
                // Promotion gain over the pawn.
                gain += piece_cp(m.prompiece()) - 100;
            }

            if stand_pat + gain + DELTA_MARGIN <= alpha {
                continue;
            }
        }
        
        if let Some(_stand_pat) = stand_pat_opt {

            if !in_check && qply >= 1 && m.iscapture() && !m.isprom() {
                let mover = board.piecelocs.piece_at(m.getSrc());
                let mover_val = piece_cp(mover.get_piece_type());

                let captured_val = if m.isep() {
                    100
                } else {
                    let cap = board.piecelocs.piece_at(m.getDst());
                    if cap != crate::core::piece::Piece::None {
                        piece_cp(cap.get_piece_type())
                    } else {
                        0
                    }
                };

                // GATE FIRST: only if capturing down do we pay for defense test.
                if mover_val > captured_val + BAD_CAP_MARGIN {
                    let us = board.turn;
                    let them = us ^ 1;
                    let dst = m.getDst();


                    let src_bb = 1u64 << (m.getSrc() as u64);
                    let dst_bb = 1u64 << (dst as u64);

                    // base: move piece src->dst
                    let mut occ_after = (board.occupied ^ src_bb) | dst_bb;

                    // if normal capture, remove captured piece on dst (it was included in occupied already)
                    // BUT since we set dst_bb, we don’t need to clear it; occupied already had dst_bb set.
                    // For sliders, the key change is src cleared, not dst cleared. Still, handle EP properly.

                    if m.isep() {
                        // EP removes the pawn behind the EP square.
                        // You must compute the actual captured pawn square.
                        // Typical: if us==0 (white capturing), captured pawn is dst-8; else dst+8.
                        let us = board.turn;
                        let cap_sq = if us == 0 { m.getDst() - 8 } else { m.getDst() + 8 };
                        occ_after &= !(1u64 << (cap_sq as u64));
                    }
                    // Targeted: is dst defended by the enemy, given current occ?
                    // If yes, this capture is very likely losing noise in qsearch.
                    if generator.is_square_attacked_by(board, occ_after, them, dst) {
                        continue;
                    }
                }
            }
        }
        board.push(m, generator, &search.nnue);

        let score = -qsearch(search, board, generator, -beta, -alpha, qply + 1);

        board.pop(generator, &search.nnue);

        if score >= beta {
            return beta; // fail-hard
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

    #[inline(always)]
    fn piece_cp(pt: crate::core::piece::PieceType) -> i32 {
        use crate::core::piece::PieceType::*;
        match pt {
            P => 100,
            N => 320,
            B => 330,
            R => 500,
            Q => 900,
            _ => 0,
        }
    }
    const DELTA_MARGIN: i32 = 50;
    const BAD_CAP_MARGIN: i32 = 120; // ~one pawn + change