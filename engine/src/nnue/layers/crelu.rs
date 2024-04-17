use std::{
    arch::x86_64::{
        __m256i, _mm256_loadu_si256, _mm256_max_epi8, _mm256_packs_epi16, _mm256_packs_epi32,
        _mm256_permute4x64_epi64, _mm256_permutevar8x32_epi32, _mm256_set_epi32,
        _mm256_setzero_si256, _mm256_storeu_si256,
    },
    marker::PhantomData,
};

use super::Layer;

#[derive(Default)]
pub struct ReluLayer<I, O, const N: usize>(PhantomData<I>, PhantomData<O>);

impl<const N: usize> Layer<i16, i8, N, N> for ReluLayer<i16, i8, N> {
    fn propagate(&self, input: &[i16; N], output: &mut [i8; N]) {
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

    fn get_hash(prev_hash: u32) -> u32 {
        0x538D24C7u32.overflowing_add(prev_hash).0
    }
}

impl<const N: usize> Layer<i32, i8, N, N> for ReluLayer<i32, i8, N> {
    fn propagate(&self, input: &[i32; N], output: &mut [i8; N]) {
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

    fn get_hash(prev_hash: u32) -> u32 {
        0x538D24C7u32.overflowing_add(prev_hash).0
    }
}
