use core::arch::x86_64::*;
use core::slice;
use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

pub struct LinearLayer<const I: usize, const O: usize> {
    pub bias: [i32; O],
    pub weights: [[i8; I]; O],
}

impl<const I: usize, const O: usize> LinearLayer<I, O> {
    pub fn load(&mut self, r: &mut impl Read) {
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

    pub fn get_hash(prev_hash: u32) -> u32 {
        let mut hash_value = 0xCC03DAE4u32;
        hash_value = hash_value.overflowing_add(O as u32).0;
        hash_value ^= prev_hash >> 1;
        hash_value ^= prev_hash << 31;
        hash_value
    }

    /// Computes:
    ///
    /// output = clamp((bias + dot(input, weights)) >> 6, 0, 127)
    ///
    /// Four output neurons are processed at once.
    #[cfg(target_feature = "avx2")]
    #[inline]
    pub fn propagate_relu(&self, input: &[i8; I], output: &mut [i8; O]) {
        const REGISTER_WIDTH: usize = 256 / 8;

        assert!(I % REGISTER_WIDTH == 0, "We're processing 32 elements at a time");

        assert!(O % 4 == 0, "We're processing 4 outputs at a time");

        let num_in_chunks = I / REGISTER_WIDTH;
        let num_out_chunks = O / 4;

        unsafe {
            let zero = _mm_setzero_si128();

            for i in 0..num_out_chunks {
                let mut sum0 = _mm256_setzero_si256();
                let mut sum1 = _mm256_setzero_si256();
                let mut sum2 = _mm256_setzero_si256();
                let mut sum3 = _mm256_setzero_si256();

                for j in 0..num_in_chunks {
                    let offset = j * REGISTER_WIDTH;

                    let x = _mm256_loadu_si256(input.as_ptr().add(offset) as *const __m256i);

                    sum0 = mm256_dpbusd_epi32(
                        sum0,
                        x,
                        _mm256_loadu_si256(
                            self.weights[i * 4].as_ptr().add(offset) as *const __m256i
                        ),
                    );

                    sum1 = mm256_dpbusd_epi32(
                        sum1,
                        x,
                        _mm256_loadu_si256(
                            self.weights[i * 4 + 1].as_ptr().add(offset) as *const __m256i
                        ),
                    );

                    sum2 = mm256_dpbusd_epi32(
                        sum2,
                        x,
                        _mm256_loadu_si256(
                            self.weights[i * 4 + 2].as_ptr().add(offset) as *const __m256i
                        ),
                    );

                    sum3 = mm256_dpbusd_epi32(
                        sum3,
                        x,
                        _mm256_loadu_si256(
                            self.weights[i * 4 + 3].as_ptr().add(offset) as *const __m256i
                        ),
                    );
                }

                let bias = _mm_loadu_si128(self.bias.as_ptr().add(i * 4) as *const __m128i);

                // Four complete i32 neuron outputs.
                let result = m256_haddx4(sum0, sum1, sum2, sum3, bias);

                // Quantization scale: / 64.
                let result = _mm_srai_epi32::<6>(result);

                // ReLU.
                let result = _mm_max_epi32(result, zero);

                // 4 x i32 -> 4 x i16.
                let result = _mm_packs_epi32(result, zero);

                // 4 x i16 -> 4 x i8.
                //
                // Positive values > 127 saturate to 127, giving us
                // the upper clipping for free.
                let result = _mm_packs_epi16(result, zero);

                // The first four bytes contain our four outputs.
                let packed = _mm_cvtsi128_si32(result);
                (output.as_mut_ptr().add(i * 4) as *mut u32).write_unaligned(packed as u32);
            }
        }
    }

    #[cfg(not(target_feature = "avx2"))]
    #[inline]
    pub fn propagate_relu(&self, input: &[i8; I], output: &mut [i8; O]) {
        for i in 0..O {
            let mut sum = self.bias[i];

            for j in 0..I {
                sum += input[j] as i32 * self.weights[i][j] as i32;
            }

            output[i] = (sum >> 6).clamp(0, 127) as i8;
        }
    }
}

impl<const I: usize> LinearLayer<I, 1> {
    #[inline(always)]
    pub fn propagate_final(&self, input: &[i8; I]) -> i32 {
        let mut sum = self.bias[0];

        for j in 0..I {
            sum += input[j] as i32 * self.weights[0][j] as i32;
        }

        sum
    }
}

#[cfg(target_feature = "avx2")]
#[inline(always)]
unsafe fn mm256_dpbusd_epi32(src: __m256i, a: __m256i, b: __m256i) -> __m256i {
    #[cfg(target_feature = "avx512vnni")]
    {
        _mm256_dpbusd_epi32(src, a, b)
    }
    #[cfg(not(target_feature = "avx512vnni"))]
    {
        let product = _mm256_maddubs_epi16(a, b);

        let one = _mm256_set1_epi16(1);
        let product = _mm256_madd_epi16(product, one);

        _mm256_add_epi32(src, product)
    }
}

#[cfg(target_feature = "avx2")]
#[inline(always)]
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
