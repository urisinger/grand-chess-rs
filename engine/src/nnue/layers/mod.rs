pub mod crelu;
pub mod linear_layer;

use std::io::Read;

pub trait Layer<IT, OT, const I: usize, const O: usize> {
    fn propagate(&self, input: &[IT; I], output: &mut [OT; O]);

    fn load<R: Read>(&mut self, _: &mut R) {}

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
            let input: [i16; 128] = from_fn(|index| (index as i16).wrapping_mul(1000));
            let mut output = [0i8; 128];

            let relu = ReluLayer::default();

            relu.propagate(&input, &mut output);

            for i in 0..128 {
                assert_eq!(input[i].clamp(0, 127) as i8, output[i]);
            }
        }

        {
            let input: [i32; 256] = from_fn(|index| (index as i32).wrapping_mul(1000));
            let mut output = [0i8; 256];

            let relu = ReluLayer::default();

            relu.propagate(&input, &mut output);

            for i in 0..32 {
                assert_eq!(input[i].clamp(0, 127) as i8, output[i]);
            }
        }
    }

    #[bench]
    fn relu_bench(bencher: &mut Bencher) {
        bencher.iter(|| {
            let input: [i32; 32] = from_fn(|index| (index as i32).wrapping_mul(1000));
            let mut output = [0i8; 32];

            let relu = ReluLayer::default();

            relu.propagate(&input, &mut output);

            output
        });
        bencher.iter(|| {
            let input: [i32; 32] = from_fn(|index| (index as i32).wrapping_mul(1000));
            let mut output = [0i8; 32];

            let relu = ReluLayer::default();

            relu.propagate(&input, &mut output);

            output
        })
    }
}
