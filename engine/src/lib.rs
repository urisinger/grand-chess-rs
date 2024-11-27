#![feature(generic_const_exprs, test)]
#![allow(incomplete_features, clippy::identity_op, clippy::needless_range_loop)]
pub mod board;
pub mod nnue;
mod transposition;
pub mod uci;

static NET: &[u8] = include_bytes!(env!("EVALFILE"));

use std::{sync::mpsc::Receiver, time::Instant};

use board::{
    movegen::{generate_captures, generate_moves},
    piece::{PieceColor, PieceType},
    r#move::{Move, MoveType},
    Board,
};
use nnue::{half_kp::HalfKP, network::TripleLayerNetwork, Nnue};

use self::transposition::{HashFlags, THash, TTable};

const MIN_SCORE: i32 = -50000;
const MAX_SCORE: i32 = 50000;

const MATE_VALUE: i32 = 49000;

const MATE_SCORE: i32 = 48000;

const MAX_PLY: usize = 128;

const STOPPED: i32 = -1000000;

pub struct GrandChessEngine {
    node_count: u64,

    max_time: Option<Instant>,

    ply_offset: u32,

    tt: TTable,

    pv_length: [usize; MAX_PLY],
    pv_table: [[Move; MAX_PLY]; MAX_PLY],

    killer_moves: [[Move; MAX_PLY]; 2],
    history_moves: [[u32; 64]; 12],

    repetition_table: [u64; MAX_PLY],

    board: Board,

    nnue: Box<Nnue<TripleLayerNetwork<512, 32, 32>, HalfKP, MAX_PLY, 512>>,

    stop: bool,

    dont_stop: bool,
}

impl GrandChessEngine {
    pub fn new(tt_bytes: usize) -> Self {
        Self {
            node_count: 0,
            max_time: None,
            ply_offset: 0,
            tt: TTable::new(tt_bytes),
            pv_length: [0; MAX_PLY],
            pv_table: [[Move::null(); MAX_PLY]; MAX_PLY],
            killer_moves: [[Move::null(); MAX_PLY]; 2],
            history_moves: [[0; 64]; 12],
            repetition_table: [0; MAX_PLY],
            board: Board::default(),
            nnue: Nnue::new_boxed(&mut std::io::Cursor::new(NET)),
            stop: false,
            dont_stop: false,
        }
    }

    pub fn bench(&mut self, benches: &[&str], depth: u32) {
        let start_time = Instant::now();
        self.node_count = 0;
        for fen in benches {
            for i in 1..depth {
                self.neg_max(
                    i as i32,
                    0,
                    &Board::from_fen(fen).unwrap(),
                    MIN_SCORE,
                    MAX_SCORE,
                    None,
                );
            }

            self.tt.clear();
            self.pv_table.fill([Move::null(); 128]);
            self.pv_length.fill(0);
            self.history_moves.fill([0; 64]);
            self.killer_moves.fill([Move::null(); MAX_PLY]);
            self.repetition_table.fill(0);
        }
        println!(
            "{} nodes {} nps",
            self.node_count,
            (self.node_count as f32 / start_time.elapsed().as_secs_f32()) as u64
        );
    }

    fn quiescence(&mut self, ply: usize, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        let best_move = Move::null();
        let stand_pat = (self.nnue.eval(ply, board.current_color) + board.eval()) / 2;

        if stand_pat >= beta {
            return beta;
        }

        alpha = alpha.max(stand_pat);

        let mut moves = generate_captures(board);

        for i in 0..moves.len() {
            let mut max_index = i;
            for j in i..moves.len() {
                if self.score_capture(moves[j], best_move)
                    > self.score_capture(moves[max_index], best_move)
                {
                    max_index = j;
                }
            }

            moves.swap(i, max_index);

            let mut new_board = board.clone();

            self.nnue.make_move(moves[i], &mut new_board, ply);

            if new_board.is_king_attacked(board.current_color) {
                break;
            }

            if moves[i].move_type() == MoveType::KingCastle {
                let castle_target = if board.current_color == PieceColor::White { 5 } else { 61 };
                if new_board.is_square_attacked(castle_target, new_board.current_color) {
                    break;
                }

                if board.is_king_attacked(board.current_color) {
                    break;
                }
            } else if moves[i].move_type() == MoveType::QueenCastle {
                let castle_target = if board.current_color == PieceColor::White { 3 } else { 59 };
                if new_board.is_square_attacked(castle_target, new_board.current_color) {
                    break;
                }
                if board.is_king_attacked(board.current_color) {
                    break;
                }
            }

            let score = -self.quiescence(ply + 1, &new_board, -beta, -alpha);

            if score > alpha {
                alpha = score;
            }

            if score >= beta {
                return beta;
            }
        }

        alpha
    }

