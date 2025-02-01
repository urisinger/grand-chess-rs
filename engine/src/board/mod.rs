#![allow(dead_code)]
pub mod bit_board;
mod hash;
pub mod r#move;
pub mod movegen;
pub mod piece;
mod scores;

use std::{
    fmt::Write,
    ops::{Deref, DerefMut},
};

use bit_board::*;
use bitflags::bitflags;
use piece::*;

use std::fmt;

use self::{
    hash::{CASTLE_KEYS, DOUBLE_PUSH_KEYS, PIECE_KEYS, SIDE_KEY},
    movegen::bitmasks::{bishop_attacks, rook_attacks, KING_ATTACKS, KNIGHT_ATTACKS},
    r#move::{Move, MoveType},
    scores::{POSITIONAL_SCORES, SCORES},
};

bitflags! {

    #[derive(Default,Debug,Clone)]
    pub struct CastleFlags : u8{
        const WHITE_KINGSIDE_CASTLING = 0x1;
        const WHITE_QUEENSIDE_CASTLING = 0x2;
        const BLACK_KINGSIDE_CASTLING = 0x4;
        const BLACK_QUEENSIDE_CASTLING = 0x8;
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub bit_boards: BitBoards,
    pub pieces: [Piece; 64],

    pub current_color: PieceColor,
    pub castle_flags: CastleFlags,
    pub last_double: Option<u32>,

    pub hash: u64,

    eval: i32,
}

#[derive(Debug)]
pub enum FenError {
    NoSuchPiece(NoSuchPieceError),
    NoSuchColor(String),
    NoSuchCastle(char),
    EnPessentNotInRange(u32),
    InvalidEnPressant(String),
    NotEnoughInfo(),
}

impl From<NoSuchPieceError> for FenError {
    fn from(e: NoSuchPieceError) -> FenError {
        FenError::NoSuchPiece(e)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, piece) in self.bit_boards.to_mailbox().into_iter().enumerate() {
            if index % 8 == 0 && index != 0 {
                f.write_char('\n')?;
            }
            f.write_fmt(format_args!("{}|", char::from(piece)))?;
        }

        Ok(())
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl Board {
    pub fn is_occupied(&self, square: usize) -> bool {
        self.bit_boards.occupancy() & (1u64 << square) != 0
    }
}

#[derive(Debug)]
pub enum ParseMoveError {
    StringTooSmall,
    InvalidPromotionPiece,
}

#[derive(Debug, Clone, Copy)]
pub struct PieceDelta {
    pub to: u32,
    pub from: u32,
    pub piece: Piece,
}

pub struct PiecesDelta {
    pieces: [PieceDelta; 3],
    len: usize,
}

impl Default for PiecesDelta {
    fn default() -> Self {
        Self::new()
    }
}

impl PiecesDelta {
    pub fn new() -> Self {
        Self { pieces: [PieceDelta { to: 64, from: 64, piece: Piece::Empty }; 3], len: 0 }
    }

    pub fn push(&mut self, delta: PieceDelta) {
        self.pieces[self.len] = delta;
        self.len += 1;
    }
}

impl Deref for PiecesDelta {
    type Target = [PieceDelta];
    fn deref(&self) -> &Self::Target {
        &self.pieces[0..self.len]
    }
}

impl DerefMut for PiecesDelta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pieces[0..self.len]
    }
}

pub trait MoveDeltaRecorder {
    fn record(&mut self, delta: PieceDelta);
}

pub struct NoDelta;
impl MoveDeltaRecorder for NoDelta {
    #[inline(always)]
    fn record(&mut self, _delta: PieceDelta) {}
}

impl MoveDeltaRecorder for &mut PiecesDelta {
    #[inline(always)]
    fn record(&mut self, delta: PieceDelta) {
        self.push(delta);
    }
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
        let mut pieces = [Piece::default(); 64];
        let mut hash = 0;
        let mut eval = 0;

        let mut bit_boards = BitBoards::default();

        let mut words = fen.split_whitespace();

        let mut rank = 7;
        let mut file = 0;

        for c in words.next().ok_or(FenError::NotEnoughInfo())?.chars() {
            match c {
                '/' => {
                    rank -= 1;
                    file = 0;
                }
                '0'..='9' => file += c as i32 - '0' as i32,
                _ => {
                    let piece = Piece::try_from(c)?;

                    let square = (rank * 8 + file) as usize;

                    hash ^= PIECE_KEYS[piece as usize][square];

                    eval += if piece.get_color() == PieceColor::White { -1 } else { 1 }
                        * (SCORES[piece.get_type() as usize]
                            + POSITIONAL_SCORES[piece.get_type() as usize]
                                [square ^ (56 * !piece.get_color() as usize)]);

                    bit_boards.set_piece(square, piece);
                    pieces[square] = piece;
                    file += 1;
                }
            }
            if rank < 0 {
                break;
            }
        }

