use std::{
    arch::x86_64::{
        _mm256_add_epi16, _mm256_loadu_si256, _mm256_max_epi8, _mm256_packs_epi16,
        _mm256_permute4x64_epi64, _mm256_setzero_si256, _mm256_storeu_si256, _mm256_sub_epi16,
    },
    io::Read,
};

use crate::board::piece::PieceColor;
use byteorder::{LittleEndian, ReadBytesExt};

#[repr(align(64))]
pub struct Accumulator<T, const OUT: usize> {
    pub accumulators: [T; OUT],
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

    pub fn transform(
        &self,
        acc: &Accumulator<i16, OUT>,
        output: &mut [i8; OUT],
        prespective: PieceColor,
    ) {
        let prespectives = [prespective, !prespective];
        for c in 0..=1 {
            let offset = if prespectives[c] == PieceColor::White { 0 } else { 1 } * OUT / 2;

            const IN_REGISTER_WIDTH: usize = 256 / 16;
            const OUT_REGISTER_WIDTH: usize = 256 / 8;
            assert!(OUT % OUT_REGISTER_WIDTH == 0, "We're processing 32 elements at a time");
            let num_of_chunks: usize = (OUT / 2) / OUT_REGISTER_WIDTH;

            let zero = unsafe { _mm256_setzero_si256() };
            const CONTROL: i32 = 0b11011000;

            for i in 0..num_of_chunks {
                unsafe {
                    let in0 = _mm256_loadu_si256(
                        &acc.accumulators[offset + (i * 2 + 0) * IN_REGISTER_WIDTH] as *const i16
                            as *const _,
                    );
                    let in1 = _mm256_loadu_si256(
                        &acc.accumulators[offset + (i * 2 + 1) * IN_REGISTER_WIDTH] as *const i16
                            as *const _,
                    );

                    let result = _mm256_permute4x64_epi64(
                        _mm256_max_epi8(_mm256_packs_epi16(in0, in1), zero),
                        CONTROL,
                    );

                    _mm256_storeu_si256(
                        &mut output[c * OUT / 2 + i * OUT_REGISTER_WIDTH] as *mut i8 as *mut _,
                        result,
                    );
                }
            }
        }
    }

    pub fn refresh(
        &self,
        acc: &mut Accumulator<i16, OUT>,
        features: &[usize],
        perspective: PieceColor,
    ) {
        const REGISTER_WIDTH: usize = 256 / 16;
        let offset = if perspective == PieceColor::White { 0 } else { 1 } * OUT / 2;

        const NUM_REGISTERS: usize = 16;

        let num_chunks = OUT / (2 * NUM_REGISTERS * REGISTER_WIDTH);

        let zero = unsafe { _mm256_setzero_si256() };
        let mut regs = [zero; NUM_REGISTERS];

        for i in 0..num_chunks {
            for j in 0..NUM_REGISTERS {
                unsafe {
                    regs[j] = _mm256_loadu_si256(
                        &self.bias[NUM_REGISTERS * REGISTER_WIDTH * i + j * REGISTER_WIDTH]
                            as *const i16 as *const _,
                    );
                }
            }

            for feature in features.iter() {
                for j in 0..NUM_REGISTERS {
                    regs[j] = unsafe {
                        _mm256_add_epi16(
                            regs[j],
                            _mm256_loadu_si256(
                                &self.weights[*feature]
                                    [NUM_REGISTERS * REGISTER_WIDTH * i + j * REGISTER_WIDTH]
                                    as *const i16 as *const _,
                            ),
                        )
                    };
                }
            }

            for j in 0..NUM_REGISTERS {
                unsafe {
                    _mm256_storeu_si256(
                        &mut acc.accumulators
                            [offset + i * NUM_REGISTERS * REGISTER_WIDTH + j * REGISTER_WIDTH]
                            as *mut i16 as *mut _,
                        regs[j],
                    );
                }
            }
        }
    }

    pub fn update_incremental(
        &mut self,
        acc: &mut Accumulator<i16, OUT>,
        prev_acc: &Accumulator<i16, OUT>,
        added_features: &[usize],
        removed_features: &[usize],
        prespective: PieceColor,
    ) {
        if cfg!(target_arch = "x86_64") {
            const REGISTER_WIDTH: usize = 256 / 16;
            let offset = if prespective == PieceColor::White { 0 } else { 1 } * OUT / 2;

            const NUM_REGISTERS: usize = 16;

            let num_chunks = OUT / (2 * NUM_REGISTERS * REGISTER_WIDTH);

            let zero = unsafe { _mm256_setzero_si256() };
            let mut regs = [zero; NUM_REGISTERS];

            for i in 0..num_chunks {
                for j in 0..NUM_REGISTERS {
                    unsafe {
                        regs[j] = _mm256_loadu_si256(
                            &prev_acc.accumulators
                                [offset + NUM_REGISTERS * REGISTER_WIDTH * i + j * REGISTER_WIDTH]
                                as *const i16 as *const _,
                        );
                    }
                }

                for feature in added_features {
                    for j in 0..NUM_REGISTERS {
                        regs[j] = unsafe {
                            _mm256_add_epi16(
                                regs[j],
                                _mm256_loadu_si256(
                                    &self.weights[*feature]
                                        [NUM_REGISTERS * REGISTER_WIDTH * i + j * REGISTER_WIDTH]
                                        as *const i16
                                        as *const _,
                                ),
                            )
                        };
                    }
                }

                for feature in removed_features {
                    for j in 0..NUM_REGISTERS {
                        regs[j] = unsafe {
                            _mm256_sub_epi16(
                                regs[j],
                                _mm256_loadu_si256(
                                    &self.weights[*feature]
                                        [NUM_REGISTERS * REGISTER_WIDTH * i + j * REGISTER_WIDTH]
                                        as *const i16
                                        as *const _,
                                ),
                            )
                        };
                    }
                }

                for j in 0..NUM_REGISTERS {
                    unsafe {
                        _mm256_storeu_si256(
                            &mut acc.accumulators
                                [offset + i * NUM_REGISTERS * REGISTER_WIDTH + j * REGISTER_WIDTH]
                                as *mut i16 as *mut _,
                            regs[j],
                        );
                    }
                }
            }
        } else {
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
}
