use std::ops::Shl;

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
