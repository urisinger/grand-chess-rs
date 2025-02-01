use std::ops::Index;
use std::ops::IndexMut;

use super::piece::Piece;
use super::piece::PieceColor;

#[derive(Default, Debug, Clone)]
pub struct BitBoards {
    pub pieces: [u64; 12],
    pub occupancy: [u64; 2],
}

impl Index<Piece> for BitBoards {
    type Output = u64;
    #[inline(always)]
    fn index(&self, index: Piece) -> &Self::Output {
        &self.pieces[index as usize]
    }
}

impl IndexMut<Piece> for BitBoards {
    #[inline(always)]
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self.pieces[index as usize]
    }
}

impl BitBoards {
    #[inline]
    pub fn col_occupancy(&self, color: PieceColor) -> u64 {
        self.occupancy[color as usize]
    }

    #[inline]
    pub fn occupancy(&self) -> u64 {
        self.occupancy[0] | self.occupancy[1]
    }

    pub fn to_mailbox(&self) -> [Piece; 64] {
        let mut arr = [Piece::default(); 64];

        for i in 0..Piece::Empty as usize {
            let mut mask = self.pieces[i];
            while mask != 0 {
                let piece = (i as u8).into();
                let index = mask.trailing_zeros();
                arr[index as usize] = piece;

                mask &= mask - 1;
            }
        }
        arr
    }

    #[inline]
    pub fn set_piece(&mut self, square: usize, piece: Piece) {
        let color = piece.get_color();
        self.occupancy[color as usize] |= 1 << square;

        self[piece] |= 1 << square;
    }

    #[inline]
    pub fn clear_square(&mut self, square: usize) {
        self.occupancy[0] &= !(1u64 << square);
        self.occupancy[1] &= !(1u64 << square);
        for i in 0..Piece::Empty as usize {
            self.pieces[i] &= !(1 << square);
        }
    }

    #[inline]
    pub fn clear_piece(&mut self, square: usize, piece: Piece) {
        self.occupancy[piece.get_color() as usize] &= !(1u64 << square);
        self[piece] &= !(1 << square);
    }

    #[inline]
    pub fn piece_at(&self, index: usize) -> Piece {
        for i in 0..Piece::Empty as usize {
            if (1 << index) & self.pieces[i] != 0 {
                return (i as u8).into();
            }
        }
        Piece::Empty
    }
}
