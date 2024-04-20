use std::{io::Read, slice};

use board::piece::PieceColor;
use byteorder::{LittleEndian, ReadBytesExt};

#[repr(align(64))]
pub struct Accumulator<T, const OUT: usize>
where
    [(); OUT / 2]:,
{
    pub accumulators: [[T; OUT / 2]; 2],
}

impl<T, const OUT: usize> Accumulator<T, OUT>
where
    [(); OUT / 2]:,
{
    pub fn new_boxed() -> Box<Self> {
        unsafe { Box::from_raw(std::alloc::alloc(std::alloc::Layout::new::<Self>()) as *mut Self) }
    }
}

pub struct FeatureTransformer<WT, BT, const IN: usize, const OUT: usize> {
    bias: [BT; OUT],
    weights: [[WT; OUT]; IN],
}

impl<const OUT: usize, const IN: usize> FeatureTransformer<i16, i16, IN, OUT>
where
    [(); OUT / 2]:,
{
    pub fn load<R: Read>(&mut self, r: &mut R) {
        for i in 0..OUT / 2 {
            self.bias[i] = r.read_i16::<LittleEndian>().unwrap();
        }

        for i in 0..IN {
            for j in 0..OUT / 2 {
                self.weights[i][j] = r.read_i16::<LittleEndian>().unwrap();
            }
        }
    }

    pub const fn get_hash() -> u32 {
        OUT as u32
    }

    pub fn transform(&self, acc: &Accumulator<i16, OUT>, output: &mut [i8; OUT]) {
        let input: [i16; OUT] =
            unsafe { slice::from_raw_parts(acc.accumulators.as_ptr() as *const i16, OUT) }
                .try_into()
                .unwrap();
        for i in 0..OUT {
            output[i] = input[i].max(0).min(127) as i8;

            print!("{} ", input[i]);
        }

        println!("");
    }

    pub fn refresh(
        &self,
        acc: &mut Accumulator<i16, OUT>,
        features: &Vec<usize>,
        perspective: PieceColor,
    ) {
        for i in 0..OUT / 2 {
            acc.accumulators[perspective as usize][i] = self.bias[i];
        }

        for feature in features {
            for i in 0..OUT / 2 {
                acc.accumulators[perspective as usize][i] += self.weights[*feature][i];
            }
        }
    }

    pub fn update_incremental(
        &mut self,
        acc: &mut Accumulator<i16, OUT>,
        prev_acc: &Accumulator<i16, OUT>,
        added_features: &Vec<usize>,
        removed_features: &Vec<usize>,
        prespective: PieceColor,
    ) {
        acc.accumulators[prespective as usize]
            .copy_from_slice(&prev_acc.accumulators[prespective as usize]);

        for r in removed_features {
            for i in 0..OUT / 2 {
                acc.accumulators[prespective as usize][i] -= self.weights[*r][i];
            }
        }

        for a in added_features {
            for i in 0..OUT / 2 {
                acc.accumulators[prespective as usize][i] += self.weights[*a][i];
            }
        }
    }
}
