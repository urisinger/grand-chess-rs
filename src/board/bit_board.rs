use std::ops::Index;
use std::ops::IndexMut;

use super::piece::Piece;
use super::piece::PieceColor;
use super::piece::PieceIter;

#[derive(Default, Debug)]
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

        for piece in PieceIter::new() {
            let mut mask = self[piece];
            while mask != 0 {
                let index = mask.trailing_zeros();
                arr[index as usize] = piece;

                mask &= mask - 1;
            }
        }
        arr
    }

    pub fn set_piece(&mut self, square: usize, piece: Piece) {
        if let Some(color) = piece.get_color() {
            self.occupancy[color as usize] |= 1 << square;
            self.occupancy[!color as usize] &= !(1 << square);
            for piece in PieceIter::new() {
                self[piece] &= !(1 << square);
            }

            self[piece] |= 1 << square;
        } else {
            self.occupancy[PieceColor::White as usize] &= !(1 << square);
            self.occupancy[PieceColor::Black as usize] &= !(1 << square);
            for piece in PieceIter::new() {
                self[piece] &= !(1 << square);
            }
        }
    }

    pub fn piece_at(&self, index: usize) -> Piece {
        for piece in PieceIter::new() {
            if (1 << index) & self[piece] != 0 {
                return piece;
            }
        }
        Piece::Empty
    }
}