        let current_color = match words.next().ok_or(FenError::NotEnoughInfo())? {
            "w" | "W" => PieceColor::White,
            "b" | "B" => PieceColor::Black,
            s => return Err(FenError::NoSuchColor(s.to_string())),
        };

        if current_color == PieceColor::White {
            eval *= -1
        };

        let mut castle_flags = CastleFlags::empty();
        for c in words.next().ok_or(FenError::NotEnoughInfo())?.chars() {
            castle_flags |= match c {
                'K' => CastleFlags::WHITE_KINGSIDE_CASTLING,
                'k' => CastleFlags::BLACK_KINGSIDE_CASTLING,
                'Q' => CastleFlags::WHITE_QUEENSIDE_CASTLING,
                'q' => CastleFlags::BLACK_QUEENSIDE_CASTLING,
                '-' => break,
                _ => return Err(FenError::NoSuchCastle(c)),
            }
        }

        let last_double = {
            let word = words.next().ok_or(FenError::NotEnoughInfo())?;

            if word != "-" {
                if word.len() < 2 {
                    return Err(FenError::InvalidEnPressant(word.to_owned()));
                }

                let bytes = word.as_bytes();
                let square = (bytes[0] - b'a' + 8 * (bytes[1] - b'1')) as u32;
                match square {
                    n @ 0..=48 => {
                        hash ^= DOUBLE_PUSH_KEYS[n as usize];
                        Some(n)
                    }
                    n => return Err(FenError::EnPessentNotInRange(n)),
                }
            } else {
                None
            }
        };

        hash ^= CASTLE_KEYS[castle_flags.bits() as usize];

        hash ^= *SIDE_KEY * !current_color as u64;

