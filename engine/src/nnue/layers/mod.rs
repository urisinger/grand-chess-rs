use std::{
    arch::x86_64::{
        __m256i, _mm256_load_si256, _mm256_loadu_si256, _mm256_max_epi8, _mm256_packs_epi16,
        _mm256_packs_epi32, _mm256_permute4x64_epi64, _mm256_permutevar8x32_epi32,
        _mm256_set_epi32, _mm256_setzero_si256, _mm256_store_si256, _mm256_storeu_si256,
    },
    io::Read,
    marker::PhantomData,
};

use byteorder::{LittleEndian, ReadBytesExt};

pub trait Layer<IT, OT, const I: usize, const O: usize> {
    fn propagate_unaligned(&self, input: &[IT; I], output: &mut [OT; O]);

    // aligned to 32 bytes
    fn propagate_aligned(&self, input: &[IT; I], output: &mut [OT; O]) {
        self.propagate_unaligned(input, output)
    }

    fn load<R: Read>(&mut self, r: &mut R) {}
}

pub struct LinearLayer<WT, BT, const I: usize, const O: usize> {
    pub bias: [BT; O],
    pub weights: [[WT; O]; I],
}

impl<const I: usize, const O: usize> Layer<i8, i32, I, O> for LinearLayer<i8, i32, I, O> {
    fn load<R: Read>(&mut self, r: &mut R) {
        for i in 0..O {
            self.bias[i] = r.read_i32::<LittleEndian>().unwrap();
        }

        for i in 0..I {
            for j in 0..O {
                self.weights[i][j] = r.read_i8().unwrap();
            }
        }
    }

    fn propagate_unaligned(&self, input: &[i8; I], output: &mut [i32; O]) {
        for i in 0..O {
            output[i] = self.bias[i];
        }

        for i in 0..I {
            for j in 0..O {
                output[i] += input[i] as i32 * self.weights[i][j] as i32;
            }
        }
    }
}

#[derive(Default)]
pub struct ReluLayer<I, O, const N: usize>(PhantomData<I>, PhantomData<O>);

impl<const N: usize> Layer<i16, i8, N, N> for ReluLayer<i16, i8, N> {
    fn propagate_unaligned(&self, input: &[i16; N], output: &mut [i8; N]) {
        const IN_REGISTER_WIDTH: usize = 256 / 16;
        const OUT_REGISTER_WIDTH: usize = 256 / 8;
        assert!(N % OUT_REGISTER_WIDTH == 0, "We're processing 32 elements at a time");
        let num_of_chunks: usize = N / OUT_REGISTER_WIDTH;

        let zero = unsafe { _mm256_setzero_si256() };
        const CONTROL: i32 = 0b11011000;

        for i in 0..num_of_chunks {
            unsafe {
                let in0 = _mm256_loadu_si256(
                    &input[(i * 2 + 0) * IN_REGISTER_WIDTH] as *const i16 as *const __m256i,
                );

                let in1 = _mm256_loadu_si256(
                    &input[(i * 2 + 1) * IN_REGISTER_WIDTH] as *const i16 as *const __m256i,
                );

                let result = _mm256_permute4x64_epi64(
                    // clamp from below
                    _mm256_max_epi8(
                        // packs saturates to 127, so we only need to clamp from below
                        _mm256_packs_epi16(in0, in1),
                        zero,
                    ),
                    CONTROL,
                );

                _mm256_storeu_si256(
                    (&mut output[i * OUT_REGISTER_WIDTH]) as *mut i8 as *mut __m256i,
                    result,
                );
            }
        }
    }

    fn propagate_aligned(&self, input: &[i16; N], output: &mut [i8; N]) {
        const IN_REGISTER_WIDTH: usize = 256 / 16;
        const OUT_REGISTER_WIDTH: usize = 256 / 8;
        assert!(N % OUT_REGISTER_WIDTH == 0, "We're processing 32 elements at a time");
        let num_of_chunks: usize = N / OUT_REGISTER_WIDTH;

        let zero = unsafe { _mm256_setzero_si256() };
        const CONTROL: i32 = 0b11011000;

        for i in 0..num_of_chunks {
            unsafe {
                let in0 = _mm256_load_si256(
                    &input[(i * 2 + 0) * IN_REGISTER_WIDTH] as *const i16 as *const __m256i,
                );

                let in1 = _mm256_load_si256(
                    &input[(i * 2 + 1) * IN_REGISTER_WIDTH] as *const i16 as *const __m256i,
                );

                let result = _mm256_permute4x64_epi64(
                    // clamp from below
                    _mm256_max_epi8(
                        // packs saturates to 127, so we only need to clamp from below
                        _mm256_packs_epi16(in0, in1),
                        zero,
                    ),
                    CONTROL,
                );

                _mm256_store_si256(
                    (&mut output[i * OUT_REGISTER_WIDTH]) as *mut i8 as *mut __m256i,
                    result,
                );
            }
        }
    }
}

