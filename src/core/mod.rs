pub mod castling;
pub mod movegen;
pub mod piece;
pub mod constlib;
pub mod r#move;
pub mod state;
pub mod cli;
pub mod zobrist;
#[cfg(test)]
pub mod tests;

pub use piece::Piece;
pub use piece::PieceLocations;
pub use piece::PieceType;
pub use piece::PieceIndex;
pub use r#move::Move;
pub use castling::CastlingRights;
pub use state::Undo;

pub use crate::evaluate::nnue::Nnue;
use crate::evaluate::nnue::{nnue_add_piece,nnue_sub_piece};
use crate::core::zobrist::{Z_PIECE_SQ, Z_SIDE, Z_CASTLING, Z_EP_FILE};
use crate::perf;
use std::time::Instant;

// a struct defining the physical aspects of the board
pub struct Board {
    pub occupied: u64,
    pub pieces: [u64; 12],
    pub playerpieces: [u64; 2],
    pub turn: u8,
    pub piecelocs: PieceLocations,

    // ---- true game state (previously stored in BoardState/Arc chain) ----
    pub castling_rights: u8,
    pub ep_square: u8,
    pub hash: u64,

    // ---- derived caches (kept for now to reduce blast radius) ----
    pub pinned: u64,
    pub pinners: u64,
    /// attacked[board.turn] == squares attacked by ENEMY (enemy-of-turn)
    pub attacked: [u64; 2],

    // ---- undo stack ----
    pub history: Vec<Undo>,

    pub ply: u16,
    pub nnue_acc_w: [i32; 256],
    pub nnue_acc_b: [i32; 256],
    pub nnue_inited: bool,
}

impl Board {
    pub fn new() -> Self {
        Self {
            occupied: 0,
            pieces: [0; 12],
            playerpieces: [0; 2],
            turn: 0,
            piecelocs: PieceLocations::new(),

            castling_rights: 0,
            ep_square: 64,
            hash: 0,

            pinned: 0,
            pinners: 0,
            attacked: [0; 2],

            history: Vec::new(),

            ply: 0,
            nnue_acc_b: [0; 256],
            nnue_acc_w: [0; 256],
            nnue_inited: false,
        }
    }


    pub fn clone_position(&self) -> Board {
        Board {
            occupied: self.occupied,
            pieces: self.pieces,
            playerpieces: self.playerpieces,
            turn: self.turn,
            piecelocs: self.piecelocs,

            castling_rights: self.castling_rights,
            ep_square: self.ep_square,
            hash: self.hash,

            pinned: self.pinned,
            pinners: self.pinners,
            attacked: self.attacked,

            history: Vec::new(),   // key point: new empty stack
            ply: self.ply,
            nnue_acc_w: self.nnue_acc_w,
            nnue_acc_b: self.nnue_acc_b,
            nnue_inited: self.nnue_inited,
        }
    }
    #[inline(always)]
    fn file_of(sq: u8) -> usize {
        (sq & 7) as usize
    }

