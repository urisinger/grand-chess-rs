use std::fmt;

use self::bitmasks::{
    magic_key, BISHOP_ATTACKS, BISHOP_MAGICS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS,
    ROOK_ATTACKS, ROOK_MAGICS, ROOK_MASKS,
};

use super::{
    r#move::{Move, MoveType},
    Board, Piece, PieceColor, PieceType,
};

pub mod bitmasks;

#[derive(Clone)]
pub struct Moves {
    moves: [Move; 256],
    size: usize,
}

impl fmt::Debug for Moves {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.size {
            f.write_fmt(format_args!("{:?} ", self.moves[i]))?;
        }
        Ok(())
    }
}

impl Default for Moves {
    fn default() -> Self {
        Self { moves: [Default::default(); 256], size: 0 }
    }
}

impl Moves {
    pub fn push(&mut self, r#move: Move) {
        assert!(self.size < 256);
        self.moves[self.size] = r#move;
        self.size += 1;
    }
}

pub fn generate_moves(board: Board) -> Moves {
    let mut moves = Moves::default();

    let mut bishops = board.bit_boards[Piece::new(PieceType::Bishop, board.current_color)];
    while bishops != 0 {
        let square = bishops.trailing_zeros() as usize;

        generate_sliding_moves(
            &board,
            &mut moves,
            square,
            BISHOP_MASKS[square],
            BISHOP_MAGICS[square],
            &BISHOP_ATTACKS[square],
        );

        bishops &= bishops - 1;
    }

    let mut rooks = board.bit_boards[Piece::new(PieceType::Rook, board.current_color)];
    while rooks != 0 {
        let square = rooks.trailing_zeros() as usize;

        generate_sliding_moves(
            &board,
            &mut moves,
            square,
            ROOK_MASKS[square],
            ROOK_MAGICS[square],
            &ROOK_ATTACKS[square],
        );

        rooks &= rooks - 1;
    }

    let mut queens = board.bit_boards[Piece::new(PieceType::Queen, board.current_color)];
    while queens != 0 {
        let square = queens.trailing_zeros() as usize;

        generate_sliding_moves(
            &board,
            &mut moves,
            square,
            ROOK_MASKS[square],
            ROOK_MAGICS[square],
            &ROOK_ATTACKS[square],
        );
        generate_sliding_moves(
            &board,
            &mut moves,
            square,
            BISHOP_MASKS[square],
            BISHOP_MAGICS[square],
            &BISHOP_ATTACKS[square],
        );

        queens &= queens - 1;
    }

    let mut knights = board.bit_boards[Piece::new(PieceType::Knight, board.current_color)];
    while knights != 0 {
        let square = knights.trailing_zeros() as usize;

        generate_piece_moves(
            &board,
            &mut moves,
            square,
            KNIGHT_ATTACKS[square] & !board.bit_boards.col_occupancy(board.current_color),
        );

        knights &= knights - 1;
    }

    generate_pawn_moves(&board, &mut moves);
    {
        let square = board.bit_boards[Piece::new(PieceType::King, board.current_color)]
            .trailing_zeros() as usize;

        if square < 64 {
            generate_piece_moves(
                &board,
                &mut moves,
                square,
                KING_ATTACKS[square] & !board.bit_boards.col_occupancy(board.current_color),
            );
        }
    }

    moves
}

#[inline]
fn generate_sliding_moves(
    board: &Board,
    moves: &mut Moves,
    square: usize,
    mask: u64,
    magic: u64,
    attacks: &[u64; 4096],
) {
    generate_piece_moves(
        board,
        moves,
        square,
        attacks[magic_key(magic, mask & board.bit_boards.occupancy(), mask.count_ones())]
            & !board.bit_boards.col_occupancy(board.current_color),
    );
}

#[inline]
fn generate_piece_moves(board: &Board, moves: &mut Moves, square: usize, mut attacks: u64) {
    let piece = board.bit_boards.piece_at(square);

    while attacks != 0 {
        let attack = attacks.trailing_zeros();

        let capture = board.bit_boards.piece_at(attack as usize).get_type();
        moves.push(Move::new(
            square as u32,
            attack,
            if capture == PieceType::Empty { MoveType::QUIET_MOVE } else { MoveType::CAPTURE },
            piece,
            capture,
        ));

        attacks &= attacks - 1;
    }
}

