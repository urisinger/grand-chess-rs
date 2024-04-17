use core::slice;
use std::io::Read;

use board::piece::{Piece, PieceColor};
use byteorder::{LittleEndian, ReadBytesExt};

use super::{
    feature_transformer::{Accumulator, FeatureTransformer},
    network::{LayersBuffer, Network},
};

fn half_kp_index(king_sq: u32, piece_sq: u32, piece: Piece, prespective: PieceColor) -> usize {
    let piece_idx =
        if piece.get_color() == prespective { piece as usize | 1 } else { piece as usize & !1 };
    piece_sq as usize + (piece_idx + king_sq as usize * 10) * 64
}

//Num squares * (num_square * (num_pieces without king) + 1)
pub const HALFKP_FEATURES: usize = 64 * (10 * 64 + 1);

pub struct HalfKP<const OUT: usize, const L_1_IN: usize, const L_2_IN: usize> {
    pub network: Network<OUT, L_1_IN, L_2_IN>,
    pub network_buffer: LayersBuffer<OUT, L_1_IN, L_2_IN>,

    pub feature_transformer: FeatureTransformer<i16, i16, HALFKP_FEATURES, OUT>,
}

impl<const OUT: usize, const L_1_IN: usize, const L_2_IN: usize> HalfKP<OUT, L_1_IN, L_2_IN>
where
    [(); OUT / 2]:,
{
    pub fn new_boxed() -> Box<Self> {
        unsafe { Box::from_raw(std::alloc::alloc(std::alloc::Layout::new::<Self>()) as *mut Self) }
    }

    pub fn load_boxed<R: Read>(r: &mut R) -> Box<Self> {
        let mut boxed = unsafe {
            Box::from_raw(std::alloc::alloc(std::alloc::Layout::new::<Self>()) as *mut Self)
        };
        boxed.load(r);
        boxed
    }

    pub fn load<R: Read>(&mut self, r: &mut R) {
        let version = r.read_i32::<LittleEndian>().unwrap();
        println!("version: 0x{:x}", version);
        let kp_hash: u32 =
            0x5D69D5B9 ^ 1 ^ FeatureTransformer::<i16, i16, HALFKP_FEATURES, OUT>::get_hash();
        let correct_hash = kp_hash ^ Network::<OUT, L_1_IN, L_2_IN>::get_hash();

        let hash = r.read_u32::<LittleEndian>().unwrap();

        if hash != correct_hash {
            eprintln!("Incorrect hash!: expected {}, found {}", correct_hash, hash);
        }

        let size = r.read_i32::<LittleEndian>().unwrap() as usize;

        let mut buf = vec![0u8; size];
        r.read_exact(&mut buf).unwrap();
        let str = String::from_utf8(buf).unwrap();

        println!("Network description is: {}", str);

        let hash = r.read_u32::<LittleEndian>().unwrap();

        assert_eq!(hash, kp_hash, "Incorrect feature hash! expected {}, found {}", hash, kp_hash);

        self.feature_transformer.load(r);
        let correct_hash = Network::<OUT, L_1_IN, L_2_IN>::get_hash();
        let hash = r.read_u32::<LittleEndian>().unwrap();

        assert_eq!(
            hash, correct_hash,
            "Incorrect network hash! expected {}, found {}",
            hash, correct_hash
        );
        self.network.load(r);
    }

    pub fn eval(&mut self, accumulator: &Accumulator<i8, OUT>) -> i32 {
        self.network.propagate(
            unsafe { slice::from_raw_parts(accumulator.accumulators.as_ptr() as *const i8, OUT) }
                .try_into()
                .unwrap(),
            &mut self.network_buffer,
        )
    }
}
