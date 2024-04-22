pub mod crelu;
pub mod linear_layer;

use std::io::Read;

pub trait Layer<IT, OT, const I: usize, const O: usize> {
    fn propagate(&self, input: &[IT; I], output: &mut [OT; O]);

    fn load(&mut self, _: &mut impl Read) {}

    fn get_hash(prev_hash: u32) -> u32;
}

#[cfg(test)]
mod tests {
    use std::array::from_fn;

    use test::Bencher;

    use super::{crelu::ReluLayer, Layer};

    extern crate test;

    #[test]
    fn relu_test() {
        {
            let input: [i32; 256 * 256] = from_fn(|index| (i32::MIN) + index as i32);
            let mut output = [0i8; 256 * 256];

            let relu = ReluLayer::default();

            relu.propagate(&input, &mut output);

            for i in 0..256 * 256 {
                assert_eq!((input[i] >> 6).clamp(0, 127) as i8, output[i]);
            }
        }

        {
            let input: [i32; 256 * 256] = from_fn(|index| (i32::MAX) - index as i32);
            let mut output = [0i8; 256 * 256];

            let relu = ReluLayer::default();

            relu.propagate(&input, &mut output);

            for i in 0..256 * 256 {
                assert_eq!((input[i] >> 6).clamp(0, 127) as i8, output[i]);
            }
        }
    }

    #[bench]
    fn relu_bench(bencher: &mut Bencher) {
        bencher.iter(|| {
            let input: [i32; 256] = from_fn(|index| (index as i32).wrapping_mul(1000));
            let mut output = [0i8; 256];

            let relu = ReluLayer::default();

            relu.propagate(&input, &mut output);

            output
        })
    }
}