#[inline]
fn generate_pawn_moves(board: &Board, moves: &mut Moves) {
    let is_white = board.current_color == PieceColor::White;
    let direction = if is_white { 1 } else { -1 };

    let pawns = board.bit_boards[Piece::new(PieceType::Pawn, board.current_color)];
    let mut pawn_moves =
        if is_white { pawns << 8 } else { pawns >> 8 } & !board.bit_boards.occupancy();

    while pawn_moves != 0 {
        let target_square = pawn_moves.trailing_zeros() as i32;
        let source_square = target_square as i32 - (direction * 8);

        let starting_rank = if is_white { 1 } else { 6 };

        if source_square / 8 == starting_rank {
            if !board.is_occupied((target_square + direction * 8) as usize) {
                moves.push(Move::new(
                    source_square as u32,
                    (target_square + direction * 8) as u32,
                    MoveType::DOUBLE_PUSH,
                    Piece::new(PieceType::Pawn, board.current_color),
                    PieceType::Empty,
                ))
            }
        }

        let promotion_rank = if is_white { 7 } else { 0 };

        if target_square / 8 == promotion_rank {
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Queen, board.current_color),
                PieceType::Empty,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Knight, board.current_color),
                PieceType::Empty,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Rook, board.current_color),
                PieceType::Empty,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Bishop, board.current_color),
                PieceType::Empty,
            ));
        } else {
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::QUIET_MOVE,
                Piece::new(PieceType::Pawn, board.current_color),
                PieceType::Empty,
            ));
        }

        pawn_moves &= pawn_moves - 1;
    }
    // Shift pawns by the direction, shift them once, apply a mask to exlude pawns who
    // were on the edge and mask those who cant be captured
    let mut left_captures = ((if is_white { pawns << 8 } else { pawns >> 8 } >> 1)
        & !(0x8080808080808080u64))
        & board.bit_boards.col_occupancy(!board.current_color);
    let mut right_captures = ((if is_white { pawns << 8 } else { pawns >> 8 } << 1)
        & !(0x0101010101010101u64))
        & board.bit_boards.col_occupancy(!board.current_color);

    while left_captures != 0 {
        let target_square = left_captures.trailing_zeros();
        let source_square = left_captures as i32 - (direction * 8) - 1;

        let promotion_rank = if is_white { 7 } else { 0 };

        let captured_piece = board.bit_boards.piece_at(target_square as usize).get_type();

        if target_square / 8 == promotion_rank {
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Queen, board.current_color),
                captured_piece,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Knight, board.current_color),
                captured_piece,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Rook, board.current_color),
                captured_piece,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Bishop, board.current_color),
                captured_piece,
            ));
        } else {
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::QUIET_MOVE,
                Piece::new(PieceType::Pawn, board.current_color),
                captured_piece,
            ));
        }

        left_captures &= left_captures - 1;
    }

    while right_captures != 0 {
        let target_square = right_captures.trailing_zeros();
        let source_square = right_captures as i32 - (direction * 8) - 1;

        let promotion_rank = if is_white { 7 } else { 0 };

        let captured_piece = board.bit_boards.piece_at(target_square as usize).get_type();

        if target_square / 8 == promotion_rank {
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Queen, board.current_color),
                captured_piece,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Knight, board.current_color),
                captured_piece,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Rook, board.current_color),
                captured_piece,
            ));
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::PROMOTE,
                Piece::new(PieceType::Bishop, board.current_color),
                captured_piece,
            ));
        } else {
            moves.push(Move::new(
                source_square as u32,
                target_square as u32,
                MoveType::QUIET_MOVE,
                Piece::new(PieceType::Pawn, board.current_color),
                captured_piece,
            ));
        }

        right_captures &= right_captures - 1;
    }

    if let Some(last_double) = board.last_double {
        let left_capture_square = last_double as i32 - 1;
        let right_capture_square = last_double as i32 + 1;

        let en_passant_file = last_double % 8;
        if en_passant_file > 0 && left_capture_square >= 0 && left_capture_square < 64 {
            if pawns & (1 << left_capture_square) != 0 {
                moves.push(Move::new(
                    left_capture_square as u32,
                    (last_double as i32 + direction * 8) as u32,
                    MoveType::EN_PASSANT_CAPTURE,
                    Piece::new(PieceType::Pawn, board.current_color),
                    PieceType::Pawn,
                ));
            }
        }

        if en_passant_file < 7 && right_capture_square >= 0 && left_capture_square < 64 {
            if pawns & (1 << right_capture_square) != 0 {
                moves.push(Move::new(
                    right_capture_square as u32,
                    (last_double as i32 + direction * 8) as u32,
                    MoveType::EN_PASSANT_CAPTURE,
                    Piece::new(PieceType::Pawn, board.current_color),
                    PieceType::Pawn,
                ));
            }
        }
    }
}
