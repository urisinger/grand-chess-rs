use std::io::Read;

use super::layers::{Layer, LinearLayer, ReluLayer};

#[repr(align(64))]
pub struct Network<const IN: usize, const L_1_IN: usize, const L_2_IN: usize> {
    r_0: ReluLayer<i16, i8, IN>,
    l_1: LinearLayer<i8, i32, IN, L_1_IN>,
    r_1: ReluLayer<i32, i8, L_1_IN>,
    l_2: LinearLayer<i8, i32, L_1_IN, L_2_IN>,
    r_2: ReluLayer<i32, i8, L_2_IN>,
    l_3: LinearLayer<i8, i32, L_2_IN, 1>,
}

impl<const IN: usize, const L_1_IN: usize, const L_2_IN: usize> Network<IN, L_1_IN, L_2_IN> {
    pub fn load<R: Read>(&mut self, r: &mut R) {
        self.r_0.load(r);
    }

    pub fn propagate(
        &self,
        input: &[i16; IN],
        buffer: &mut LayersBuffer<IN, L_1_IN, L_2_IN>,
    ) -> i32 {
        self.r_0.propagate_unaligned(input, &mut buffer.r_0);
        self.l_1.propagate_unaligned(&buffer.r_0, &mut buffer.l_1);
        self.r_1.propagate_unaligned(&buffer.l_1, &mut buffer.r_1);
        self.l_2.propagate_unaligned(&buffer.r_1, &mut buffer.l_2);
        self.r_2.propagate_unaligned(&buffer.l_2, &mut buffer.r_2);
        self.l_3.propagate_unaligned(&buffer.r_2, &mut buffer.out);

        buffer.out[0]
    }
}

#[repr(align(64))]
pub struct LayersBuffer<const IN: usize, const L_1_IN: usize, const L_2_IN: usize> {
    pub r_0: [i8; IN],
    pub l_1: [i32; L_1_IN],
    pub r_1: [i8; L_1_IN],
    pub l_2: [i32; L_2_IN],
    pub r_2: [i8; L_2_IN],
    pub out: [i32; 1],
}
