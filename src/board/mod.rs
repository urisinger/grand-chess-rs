mod bit_board;
mod piece;

use std::{fmt::Write, num::ParseIntError};

use bit_board::*;
use bitflags::bitflags;
use piece::*;

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

#[derive(Debug)]
pub struct Board {
    pub bit_boards: BitBoards,

    pub current_color: PieceColor,
    pub castle_flags: CastleFlags,
    pub en_pessant_sqaure: u32,
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

                    bit_boards[piece].set_bit(rank * 8 + file);
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
                'K' => CastleFlags::WHITE_KING_SIDE_CASTELING,
                'k' => CastleFlags::BLACK_KING_SIDE_CASTELING,
                'Q' => CastleFlags::WHITE_QUEEN_SIDE_CASTELING,
                'q' => CastleFlags::BLACK_QUEEN_SIDE_CASTELING,
                _ => return Err(FenError::NoSuchCastle(c)),
            }
        }

        let en_pessant_sqaure = {
            let word = words.next().ok_or(FenError::NotEnoughInfo())?;

            if word != "-" {
                match word.parse()? {
                    n @ 0..=48 => n,
                    n => return Err(FenError::EnPessentNotInRange(n)),
                }
            } else {
                0
            }
        };

        Ok(Self {
            bit_boards,
            current_color,
            castle_flags,
            en_pessant_sqaure,
        })
    }
}
