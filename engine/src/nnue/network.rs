use std::io::Read;

use super::{
    layers::{crelu::ReluLayer, linear_layer::LinearLayer, Layer},
    Network,
};

#[repr(align(64))]
pub struct TripleLayerNetwork<const L_1: usize, const L_2: usize, const L_3: usize> {
    pub l_1: LinearLayer<i32, L_1, L_2>,
    pub r_1: ReluLayer<i32, i8, L_2>,
    pub l_2: LinearLayer<i32, L_2, L_3>,
    pub r_2: ReluLayer<i32, i8, L_3>,
    pub l_3: LinearLayer<i32, L_3, 1>,
}

impl<const L_1: usize, const L_2: usize, const L_3: usize> Network
    for TripleLayerNetwork<L_1, L_2, L_3>
{
    const IN: usize = L_1;
    const HALF_IN: usize = L_1 / 2;
    type Buffer = LayersBuffer<L_2, L_3>;
    fn load(&mut self, r: &mut impl Read) {
        self.l_1.load(r);

        self.l_2.load(r);
        self.l_3.load(r);
    }

    fn hash() -> u32 {
        let mut hash = 0xEC42E90Du32 ^ (L_1 as u32);

        hash = LinearLayer::<i32, L_1, L_2>::get_hash(hash);

        hash = ReluLayer::<i32, i8, L_2>::get_hash(hash);

        hash = LinearLayer::<i32, L_2, L_3>::get_hash(hash);

        hash = ReluLayer::<i32, i8, L_3>::get_hash(hash);

        hash = LinearLayer::<i32, L_3, 1>::get_hash(hash);

        hash
    }

    fn eval(&self, input: &[i8], buffer: &mut LayersBuffer<L_2, L_3>) -> i32 {
        let input: &[i8; L_1] = input.try_into().unwrap();
        self.l_1.propagate(input, &mut buffer.l_1);

        self.r_1.propagate(&buffer.l_1, &mut buffer.r_1);
        self.l_2.propagate(&buffer.r_1, &mut buffer.l_2);
        self.r_2.propagate(&buffer.l_2, &mut buffer.r_2);
        self.l_3.propagate(&buffer.r_2, &mut buffer.out);

        buffer.out[0] / 16
    }
}

#[repr(align(64))]
pub struct LayersBuffer<const L_2: usize, const L_3: usize> {
    pub l_1: [i32; L_2],
    pub r_1: [i8; L_2],
    pub l_2: [i32; L_3],
    pub r_2: [i8; L_3],
    pub out: [i32; 1],
}