    pub fn push(&mut self, bm: Move, movegen: &movegen::MoveGenerator, nnue: &Nnue) {
        let push_timer = Instant::now();
        let color = self.turn;
        let enemy = color ^ 1;

        let ep = bm.isep();
        let castle = bm.iscastle();
        let capture = bm.iscapture();
        let prom = bm.isprom();
        let dbpush = bm.isdoublepawn();

        let from = bm.getSrc();
        let to = bm.getDst();
        let piece = self.piecelocs.piece_at(from);
        assert!(piece != Piece::None);

        // ---- save undo (previous true state) ----
        if !self.nnue_inited {
            self.nnue_rebuild(nnue);
        }
        let mut undo = Undo::new(bm, self.castling_rights, self.ep_square, self.hash, self.nnue_acc_w, self.nnue_acc_b);

        // ---- incremental zobrist: start from previous hash and remove old EP/castling ----
        let old_castle = self.castling_rights;
        let old_ep = self.ep_square;
        let mut h = self.hash;
        if old_ep != 64 {
            h ^= Z_EP_FILE[Self::file_of(old_ep)];
        }
        h ^= Z_CASTLING[(old_castle & 0x0F) as usize];

        self.ply += 1;

        if castle {
            // apply_castling expects (king_src, rook_src) encoded in the move
            self.apply_castling(from as i8, to as i8);

            // hash update for castling (king + rook squares)
            let ksrc = from as i8;
            let rsrc = to as i8;
            let kingside = ksrc < rsrc;
            let (kdst, rdst) = if kingside { (ksrc + 2, ksrc + 1) } else { (ksrc - 2, ksrc - 1) };

            let king_idx = (6 * color as usize) + PieceIndex::K.index();
            let rook_idx = (6 * color as usize) + PieceIndex::R.index();

            h ^= Z_PIECE_SQ[king_idx][ksrc as usize];
            h ^= Z_PIECE_SQ[king_idx][kdst as usize];
            h ^= Z_PIECE_SQ[rook_idx][rsrc as usize];
            h ^= Z_PIECE_SQ[rook_idx][rdst as usize];

            // castling never creates an EP square
            self.ep_square = 64;
        } else {
            // ---- update occupied ----
            self.occupied ^= 1u64 << from;
            self.occupied |= 1u64 << to;

            // ---- update piece bitboards ----
            let pieceidx = piece.getidx();
            self.pieces[pieceidx] ^= 1u64 << from;
            self.pieces[pieceidx] |= 1u64 << to;

            // ---- capture handling ----
            if capture {
                let mut capsq = to;

                if ep {
                    // EP capture: captured pawn is behind the destination square
                    capsq = if color == 0 { to - 8 } else { to + 8 };
                    let captured_piece = Piece::make(enemy, PieceType::P);

                    // remove the captured pawn from board
                    self.occupied ^= 1u64 << capsq;
                    self.piecelocs.remove(capsq);

                    let capturedidx = (6 * enemy as usize) + PieceIndex::P.index();
                    self.pieces[capturedidx] ^= 1u64 << capsq;
                    self.playerpieces[enemy as usize] ^= 1u64 << capsq;
                    h ^= Z_PIECE_SQ[capturedidx][capsq as usize];

                    undo.captured_piece = captured_piece;
                    undo.captured_sq = capsq;
                } else {
                    let captured_piece = self.piecelocs.piece_at(to);
                    debug_assert!(captured_piece != Piece::None, "Capture move but dst is empty");
                    debug_assert!(captured_piece.get_piece_type() != PieceType::K, "Illegal king capture");

                    let capturedidx = captured_piece.getidx();

                    // remove captured piece
                    self.pieces[capturedidx] ^= 1u64 << capsq;
                    self.playerpieces[enemy as usize] ^= 1u64 << capsq;
                    self.piecelocs.remove(capsq);
                    h ^= Z_PIECE_SQ[capturedidx][capsq as usize];

                    undo.captured_piece = captured_piece;
                    undo.captured_sq = capsq;
                }
            }

            // ---- toggle our playerpieces ----
            self.playerpieces[color as usize] ^= (1u64 << from) | (1u64 << to);

            // ---- update piecelocs ----
            self.piecelocs.place(to, piece);
            self.piecelocs.remove(from);

            // ---- EP square update ----
            if piece.get_piece_type() == PieceType::P {
                if dbpush {
                    self.ep_square = if color == 0 { to - 8 } else { to + 8 };
                } else {
                    self.ep_square = 64;
                }

                if prom {
                    let prompiece = bm.prompiece().to_piece(color);
                    let promidx = prompiece.getidx();

                    self.pieces[promidx] |= 1u64 << to;
                    self.pieces[pieceidx] ^= 1u64 << to; // remove pawn from to
                    self.piecelocs.place(to, prompiece);

                    // hash: remove pawn at to, add promoted at to
                    h ^= Z_PIECE_SQ[pieceidx][to as usize];
                    h ^= Z_PIECE_SQ[promidx][to as usize];
                }
            } else {
                self.ep_square = 64;
            }

            // hash for moved piece (non-castle)
            h ^= Z_PIECE_SQ[pieceidx][from as usize];
            h ^= Z_PIECE_SQ[pieceidx][to as usize];
        }

        // ---- castling rights update (based on from/to squares) ----
        let updatecastlemask = !(castling::get_rights(to) | castling::get_rights(from));
        self.castling_rights &= updatecastlemask;

        // ---- side to move ----
        self.turn = enemy;
        h ^= Z_SIDE;

        // ---- incremental zobrist: add new castling/EP ----
        h ^= Z_CASTLING[(self.castling_rights & 0x0F) as usize];
        if self.ep_square != 64 {
            h ^= Z_EP_FILE[Self::file_of(self.ep_square)];
        }
        self.hash = h;

        // ---- update derived caches (kept for now) ----
        // let pin = movegen.getpinned(self);
        // self.pinned = pin.0;
        // self.pinners = pin.1;
        // self.attacked[self.turn as usize] = movegen.makeattackedmask(self, self.occupied);


        // ---- NNUE INCREMENTAL ACC update ----
        let nnue_timer = Instant::now();
        // Assumes self.nnue_acc_w / self.nnue_acc_b currently represent the PRE-move position
        // (we ensured initialization + created Undo snapshot before making changes).

        let mover_is_king = piece.get_piece_type() == PieceType::K;

        if castle || mover_is_king {
            // HalfKP depends on king square: easiest correct rule is rebuild on king moves / castling.
            self.nnue_rebuild(nnue);
        } else {
            let wk_sq = self.pieces[PieceIndex::K.index()].trailing_zeros() as usize;
            let bk_sq = self.pieces[6 + PieceIndex::K.index()].trailing_zeros() as usize;

            let mover_idx = piece.getidx();

            if prom {
                // Promotion: pawn(from) removed, promoted(to) added, plus capture removed if any.
                nnue_sub_piece(
                    nnue,
                    &mut self.nnue_acc_w,
                    &mut self.nnue_acc_b,
                    wk_sq,
                    bk_sq,
                    mover_idx,
                    from as usize,
                );

                if undo.captured_piece != Piece::None {
                    nnue_sub_piece(
                        nnue,
                        &mut self.nnue_acc_w,
                        &mut self.nnue_acc_b,
                        wk_sq,
                        bk_sq,
                        undo.captured_piece.getidx(),
                        undo.captured_sq as usize,
                    );
                }

                let prompiece = bm.prompiece().to_piece(color);
                nnue_add_piece(
                    nnue,
                    &mut self.nnue_acc_w,
                    &mut self.nnue_acc_b,
                    wk_sq,
                    bk_sq,
                    prompiece.getidx(),
                    to as usize,
                );
            } else {
                // Normal move: mover from->to
                nnue_sub_piece(
                    nnue,
                    &mut self.nnue_acc_w,
                    &mut self.nnue_acc_b,
                    wk_sq,
                    bk_sq,
                    mover_idx,
                    from as usize,
                );
                nnue_add_piece(
                    nnue,
                    &mut self.nnue_acc_w,
                    &mut self.nnue_acc_b,
                    wk_sq,
                    bk_sq,
                    mover_idx,
                    to as usize,
                );

                // Capture (including EP): remove captured at captured_sq
                if undo.captured_piece != Piece::None {
                    nnue_sub_piece(
                        nnue,
                        &mut self.nnue_acc_w,
                        &mut self.nnue_acc_b,
                        wk_sq,
                        bk_sq,
                        undo.captured_piece.getidx(),
                        undo.captured_sq as usize,
                    );
                }
            }
        }
        perf::record_push_nn_update(nnue_timer.elapsed());

        {
            let recomputed = Self::compute_hash(self); // rebuild from pieces + side + rights + ep
            assert_eq!(self.hash, recomputed, "Zobrist mismatch after push/pop");
        }

        
        // ---- finalize ----
        self.history.push(undo);

        #[cfg(debug_assertions)]
self.debug_validate();
        // ---------DEBUG ONLY MODE -=---------
        #[cfg(debug_assertions)]
        {
            // After push(), self.turn is side-to-move (stm). The mover is stm^1.
            let stm = self.turn;
            let mover = stm ^ 1;

            let saved_turn = self.turn;
            self.turn = mover;

            // If mover is in check after making the move, it was illegal.
            let checkers = movegen.getcheckers(self, self.occupied);

            self.turn = saved_turn;

            if checkers != 0 {
                eprintln!("ILLEGAL SELF-CHECK CREATED BY MOVE:");
                bm.print();
                self.print();
                eprintln!("checkers bb:");
                crate::core::constlib::print_bitboard(checkers);
                panic!("push() produced illegal position: mover king in check");
            }
        }

        perf::record_push(push_timer.elapsed());

    }