        Ok(Self { bit_boards, current_color, castle_flags, last_double, hash, eval, pieces })
    }

    pub fn piece_at(&self, square: usize) -> Piece {
        let piece = self.bit_boards.piece_at(square);
        if piece != self.pieces[square] {
            Self::print_board(|square| self.bit_boards.piece_at(square));
            Self::print_board(|square| self.pieces[square]);
            panic!("mismatch {:?}, {:?}", piece, self.pieces[square]);
        } else {
            piece
        }
    }

    pub fn print_board(piece_at: impl Fn(usize) -> Piece) {
        println!("  a b c d e f g h");
        println!("  ---------------");

        for rank in (0..8).rev() {
            print!("{}|", rank + 1); // Print rank number
            for file in 0..8 {
                let square = rank * 8 + file;
                let piece = piece_at(square);

                let piece_char = if piece == Piece::Empty { '.' } else { char::from(piece) };

                print!("{} ", piece_char);
            }
            println!("|"); // End of row
        }

        println!("  ---------------");
    }

    pub fn is_square_attacked(&self, square: usize, attacker_color: PieceColor) -> bool {
        if KNIGHT_ATTACKS[square] & self.bit_boards[Piece::new(PieceType::Knight, attacker_color)]
            != 0
        {
            return true;
        };
        if KING_ATTACKS[square] & self.bit_boards[Piece::new(PieceType::King, attacker_color)] != 0
        {
            return true;
        }

        let pawns = self.bit_boards[Piece::new(PieceType::Pawn, attacker_color)];

        if attacker_color == PieceColor::Black {
            if square % 8 != 7 && (pawns & 1u64.overflowing_shl(square as u32 + 9).0 != 0) {
                return true;
            }

            if square % 8 != 0 && (pawns & (1u64.overflowing_shl(square as u32 + 7).0) != 0) {
                return true;
            }
        } else {
            if square % 8 != 0
                && (pawns & (1u64.overflowing_shl(square.overflowing_sub(9).0 as u32).0) != 0)
            {
                return true;
            }

            if square % 8 != 7
                && (pawns & (1u64.overflowing_shl(square.overflowing_sub(7).0 as u32).0) != 0)
            {
                return true;
            }
        }

        let bishop_queens = self.bit_boards[Piece::new(PieceType::Bishop, attacker_color)]
            | self.bit_boards[Piece::new(PieceType::Queen, attacker_color)];

        if bishop_attacks(square, self.bit_boards.occupancy()) & bishop_queens != 0 {
            return true;
        }

        let rook_queens = self.bit_boards[Piece::new(PieceType::Rook, attacker_color)]
            | self.bit_boards[Piece::new(PieceType::Queen, attacker_color)];

        if rook_attacks(square, self.bit_boards.occupancy()) & rook_queens != 0 {
            return true;
        }

        false
    }

    pub fn is_king_attacked(&self, color: PieceColor) -> bool {
        let kings = self.bit_boards[Piece::new(PieceType::King, color)];
        if kings != 0 {
            self.is_square_attacked(kings.trailing_zeros() as usize, !color)
        } else {
            true
        }
    }

    pub fn make_null_move(&mut self) {
        if let Some(last_double) = self.last_double {
            self.hash ^= DOUBLE_PUSH_KEYS[last_double as usize];
        }
        self.last_double = None;
        self.eval *= -1;
        self.hash ^= *SIDE_KEY;
        self.current_color = !self.current_color;
    }

    pub fn make_move(&mut self, r#move: Move, mut delta: impl MoveDeltaRecorder) {
        let (from, to, move_type, piece, capture) = r#move.unpack();

        if capture != PieceType::Empty {
            self.bit_boards.clear_piece(to, Piece::new(capture, !self.current_color));

            self.hash ^= PIECE_KEYS[Piece::new(capture, !self.current_color) as usize][to];

            self.eval += SCORES[capture as usize]
                + POSITIONAL_SCORES[capture as usize][to ^ (56 * self.current_color as usize)];

            if move_type == MoveType::EnPassantCapture {
                delta.record(PieceDelta {
                    to: 64,
                    from: (to as i32 + if piece == Piece::WhitePawn { -8 } else { 8 }) as u32,
                    piece: Piece::new(capture, !self.current_color),
                })
            } else {
                delta.record(PieceDelta {
                    to: 64,
                    from: to as u32,
                    piece: Piece::new(capture, !self.current_color),
                });
            }
        }

        self.bit_boards.set_piece(to, piece);

        self.pieces[to] = piece;

        self.hash ^= PIECE_KEYS[piece as usize][to];

        if move_type == MoveType::Promote {
            self.bit_boards.clear_piece(from, Piece::new(PieceType::Pawn, self.current_color));

            self.hash ^= PIECE_KEYS[Piece::new(PieceType::Pawn, self.current_color) as usize][from];

            self.eval -= SCORES[PieceType::Pawn as usize]
                + POSITIONAL_SCORES[PieceType::Pawn as usize]
                    [from ^ (56 * !self.current_color as usize)];

            self.eval += SCORES[piece.get_type() as usize]
                + POSITIONAL_SCORES[piece.get_type() as usize]
                    [to ^ (56 * !self.current_color as usize)];

            delta.record(PieceDelta {
                to: 64,
                from: from as u32,
                piece: Piece::new(PieceType::Pawn, self.current_color),
            });
            delta.record(PieceDelta { to: to as u32, from: 64, piece });
        } else {
            self.hash ^= PIECE_KEYS[piece as usize][from];

            self.bit_boards.clear_piece(from, piece);

            self.eval += POSITIONAL_SCORES[piece.get_type() as usize]
                [to ^ (56 * !self.current_color as usize)]
                - POSITIONAL_SCORES[piece.get_type() as usize]
                    [from ^ (56 * !self.current_color as usize)];

            delta.record(PieceDelta { to: to as u32, from: from as u32, piece });
        }

        self.pieces[from] = Piece::Empty;

        self.hash ^= CASTLE_KEYS[self.castle_flags.bits() as usize];
        if move_type == MoveType::KingCastle || move_type == MoveType::QueenCastle {
            let rook_from = match to {
                // C1 => A1,
                2 => 0,
                // G1 => H1,
                6 => 7,
                //C8 => A8,
                58 => 56,
                //G8 => H8,
                62 => 63,
                _ => panic!("Castle move with invalid to square"),
            };

            let rook_to = match to {
                // C1 => D1,
                2 => 3,
                // G1 => F1,
                6 => 5,
                //C8 => D8,
                58 => 59,
                //G8 => F8,
                62 => 61,
                _ => panic!("Castle move with invalid to square"),
            };

            let rook = Piece::new(PieceType::Rook, self.current_color);

            self.bit_boards.set_piece(rook_to, rook);
            self.bit_boards.clear_piece(rook_from, rook);

            self.pieces[rook_to] = rook;
            self.pieces[rook_from] = Piece::Empty;

            self.hash ^= PIECE_KEYS[rook as usize][rook_from];
            self.hash ^= PIECE_KEYS[rook as usize][rook_to];

            self.eval += POSITIONAL_SCORES[PieceType::Rook as usize]
                [rook_to ^ (56 * !self.current_color as usize)]
                - POSITIONAL_SCORES[PieceType::Rook as usize]
                    [rook_from ^ (56 * !self.current_color as usize)];

            delta.record(PieceDelta { to: rook_to as u32, from: rook_from as u32, piece: rook });
        }

        if piece == Piece::WhiteRook {
            if from == 0 {
                self.castle_flags &= !CastleFlags::WHITE_QUEENSIDE_CASTLING;
            } else if from == 7 {
                self.castle_flags &= !CastleFlags::WHITE_KINGSIDE_CASTLING;
            }
        } else if piece == Piece::BlackRook {
            if from == 56 {
                self.castle_flags &= !CastleFlags::BLACK_QUEENSIDE_CASTLING;
            } else if from == 63 {
                self.castle_flags &= !CastleFlags::BLACK_KINGSIDE_CASTLING;
            }
        }

        if piece == Piece::WhiteKing {
            self.castle_flags &=
                !(CastleFlags::WHITE_KINGSIDE_CASTLING | CastleFlags::WHITE_QUEENSIDE_CASTLING);
        } else if piece == Piece::BlackKing {
            self.castle_flags &=
                !(CastleFlags::BLACK_KINGSIDE_CASTLING | CastleFlags::BLACK_QUEENSIDE_CASTLING);
        }

        if capture == PieceType::Rook {
            match to {
                0 => self.castle_flags &= !CastleFlags::WHITE_QUEENSIDE_CASTLING,
                7 => self.castle_flags &= !CastleFlags::WHITE_KINGSIDE_CASTLING,
                56 => self.castle_flags &= !CastleFlags::BLACK_QUEENSIDE_CASTLING,
                63 => self.castle_flags &= !CastleFlags::BLACK_KINGSIDE_CASTLING,
                _ => {}
            }
        }

        // Handle en passant capture
        if move_type == MoveType::EnPassantCapture {
            let captured_pawn_square = to as i32 + if piece == Piece::WhitePawn { -8 } else { 8 };

            let captured_piece = Piece::new(PieceType::Pawn, !self.current_color);
            self.bit_boards.clear_piece(captured_pawn_square as usize, captured_piece);

            self.pieces[captured_pawn_square as usize] = Piece::Empty;

            self.hash ^= PIECE_KEYS[captured_piece as usize][captured_pawn_square as usize];
            self.hash ^= PIECE_KEYS[captured_piece as usize][to];

            self.eval += POSITIONAL_SCORES[PieceType::Pawn as usize]
                [captured_pawn_square as usize ^ (56 * self.current_color as usize)]
                - POSITIONAL_SCORES[PieceType::Pawn as usize]
                    [to ^ (56 * self.current_color as usize)];
        }

        self.hash ^= CASTLE_KEYS[self.castle_flags.bits() as usize];

        if let Some(last_double) = self.last_double {
            self.hash ^= DOUBLE_PUSH_KEYS[last_double as usize];
        }

        self.last_double = if move_type == MoveType::DoublePush {
            self.hash ^= DOUBLE_PUSH_KEYS[to];
            Some(to as u32)
        } else {
            None
        };

        self.eval *= -1;
        self.hash ^= *SIDE_KEY;
        self.current_color = !self.current_color;
    }

    pub fn eval(&self) -> i32 {
        self.eval
    }
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

    use crate::board::{
        hash::{CASTLE_KEYS, DOUBLE_PUSH_KEYS, PIECE_KEYS, SIDE_KEY},
        movegen::generate_moves,
        r#move::MoveType,
        scores::{POSITIONAL_SCORES, SCORES},
        PieceColor, PiecesDelta,
    };

    use super::Board;

    #[test]
    fn perft_test() {
        let fen_tests = [
            (Board::default(), 6, 119_060_324),
            (
                Board::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                )
                .unwrap(),
                5,
                193_690_690,
            ),
            (Board::from_fen("4k3/8/8/8/8/8/8/4K2R w K - 0 1").unwrap(), 6, 764_643),
            (Board::from_fen("8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1").unwrap(), 6, 8_503_277),
            (Board::from_fen("8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1").unwrap(), 6, 3_147_566),
            (Board::from_fen("8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1").unwrap(), 6, 4_405_103),
            (Board::from_fen("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1").unwrap(), 6, 22_823_890),
            (Board::from_fen("r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1").unwrap(), 6, 185_959_088),
            (Board::from_fen("r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1").unwrap(), 6, 190_755_813),
            (Board::from_fen("R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1").unwrap(), 6, 525_169_084),
            (Board::from_fen("8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1").unwrap(), 6, 34_822),
            (Board::from_fen("8/8/8/8/8/4k3/4P3/4K3 w - - 0 1").unwrap(), 6, 11_848),
            (Board::from_fen("8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1").unwrap(), 6, 158_065),
        ];

        let start = Instant::now();
        fen_tests.iter().for_each(|(board, depth, target)| {
            assert_eq!(par_perft(board, *depth), *target);
        });
        let nodes: u64 = fen_tests.iter().map(|b| b.2).sum();
        let time = start.elapsed().as_secs_f64();
        println!("time: {}, nps: {}", time, (nodes as f64 / time) as u64);
    }

    pub fn generate_hash(board: &Board) -> u64 {
        let mut hash = 0u64;

        let mut pieces = board.bit_boards.occupancy();
        while pieces != 0 {
            let square = pieces.trailing_zeros() as usize;
            let piece = board.piece_at(square);

            hash ^= PIECE_KEYS[piece as usize][square];
            pieces &= pieces - 1;
        }

        if let Some(double_push) = board.last_double {
            hash ^= DOUBLE_PUSH_KEYS[double_push as usize];
        }

        hash ^= CASTLE_KEYS[board.castle_flags.bits() as usize];
        hash ^= !board.current_color as u64 * *SIDE_KEY;

        hash
    }

    pub fn eval_board(board: &Board) -> i32 {
        let mut eval = 0;

        let mut pieces = board.bit_boards.occupancy();
        while pieces != 0 {
            let square = pieces.trailing_zeros() as usize;
            let piece = board.piece_at(square);

            eval += if piece.get_color() == board.current_color { 1 } else { -1 }
                * (SCORES[piece.get_type() as usize]
                    + POSITIONAL_SCORES[piece.get_type() as usize]
                        [square ^ (56 * !piece.get_color() as usize)]);

            pieces &= pieces - 1;
        }

        eval
    }

    pub fn par_perft(board: &Board, depth: u32) -> u64 {
        let moves = generate_moves(board);

        let nodes = moves
            .par_iter()
            .map(|&r#move| {
                let mut new_board = board.clone();

                let mut delta = PiecesDelta::new();
                new_board.make_move(r#move, &mut delta);
                if new_board.is_king_attacked(board.current_color) {
                    return 0;
                }

                if r#move.move_type() == MoveType::KingCastle {
                    let castle_target =
                        if board.current_color == PieceColor::White { 5 } else { 61 };
                    if new_board.is_square_attacked(castle_target, new_board.current_color) {
                        return 0;
                    }

                    if board.is_king_attacked(board.current_color) {
                        return 0;
                    }
                } else if r#move.move_type() == MoveType::QueenCastle {
                    let castle_target =
                        if board.current_color == PieceColor::White { 3 } else { 59 };
                    if new_board.is_square_attacked(castle_target, new_board.current_color) {
                        return 0;
                    }
                    if board.is_king_attacked(board.current_color) {
                        return 0;
                    }
                }

                let result = perft_helper(new_board, depth - 1);
                println!("{} {}", r#move, result);
                result
            })
            .sum();

        println!("\n{}", nodes);
        nodes
    }

    fn perft_helper(board: Board, depth: u32) -> u64 {
        if depth <= 0 {
            return 1;
        }

        let moves = generate_moves(&board);

        let mut nodes = 0;
        for r#move in moves {
            let mut new_board = board.clone();

            let mut delta = PiecesDelta::new();
            new_board.make_move(r#move, &mut delta);
            if new_board.is_king_attacked(board.current_color) {
                continue;
            }

            if r#move.move_type() == MoveType::KingCastle {
                let castle_target = if board.current_color == PieceColor::White { 5 } else { 61 };
                if new_board.is_square_attacked(castle_target, new_board.current_color) {
                    continue;
                }
                if board.is_king_attacked(board.current_color) {
                    continue;
                }
            } else if r#move.move_type() == MoveType::QueenCastle {
                let castle_target = if board.current_color == PieceColor::White { 3 } else { 59 };
                if new_board.is_square_attacked(castle_target, new_board.current_color) {
                    continue;
                }
                if board.is_king_attacked(board.current_color) {
                    continue;
                }
            }

            let result = perft_helper(new_board, depth - 1);
            nodes += result;
        }

        nodes
    }
}
