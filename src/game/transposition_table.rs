pub struct TranspositionTable {
    entries: Vec<TTEntry>,
}

#[derive(Copy, Clone, Default)]
pub enum Bound {
    #[default]
    Exact,
    Lower,
    Upper,
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

        if existing.hash != hash {
            return None;
        }

        Some(existing)
    }
}
