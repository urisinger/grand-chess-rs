use std::io::Read;

use board::piece::{Piece, PieceColor};
use byteorder::{LittleEndian, ReadBytesExt};

use self::feature_transformer::{Accumulator, FeatureTransformer};

pub mod feature_transformer;
pub mod half_kp;
mod layers;
mod network;

pub trait FeatureSet {
    const HALF_SIZE: usize;

    fn make_index(king_sq: u32, piece_sq: u32, piece: Piece, prespective: PieceColor) -> usize;
    fn hash() -> u32;
}

pub trait Network {
    const IN: usize;
    type Buffer;

    fn load(&mut self, r: impl Read);
    fn hash() -> u32;

    fn eval(&self, input: &[i8; Self::IN], buffer: &Self::Buffer);
}

pub struct Nnue<NET: Network, SET: FeatureSet, const STACK_SIZE: usize>
where
    [(); SET::HALF_SIZE * 2]:,
    [(); NET::IN]:,
    [(); NET::IN / 2]:,
{
    net: NET,
    transformer: FeatureTransformer<i16, i16, { SET::HALF_SIZE * 2 }, { NET::IN }>,

    acc_stack: [Accumulator<i16, { SET::HALF_SIZE * 2 }>; STACK_SIZE],
    stack_head: usize,
}

impl<NET: Network, SET: FeatureSet, const STACK_SIZE: usize> Nnue<NET, SET, STACK_SIZE>
where
    [(); SET::HALF_SIZE * 2]:,
    [(); NET::IN]:,
    [(); NET::IN / 2]:,
{
    pub fn new_boxed(r: &mut impl Read) -> Box<Self> {
        let mut boxed = unsafe {
            Box::from_raw(std::alloc::alloc(std::alloc::Layout::new::<Self>()) as *mut Self)
        };
        boxed.load(r);
        boxed
    }

    pub fn load(&mut self, r: &mut impl Read) {
        let version = r.read_i32::<LittleEndian>().unwrap();
        println!("version: 0x{:x}", version);
        let kp_hash: u32 = SET::hash() ^ NET::IN as u32;
        let correct_hash = kp_hash ^ NET::hash();

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

        self.transformer.load(r);
        let correct_hash = NET::hash();
        let hash = r.read_u32::<LittleEndian>().unwrap();

        assert_eq!(
            hash, correct_hash,
            "Incorrect network hash! expected {}, found {}",
            hash, correct_hash
        );

        self.net.load(r);

        println!("network loaded");
    }
}
