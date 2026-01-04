#[derive(Copy, Clone)]
pub struct TTEntry {
    pub key: u64,        // full zobrist key
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
    bucket_mask: usize,     // masks bucket index (bucket_count - 1)
    table: Vec<TTEntry>,    // flat array of size bucket_count * WAYS
}

impl TranspositionTable {
    const WAYS: usize = 4;

    pub fn new_mb(megabytes: usize) -> Self {
        let bytes = megabytes * 1024 * 1024;
        let entry_size = std::mem::size_of::<TTEntry>();

        // Total entries that fit:
        let mut total_entries = bytes / entry_size;
        if total_entries < Self::WAYS {
            total_entries = Self::WAYS;
        }

        // Convert to bucket count (each bucket holds WAYS entries)
        let mut buckets = total_entries / Self::WAYS;
        if buckets < 1 {
            buckets = 1;
        }

        // buckets must be a power of two for masking
        let raw = buckets;
        buckets = buckets.next_power_of_two();
        if buckets > raw {
            buckets >>= 1; // largest power-of-two <= raw
            if buckets < 1 { buckets = 1; }
        }

        let table_len = buckets * Self::WAYS;
        let table = vec![TTEntry::default(); table_len];

        Self {
            bucket_mask: buckets - 1,
            table,
        }
    }

    #[inline(always)]
    fn bucket_index(&self, key: u64) -> usize {
        (key as usize) & self.bucket_mask
    }

    #[inline(always)]
    fn bucket_start(&self, key: u64) -> usize {
        self.bucket_index(key) * Self::WAYS
    }

    /// Probe for a key. Returns TT_EMPTY entry on miss.
    #[inline(always)]
    pub fn probe(&self, key: u64) -> TTEntry {
        let start = self.bucket_start(key);
        // Scan 4-way bucket
        for j in 0..Self::WAYS {
            let e = self.table[start + j];
            if e.flag != TT_EMPTY && e.key == key {
                return e;
            }
        }
        TTEntry::default()
    }

    /// Store entry with simple 4-way replacement policy:
    /// 1) replace same key
    /// 2) else fill empty slot
    /// 3) else replace shallowest depth in bucket
    #[inline(always)]
    pub fn store(&mut self, key: u64, depth: u8, flag: u8, score: i32, best: u16) {
        let start = self.bucket_start(key);
        let newe = TTEntry { key, depth, flag, score, best };

        // 1) Same key replacement
        for j in 0..Self::WAYS {
            let i = start + j;
            let e = self.table[i];
            if e.flag != TT_EMPTY && e.key == key {
                // Prefer deeper (or equal) info; but also allow updating best move.
                if depth >= e.depth || flag == TT_EXACT {
                    self.table[i] = newe;
                } else if best != 0 && e.best == 0 {
                    // Preserve deeper bounds but keep a move if we didn't have one
                    let mut patched = e;
                    patched.best = best;
                    self.table[i] = patched;
                }
                return;
            }
        }

        // 2) Fill empty slot
        for j in 0..Self::WAYS {
            let i = start + j;
            if self.table[i].flag == TT_EMPTY {
                self.table[i] = newe;
                return;
            }
        }

        // 3) Replace shallowest depth
        let mut repl = start;
        let mut best_depth = self.table[start].depth;
        for j in 1..Self::WAYS {
            let i = start + j;
            let d = self.table[i].depth;
            if d < best_depth {
                best_depth = d;
                repl = i;
            }
        }

        self.table[repl] = newe;
    }

    pub fn clear(&mut self) {
        for e in self.table.iter_mut() {
            *e = TTEntry::default();
        }
    }
}