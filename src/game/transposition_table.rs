use crate::board::square::{Move, Square};

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
}

#[derive(Copy, Clone)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Copy, Clone)]
pub struct TTEntry {
    hash: u64,
    depth: u8,
    score: isize,
    bound: Bound,
    best: Move,
}

impl TTEntry {
    pub fn new(hash: u64, depth: u8, score: isize, best: Move, bound: Bound) -> Self {
        Self { hash, depth, score, best, bound }
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn score(&self) -> isize {
        self.score
    }

    pub fn best(&self) -> Move {
        self.best
    }

    pub fn bound(&self) -> Bound {
        self.bound
    }
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self {
            entries: vec![TTEntry::new(0, 0, 0, Move::new(Square::new(0, 0), Square::new(0, 0), None), Bound::Exact); 1 << size],
        }
    }

    fn hash_to_index(&self, hash: u64) -> usize {
        (hash & (self.entries.len() - 1) as u64) as usize
    }

    pub fn insert(&mut self, entry: TTEntry) {
        let index = self.hash_to_index(entry.hash);

        self.entries[index] = entry;
    }

    pub fn contains(&self, hash: u64) -> bool {
        let index = self.hash_to_index(hash);
        let existing = self.entries[index];

        existing.hash == hash
    }

    pub fn get(&self, hash: u64) -> TTEntry {
        let index = self.hash_to_index(hash);

        self.entries[index]
    }
}