impl<const N: usize> Layer<i32, i8, N, N> for ReluLayer<i32, i8, N> {
    fn propagate_unaligned(&self, input: &[i32; N], output: &mut [i8; N]) {
        const IN_REGISTER_WIDTH: usize = 256 / 32;
        const OUT_REGISTER_WIDTH: usize = 256 / 8;
        assert!(N % OUT_REGISTER_WIDTH == 0, "We're processing 32 elements at a time");
        let num_of_chunks: usize = N / OUT_REGISTER_WIDTH;

        let zero = unsafe { _mm256_setzero_si256() };
        let control = unsafe { _mm256_set_epi32(7, 3, 6, 2, 5, 1, 4, 0) };

        for i in 0..num_of_chunks {
            unsafe {
                let in0 = _mm256_packs_epi32(
                    _mm256_loadu_si256(
                        &input[(i * 4 + 0) * IN_REGISTER_WIDTH] as *const i32 as *const _,
                    ),
                    _mm256_loadu_si256(
                        &input[(i * 4 + 1) * IN_REGISTER_WIDTH] as *const i32 as *const _,
                    ),
                );
                let in1 = _mm256_packs_epi32(
                    _mm256_loadu_si256(
                        &input[(i * 4 + 2) * IN_REGISTER_WIDTH] as *const i32 as *const _,
                    ),
                    _mm256_loadu_si256(
                        &input[(i * 4 + 3) * IN_REGISTER_WIDTH] as *const i32 as *const _,
                    ),
                );

                let result = _mm256_permutevar8x32_epi32(
                    _mm256_max_epi8(_mm256_packs_epi16(in0, in1), zero),
                    control,
                );

                _mm256_storeu_si256(
                    &mut output[i * OUT_REGISTER_WIDTH] as *mut i8 as *mut _,
                    result,
                );
            }
        }
    }

    fn propagate_aligned(&self, input: &[i32; N], output: &mut [i8; N]) {
        const IN_REGISTER_WIDTH: usize = 256 / 32;
        const OUT_REGISTER_WIDTH: usize = 256 / 8;
        assert!(N % OUT_REGISTER_WIDTH == 0, "We're processing 32 elements at a time");
        let num_of_chunks: usize = N / OUT_REGISTER_WIDTH;

        let zero = unsafe { _mm256_setzero_si256() };
        let control = unsafe { _mm256_set_epi32(7, 3, 6, 2, 5, 1, 4, 0) };

        for i in 0..num_of_chunks {
            unsafe {
                let in0 = _mm256_packs_epi32(
                    _mm256_load_si256(
                        &input[(i * 4 + 0) * IN_REGISTER_WIDTH] as *const i32 as *const _,
                    ),
                    _mm256_load_si256(
                        &input[(i * 4 + 1) * IN_REGISTER_WIDTH] as *const i32 as *const _,
                    ),
                );
                let in1 = _mm256_packs_epi32(
                    _mm256_load_si256(
                        &input[(i * 4 + 2) * IN_REGISTER_WIDTH] as *const i32 as *const _,
                    ),
                    _mm256_load_si256(
                        &input[(i * 4 + 3) * IN_REGISTER_WIDTH] as *const i32 as *const _,
                    ),
                );

                let result = _mm256_permutevar8x32_epi32(
                    _mm256_max_epi8(_mm256_packs_epi16(in0, in1), zero),
                    control,
                );

                _mm256_store_si256(
                    &mut output[i * OUT_REGISTER_WIDTH] as *mut i8 as *mut _,
                    result,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::array::from_fn;

    use super::{Layer, ReluLayer};

    #[test]
    fn relu_test() {
        {
            let input: [i16; 128] = from_fn(|index| (index as i16).wrapping_mul(1000));
            let mut output = [0i8; 128];

            let relu = ReluLayer::default();

            relu.propagate_unaligned(&input, &mut output);

            for i in 0..128 {
                assert_eq!(input[i].clamp(0, 127) as i8, output[i]);
            }
        }

        {
            let input: [i32; 256] = from_fn(|index| (index as i32).wrapping_mul(1000));
            let mut output = [0i8; 256];

            let relu = ReluLayer::default();

            relu.propagate_unaligned(&input, &mut output);

            for i in 0..32 {
                assert_eq!(input[i].clamp(0, 127) as i8, output[i]);
            }
        }
    }
}
