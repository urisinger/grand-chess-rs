use crate::board::r#move::Move;

use super::MATE_SCORE;

#[derive(Default, Clone, Copy)]
pub enum HashFlags {
    #[default]
    Exsact = 0,
    Alpha,
    Beta,
}

#[derive(Default, Clone, Copy)]
pub struct THash {
    key: u64,
    depth: u32,
    score: i32,
    flags: HashFlags,
    best_move: Move,
}

impl THash {
    pub fn new(key: u64, depth: u32, score: i32, flags: HashFlags, best_move: Move) -> Self {
        Self { key, depth, score, flags, best_move }
    }
}

pub struct TTable {
    entries: Box<[THash]>,
}

impl TTable {
    pub fn new(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        entries.resize_with(size, Default::default);

        Self { entries: entries.into() }
    }

    pub fn resize(&mut self, new_size: usize) {
        let mut entries = Vec::with_capacity(new_size);
        entries.resize(new_size, Default::default());

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

        self.entries[(entry.key % self.entries.len() as u64) as usize] = entry;
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