    pub fn neg_max(
        &mut self,
        mut depth: i32,
        ply: usize,
        board: &Board,
        mut alpha: i32,
        beta: i32,
        reciver: Option<&Receiver<()>>,
    ) -> i32 {
        self.pv_length[ply] = ply;

        if self.is_repetition(board.hash, ply) {
            return 0;
        }
        let in_check = board.is_king_attacked(board.current_color);

        depth += in_check as i32;

        if depth <= 0 {
            return self.quiescence(ply, board, alpha, beta);
        }

        if self.node_count & 16383 == 0
            && ((self.max_time.is_some() && Instant::now() > self.max_time.unwrap())
                || reciver.map(|recv| recv.try_recv().is_ok()).unwrap_or(false))
            && !self.dont_stop
        {
            self.stop = true;
            return STOPPED;
        }

        let is_pv = (beta - alpha) > 1;

        let mut best_move = Move::null();

        let entry = self.tt.probe_entry(board.hash, ply as u32);
        if let Some(entry) = entry {
            best_move = entry.best_move;

            if entry.depth >= depth && !is_pv {
                match entry.flags {
                    HashFlags::Exsact => return entry.score,
                    HashFlags::Alpha => {
                        if entry.score <= alpha {
                            return alpha;
                        }
                    }
                    HashFlags::Beta => {
                        if entry.score >= beta {
                            return beta;
                        }
                    }
                }
            }
        };

        if !in_check && ply != 0 && !is_pv {
            let mut null_board = board.clone();

            null_board.make_null_move();
            const R: i32 = 2;

            let null_score =
                -self.neg_max(depth - 1 - R, ply, &null_board, -beta, -beta + 1, reciver);

            if null_score >= beta {
                return beta;
            }
        }

        let mut hash_flag = HashFlags::Alpha;

        let mut moves = generate_moves(board);

        let mut moves_searched = 0;

        for i in 0..moves.len() {
            let mut max_index = i;
            for j in i..moves.len() {
                if self.score_move(moves[j], ply, best_move)
                    > self.score_move(moves[max_index], ply, best_move)
                {
                    max_index = j;
                }
            }

            moves.swap(i, max_index);

            let mut new_board = board.clone();

            self.nnue.make_move(moves[i], &mut new_board, ply);

            if new_board.is_king_attacked(board.current_color) {
                continue;
            }

            if moves[i].move_type() == MoveType::KingCastle {
                let castle_target = if board.current_color == PieceColor::White { 5 } else { 61 };
                if new_board.is_square_attacked(castle_target, !board.current_color) {
                    continue;
                }
                if in_check {
                    continue;
                }
            } else if moves[i].move_type() == MoveType::QueenCastle {
                let castle_target = if board.current_color == PieceColor::White { 3 } else { 59 };
                if new_board.is_square_attacked(castle_target, !board.current_color) {
                    continue;
                }
                if in_check {
                    continue;
                }
            }

            let score = if moves_searched == 0 {
                self.repetition_table[ply + 1] = new_board.hash;
                -self.neg_max(depth - 1, ply + 1, &new_board, -beta, -alpha, reciver)
            } else {
                let score = if moves_searched >= 4
                    && depth >= 3
                    && !in_check
                    && moves[i].move_type() < MoveType::EnPassantCapture
                {
                    self.repetition_table[ply + 1] = new_board.hash;
                    -self.neg_max(depth - 2, ply + 1, &new_board, -alpha - 1, -alpha, reciver)
                } else {
                    alpha + 1
                };

                if score > alpha {
                    self.repetition_table[ply + 1] = new_board.hash;
                    -self.neg_max(depth - 1, ply + 1, &new_board, -beta, -alpha, reciver)
                } else {
                    score
                }
            };

            self.node_count += 1;
            moves_searched += 1;

            if self.stop {
                return STOPPED;
            }

            if score > alpha && !self.stop {
                alpha = score;
                best_move = moves[i];
                hash_flag = HashFlags::Exsact;

                if moves[i].captured() != PieceType::Empty {
                    self.history_moves[moves[i].piece() as usize][moves[i].to() as usize] +=
                        (depth) as u32;
                }

                self.pv_table[ply][ply] = moves[i];

                for next_ply in (ply + 1)..self.pv_length[ply + 1] {
                    self.pv_table[ply][next_ply] = self.pv_table[ply + 1][next_ply];
                }

                self.pv_length[ply] = self.pv_length[ply + 1];
            }

            if score >= beta {
                self.tt.write_entry(
                    THash::new(board.hash, depth, score, best_move, HashFlags::Beta),
                    ply as u32,
                );
                if moves[i].captured() != PieceType::Empty {
                    self.killer_moves[1][ply] = self.killer_moves[0][ply];
                    self.killer_moves[0][ply] = moves[i];
                }
                return beta;
            }
        }

        if moves_searched == 0 {
            return in_check as i32 * -(MATE_VALUE + depth);
        } else {
            self.tt.write_entry(
                THash::new(board.hash, depth, alpha, best_move, hash_flag),
                ply as u32,
            );
        }

        alpha
    }

    fn is_repetition(&self, hash: u64, ply: usize) -> bool {
        for i in 0..ply {
            if hash == self.repetition_table[i] {
                return true;
            }
        }
        false
    }

    fn score_move(&self, r#move: Move, ply: usize, best_move: Move) -> u32 {
        if r#move == best_move {
            return 200000;
        }
        if r#move == self.pv_table[0][ply] {
            return 100000;
        }

        let (_, to, _, piece, capture) = r#move.unpack();

        if capture != PieceType::Empty {
            (6 - piece.get_type() as u32) + capture as u32 * 100 + 10000
        } else if self.killer_moves[0][ply] == r#move {
            9000
        } else if self.killer_moves[1][ply] == r#move {
            8000
        } else {
            self.history_moves[piece as usize][to]
        }
    }

    fn score_capture(&self, r#move: Move, best_move: Move) -> u32 {
        if r#move == best_move {
            return 100000;
        }
        (6 - r#move.piece().get_type() as u32) + r#move.captured() as u32 * 100
    }
}