    pub fn pop(&mut self, movegen: &movegen::MoveGenerator, nnue: &Nnue) {
        let pop_timer = Instant::now();
        let undo = match self.history.pop() {
            Some(u) => u,
            None => {
                eprintln!("Board::pop called with empty history; ignoring");
                return;
            }
        };

        let bm = undo.mv;

        // In the post-move position, `turn` is the opponent of the mover.
        let color = self.turn ^ 1; // mover
        let enemy = self.turn;     // side who was to move after the move

        let castle = bm.iscastle();
        let capture = bm.iscapture();
        let prom = bm.isprom();

        let from = bm.getSrc();
        let to = bm.getDst();

        debug_assert!(
            self.pieces[PieceIndex::K.index()] != 0 && self.pieces[PieceIndex::k.index()] != 0,
            "King missing at pop() entry"
        );

        if castle {
            self.undo_castling(from as i8, to as i8);
        } else if prom {
            // Promotion undo:
            // 1) remove promoted piece from `to`
            let prom_piece = self.piecelocs.piece_at(to);
            let promidx = prom_piece.getidx();
            self.pieces[promidx] ^= 1u64 << to;
            self.piecelocs.remove(to);

            // 2) restore pawn at `from`
            let pawn = Piece::make(color, PieceType::P);
            let pawnidx = pawn.getidx();
            self.pieces[pawnidx] |= 1u64 << from;
            self.piecelocs.place(from, pawn);

            // 3) occupied and playerpieces treat this like a normal from<->to move
            self.occupied ^= (1u64 << from) | (1u64 << to);
            self.playerpieces[color as usize] ^= (1u64 << from) | (1u64 << to);

            // 4) restore capture (promotion-capture is possible; EP is not)
            if capture {
                debug_assert!(undo.captured_sq != 64 && undo.captured_piece != Piece::None);
                let capsq = undo.captured_sq;
                let captured = undo.captured_piece;
                let capturedidx = captured.getidx();

                self.pieces[capturedidx] |= 1u64 << capsq;
                self.playerpieces[enemy as usize] |= 1u64 << capsq;
                self.piecelocs.place(capsq, captured);
                self.occupied |= 1u64 << capsq;
            }
        } else {
            // Generic non-castle, non-promotion undo
            let piece = self.piecelocs.piece_at(to);
            debug_assert!(piece != Piece::None);
            debug_assert!(piece.get_color() == color);

            self.occupied ^= (1u64 << from) | (1u64 << to);

            self.piecelocs.remove(to);
            self.piecelocs.place(from, piece);

            let pieceidx = piece.getidx();
            self.pieces[pieceidx] ^= (1u64 << to) | (1u64 << from);

            if capture {
                debug_assert!(undo.captured_sq != 64 && undo.captured_piece != Piece::None);
                let capsq = undo.captured_sq;
                let captured = undo.captured_piece;
                let capturedidx = captured.getidx();

                self.pieces[capturedidx] |= 1u64 << capsq;
                self.playerpieces[enemy as usize] |= 1u64 << capsq;
                self.piecelocs.place(capsq, captured);
                self.occupied |= 1u64 << capsq;
            }

            self.playerpieces[color as usize] ^= (1u64 << from) | (1u64 << to);
        }

        // Restore true state
        self.castling_rights = undo.castling_rights;
        self.ep_square = undo.ep_square;
        self.hash = undo.hash;

        // Restore side-to-move and ply
        self.turn = color;
        self.ply -= 1;


        self.nnue_acc_w = undo.nnue_acc_w;
        self.nnue_acc_b = undo.nnue_acc_b;
        self.nnue_inited = true;
        // Refresh derived caches
        // let pin = movegen.getpinned(self);
        // self.pinned = pin.0;
        // self.pinners = pin.1;
        // self.attacked[self.turn as usize] = movegen.makeattackedmask(self, self.occupied);
                #[cfg(debug_assertions)]
self.debug_validate();
        {
            debug_assert_eq!(self.hash, Self::compute_hash(self));
            self.assert_kings_present();
        }

        perf::record_pop(pop_timer.elapsed());

    }

