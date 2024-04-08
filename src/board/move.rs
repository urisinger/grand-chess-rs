use std::fmt;

use super::{Piece, PieceType};

#[derive(Default, Clone, Copy)]
pub struct Move(pub u32);

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from_file: char = (b'a' + (self.from() % 8) as u8).into();
        let to_file: char = (b'a' + (self.to() % 8) as u8).into();
        f.write_fmt(format_args!(
            "{}{}{}{}",
            from_file,
            self.from() / 8 + 1,
            to_file,
            self.to() / 8 + 1
        ))?;

        if self.move_type() == MoveType::Promote {
            f.write_str(&self.piece().to_string().to_lowercase())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    QuietMove = 0,
    DoublePush = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Capture = 4,
    EnPassantCapture = 5,
    Promote = 6,
}

impl From<u8> for MoveType {
    fn from(value: u8) -> Self {
        unsafe { core::mem::transmute(value.clamp(0, 7)) }
    }
}

impl Move {
    #[inline]
    pub fn new(from: u32, to: u32, move_type: MoveType, piece: Piece, captured: PieceType) -> Self {
        let inner = ((captured as u32) & 0b111) << 20
            | ((piece as u32) & 0b1111) << 16
            | ((move_type as u32) & 0b1111) << 12
            | ((from & 0b111111) << 6)
            | (to & 0b111111);

        Self(inner)
    }

    #[inline]
    pub fn to(self) -> u32 {
        self.0 & 0b111111
    }

    #[inline]
    pub fn from(self) -> u32 {
        (self.0 >> 6) & 0b111111
    }

    #[inline]
    pub fn move_type(self) -> MoveType {
        (((self.0 >> 12) & 0b1111) as u8).into()
    }

    #[inline]
    pub fn piece(self) -> Piece {
        Piece::from(((self.0 >> 16) & 0b1111) as u8)
    }

    #[inline]
    pub fn captured(self) -> PieceType {
        PieceType::from(((self.0 >> 20) & 0b111) as u8)
    }

    #[inline]
    pub fn set_to(&mut self, to: u32) {
        self.0 &= !0b111111;
        self.0 |= to & 0b111111;
    }

    #[inline]
    pub fn set_from(&mut self, from: u32) {
        self.0 &= !(0b111111 << 6);
        self.0 |= (from & 0b111111) << 6;
    }

    #[inline]
    pub fn set_type(&mut self, move_type: MoveType) {
        self.0 &= !(0b1111 << 12);
        self.0 |= (move_type as u32) << 12;
    }

    #[inline]
    pub fn set_piece(&mut self, piece: Piece) {
        self.0 &= !(0b1111 << 16);
        self.0 |= (piece as u32 & 0b1111) << 16;
    }

    #[inline]
    pub fn set_captured(&mut self, captured: PieceType) {
        self.0 &= !(0b1111 << 20);
        self.0 |= (captured as u32 & 0b1111) << 20;
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{r#move::Move, Piece, PieceType};

    use super::MoveType;

    #[test]
    fn test_moves() {
        test_move(0, 63, MoveType::Promote, Piece::BlackKing, PieceType::Empty);

        test_move(0, 63, MoveType::KingCastle, Piece::WhiteBishop, PieceType::Rook);
    }

    fn test_move(from: u32, to: u32, move_type: MoveType, piece: Piece, captured: PieceType) {
        {
            let r#move = Move::new(from, to, move_type, piece, captured);

            assert_eq!(r#move.from(), from);
            assert_eq!(r#move.to(), to);
            assert_eq!(r#move.move_type(), move_type);
            assert_eq!(r#move.piece(), piece);
            assert_eq!(r#move.captured(), captured);
        }

        {
            let mut r#move = Move::default();

            r#move.set_to(to);
            r#move.set_from(from);
            r#move.set_type(move_type);
            r#move.set_piece(piece);
            r#move.set_captured(captured);

            assert_eq!(r#move.to(), to);
            assert_eq!(r#move.from(), from);
            assert_eq!(r#move.move_type(), move_type);
            assert_eq!(r#move.piece(), piece);
            assert_eq!(r#move.captured(), captured);
        }
    }
}
