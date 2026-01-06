use crate::core::{movegen::MoveGenerator, Board, Move, Piece, PieceIndex, PieceType};
use crate::evaluate::nnue::Nnue;
use crate::evaluate::{evaluate, evaluate_neural, evaluate_neural_fast};
use crate::perf;
use crate::search::alphabeta::alphabeta;
use crate::search::tt::TranspositionTable;

const MAX_PLY: usize = 128;

pub struct Search {
    pub nodes: u64,
    pub qnodes: u64,
    pub lmr_reductions: u64,
    pub lmr_researches: u64,
    pub pvs_researches: u64,
    pub asp_fail_low: u64,
    pub asp_fail_high: u64,
    killers: [[Move; 2]; MAX_PLY],
    history: [[i32; 64]; 64],

    pub tt_probes: u64,
    pub tt_hits: u64,
    pub tt_key_hits: u64,
    pub tt_cutoffs: u64,
    pub tt_exact: u64,
    pub tt_cut_lower: u64,
    pub tt_cut_upper: u64,
    pub tt_move_used: u64,
    pub tt: TranspositionTable,

    pub use_nnue: bool,
    pub nnue: Nnue,
}

impl Search {
    pub fn new(use_nnue: bool) -> Self {
        let null = Move::new();
        let killers = std::array::from_fn(|_| [null; 2]);
        let history = [[0i32; 64]; 64];
        Self {
            nodes: 0,
            qnodes: 0,
            lmr_reductions: 0,
            lmr_researches: 0,
            pvs_researches: 0,
            asp_fail_low: 0,
            asp_fail_high: 0,
            killers,
            history,
            tt: TranspositionTable::new_mb(128),
            tt_probes: 0,
            tt_hits: 0,
            tt_key_hits: 0,
            tt_cutoffs: 0,
            tt_exact: 0,
            tt_cut_lower: 0,
            tt_cut_upper: 0,
            tt_move_used: 0,
            use_nnue,
            nnue: Nnue::load("data/processed/nnue.bin").expect("failed to load NNUE file"),
        }
    }

    #[inline(always)]
    pub fn eval(&self, board: &Board, mg: &MoveGenerator) -> i32 {
        if self.use_nnue {
            evaluate_neural(board, &self.nnue)
        } else {
            evaluate(board, mg)
        }
    }

    #[inline(always)]
    pub fn eval_fast(&self, board: &Board, mg: &MoveGenerator) -> i32 {
        if self.use_nnue {
            evaluate_neural_fast(board, &self.nnue)
        } else {
            evaluate(board, mg)
        }
    }

    pub fn search_root_yes(
        &mut self,
        board: &mut Board,
        depth: u8,
        mg: &MoveGenerator,
    ) -> (Move, i32) {
        let moves = mg.generate(board);
        let mut best_score = -30000;
        let mut best_move = Move::new();
        for m in moves {
            board.push(m, &mg, &self.nnue);
            //self.debug_after_push(board, mg, m);
            let score = -alphabeta(self, board, depth - 1, mg, -30000, 30000);
            board.pop(mg, &self.nnue);
            if score > best_score {
                best_move = m;
                best_score = score;
            }
        }
        println!("Best move found with score {}", best_score);
        best_move.print();
        (best_move, best_score)
    }
    pub fn search_root(&mut self, board: &mut Board, depth: u8, mg: &MoveGenerator) -> (Move, i32) {
        return self.search_iterative(board, depth, mg);
    }

