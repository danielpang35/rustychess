use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Stockfish {
    _child: Child, // keep process alive
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Stockfish {
    pub fn new(path: &str) -> Self {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start stockfish");

        let stdin = child.stdin.take().expect("stockfish stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stockfish stdout"));

        let mut sf = Self {
            _child: child,
            stdin,
            stdout,
        };

        sf.init();
        sf
    }

    fn init(&mut self) {
        self.send("uci");
        self.send("isready");
        self.read_until("readyok");
        // Optional but useful: avoid leftover chatter impacting first query
    }

    #[inline]
    fn send(&mut self, cmd: &str) {
        // UCI commands are line-based
        writeln!(self.stdin, "{}", cmd).expect("write to stockfish");
        self.stdin.flush().expect("flush stockfish stdin");
    }

    #[inline]
    fn read_line(&mut self) -> String {
        let mut line = String::new();
        self.stdout.read_line(&mut line).expect("read from stockfish");
        line.trim().to_string()
    }

    fn read_until(&mut self, token: &str) {
        loop {
            let line = self.read_line();
            if line.contains(token) {
                break;
            }
        }
    }

pub fn is_uci_move(s: &str) -> bool {
    let b = s.as_bytes();

    // UCI: "e2e4" or "e7e8q"
    if !(b.len() == 4 || b.len() == 5) {
        return false;
    }

    let file_ok = |c: u8| (b'a'..=b'h').contains(&c);
    let rank_ok = |c: u8| (b'1'..=b'8').contains(&c);

    if !file_ok(b[0]) || !rank_ok(b[1]) || !file_ok(b[2]) || !rank_ok(b[3]) {
        return false;
    }

    if b.len() == 5 {
        matches!(b[4], b'n' | b'b' | b'r' | b'q')
    } else {
        true
    }
}

    /// Returns legal moves in UCI for the given FEN using `go perft 1`.
pub fn legal_moves(&mut self, fen: &str) -> Vec<String> {
    self.send(&format!("position fen {}", fen));
    self.send("go perft 1");

    let mut moves = Vec::new();

    loop {
        let line = self.read_line();

        // Perft terminator (typical Stockfish output)
        if line.starts_with("Nodes searched") {
            break;
        }

        // Perft lines are usually "e2e4: 1"
        if let Some((mv, _count)) = line.split_once(':') {
            let mv = mv.trim();
            if Self::is_uci_move(mv) {
                moves.push(mv.to_string());
            }
        }
    }

    moves
}


fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

pub fn random_fens(&mut self, n: usize, plies_min: usize, plies_max: usize) -> Vec<String> {
    let mut out = Vec::with_capacity(n);

    // Seed RNG from time (good enough for randomized testing)
    let mut rng_state = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    for i in 0..n {
        if i % 1000 == 0 {
            eprintln!("generated {} / {}", i, n);
        }

        // Choose a random ply count in [plies_min, plies_max]
        let span = (plies_max - plies_min + 1) as u64;
        let plies = plies_min + (Self::xorshift64(&mut rng_state) % span) as usize;

        // Accumulate a random line of moves from startpos
        let mut moves_line: Vec<String> = Vec::new();

        for _ in 0..plies {
            // Get legal moves for current position
            let fen = if moves_line.is_empty() {
                // Startpos FEN not needed; we can just query startpos directly:
                // But your legal_moves() takes fen; so we ask stockfish for FEN after "position startpos moves ..."
                // We'll do it by setting position then calling "d" once to get FEN.
                self.send("position startpos");
                self.send("d");
                self.read_fen_from_d()
            } else {
                self.send(&format!("position startpos moves {}", moves_line.join(" ")));
                self.send("d");
                self.read_fen_from_d()
            };

            let legal = self.legal_moves(&fen);
            if legal.is_empty() {
                break;
            }

            let idx = (Self::xorshift64(&mut rng_state) % (legal.len() as u64)) as usize;
            moves_line.push(legal[idx].clone());
        }

        // Final position -> FEN
        if moves_line.is_empty() {
            self.send("position startpos");
        } else {
            self.send(&format!("position startpos moves {}", moves_line.join(" ")));
        }
        self.send("d");
        out.push(self.read_fen_from_d());
    }

    out
}

/// Helper: assumes you already sent "d". Reads until it finds "Fen: ...".
fn read_fen_from_d(&mut self) -> String {
    let mut fen: Option<String> = None;

    // Hard bound prevents infinite loops if something changes
    for _ in 0..200 {
        let line = self.read_line();

        if fen.is_none() {
            if let Some(rest) = line.strip_prefix("Fen: ") {
                fen = Some(rest.to_string());
            }
            continue;
        }

        // After Fen is found, keep draining until we hit a stable "end-ish" marker.
        // Stockfish's `d` output includes "Key:" near the end.
        if line.starts_with("Key:") {
            break;
        }
    }

    fen.expect("Stockfish `d` output did not contain a Fen: line")
}


pub fn fen_from_startpos_moves(&mut self, moves: &[String]) -> String {
    if moves.is_empty() {
        self.send("position startpos");
    } else {
        self.send(&format!("position startpos moves {}", moves.join(" ")));
    }
    self.send("d");
    self.read_fen_from_d()
}


}
