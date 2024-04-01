use crate::common::BitsIterator;
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

impl BitBoards {
    pub fn get_occupancy(&self, color: PieceColor) -> u64 {
        self.occupancy[color as usize]
    }
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
    pub fn to_mailbox(&self) -> [Piece; 64] {
        let mut arr = [Piece::default(); 64];

        for piece in PieceIter::new() {
            for index in BitsIterator::new(self[piece]) {
                arr[index] = piece;
            }
        }
        arr
    }
}
