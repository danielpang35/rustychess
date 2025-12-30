#[derive(Copy, Clone)]
pub struct TTEntry {
    pub key: u64,        // full zobrist key (or key^score; keep full for correctness first)
    pub depth: u8,       // remaining depth at node
    pub flag: u8,        // 0=EMPTY, 1=EXACT, 2=LOWER, 3=UPPER
    pub score: i32,      // stored (mate-normalized)
    pub best: u16,       // Move packed bits
}

pub const TT_EMPTY: u8 = 0;
pub const TT_EXACT: u8 = 1;
pub const TT_LOWER: u8 = 2;
pub const TT_UPPER: u8 = 3;

impl Default for TTEntry {
    fn default() -> Self {
        Self { key: 0, depth: 0, flag: TT_EMPTY, score: 0, best: 0 }
    }
}

pub struct TranspositionTable {
    mask: usize,
    table: Vec<TTEntry>,
}

impl TranspositionTable {
    pub fn new_mb(megabytes: usize) -> Self {
        let bytes = megabytes * 1024 * 1024;
        let entry_size = std::mem::size_of::<TTEntry>();
        let mut n = bytes / entry_size;
        // round down to power of two
        n = n.next_power_of_two() >> 1;
        if n < 1 { n = 1; }
        let table = vec![TTEntry::default(); n];
        Self { mask: n - 1, table }
    }

    #[inline(always)]
    fn idx(&self, key: u64) -> usize {
        (key as usize) & self.mask
    }

    #[inline(always)]
    pub fn probe(&self, key: u64) -> TTEntry {
        self.table[self.idx(key)]
    }

    #[inline(always)]
    pub fn store(&mut self, key: u64, depth: u8, flag: u8, score: i32, best: u16) {
        let i = self.idx(key);
        let e = self.table[i];

        // Replacement: prefer deeper entries or empty or same key
        if e.flag == TT_EMPTY || e.key == key || depth >= e.depth {
            self.table[i] = TTEntry { key, depth, flag, score, best };
        }
    }

    pub fn clear(&mut self) {
        for e in self.table.iter_mut() {
            *e = TTEntry::default();
        }
    }
}