    pub fn apply_castling(&mut self, ksrc: i8, rsrc: i8) {
        let kingside = ksrc < rsrc;
        let us = self.turn;
        let (kdst, rdst) = if kingside { (ksrc + 2, ksrc + 1) } else { (ksrc - 2, ksrc - 1) };

        let kingidx = 6 * us as usize + PieceIndex::K.index();
        let rookidx = 6 * us as usize + PieceIndex::R.index();

        self.pieces[kingidx] ^= (1u64 << ksrc) | (1u64 << kdst);
        self.pieces[rookidx] ^= (1u64 << rsrc) | (1u64 << rdst);

        self.occupied ^= (1u64 << ksrc) | (1u64 << kdst) | (1u64 << rsrc) | (1u64 << rdst);

        let king = self.piecelocs.piece_at(ksrc as u8);
        let rook = self.piecelocs.piece_at(rsrc as u8);

        self.piecelocs.remove(ksrc as u8);
        self.piecelocs.place(kdst as u8, king);

        self.piecelocs.remove(rsrc as u8);
        self.piecelocs.place(rdst as u8, rook);

        self.playerpieces[us as usize] ^= (1u64 << ksrc) | (1u64 << kdst) | (1u64 << rsrc) | (1u64 << rdst);
    }

    pub fn undo_castling(&mut self, ksrc: i8, rsrc: i8) {
        let kingside = ksrc < rsrc;
        let us = self.turn;
        let enemy = us ^ 1;
        let (kdst, rdst) = if kingside { (ksrc + 2, ksrc + 1) } else { (ksrc - 2, ksrc - 1) };

        let kingidx = 6 * enemy as usize + PieceIndex::K.index();
        let rookidx = 6 * enemy as usize + PieceIndex::R.index();

        self.pieces[kingidx] ^= (1u64 << ksrc) | (1u64 << kdst);
        self.pieces[rookidx] ^= (1u64 << rsrc) | (1u64 << rdst);

        self.occupied ^= (1u64 << ksrc) | (1u64 << kdst) | (1u64 << rsrc) | (1u64 << rdst);

        let king = self.piecelocs.piece_at(kdst as u8);
        let rook = self.piecelocs.piece_at(rdst as u8);

        self.piecelocs.remove(kdst as u8);
        self.piecelocs.place(ksrc as u8, king);

        self.piecelocs.remove(rdst as u8);
        self.piecelocs.place(rsrc as u8, rook);

        self.playerpieces[enemy as usize] ^= (1u64 << ksrc) | (1u64 << kdst) | (1u64 << rsrc) | (1u64 << rdst);
    }

