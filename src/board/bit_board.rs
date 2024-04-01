use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Shl;

use super::piece::Piece;
use super::piece::PieceColor;
use super::piece::PieceIter;

pub struct BitsIterator {
    bits: u64,
}

impl BitsIterator {
    pub fn new(bits: u64) -> Self {
        Self { bits }
    }
}

impl Iterator for BitsIterator {
    type Item = usize;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let index = self.bits.trailing_zeros() as usize;
            self.bits &= !(1 << index);
            Some(index)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.bits.count_ones() as usize;

        (size, Some(size))
    }
}

pub trait BitManipulation<T> {
    fn set_bit(&mut self, index: T);
    fn clear_bit(&mut self, index: T);
    fn flip_bit(&mut self, index: T);
}

impl<T> BitManipulation<T> for u64
where
    u64: Shl<T, Output = u64>,
{
    #[inline(always)]
    fn set_bit(&mut self, index: T) {
        *self |= 1u64 << index;
    }

    #[inline(always)]
    fn clear_bit(&mut self, index: T) {
        *self &= !(1u64 << index);
    }

    #[inline(always)]
    fn flip_bit(&mut self, index: T) {
        *self ^= 1u64 << index;
    }
}

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
