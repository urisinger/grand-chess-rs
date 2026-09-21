use std::io::Read;

use crate::nnue::layer::LinearLayer;
macro_rules! net {
    ($in:expr, $out:expr) => {
        (
            $crate::nnue::LinearLayer<$in, $out>,
            $crate::nnue::LinearLayer<$out, 1>,
        )
    };

    ($in:expr, $out:expr, $($rest:expr),+) => {
        (
            $crate::nnue::LinearLayer<$in, $out>,
            $crate::nnue::net!($out, $($rest),+),
        )
    };
}

pub(crate) use net;

fn relu_hash(prev_hash: u32) -> u32 {
    0x538D24C7u32.overflowing_add(prev_hash).0
}

pub fn linear_hash<const O: usize>(prev_hash: u32) -> u32 {
    let mut hash_value = 0xCC03DAE4u32;
    hash_value = hash_value.overflowing_add(O as u32).0;
    hash_value ^= prev_hash >> 1;
    hash_value ^= prev_hash << 31;
    hash_value
}

pub trait Network {
    type Buffer;

    fn load(&mut self, r: &mut impl Read);

    fn hash(hash: u32) -> u32;

    fn eval(&self, input: &[i8], buffer: &mut Self::Buffer) -> i32;
}

impl<const I: usize> Network for LinearLayer<I, 1> {
    type Buffer = ();

    fn load(&mut self, r: &mut impl Read) {
        LinearLayer::load(self, r);
    }

    fn hash(hash: u32) -> u32 {
        Self::get_hash(hash)
    }

    #[inline(always)]
    fn eval(&self, input: &[i8], _: &mut Self::Buffer) -> i32 {
        let input: &[i8; I] = input.try_into().unwrap();

        self.propagate_final(input) / 16
    }
}

impl<const IN: usize, const OUT: usize, REST> Network for (LinearLayer<IN, OUT>, REST)
where
    REST: Network,
{
    type Buffer = ([i8; OUT], REST::Buffer);

    fn load(&mut self, r: &mut impl Read) {
        self.0.load(r);

        self.1.load(r);
    }

    fn hash(mut hash: u32) -> u32 {
        hash = linear_hash::<OUT>(hash);

        hash = relu_hash(hash);

        REST::hash(hash)
    }

    fn eval(&self, input: &[i8], buffer: &mut Self::Buffer) -> i32 {
        let input: &[i8; IN] = input.try_into().unwrap();

        self.0.propagate_relu(input, &mut buffer.0);
        self.1.eval(&buffer.0, &mut buffer.1)
    }
}
