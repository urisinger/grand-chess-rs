use std::{io::Read, slice};

use board::piece::PieceColor;
use byteorder::{LittleEndian, ReadBytesExt};

#[repr(align(64))]
pub struct Accumulator<T, const OUT: usize> {
    pub accumulators: [T; OUT],
}

impl<T, const OUT: usize> Accumulator<T, OUT> {
    pub fn new_boxed() -> Box<Self> {
        unsafe { Box::from_raw(std::alloc::alloc(std::alloc::Layout::new::<Self>()) as *mut Self) }
    }
}

pub struct FeatureTransformer<WT, BT, const IN: usize, const OUT: usize>
where
    [(); OUT / 2]:,
{
    bias: [BT; OUT / 2],
    weights: [[WT; OUT / 2]; IN],
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

    pub fn transform(
        &self,
        acc: &Accumulator<i16, OUT>,
        output: &mut [i8; OUT],
        prespective: PieceColor,
    ) {
        let prespectives = [prespective, !prespective];
        for c in 0..=1 {
            let offset = if prespectives[c] == PieceColor::White { 0 } else { 1 } * OUT / 2;

            for i in 0..OUT / 2 {
                output[c * OUT / 2 + i] = acc.accumulators[offset + i].clamp(0, 127) as i8;
            }
        }
        println!("");
    }

    pub fn refresh(
        &self,
        acc: &mut Accumulator<i16, OUT>,
        features: &Vec<usize>,
        perspective: PieceColor,
    ) {
        let offset = if perspective == PieceColor::White { 0 } else { 1 } * OUT / 2;

        for i in 0..OUT / 2 {
            acc.accumulators[offset + i] = self.bias[i];
        }

        for feature in features {
            for i in 0..OUT / 2 {
                acc.accumulators[offset + i] += self.weights[*feature][i];
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
        let offset = prespective as usize * OUT / 2;
        acc.accumulators[offset..offset + OUT / 2]
            .copy_from_slice(&prev_acc.accumulators[offset..offset + OUT / 2]);

        for r in removed_features {
            for i in 0..OUT / 2 {
                acc.accumulators[offset + i] -= self.weights[*r][i];
            }
        }

        for a in added_features {
            for i in 0..OUT / 2 {
                acc.accumulators[offset + i] += self.weights[*a][i];
            }
        }
    }
}
