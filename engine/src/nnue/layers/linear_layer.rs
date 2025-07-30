use core::slice;
use std::io::Read;

use core::arch::x86_64::*;

use byteorder::{LittleEndian, ReadBytesExt};

use super::Layer;

pub struct LinearLayer<BT, const I: usize, const O: usize> {
    pub bias: [BT; O],
    pub weights: [[i8; I]; O],
}

impl<const I: usize, const O: usize> Layer<i8, i32, I, O> for LinearLayer<i32, I, O> {
    fn load(&mut self, r: &mut impl Read) {
        for i in 0..O {
            self.bias[i] = r.read_i32::<LittleEndian>().unwrap();
        }

        for i in 0..O {
            r.read_exact(unsafe {
                slice::from_raw_parts_mut(self.weights[i].as_mut_ptr() as *mut u8, I)
            })
            .unwrap();
        }
    }

    fn get_hash(prev_hash: u32) -> u32 {
        let mut hash_value = 0xCC03DAE4u32;
        hash_value = hash_value.overflowing_add(O as u32).0;
        hash_value ^= prev_hash >> 1;
        hash_value ^= prev_hash << 31;
        hash_value
    }

    #[cfg(target_feature = "avx2")]
    fn propagate(&self, input: &[i8; I], output: &mut [i32; O]) {
        if O == 1 {
            let mut sum = self.bias[0];
            for j in 0..I {
                sum += input[j] as i32 * self.weights[0][j] as i32;
            }

            output[0] = sum;
        } else {
            const REGISTER_WIDTH: usize = 256 / 8;
            assert!(I % REGISTER_WIDTH == 0, "Were proccesing 32 elements at a time");
            assert!(O % 4 == 0, "We unroll 4 at a time");

            let num_in_chunks: usize = I / REGISTER_WIDTH;
            let num_out_chunks: usize = O / 4;

            for i in 0..num_out_chunks {
                unsafe {
                    let mut sum0 = _mm256_setzero_si256();
                    let mut sum1 = _mm256_setzero_si256();
                    let mut sum2 = _mm256_setzero_si256();
                    let mut sum3 = _mm256_setzero_si256();

                    for j in 0..num_in_chunks {
                        let input =
                            _mm256_loadu_si256(&input[j * REGISTER_WIDTH] as *const i8 as *const _);

                        mm256_dpbusd_epi32(
                            &mut sum0,
                            input,
                            _mm256_loadu_si256(
                                &self.weights[i * 4 + 0][j * REGISTER_WIDTH] as *const i8
                                    as *const _,
                            ),
                        );

                        mm256_dpbusd_epi32(
                            &mut sum1,
                            input,
                            _mm256_loadu_si256(
                                &self.weights[i * 4 + 1][j * REGISTER_WIDTH] as *const i8
                                    as *const _,
                            ),
                        );
                        mm256_dpbusd_epi32(
                            &mut sum2,
                            input,
                            _mm256_loadu_si256(
                                &self.weights[i * 4 + 2][j * REGISTER_WIDTH] as *const i8
                                    as *const _,
                            ),
                        );
                        mm256_dpbusd_epi32(
                            &mut sum3,
                            input,
                            _mm256_loadu_si256(
                                &self.weights[i * 4 + 3][j * REGISTER_WIDTH] as *const i8
                                    as *const _,
                            ),
                        );
                    }

                    let bias = _mm_loadu_si128(&self.bias[i * 4] as *const i32 as *const _);

                    _mm_storeu_si128(
                        &mut output[i * 4] as *mut i32 as *mut _,
                        m256_haddx4(sum0, sum1, sum2, sum3, bias),
                    );
                }
            }
        }
    }
    #[cfg(not(target_feature = "avx2"))]
    fn propagate(&self, input: &[i8; I], output: &mut [i32; O]) {
        for i in 0..O {
            let mut sum = self.bias[i];
            for j in 0..I {
                sum += input[j] as i32 * self.weights[i * I + j] as i32;
            }

            output[i] = sum;
        }
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn mm256_dpbusd_epi32(acc: &mut __m256i, a: __m256i, b: __m256i) {
    let product = _mm256_maddubs_epi16(a, b);

    let one = _mm256_set1_epi16(1);
    let product = _mm256_madd_epi16(product, one);

    *acc = _mm256_add_epi32(*acc, product);
}

#[cfg(target_arch = "x86_64")]
unsafe fn m256_haddx4(
    sum0: core::arch::x86_64::__m256i,
    sum1: __m256i,
    sum2: __m256i,
    sum3: __m256i,
    bias: __m128i,
) -> __m128i {
    let sum0 = _mm256_hadd_epi32(sum0, sum1);
    let sum2 = _mm256_hadd_epi32(sum2, sum3);

    let sum0 = _mm256_hadd_epi32(sum0, sum2);

    _mm_add_epi32(
        _mm_add_epi32(_mm256_castsi256_si128(sum0), _mm256_extracti128_si256::<1>(sum0)),
        bias,
    )
}
