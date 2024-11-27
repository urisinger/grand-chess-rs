use std::{
    fmt::{Display, Write},
    ops::Not,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(u8)]
#[derive(Default)]
pub enum PieceColor {
    #[default]
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

impl TryFrom<u8> for PieceColor {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::Black as u8 {
            Err(())
        } else {
            unsafe { Ok(core::mem::transmute::<u8, PieceColor>(value)) }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(u8)]
#[derive(Default)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
    #[default]
    Empty = 6,
}

impl Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match *self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
            PieceType::Empty => ' ',
        })
    }
}

impl FromStr for PieceType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.as_bytes().first().ok_or(())? {
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

impl From<u8> for PieceType {
    #[inline]
    fn from(value: u8) -> Self {
        if value > 6 {
            return PieceType::Empty;
        }

        unsafe { core::mem::transmute(value) }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(u8)]
#[derive(Default)]
pub enum Piece {
    WhitePawn = 0,
    BlackPawn = 1,
    WhiteKnight = 2,
    BlackKnight = 3,
    WhiteBishop = 4,
    BlackBishop = 5,
    WhiteRook = 6,
    BlackRook = 7,
    WhiteQueen = 8,
    BlackQueen = 9,
    WhiteKing = 10,
    BlackKing = 11,
    #[default]
    Empty = 12,
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(char::from(*self))
    }
}

impl Piece {
    #[inline]
    pub fn new(piece_type: PieceType, piece_color: PieceColor) -> Piece {
        (piece_color as u8 + piece_type as u8 * 2).into()
    }

    #[inline]
    pub fn flip_color(self) -> Piece {
        unsafe { core::mem::transmute(self as u8 ^ 1) }
    }

    #[inline]
    pub fn get_color(&self) -> PieceColor {
        unsafe { core::mem::transmute(*self as u8 & 1) }
    }

    #[inline]
    pub fn get_type(&self) -> PieceType {
        unsafe { core::mem::transmute(*self as u8 / 2) }
    }
}

impl From<u8> for Piece {
    fn from(value: u8) -> Self {
        unsafe { core::mem::transmute(value.clamp(0, 12)) }
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