    pub fn piece_exists_at(&self, rank: usize, file: usize) -> bool {
        let result = self.occupied >> (rank * 8 + file);
        result & 1 == 1
    }

    pub fn getpinned(&self) -> u64 { self.pinned }
    pub fn getattacked(&self) -> [u64; 2] { self.attacked }
    pub fn getep(&self) -> u8 { self.ep_square }

    // creates board from fen string
    pub fn from_fen(&mut self, fen: String, nnue: &Nnue) {
        let mut fields = fen.split(" ");
        let pieces = fields.next().unwrap().chars();
        let mut rank: usize = 7;
        let mut file: usize = 0;

        // reset
        *self = Board::new();

        for c in pieces {
            if c.is_numeric() {
                file += c.to_digit(10).unwrap() as usize;
            } else if c.is_alphabetic() {
                self.put_piece(c, rank, file);
                file += 1;
            } else if c == '/' {
                rank -= 1;
                file = 0;
            }
        }

        let color = fields.next().unwrap();
        let castling_rights = fields.next().unwrap();
        let ep = fields.next().unwrap();

        self.ep_square = if ep == "-" { 64 } else { constlib::square_from_string(ep) };
        self.turn = if color.chars().next().unwrap() == 'w' { 0 } else { 1 };
        self.castling_rights = castling::get_castling_mask(castling_rights);

        self.nnue_inited = false;
        self.nnue_rebuild(nnue);

        // initialize caches and hash
        let mg = movegen::MoveGenerator::new();
        let pininfo = mg.getpinned(self);
        self.pinned = pininfo.0;
        self.pinners = pininfo.1;
        self.attacked[self.turn as usize] = mg.makeattackedmask(self, self.turn, self.occupied);
        self.hash = Self::compute_hash(self);

    }

