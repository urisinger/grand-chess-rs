use std::mem::size_of;

use crate::board::r#move::Move;

use super::MATE_SCORE;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum HashFlags {
    #[default]
    Exsact = 0,
    Alpha,
    Beta,
}

#[derive(Default, Clone, Copy)]
pub struct THash {
    pub key: u64,
    pub depth: i32,
    pub score: i32,
    pub flags: HashFlags,
    pub best_move: Move,
}

impl THash {
    pub fn new(key: u64, depth: i32, score: i32, best_move: Move, flags: HashFlags) -> Self {
        Self { key, depth, score, flags, best_move }
    }
}

pub struct TTable {
    entries: Box<[THash]>,
}

impl TTable {
    pub fn new(size: usize) -> Self {
        let num_entries = size / size_of::<THash>();
        let mut entries = Vec::with_capacity(num_entries);
        entries.resize_with(num_entries, Default::default);

        Self { entries: entries.into() }
    }

    pub fn resize(&mut self, new_size: usize) {
        let num_entries = new_size / size_of::<THash>();
        let mut entries = Vec::with_capacity(num_entries);
        entries.resize(num_entries, Default::default());

        self.entries = entries.into();
    }

    pub fn clear(&mut self) {
        self.entries.fill(Default::default());
    }

    pub fn write_entry(&mut self, mut entry: THash, ply: u32) {
        if entry.score > MATE_SCORE {
            entry.score += ply as i32;
        } else if entry.score < -MATE_SCORE {
            entry.score -= ply as i32;
        }

        if entry.depth >= self.entries[(entry.key % self.entries.len() as u64) as usize].depth {
            self.entries[(entry.key % self.entries.len() as u64) as usize] = entry;
        }
    }

    pub fn probe_entry(&self, key: u64, ply: u32) -> Option<THash> {
        let mut hash_entry = self.entries[(key % self.entries.len() as u64) as usize];

        if hash_entry.score > MATE_SCORE {
            hash_entry.score -= ply as i32;
        } else if hash_entry.score < -MATE_SCORE {
            hash_entry.score += ply as i32;
        }

        if hash_entry.key == key {
            Some(hash_entry)
        } else {
            None
        }
    }
}
