#![feature(generic_const_exprs, test)]
#![allow(incomplete_features)]
pub mod nnue;
mod transposition;
pub mod uci;

use std::{sync::mpsc::Receiver, time::Instant};

use board::{
    movegen::{generate_captures, generate_moves},
    piece::{PieceColor, PieceType},
    r#move::{Move, MoveType},
    Board,
};

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

    stop: bool,
}

impl GrandChessEngine {
    pub fn new(size: usize) -> Self {
        Self {
            node_count: 0,
            max_time: None,
            ply_offset: 0,
            tt: TTable::new(size),
            pv_length: [0; MAX_PLY],
            pv_table: [[Move::null(); MAX_PLY]; MAX_PLY],
            killer_moves: [[Move::null(); MAX_PLY]; 2],
            history_moves: [[0; 64]; 12],
            repetition_table: [0; MAX_PLY],
            board: Board::default(),
            stop: false,
        }
    }

    pub fn quiescence(&mut self, ply: usize, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = board.eval();

        if stand_pat >= beta {
            return beta;
        }

        alpha = alpha.max(stand_pat);

        let mut moves = generate_captures(board);

        for i in 0..moves.len() {
            let mut max_index = i;
            for j in i..moves.len() {
                if self.score_capture(moves[j]) > self.score_capture(moves[max_index]) {
                    max_index = j;
                }
            }

            moves.swap(i, max_index);

            let mut new_board = board.clone();

            new_board.make_move(moves[i]);

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
        reciver: &Receiver<()>,
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

        if self.node_count % 2048 == 0 {
            if (self.max_time.is_some() && Instant::now() > self.max_time.unwrap())
                || reciver.try_recv().is_ok()
            {
                self.stop = true;
                return STOPPED;
            }
        }

        let is_pv = (beta - alpha) > 1;

        let mut best_move = Move::null();

        let mut guessed_score = board.eval();

        let entry = self.tt.probe_entry(board.hash, ply as u32);
        if let Some(entry) = entry {
            best_move = entry.best_move;

            if entry.depth >= depth && !is_pv {
                guessed_score = entry.score;
                match entry.flags {
                    HashFlags::Exsact => return guessed_score,
                    HashFlags::Alpha => {
                        if guessed_score <= alpha {
                            return alpha;
                        }
                    }
                    HashFlags::Beta => {
                        if guessed_score >= beta {
                            return beta;
                        }
                    }
                };
            } else {
                if entry.flags == HashFlags::Exsact {
                    guessed_score = entry.score
                };
            }
        }

        if !in_check && ply != 0 && !is_pv {
            if depth <= 5 && guessed_score - (depth * depth * 20) >= beta {
                return guessed_score;
            }

            let mut null_board = board.clone();

            null_board.make_null_move();

            const R: i32 = 2;

            let null_score = -self.neg_max(
                depth - 1 - R,
                ply + 1 + R as usize,
                &null_board,
                -beta,
                -beta + 1,
                reciver,
            );

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

            new_board.make_move(moves[i]);

            const FMAGRIN: [i32; 4] = [0, 200, 300, 700];
            if !in_check
                && !is_pv
                && moves_searched != 0
                && depth <= 3
                && moves[i].move_type() < MoveType::EnPassantCapture
                && new_board.eval() + FMAGRIN[depth as usize] <= alpha
                && !new_board.is_king_attacked(!board.current_color)
            {
                continue;
            }

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

            let guessed_score = if moves_searched == 0 {
                self.repetition_table[ply + 1] = new_board.hash;
                -self.neg_max(depth - 1, ply + 1, &new_board, -beta, -alpha, reciver)
            } else {
                let score = if moves_searched >= 4
                    && depth >= 3
                    && !in_check
                    && moves[i].move_type() < MoveType::EnPassantCapture
                {
                    self.repetition_table[ply + 2] = new_board.hash;
                    -self.neg_max(depth - 2, ply + 2, &new_board, -alpha - 1, -alpha, reciver)
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

            if guessed_score > alpha && !self.stop {
                alpha = guessed_score;
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

            if guessed_score >= beta {
                self.tt.write_entry(
                    THash::new(board.hash, depth, guessed_score, best_move, HashFlags::Beta),
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
            self.tt.write_entry(
                THash::new(
                    board.hash,
                    depth,
                    in_check as i32 * -(MATE_VALUE + depth as i32),
                    best_move,
                    HashFlags::Exsact,
                ),
                ply as u32,
            );

            return in_check as i32 * -(MATE_VALUE + depth as i32);
        } else {
            self.tt.write_entry(
                THash::new(board.hash, depth, alpha, best_move, hash_flag),
                ply as u32,
            );
        }

        return alpha;
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
        } else {
            if self.killer_moves[0][ply] == r#move {
                9000
            } else if self.killer_moves[0][ply] == r#move {
                8000
            } else {
                self.history_moves[piece as usize][to as usize]
            }
        }
    }

    fn score_capture(&self, r#move: Move) -> u32 {
        (6 - r#move.piece().get_type() as u32) + r#move.captured() as u32 * 100 + 10000
    }
}
