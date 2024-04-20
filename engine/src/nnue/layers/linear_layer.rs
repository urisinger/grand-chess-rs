use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use super::Layer;

pub struct LinearLayer<BT, const I: usize, const O: usize> {
    pub bias: [BT; O],
    pub weights: [[i8; O]; I],
}

impl<const I: usize, const O: usize> Layer<i8, i32, I, O> for LinearLayer<i32, I, O> {
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

    fn get_hash(prev_hash: u32) -> u32 {
        let mut hash_value = 0xCC03DAE4u32;
        hash_value = hash_value.overflowing_add(O as u32).0;
        hash_value ^= prev_hash >> 1;
        hash_value ^= prev_hash << 31;
        hash_value
    }

    fn propagate(&self, input: &[i8; I], output: &mut [i32; O]) {
        for i in 0..O {
            output[i] = self.bias[i];
        }

        for i in 0..O {
            let mut sum = 0;
            for j in 0..I {
                sum += (input[j] as i32 * self.weights[j][i] as i32) as i32;
            }

            output[i] += sum;
        }

        println!("");
    }
}
