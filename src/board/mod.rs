pub mod bit_board;
pub mod r#move;
pub mod movegen;
pub mod piece;
mod util;

use std::{fmt::Write, num::ParseIntError, sync::Mutex};

use bit_board::*;
use bitflags::bitflags;
use piece::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::fmt;

use crate::board::movegen::generate_moves;

use self::{
    movegen::bitmasks::{bishop_attacks, rook_attacks, KING_ATTACKS, KNIGHT_ATTACKS},
    r#move::{Move, MoveType},
};

bitflags! {

    #[derive(Default)]
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

    pub current_color: PieceColor,
    pub castle_flags: CastleFlags,
    pub last_double: Option<u32>,
}

#[derive(Debug)]
pub enum FenError {
    NoSuchPiece(NoSuchPieceError),
    NoSuchColor(String),
    NoSuchCastle(char),
    EnPessentNotInRange(u32),
    ParseIntError(ParseIntError),
    NotEnoughInfo(),
}

impl From<NoSuchPieceError> for FenError {
    fn from(e: NoSuchPieceError) -> FenError {
        FenError::NoSuchPiece(e)
    }
}

impl From<ParseIntError> for FenError {
    fn from(e: ParseIntError) -> FenError {
        FenError::ParseIntError(e)
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

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
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

                    bit_boards.set_piece((rank * 8 + file) as usize, piece);
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
                match word.parse()? {
                    n @ 0..=48 => Some(n),
                    n => return Err(FenError::EnPessentNotInRange(n)),
                }
            } else {
                None
            }
        };

        Ok(Self { bit_boards, current_color, castle_flags, last_double })
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
            if square % 8 != 7 && (pawns & (1 << (square + 9)) != 0) {
                return true;
            }

            if square % 8 != 0 && (pawns & (1 << (square + 7)) != 0) {
                return true;
            }
        } else {
            if square % 8 != 0 && (pawns & (1 << (square - 9)) != 0) {
                return true;
            }

            if square % 8 != 7 && (pawns & (1 << (square - 7)) != 0) {
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

    pub fn make_move(&mut self, r#move: Move) {
        let to = r#move.to();
        let from = r#move.from();
        let piece = r#move.piece();
        let capture = r#move.captured();
        let move_type = r#move.move_type();

        if capture != PieceType::Empty {
            self.bit_boards.clear_piece(to as usize, Piece::new(capture, !self.current_color));
        }
        self.bit_boards.set_piece(to as usize, piece);
        if move_type == MoveType::Promote {
            self.bit_boards
                .clear_piece(from as usize, Piece::new(PieceType::Pawn, self.current_color));
        } else {
            self.bit_boards.clear_piece(from as usize, piece);
        }

        if move_type == MoveType::KingCastle {
            if piece.get_color() == Some(PieceColor::White) {
                // Move the rook from H1 to F1
                const ROOK_FROM: usize = 7;
                const ROOK_TO: usize = 5;
                self.castle_flags &=
                    !(CastleFlags::WHITE_KINGSIDE_CASTLING | CastleFlags::WHITE_QUEENSIDE_CASTLING);
                self.bit_boards.set_piece(ROOK_TO, Piece::WhiteRook);
                self.bit_boards.clear_piece(ROOK_FROM, Piece::WhiteRook);
            } else {
                // Move the rook from H8 to F8
                const ROOK_FROM: usize = 63;
                const ROOK_TO: usize = 61;
                self.castle_flags &=
                    !(CastleFlags::BLACK_KINGSIDE_CASTLING | CastleFlags::BLACK_QUEENSIDE_CASTLING);
                self.bit_boards.set_piece(ROOK_TO, Piece::BlackRook);
                self.bit_boards.clear_piece(ROOK_FROM, Piece::BlackRook);
            }
        } else if move_type == MoveType::QueenCastle {
            if piece.get_color() == Some(PieceColor::White) {
                // Move the rook from A1 to D1
                const ROOK_FROM: usize = 0;
                const ROOK_TO: usize = 3;
                self.castle_flags &=
                    !(CastleFlags::WHITE_KINGSIDE_CASTLING | CastleFlags::WHITE_QUEENSIDE_CASTLING);
                self.bit_boards.set_piece(ROOK_TO, Piece::WhiteRook);
                self.bit_boards.clear_piece(ROOK_FROM, Piece::WhiteRook);
            } else {
                // Move the rook from A8 to D8
                const ROOK_FROM: usize = 56;
                const ROOK_TO: usize = 59;
                self.castle_flags &=
                    !(CastleFlags::BLACK_KINGSIDE_CASTLING | CastleFlags::BLACK_QUEENSIDE_CASTLING);
                self.bit_boards.set_piece(ROOK_TO, Piece::BlackRook);
                self.bit_boards.clear_piece(ROOK_FROM, Piece::BlackRook);
            }
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
        } else if piece == Piece::WhiteKing {
            self.castle_flags &= !CastleFlags::WHITE_KINGSIDE_CASTLING;
            self.castle_flags &= !CastleFlags::WHITE_QUEENSIDE_CASTLING;
        } else if piece == Piece::BlackKing {
            self.castle_flags &= !CastleFlags::BLACK_KINGSIDE_CASTLING;
            self.castle_flags &= !CastleFlags::BLACK_QUEENSIDE_CASTLING;
        }

        if capture == PieceType::Rook {
            if to == 0 {
                self.castle_flags &= !CastleFlags::WHITE_QUEENSIDE_CASTLING;
            } else if to == 7 {
                self.castle_flags &= !CastleFlags::WHITE_KINGSIDE_CASTLING;
            } else if to == 56 {
                self.castle_flags &= !CastleFlags::BLACK_QUEENSIDE_CASTLING;
            } else if to == 63 {
                self.castle_flags &= !CastleFlags::BLACK_KINGSIDE_CASTLING;
            }
        }

        // Handle en passant capture
        if move_type == MoveType::EnPassantCapture {
            let captured_pawn_square = to as i32 + if piece == Piece::WhitePawn { -8 } else { 8 };

            self.bit_boards.clear_piece(
                captured_pawn_square as usize,
                Piece::new(PieceType::Pawn, !self.current_color),
            );
        }

        self.last_double = if move_type == MoveType::DoublePush { Some(to) } else { None };

        self.current_color = !self.current_color;
    }

    pub fn parse_move(&self, s: &str) -> Result<Move, ParseMoveError> {
        let bytes = s.as_bytes();

        let from_file = bytes.get(0).ok_or(ParseMoveError::StringTooSmall)?;
        let from_rank = bytes.get(1).ok_or(ParseMoveError::StringTooSmall)?;
        let to_file = bytes.get(2).ok_or(ParseMoveError::StringTooSmall)?;
        let to_rank = bytes.get(3).ok_or(ParseMoveError::StringTooSmall)?;

        let from = ((from_file - b'a') + (8 * (from_rank - b'1'))) as usize;
        let to = ((to_file - b'a') + (8 * (to_rank - b'1'))) as usize;

        if self.bit_boards.piece_at(from).get_type() == PieceType::Pawn
            && self.bit_boards.piece_at(to) == Piece::Empty
        {
            if self.current_color == PieceColor::White
                && *from_rank == b'2'
                && *to_rank == b'4'
                && from_file.abs_diff(*to_file) == 0
            {
                return Ok(Move::new(
                    from as u32,
                    to as u32,
                    MoveType::DoublePush,
                    Piece::new(PieceType::Pawn, self.current_color),
                    PieceType::Empty,
                ));
            } else if self.current_color == PieceColor::Black
                && *from_rank == b'7'
                && *to_rank == b'5'
                && from_file.abs_diff(*to_file) == 0
            {
                return Ok(Move::new(
                    from as u32,
                    to as u32,
                    MoveType::DoublePush,
                    Piece::new(PieceType::Pawn, self.current_color),
                    PieceType::Empty,
                ));
            }
        }

        // Check if the move is a castling move
        if from == 4
            && to == 6
            && self.bit_boards.piece_at(from).get_type() == PieceType::King
            && self.current_color == PieceColor::White
        {
            return Ok(Move::new(
                from as u32,
                to as u32,
                MoveType::KingCastle,
                Piece::new(PieceType::King, self.current_color),
                PieceType::Empty,
            ));
        } else if from == 4
            && to == 2
            && self.bit_boards.piece_at(from).get_type() == PieceType::King
            && self.current_color == PieceColor::White
        {
            return Ok(Move::new(
                from as u32,
                to as u32,
                MoveType::QueenCastle,
                Piece::new(PieceType::King, self.current_color),
                PieceType::Empty,
            ));
        } else if from == 60
            && to == 62
            && self.bit_boards.piece_at(from).get_type() == PieceType::King
            && self.current_color == PieceColor::Black
        {
            return Ok(Move::new(
                from as u32,
                to as u32,
                MoveType::KingCastle,
                Piece::new(PieceType::King, self.current_color),
                PieceType::Empty,
            ));
        } else if from == 60
            && to == 58
            && self.bit_boards.piece_at(from).get_type() == PieceType::King
            && self.current_color == PieceColor::Black
        {
            return Ok(Move::new(
                from as u32,
                to as u32,
                MoveType::QueenCastle,
                Piece::new(PieceType::King, self.current_color),
                PieceType::Empty,
            ));
        }

        // Check if the move is an en passant capture
        if self.bit_boards.piece_at(from).get_type() == PieceType::Pawn
            && self.bit_boards.piece_at(to) == Piece::Empty
        {
            if self.current_color == PieceColor::Black
                && *from_rank == b'5'
                && *to_rank == b'6'
                && from_file.abs_diff(*to_file) == 1
            {
                if self.bit_boards.piece_at(to + 8).get_type() == PieceType::Pawn {
                    return Ok(Move::new(
                        from as u32,
                        to as u32,
                        MoveType::EnPassantCapture,
                        Piece::new(PieceType::Pawn, self.current_color),
                        PieceType::Pawn,
                    ));
                }
            } else if self.current_color == PieceColor::White
                && *from_rank == b'4'
                && *to_rank == b'3'
                && from_file.abs_diff(*to_file) == 1
            {
                if self.bit_boards.piece_at(to - 8).get_type() == PieceType::Pawn {
                    return Ok(Move::new(
                        from as u32,
                        to as u32,
                        MoveType::EnPassantCapture,
                        Piece::new(PieceType::Pawn, self.current_color),
                        PieceType::Pawn,
                    ));
                }
            }
        }

        // Check if there is a captured piece on the 'to' square
        let captured_piece = self.bit_boards.piece_at(to).get_type();

        // Extract promotion piece if present
        let promotion_piece = match bytes.get(4) {
            Some(promotion_piece_char) => match promotion_piece_char {
                b'q' => PieceType::Queen,
                b'r' => PieceType::Rook,
                b'b' => PieceType::Bishop,
                b'n' => PieceType::Knight,
                _ => return Err(ParseMoveError::InvalidPromotionPiece),
            },
            None => PieceType::Empty,
        };

        if promotion_piece != PieceType::Empty {
            Ok(Move::new(
                from as u32,
                to as u32,
                MoveType::Promote,
                Piece::new(promotion_piece, self.current_color),
                captured_piece,
            ))
        } else {
            Ok(Move::new(
                from as u32,
                to as u32,
                MoveType::QuietMove,
                self.bit_boards.piece_at(from),
                captured_piece,
            ))
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::board::par_perft;

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
        println!("nps: {}", (nodes as f64 / start.elapsed().as_secs_f64()) as u64);
    }
}

pub fn par_perft(board: &Board, depth: u32) -> u64 {
    let nodes = Mutex::new(0);
    let moves = generate_moves(&board);

    moves.par_iter().for_each(|&r#move| {
        let mut new_board = board.clone();

        new_board.make_move(r#move);
        if new_board.is_king_attacked(board.current_color) {
            return;
        }

        if r#move.move_type() == MoveType::KingCastle {
            let castle_target = if board.current_color == PieceColor::White { 5 } else { 61 };
            if new_board.is_square_attacked(castle_target, new_board.current_color) {
                return;
            }

            if board.is_king_attacked(board.current_color) {
                return;
            }
        } else if r#move.move_type() == MoveType::QueenCastle {
            let castle_target = if board.current_color == PieceColor::White { 3 } else { 59 };
            if new_board.is_square_attacked(castle_target, new_board.current_color) {
                return;
            }
            if board.is_king_attacked(board.current_color) {
                return;
            }
        }
        let result = perft_helper(new_board, depth - 1);
        println!("{} {}", r#move, result);
        *nodes.lock().unwrap() += result;
    });

    let result = *nodes.lock().unwrap();
    println!("\n{}", result);
    result
}

pub fn perft(board: &Board, depth: u32) -> u64 {
    let mut nodes = 0;
    let moves = generate_moves(&board);

    moves.iter().for_each(|&r#move| {
        let mut new_board = board.clone();

        new_board.make_move(r#move);
        if new_board.is_king_attacked(board.current_color) {
            return;
        }

        if r#move.move_type() == MoveType::KingCastle {
            let castle_target = if board.current_color == PieceColor::White { 5 } else { 61 };
            if new_board.is_square_attacked(castle_target, new_board.current_color) {
                return;
            }

            if board.is_king_attacked(board.current_color) {
                return;
            }
        } else if r#move.move_type() == MoveType::QueenCastle {
            let castle_target = if board.current_color == PieceColor::White { 3 } else { 59 };
            if new_board.is_square_attacked(castle_target, new_board.current_color) {
                return;
            }
            if board.is_king_attacked(board.current_color) {
                return;
            }
        }
        let result = perft_helper(new_board, depth - 1);
        println!("{} {}", r#move, result);
        nodes += result;
    });

    println!("\n{}", nodes);
    nodes
}

fn perft_helper(board: Board, depth: u32) -> u64 {
    if depth <= 0 {
        return 1;
    }

    let moves = generate_moves(&board);

    let mut nodes = 0;
    moves.iter().for_each(|&r#move| {
        let mut new_board = board.clone();

        new_board.make_move(r#move);
        if new_board.is_king_attacked(board.current_color) {
            return;
        }

        if r#move.move_type() == MoveType::KingCastle {
            let castle_target = if board.current_color == PieceColor::White { 5 } else { 61 };
            if new_board.is_square_attacked(castle_target, new_board.current_color) {
                return;
            }
            if board.is_king_attacked(board.current_color) {
                return;
            }
        } else if r#move.move_type() == MoveType::QueenCastle {
            let castle_target = if board.current_color == PieceColor::White { 3 } else { 59 };
            if new_board.is_square_attacked(castle_target, new_board.current_color) {
                return;
            }
            if board.is_king_attacked(board.current_color) {
                return;
            }
        }

        let result = perft_helper(new_board, depth - 1);
        nodes += result;
    });

    nodes
}