    pub fn compute_hash(board: &crate::core::Board) -> u64 {
        use crate::core::zobrist::{Z_PIECE_SQ, Z_SIDE, Z_CASTLING, Z_EP_FILE};
        use crate::core::constlib;

        let mut h: u64 = 0;

        for p in 0..12usize {
            let mut bb = board.pieces[p];
            while bb != 0 {
                let sq = constlib::poplsb(&mut bb) as usize;
                h ^= Z_PIECE_SQ[p][sq];
            }
        }

        if board.turn == 1 { h ^= Z_SIDE; }
        h ^= Z_CASTLING[(board.castling_rights & 0x0F) as usize];

        if board.ep_square != 64 {
            h ^= Z_EP_FILE[Self::file_of(board.ep_square)];
        }
        h
    }

    pub fn set_startpos(&mut self, nnue: &Nnue) {
        self.from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), nnue);
    }

    pub fn put_piece(&mut self, ch: char, rank: usize, file: usize) {
        let ind = rank * 8 + file;
        let color = if ch.is_uppercase() { 0 } else { 1 };
        let piece = match ch {
            'p' | 'P' => PieceType::P,
            'n' | 'N' => PieceType::N,
            'b' | 'B' => PieceType::B,
            'r' | 'R' => PieceType::R,
            'q' | 'Q' => PieceType::Q,
            'k' | 'K' => PieceType::K,
            _ => PieceType::NONE,
        };
        self.playerpieces[color] |= 1u64 << ind;
        self.pieces[piece as usize + (color * 6) - 1] |= 1u64 << ind;
        self.occupied |= 1u64 << ind;
        self.piecelocs.place(ind as u8, Piece::make(color as u8, piece));
    }

    pub fn print(&self) {
        let mut s = String::from("+---+---+---+---+---+---+---+---+");
        for r in (0..=7).rev() {
            let mut row = String::from("\n|");
            for f in 0..=7 {
                let p = self.piecelocs.piece_at(r * 8 + f);
                let mut ch = p.get_piece_type().get_piece_type().to_string();
                if p != Piece::None && p.get_color() == 0 {
                    ch = ch.to_ascii_uppercase();
                }
                row.push_str(&ch);
            }
            s.push_str(&row)
        }
        s.push_str("\n   a  b  c  d  e  f  g  h\n");
        println!("{}", s);
    }

    pub fn bbtostr(&self, mut input: String) -> String {
        let mut s = String::from("+---+---+---+---+---+---+---+---+");
        let mg = movegen::MoveGenerator::new();
        let precomputedpawns = mg.pawnattacks[0][1];
        for r in (0..=7).rev() {
            let mut row = String::from("\n|");
            for f in 0..=7 {
                let p = if (precomputedpawns >> (r * 8 + f) & 1) == 1 { "1" } else { "=" };
                row.push_str(&p);
            }
            s.push_str(&row)
        }
        s.push_str("\n   a  b  c  d  e  f  g  h\n");
        input.push_str(s.as_str());
        input
    }

    pub fn board_to_chars(&self) -> Vec<String> {
        let mut out = Vec::with_capacity(64);
        for sq in 0u8..64 {
            let p = self.piecelocs.piece_at(sq);
            let c = if p == Piece::None {
                '.'
            } else {
                let mut ch = p.get_piece_type().get_piece_type();
                if p.get_color() == 0 {
                    ch = ch.to_ascii_uppercase();
                }
                ch
            };
            out.push(c.to_string());
        }
        debug_assert_eq!(out.len(), 64);
        debug_assert!(out.iter().all(|s| s.chars().count() == 1));
        out
    }

    #[inline(always)]
    pub fn assert_kings_present(&self) {
        debug_assert!(self.pieces[PieceIndex::K.index()] != 0, "White king missing");
        debug_assert!(self.pieces[PieceIndex::k.index()] != 0, "Black king missing");
    }
    #[cfg(debug_assertions)]