    pub fn search_iterative(
        &mut self,
        board: &mut Board,
        max_depth: u8,
        mg: &MoveGenerator,
    ) -> (Move, i32) {
        const INF: i32 = 30_000;

        perf::reset();

        let mut pv: Option<Move> = None;
        let mut prev_score: i32 = 0;

        let window: i32 = 25;

        let mut final_best = Move::new();
        let mut final_score = 0;

        for depth in 1..=max_depth {
            self.nodes = 0;

            // Root search runner (kept inside this function).
            // Runs ONE root search at this depth with the provided bounds.
            let mut run_root = |mut alpha: i32, beta: i32, pv: Option<Move>| -> (Move, i32) {
                let mut moves = mg.generate(board);

                // Handle mate/stalemate at root cleanly
                if moves.is_empty() {
                    let score = if mg.in_check(board) {
                        -99999 + board.ply as i32
                    } else {
                        0
                    };
                    return (Move::new(), score);
                }

                // PV-first
                if let Some(prev) = pv {
                    Self::pv_first(&mut moves, &prev);
                }
                // Keep PV at index 0: order only the tail
                if moves.len() > 1 {
                    self.order_moves_range(&mut moves[1..], board, board.ply as usize);
                }

                let mut best_move = Move::new();
                let mut best_score = -INF;

                for (i, m) in moves.iter().copied().enumerate() {
                    board.push(m, mg, &self.nnue);

                    let score = if i == 0 {
                        // First move: full window
                        -alphabeta(self, board, depth - 1, mg, -beta, -alpha)
                    } else {
                        // PVS: null-window first
                        let mut s = -alphabeta(self, board, depth - 1, mg, -(alpha + 1), -alpha);
                        if s > alpha {
                            // Re-search full window if it looks better
                            s = -alphabeta(self, board, depth - 1, mg, -beta, -alpha);
                            self.pvs_researches += 1;
                        }
                        s
                    };

                    board.pop(mg, &self.nnue);

                    if score > best_score {
                        best_score = score;
                        best_move = m;
                    }
                    if score > alpha {
                        alpha = score;
                    }
                    if alpha >= beta {
                        break; // root cutoff
                    }
                }

                (best_move, best_score)
            };

            // --- Aspiration window attempt #1 ---
            let a0 = prev_score - window;
            let b0 = prev_score + window;

            let (mut best_move, mut best_score) = run_root(a0, b0, pv);

            // --- If failed, widen and re-search once ---
            if best_score <= a0 {
                // fail-low
                (best_move, best_score) = run_root(-INF, b0, pv);
                self.asp_fail_low += 1;
            } else if best_score >= b0 {
                // fail-high
                (best_move, best_score) = run_root(a0, INF, pv);
                self.asp_fail_high += 1;
            }

            pv = Some(best_move);
            prev_score = best_score;

            final_best = best_move;
            final_score = best_score;
            println!("Searched to depth {}: PV: ", depth);
            final_best.print();
        }
        println!(
            "nodes={} qnodes={} lmr_red={} lmr_re={} pvs_re={} aspL={} aspH={}",
            self.nodes,
            self.qnodes,
            self.lmr_reductions,
            self.lmr_researches,
            self.pvs_researches,
            self.asp_fail_low,
            self.asp_fail_high
        );
        println!(
    "TT: probes={} hits={}  ({:.1}%) key_hits={} tt_cutoff={} exact={} cutL={} cutU={} move_used={} ",
    self.tt_probes,
    self.tt_hits,
    (self.tt_hits as f64 * 100.0) / self.tt_probes.max(1) as f64,
    self.tt_key_hits,
    self.tt_cutoffs,
    self.tt_exact,
    self.tt_cut_lower,
    self.tt_cut_upper,
    self.tt_move_used,
);

        let snapshot = perf::snapshot();
        perf::print_snapshot("Performance metrics", snapshot);
        (final_best, final_score)
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
    pub fn pv_first<T: PartialEq>(moves: &mut [T], pv: &T) {
        if let Some(i) = moves.iter().position(|m| m == pv) {
            moves.swap(0, i);
        }
    }
    #[inline(always)]
    pub(crate) fn order_moves(&self, moves: &mut Vec<Move>, board: &Board, ply: usize) {
        self.order_moves_range(moves, board, ply);
    }

    #[inline(always)]
    pub(crate) fn order_moves_range(&self, moves: &mut [Move], board: &Board, ply: usize) {
        if moves.len() <= 1 {
            return;
        }

        // 1) Partition tacticals
        let mut tact_end = 0;
        for i in 0..moves.len() {
            let m = moves[i];
            if m.iscapture() || m.isprom() {
                moves.swap(i, tact_end);
                tact_end += 1;
            }
        }

        // 2) Sort tacticals
        if tact_end > 1 {
            moves[..tact_end].sort_unstable_by(|a, b| {
                let sa = Self::tactical_score(*a, board);
                let sb = Self::tactical_score(*b, board);
                sb.cmp(&sa)
            });
        }

        // 3) Promote killers
        if ply < MAX_PLY {
            let quiet_start = tact_end;
            let k0 = self.killers[ply][0];
            let k1 = self.killers[ply][1];
            Self::bring_killer_forward(moves, quiet_start, k0);
            Self::bring_killer_forward(moves, quiet_start + 1, k1);
        }

        // 4) History ordering for quiet moves
        let quiet_start = tact_end;
        if quiet_start + 1 < moves.len() {
            moves[quiet_start..].sort_unstable_by(|a, b| {
                let ha = self.history[a.getSrc() as usize][a.getDst() as usize];
                let hb = self.history[b.getSrc() as usize][b.getDst() as usize];
                hb.cmp(&ha)
            });
        }
    }

    #[inline(always)]
    fn bring_killer_forward(moves: &mut [Move], start: usize, killer: Move) {
        if start >= moves.len() {
            return;
        }
        // Treat Move::new() as "null move" sentinel.
        if killer == Move::new() {
            return;
        }
        // Promote only if it is quiet *in this position*.
        if let Some(i) = moves[start..]
            .iter()
            .position(|&m| m == killer && m.isquiet())
        {
            moves.swap(start, start + i);
        }
    }

    #[inline(always)]
    fn tactical_score(m: Move, board: &Board) -> i32 {
        let mover = board.piecelocs.piece_at(m.getSrc());
        let mover_v = Self::piece_value(mover.get_piece_type());

        let mut cap_v = 0;
        if m.iscapture() {
            if m.isep() {
                cap_v = Self::piece_value(PieceType::P);
            } else {
                let cap = board.piecelocs.piece_at(m.getDst());
                cap_v = Self::piece_value(cap.get_piece_type());
            }
        }

        let mut prom_v = 0;
        if m.isprom() {
            prom_v = Self::piece_value(m.prompiece());
        }

        (cap_v * 100) + (prom_v * 10) - mover_v
    }

    #[inline(always)]
    pub(crate) fn store_killer(&mut self, ply: usize, m: Move) {
        if ply >= MAX_PLY {
            return;
        }
        // Only quiet moves are killers.
        if !m.isquiet() {
            return;
        }
        if self.killers[ply][0] == m {
            return;
        }
        self.killers[ply][1] = self.killers[ply][0];
        self.killers[ply][0] = m;
    }

    #[inline(always)]
    pub(crate) fn store_history_cutoff(&mut self, m: Move, depth: u8) {
        if !m.isquiet() {
            return;
        }
        let from = m.getSrc() as usize;
        let to = m.getDst() as usize;

        // Depth-squared weighting: deeper cutoffs matter more.
        let d = depth as i32;
        self.history[from][to] = self.history[from][to].saturating_add(d * d);

        // Optional: very cheap decay to prevent runaway growth (rarely needed early).
        if (self.nodes & 0x3FFF) == 0 {
            self.history[from][to] >>= 1;
        }
    }

    #[inline(always)]
    fn piece_value(pt: PieceType) -> i32 {
        match pt {
            PieceType::P => 100,
            PieceType::N => 320,
            PieceType::B => 330,
            PieceType::R => 500,
            PieceType::Q => 900,
            PieceType::K => 20_000,
            PieceType::NONE => 0,
        }
    }
}
