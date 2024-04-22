use core::slice;
use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use super::Layer;

pub struct LinearLayer<BT, const I: usize, const O: usize>
where
    [(); I * O]:,
{
    pub bias: [BT; O],
    pub weights: [i8; I * O],
}

impl<const I: usize, const O: usize> Layer<i8, i32, I, O> for LinearLayer<i32, I, O>
where
    [(); I * O]:,
{
    fn load(&mut self, r: &mut impl Read) {
        for i in 0..O {
            self.bias[i] = r.read_i32::<LittleEndian>().unwrap();
        }

        r.read_exact(unsafe {
            slice::from_raw_parts_mut(self.weights.as_mut_ptr() as *mut i8 as *mut u8, O * I)
        })
        .unwrap();
    }

    fn get_hash(prev_hash: u32) -> u32 {
        let mut hash_value = 0xCC03DAE4u32;
        hash_value = hash_value.overflowing_add(O as u32).0;
        hash_value ^= prev_hash >> 1;
        hash_value ^= prev_hash << 31;
        hash_value
    }

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
