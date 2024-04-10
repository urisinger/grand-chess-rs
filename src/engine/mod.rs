use std::sync::Mutex;

use crate::board::{piece::PieceType, r#move::Move, Board};

use self::transposition::TTable;

mod transposition;

const MIN_SCORE: i32 = -50000;
const MAX_SCORE: i32 = 50000;

const MATE_VALUE: i32 = 49000;

const MATE_SCORE: i32 = 48000;

const MAX_PLY: usize = 128;

const STOPPED: i32 = -1000000;

struct GrandChessEngine {
    node_count: u64,

    ply_offset: u32,
    ply: u32,

    tt: TTable,

    pv_length: [usize; MAX_PLY],
    pv_table: [[Move; MAX_PLY]; MAX_PLY],

    stop: Mutex<bool>,
}

impl GrandChessEngine {
    fn neg_max(&mut self, depth: u32, board: &Board, alpha: i32, beta: i32) -> i32 {
        self.pv_length[self.ply as usize] = self.ply as usize;

        if self.node_count % 2048 == 0 {
            if self.stop.lock().unwrap() {
                return STOPPED;
            }
        }

        if depth <= 0 {
            return;
        }
        return alpha;
    }

    fn score_move(&self, r#move: Move) -> u32 {
        if r#move.0 == self.pv_table[0][self.ply as usize].0 {
            return 100000;
        }

        let (_, _, _, piece, capture) = r#move.unpack();
        if capture != PieceType::Empty {
            return (6 - piece.get_type() as u32) + capture as u32 * 100 + 10000;
        }
        return 0;
    }
}