#[inline(never)]
pub fn debug_validate(&self) {
    // 1) occupied = OR(pieces)
    let mut occ_from_pieces = 0u64;
    for p in 0..12 {
        occ_from_pieces |= self.pieces[p];
    }
    assert_eq!(self.occupied, occ_from_pieces, "occupied mismatch vs pieces OR");

    // 2) playerpieces = OR(color pieces)
    let white_from_pieces =
        self.pieces[0] | self.pieces[1] | self.pieces[2] | self.pieces[3] | self.pieces[4] | self.pieces[5];
    let black_from_pieces =
        self.pieces[6] | self.pieces[7] | self.pieces[8] | self.pieces[9] | self.pieces[10] | self.pieces[11];

    assert_eq!(self.playerpieces[0], white_from_pieces, "playerpieces[white] mismatch");
    assert_eq!(self.playerpieces[1], black_from_pieces, "playerpieces[black] mismatch");

    // 3) kings exist and are unique
    assert_eq!(self.pieces[PieceIndex::K.index()].count_ones(), 1, "white king count != 1");
    assert_eq!(self.pieces[PieceIndex::k.index()].count_ones(), 1, "black king count != 1");

    // 4) piecelocs matches bitboards
    for sq in 0u8..64 {
        let p = self.piecelocs.piece_at(sq);
        let bit = 1u64 << sq;
        let bit_in_occ = (self.occupied & bit) != 0;

        if p == Piece::None {
            assert!(!bit_in_occ, "piecelocs empty but occupied has bit at {}", sq);
        } else {
            assert!(bit_in_occ, "piecelocs has piece but occupied missing bit at {}", sq);
            let idx = p.getidx();
            assert!(
                (self.pieces[idx] & bit) != 0,
                "piecelocs says {:?} at {} but piece bitboard missing",
                p, sq
            );
        }
    }
}
    pub fn nnue_rebuild(&mut self, nnue: &crate::evaluate::nnue::Nnue) {
        debug_assert_eq!(nnue.b1.len(), 256);

        // Start from bias
        self.nnue_acc_w.copy_from_slice(&nnue.b1);
        self.nnue_acc_b.copy_from_slice(&nnue.b1);

        let wk_sq = self.pieces[PieceIndex::K.index()].trailing_zeros() as usize;
        let bk_sq = self.pieces[6 + PieceIndex::K.index()].trailing_zeros() as usize;

        for piece_idx in 0..12usize {
            let mut bb = self.pieces[piece_idx];
            while bb != 0 {
                let sq = constlib::poplsb(&mut bb) as usize;
                nnue_add_piece(
                    nnue,
                    &mut self.nnue_acc_w,
                    &mut self.nnue_acc_b,
                    wk_sq,
                    bk_sq,
                    piece_idx,
                    sq,
                );
            }
        }

        self.nnue_inited = true;
    }

}
