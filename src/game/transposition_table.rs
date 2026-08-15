pub struct TranspositionTable {
    stats: TTStats,
    entries: Vec<TTEntry>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TTStats {
    pub probes: u64,
    pub hits: u64,
    pub insufficient_depth: u64,
    pub usable: u64,
    pub exact_cutoffs: u64,
    pub lower_bound_hit: u64,
    pub upper_bound_hit: u64,
}

#[derive(Copy, Clone)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

impl Default for Bound {
    fn default() -> Self {
        Self::Exact
    }
}

#[derive(Copy, Clone, Default)]
pub struct TTEntry {
    hash: u64,
    depth: u8,
    score: isize,
    bound: Bound,
}

impl TTEntry {
    pub fn new(hash: u64, depth: u8, score: isize, bound: Bound) -> Self {
        Self {
            hash,
            depth,
            score,
            bound,
        }
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn score(&self) -> isize {
        self.score
    }

    pub fn bound(&self) -> Bound {
        self.bound
    }
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self {
            entries: vec![TTEntry::default(); 1 << size],
            stats: TTStats::default(),
        }
    }

    fn hash_to_index(&self, hash: u64) -> usize {
        (hash & (self.entries.len() - 1) as u64) as usize
    }

    pub fn insert(&mut self, entry: TTEntry) {
        let index = self.hash_to_index(entry.hash);

        self.entries[index] = entry;
    }

    pub fn get(&mut self, hash: u64) -> Option<TTEntry> {
        let index = self.hash_to_index(hash);
        let existing = self.entries[index];

        self.stats.probes += 1;

        if existing.hash != hash {
            return None;
        }

        self.stats.hits += 1;
        Some(existing)
    }

    pub fn stats_mut(&mut self) -> &mut TTStats {
        &mut self.stats
    }

    pub fn stats(&self) -> TTStats {
        self.stats
    }
}
