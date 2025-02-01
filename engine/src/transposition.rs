use std::mem::size_of;

use crate::board::{
    piece::PieceType,
    r#move::{Move, MoveType},
    Board,
};

use super::MATE_SCORE;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn into_u128(self) -> u128 {
        let mut val: u128 = 0;
        // Pack the 64-bit key into bits 0..63.
        val |= self.key as u128;
        // Pack the 16-bit depth into bits 64..79.
        val |= ((self.depth as u128) & 0xffff) << 64;
        // Pack the 32-bit score into bits 80..111.
        val |= ((self.score as u128) & 0xffffffff) << 79;
        // Pack the 2-bit flags into bits 112..113.
        val |= ((self.flags as u128) & 0b11) << 111;
        // Pack best_move.from (6 bits) into bits 114..119.
        val |= ((self.best_move.from() as u128) & 0x3f) << 113;
        // Pack best_move.to (6 bits) into bits 120..125.
        val |= ((self.best_move.to() as u128) & 0x3f) << 119;
        // Pack best_move.move_type (2 bits) into bits 126..127.
        // (Note: the original code used 0x7, which would be 3 bits.)
        val |= ((self.best_move.move_type() as u128) & 0x7) << 125;
        val
    }

    pub fn get_depth(val: u128) -> i32 {
        ((val >> 64) & 0x7fff) as i32
    }

    pub fn from_u128(board: &Board, val: u128) -> Self {
        let from = ((val >> 113) & 0x3f) as u32;
        let to = ((val >> 119) & 0x3f) as u32;
        let move_type: MoveType = unsafe { std::mem::transmute(((val >> 125) & 0x7) as u8) };

        let piece = board.piece_at(from as usize);
        let captured = board.piece_at(to as usize);
        let captured = if captured.get_color() != board.current_color {
            captured.get_type()
        } else {
            PieceType::Empty
        };

        Self {
            key: (val & 0xffffffffffffffff) as u64,   // Bits 0–63
            depth: ((val >> 64) & 0x7fff) as i32,     // Bits 64–78 (15 bits)
            score: ((val >> 79) & 0xffffffff) as i32, // Bits 79–110 (32 bits)
            flags: unsafe { std::mem::transmute(((val >> 111) & 0b11) as u8) }, // Bits 111–112 (2 bits)

            best_move: Move::new(from, to, move_type, piece, captured),
        }
    }
}

pub struct TTable {
    entries: Box<[u128]>,
}

impl TTable {
    pub fn new(size: usize) -> Self {
        let num_entries = size / size_of::<u128>();
        let mut entries = Vec::with_capacity(num_entries);
        entries.resize_with(num_entries, Default::default);

        Self { entries: entries.into() }
    }

    pub fn resize(&mut self, new_size: usize) {
        let num_entries = new_size / size_of::<u128>();
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

        if entry.depth
            >= THash::get_depth(self.entries[(entry.key % self.entries.len() as u64) as usize])
        {
            self.entries[(entry.key % self.entries.len() as u64) as usize] = entry.into_u128();
        }
    }

    pub fn probe_entry(&self, board: &Board, key: u64, ply: u32) -> Option<THash> {
        let mut hash_entry =
            THash::from_u128(board, self.entries[(key % self.entries.len() as u64) as usize]);

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

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::board::{movegen::generate_moves, PiecesDelta};

    use super::*;

    fn generate_test_positions() -> Vec<Board> {
        vec![
            Board::default(),
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap(),
            Board::from_fen("4k3/8/8/8/8/8/8/4K2R w K - 0 1").unwrap(),
            Board::from_fen("8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1").unwrap(),
            Board::from_fen("8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1").unwrap(),
            Board::from_fen("8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1").unwrap(),
            Board::from_fen("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1").unwrap(),
            Board::from_fen("r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1").unwrap(),
            Board::from_fen("r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1").unwrap(),
            Board::from_fen("R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1").unwrap(),
            Board::from_fen("8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1").unwrap(),
            Board::from_fen("8/8/8/8/8/4k3/4P3/4K3 w - - 0 1").unwrap(),
            Board::from_fen("8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1").unwrap(),
        ]
    }

    #[test]
    fn test_all_moves_into_u128_and_from_u128() {
        let positions = generate_test_positions();
        let mut rng = rand::thread_rng();

        for board in positions {
            let moves = generate_moves(&board);

            for r#move in moves {
                let mut new_board = board.clone();
                let mut delta = PiecesDelta::new();
                new_board.make_move(r#move, &mut delta);

                // Create an Entry with a random depth (0-32767) and random score (-10K to 10K)
                let depth = rng.gen_range(0..=(0x7fff)) as i32;
                let score = rng.gen_range(-10_000..=10_000) as i32;
                let flags = unsafe { transmute(rng.gen_range(0..=2) as u8) }; // Random flags (2 bits)

                let entry = THash { key: rng.gen::<u64>(), depth, score, flags, best_move: r#move };

                let encoded = entry.into_u128();
                let decoded = THash::from_u128(&board, encoded);

                // Verify all fields match
                assert_eq!(decoded.key, entry.key, "Key mismatch");
                assert_eq!(decoded.depth, entry.depth, "Depth mismatch");
                assert_eq!(decoded.score, entry.score, "Score mismatch");
                assert_eq!(decoded.flags, entry.flags, "Flags mismatch");
                assert_eq!(decoded.best_move, entry.best_move, "Best move mismatch");
            }
        }
    }
}
