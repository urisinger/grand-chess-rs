use std::io::Read;

use board::piece::PieceColor;
use byteorder::{LittleEndian, ReadBytesExt};

pub struct Accumulator<T, const OUT: usize>
where
    [(); OUT / 2]:,
{
    pub accumulators: [[T; OUT / 2]; 2],
}

pub struct FeatureTransformer<T, const IN: usize, const OUT: usize> {
    pub bias: [T; OUT],
    pub weights: [[T; OUT]; IN],
}

#[allow(dead_code)]
impl<const OUT: usize, const IN: usize> FeatureTransformer<i16, IN, OUT>
where
    [(); OUT / 2]:,
{
    pub fn load<R: Read>(&mut self, r: &mut R) {
        for i in 0..OUT {
            self.bias[i] = r.read_i16::<LittleEndian>().unwrap();
        }

        for i in 0..IN {
            for j in 0..OUT {
                self.weights[i][j] = r.read_i16::<LittleEndian>().unwrap();
            }
        }
    }

    pub fn refresh(
        &self,
        acc: &mut Accumulator<i16, OUT>,
        features: Vec<usize>,
        perspective: PieceColor,
    ) {
        for i in 0..OUT / 2 {
            acc.accumulators[perspective as usize][i] = self.bias[i];
        }

        for feature in features {
            for i in 0..OUT / 2 {
                acc.accumulators[perspective as usize][i] += self.weights[feature][i];
            }
        }
    }

    pub fn update_incremental(
        &mut self,
        acc: &mut Accumulator<i16, OUT>,
        prev_acc: &Accumulator<i16, OUT>,
        added_features: Vec<usize>,
        removed_features: Vec<usize>,
        prespective: PieceColor,
    ) {
        acc.accumulators[prespective as usize]
            .copy_from_slice(&prev_acc.accumulators[prespective as usize]);

        for r in removed_features {
            for i in 0..OUT / 2 {
                acc.accumulators[prespective as usize][i] -= self.weights[r][i];
            }
        }

        for a in added_features {
            for i in 0..OUT / 2 {
                acc.accumulators[prespective as usize][i] += self.weights[a][i];
            }
        }
    }
}
