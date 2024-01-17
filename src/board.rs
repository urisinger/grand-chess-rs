use std::num::ParseIntError;

use crate::{bit_board::*, piece::*};
use bitflags::bitflags;

use std::fmt;

bitflags! {

    #[derive(Default)]
    pub struct CastleFlags : u8{
        const WHITE_KING_SIDE_CASTELING = 0x1;
        const WHITE_QUEEN_SIDE_CASTELING = 0x2;
        const BLACK_KING_SIDE_CASTELING = 0x4;
        const BLACK_QUEEN_SIDE_CASTELING = 0x8;
    }
}

#[derive(Default, Debug)]
pub struct Board {
    bit_boards: BitBoards,

    current_color: PieceColor,
    castle_flags: CastleFlags,
    en_pessant_sqaure: u32,
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
impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
        let mut board = Board::default();

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

                    board.bit_boards[piece.piece_color][piece.piece_type].set_bit(rank * 8 + file);
                    file += 1;
                }
            }
            if rank < 0 {
                break;
            }
        }

        board.current_color = match words.next().ok_or(FenError::NotEnoughInfo())? {
            "w" | "W" => PieceColor::White,
            "b" | "B" => PieceColor::Black,
            s => return Err(FenError::NoSuchColor(s.to_string())),
        };

        for c in words.next().ok_or(FenError::NotEnoughInfo())?.chars() {
            board.castle_flags |= match c {
                'K' => CastleFlags::WHITE_KING_SIDE_CASTELING,
                'k' => CastleFlags::BLACK_KING_SIDE_CASTELING,
                'Q' => CastleFlags::WHITE_QUEEN_SIDE_CASTELING,
                'q' => CastleFlags::BLACK_QUEEN_SIDE_CASTELING,
                _ => return Err(FenError::NoSuchCastle(c)),
            }
        }

        {
            let word = words.next().ok_or(FenError::NotEnoughInfo())?;

            if word != "-" {
                board.en_pessant_sqaure = match word.parse()? {
                    n @ 0..=48 => n,
                    n => return Err(FenError::EnPessentNotInRange(n)),
                }
            }
        }

        Ok(board)
    }
}
