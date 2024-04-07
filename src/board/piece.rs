use core::fmt;
use std::{fmt::Write, ops::Not, str::FromStr};

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

impl Not for PieceColor {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Default for PieceColor {
    fn default() -> Self {
        PieceColor::White
    }
}

impl TryFrom<u8> for PieceColor {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::Black as u8 {
            Err(())
        } else {
            unsafe { Ok(core::mem::transmute(value)) }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
    Empty = 6,
}

impl ToString for PieceType {
    fn to_string(&self) -> String {
        match *self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
            PieceType::Empty => ' ',
        }
        .to_string()
    }
}

impl FromStr for PieceType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.as_bytes().get(0).ok_or(())? {
            b'p' => PieceType::Pawn,
            b'n' => PieceType::Knight,
            b'b' => PieceType::Bishop,
            b'r' => PieceType::Rook,
            b'q' => PieceType::Queen,
            b'k' => PieceType::King,
            b' ' => PieceType::Empty,
            _ => return Err(()),
        })
    }
}

impl Default for PieceType {
    fn default() -> Self {
        PieceType::Empty
    }
}

impl From<u8> for PieceType {
    fn from(value: u8) -> Self {
        unsafe { core::mem::transmute(value.clamp(0, 6)) }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum Piece {
    WhitePawn = 0,
    WhiteKnight = 1,
    WhiteBishop = 2,
    WhiteRook = 3,
    WhiteQueen = 4,
    WhiteKing = 5,
    BlackPawn = 6,
    BlackKnight = 7,
    BlackBishop = 8,
    BlackRook = 9,
    BlackQueen = 10,
    BlackKing = 11,
    Empty = 12,
}

impl ToString for Piece {
    fn to_string(&self) -> String {
        match *self {
            Piece::WhitePawn => 'p',
            Piece::WhiteKnight => 'n',
            Piece::WhiteBishop => 'b',
            Piece::WhiteRook => 'r',
            Piece::WhiteQueen => 'q',
            Piece::WhiteKing => 'k',
            Piece::BlackPawn => 'P',
            Piece::BlackKnight => 'N',
            Piece::BlackBishop => 'B',
            Piece::BlackRook => 'R',
            Piece::BlackQueen => 'Q',
            Piece::BlackKing => 'K',
            Piece::Empty => ' ',
        }
        .to_string()
    }
}

impl Default for Piece {
    fn default() -> Self {
        Piece::Empty
    }
}

impl Piece {
    #[inline]
    pub fn new(piece_type: PieceType, piece_color: PieceColor) -> Piece {
        if piece_type == PieceType::Empty {
            return Self::Empty;
        }
        (piece_color as u8 * 6 + piece_type as u8).try_into().unwrap()
    }

    pub fn get_color(&self) -> Option<PieceColor> {
        match *self as u8 {
            0..=5 => Some(PieceColor::White),
            6..=11 => Some(PieceColor::Black),
            _ => None,
        }
    }

    pub fn get_type(&self) -> PieceType {
        let type_u8 = (*self as u8) / 2;

        type_u8.into()
    }
}

impl From<u8> for Piece {
    fn from(value: u8) -> Self {
        unsafe { core::mem::transmute(value.clamp(0, 12)) }
    }
}

#[derive(Default)]
pub struct PieceIter {
    cur: u8,
}

impl PieceIter {
    pub fn new() -> Self {
        PieceIter { cur: 0 }
    }
}

impl Iterator for PieceIter {
    type Item = Piece;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == Piece::Empty as u8 {
            return None;
        }
        let piece = Piece::from(self.cur);
        self.cur += 1;
        Some(piece)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = Piece::Empty as usize;

        (size, Some(size))
    }
}

#[derive(Debug)]
pub struct NoSuchPieceError(char);

impl TryFrom<char> for Piece {
    type Error = NoSuchPieceError;

    fn try_from(c: char) -> Result<Self, NoSuchPieceError> {
        match c {
            'P' => Ok(Piece::WhitePawn),
            'p' => Ok(Piece::BlackPawn),
            'N' => Ok(Piece::WhiteKnight),
            'n' => Ok(Piece::BlackKnight),
            'B' => Ok(Piece::WhiteBishop),
            'b' => Ok(Piece::BlackBishop),
            'R' => Ok(Piece::WhiteRook),
            'r' => Ok(Piece::BlackRook),
            'Q' => Ok(Piece::WhiteQueen),
            'q' => Ok(Piece::BlackQueen),
            'K' => Ok(Piece::WhiteKing),
            'k' => Ok(Piece::BlackKing),
            _ => Err(NoSuchPieceError(c)),
        }
    }
}

impl From<Piece> for char {
    fn from(value: Piece) -> Self {
        match value {
            Piece::WhitePawn => 'P',
            Piece::BlackPawn => 'p',
            Piece::WhiteKnight => 'N',
            Piece::BlackKnight => 'n',
            Piece::WhiteBishop => 'B',
            Piece::BlackBishop => 'b',
            Piece::WhiteRook => 'R',
            Piece::BlackRook => 'r',
            Piece::WhiteQueen => 'Q',
            Piece::BlackQueen => 'q',
            Piece::WhiteKing => 'K',
            Piece::BlackKing => 'k',
            Piece::Empty => ' ',
        }
    }
}
