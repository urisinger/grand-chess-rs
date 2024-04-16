use core::slice;
use std::io::Read;

use board::piece::{Piece, PieceColor};
use bytemuck::cast_ref;

use super::{
    feature_transformer::{Accumulator, FeatureTransformer},
    network::{LayersBuffer, Network},
};

fn half_kp_index(king_sq: u32, piece_sq: u32, piece: Piece, prespective: PieceColor) -> usize {
    piece_sq as usize + (piece as usize + king_sq as usize * 10) * 64
}

//Num squares * (num_square * (num_pieces without prespective king))
const HALFKP_FEATURES: usize = 64 * (64 * (5 * 2 + 1));

pub struct HalfKP<const OUT: usize, const L_1_IN: usize, const L_2_IN: usize> {
    network: Network<OUT, L_1_IN, L_2_IN>,
    network_buffer: LayersBuffer<OUT, L_1_IN, L_2_IN>,

    feature_transformer: FeatureTransformer<i16, HALFKP_FEATURES, OUT>,
}

impl<const OUT: usize, const L_1_IN: usize, const L_2_IN: usize> HalfKP<OUT, L_1_IN, L_2_IN>
where
    [(); OUT / 2]:,
{
    pub fn load<R: Read>(&mut self, r: &mut R) {
        self.feature_transformer.load(r);
        self.network.load(r);
    }

    pub fn eval(&mut self, accumulator: &Accumulator<i16, OUT>) -> i32 {
        self.network.propagate(
            unsafe { slice::from_raw_parts(accumulator.accumulators.as_ptr() as *const i16, OUT) }
                .try_into()
                .unwrap(),
            &mut self.network_buffer,
        )
    }
}
