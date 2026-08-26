use std::{cell::UnsafeCell, sync::Mutex};

pub struct TranspositionTable {
    entries: UnsafeCell<Vec<TTEntry>>,
}

#[derive(Copy, Clone, Default)]
pub enum Bound {
    #[default]
    Exact,
    Lower,
    Upper,
}

#[derive(Default)]
pub struct TTEntry {
    hash: u64,
    score: i32,
    depth: u8,
    bound: Bound,
    lock: Mutex<()>,
}

pub struct TTData {
    score: i32,
    depth: u8,
    bound: Bound,
}

impl TTData {
    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn bound(&self) -> Bound {
        self.bound
    }
}

impl TTEntry {
    pub fn new(hash: u64, depth: u8, score: i32, bound: Bound) -> Self {
        Self {
            hash,
            depth,
            score,
            bound,
            lock: Mutex::new(()),
        }
    }

    pub fn to_data(&self) -> TTData {
        TTData {
            score: self.score,
            depth: self.depth,
            bound: self.bound,
        }
    }
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        let count = 1 << size;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(TTEntry::default());
        }

        Self {
            entries: entries.into(),
        }
    }

    fn hash_to_index(&self, hash: u64) -> usize {
        let len = unsafe { &*self.entries.get() }.len();
        (hash & (len - 1) as u64) as usize
    }

    pub fn insert(&self, entry: TTEntry) {
        let index = self.hash_to_index(entry.hash);
        let existing = &mut unsafe { &mut *self.entries.get() }[index];
        let lock = existing.lock.lock();

        existing.hash = entry.hash;
        existing.score = entry.score;
        existing.depth = entry.depth;
        existing.bound = entry.bound;

        drop(lock);
    }

    pub fn get(&self, hash: u64) -> Option<TTData> {
        let index = self.hash_to_index(hash);
        let existing = &mut unsafe { &mut *self.entries.get() }[index];
        let lock = existing.lock.lock();

        if existing.hash != hash {
            return None;
        }

        let data = existing.to_data();
        drop(lock);

        Some(data)
    }
}

unsafe impl Send for TranspositionTable {}
unsafe impl Sync for TranspositionTable {}
