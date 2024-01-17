use crate::piece::*;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Shl;

pub struct BitsIterator {
    bits: u64,
}

impl Iterator for BitsIterator {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let index = self.bits.trailing_zeros();
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
    type Iter: Iterator<Item = u32>;

    fn set_bit(&mut self, index: T);
    fn clear_bit(&mut self, index: T);
    fn flip_bit(&mut self, index: T);
    fn iter_ones(&self) -> Self::Iter;
}

impl<T> BitManipulation<T> for u64
where
    u64: Shl<T, Output = u64>,
{
    type Iter = BitsIterator;

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

    #[inline(always)]
    fn iter_ones(&self) -> Self::Iter {
        BitsIterator { bits: *self }
    }
}

#[derive(Default, Debug)]
pub struct PiecesBitBoards {
    pub pieces: [u64; 6],
    pub occupancy: u64,
}

impl PiecesBitBoards {
    pub fn get_occupancy(&self) -> u64 {
        self.occupancy
    }
}
impl Index<PieceType> for PiecesBitBoards {
    type Output = u64;
    #[inline(always)]
    fn index(&self, index: PieceType) -> &Self::Output {
        &self.pieces[index as usize]
    }
}

impl IndexMut<PieceType> for PiecesBitBoards {
    #[inline(always)]
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        &mut self.pieces[index as usize]
    }
}

#[derive(Default, Debug)]
pub struct BitBoards {
    pieces: [PiecesBitBoards; 2],
}

impl Index<PieceColor> for BitBoards {
    type Output = PiecesBitBoards;
    #[inline(always)]
    fn index(&self, index: PieceColor) -> &Self::Output {
        &self.pieces[index as usize]
    }
}

impl IndexMut<PieceColor> for BitBoards {
    #[inline(always)]
    fn index_mut(&mut self, index: PieceColor) -> &mut Self::Output {
        &mut self.pieces[index as usize]
    }
}

impl BitBoards {
    fn to_mailbox(&self) -> [Piece; 64] {
        let arr = [Piece::default(); 64];
            
        for index in self[PieceColor::White][PieceType::Pawn]
        arr
    }

    fn get_piece(&self, index : u32) -> Piece{
        
    }
}
