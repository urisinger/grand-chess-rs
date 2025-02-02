use std::{
    io::Read,
    ops::{Deref, DerefMut},
};

use crate::board::{piece::PieceColor, r#move::Move, Board, PiecesDelta};
use byteorder::{LittleEndian, ReadBytesExt};

use self::feature_transformer::{Accumulator, FeatureTransformer};

mod feature_transformer;
pub mod half_kp;

mod layers;
pub mod network;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefreshFlags {
    pub white: bool,
    pub black: bool,
}

impl RefreshFlags {
    pub fn from_color(color: PieceColor) -> Self {
        match color {
            PieceColor::White => RefreshFlags { white: true, black: false },
            PieceColor::Black => RefreshFlags { black: true, white: false },
        }
    }
}

pub trait FeatureSet {
    const HALF_SIZE: usize;

    fn needs_refresh(r#move: Move) -> RefreshFlags;
    fn active_features(features: &mut FeatureList<32>, board: &Board, prespective: PieceColor);
    fn features_diff<const N: usize>(
        delta: &PiecesDelta,
        added_features: &mut FeatureList<N>,
        removed_features: &mut FeatureList<N>,
        board: &Board,
        prespective: PieceColor,
    );

    fn hash() -> u32;
}

pub trait Network<const IN: usize> {
    type Buffer;

    fn load(&mut self, r: &mut impl Read);
    fn hash() -> u32;

    fn eval(&self, input: &[i8; IN], buffer: &mut Self::Buffer) -> i32;
}

#[derive(Debug)]
pub struct FeatureList<const N: usize> {
    features: [usize; N],
    len: usize,
}

impl<const N: usize> Default for FeatureList<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> FeatureList<N> {
    pub fn new() -> Self {
        Self { features: [0; N], len: 0 }
    }
    pub fn push(&mut self, feature: usize) {
        self.features[self.len] = feature;
        self.len += 1;
    }
}

impl<const N: usize> Deref for FeatureList<N> {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        &self.features[0..self.len]
    }
}

impl<const N: usize> DerefMut for FeatureList<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.features[0..self.len]
    }
}

pub struct Nnue<NET: Network<NET_IN>, SET: FeatureSet, const STACK_SIZE: usize, const NET_IN: usize>
where
    [(); SET::HALF_SIZE * 2]:,
    [(); NET_IN / 2]:,
{
    net: NET,
    net_buffer: NET::Buffer,
    transformer: FeatureTransformer<i16, i16, { SET::HALF_SIZE }, NET_IN>,

    acc_stack: [Accumulator<i16, NET_IN>; STACK_SIZE],
}

impl<NET: Network<NET_IN>, SET: FeatureSet, const STACK_SIZE: usize, const NET_IN: usize>
    Nnue<NET, SET, STACK_SIZE, NET_IN>
where
    [(); SET::HALF_SIZE * 2]:,
    [(); NET_IN / 2]:,
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
        let kp_hash: u32 = SET::hash() ^ NET_IN as u32;
        let correct_hash = kp_hash ^ NET::hash();

        let hash = r.read_u32::<LittleEndian>().unwrap();

        if hash != correct_hash {
            eprintln!("Incorrect hash!: expected {}, found {}", correct_hash, hash);
        }

        let size = r.read_i32::<LittleEndian>().unwrap() as usize;

        let mut buf = vec![0u8; size];
        r.read_exact(&mut buf).unwrap();
        let str = String::from_utf8(buf).unwrap();

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
    }

    pub fn refresh_board(&mut self, board: &Board, ply: usize) {
        let mut features = FeatureList::new();
        SET::active_features(&mut features, board, PieceColor::White);
        self.transformer.refresh(&mut self.acc_stack[ply], &features, PieceColor::White);

        let mut features = FeatureList::new();
        SET::active_features(&mut features, board, PieceColor::Black);
        self.transformer.refresh(&mut self.acc_stack[ply], &features, PieceColor::Black);
    }

    pub fn make_null_move(&mut self, board: &mut Board, ply: usize) {
        board.make_null_move();

        let split = self.acc_stack.split_at_mut(ply + 1);
        split.1[0].accumulators.copy_from_slice(&split.0[ply].accumulators);
    }

    pub fn make_move(&mut self, r#move: Move, board: &mut Board, ply: usize) {
        let mut delta = PiecesDelta::new();
        board.make_move(r#move, &mut delta);

        let needs_refresh = SET::needs_refresh(r#move);

        if needs_refresh.white {
            let mut features = FeatureList::new();
            SET::active_features(&mut features, board, PieceColor::White);
            self.transformer.refresh(&mut self.acc_stack[ply + 1], &features, PieceColor::White);
        } else {
            let mut removed_features = FeatureList::<4>::new();
            let mut added_features = FeatureList::<4>::new();
            SET::features_diff(
                &delta,
                &mut added_features,
                &mut removed_features,
                board,
                PieceColor::White,
            );

            let split = self.acc_stack.split_at_mut(ply + 1);
            self.transformer.update_incremental(
                &mut split.1[0],
                &split.0[ply],
                &added_features,
                &removed_features,
                PieceColor::White,
            );
        }

        if needs_refresh.black {
            let mut features = FeatureList::new();
            SET::active_features(&mut features, board, PieceColor::Black);

            self.transformer.refresh(&mut self.acc_stack[ply + 1], &features, PieceColor::Black);
        } else {
            let mut removed_features = FeatureList::<4>::new();
            let mut added_features = FeatureList::<4>::new();

            SET::features_diff(
                &delta,
                &mut added_features,
                &mut removed_features,
                board,
                PieceColor::Black,
            );

            let split = self.acc_stack.split_at_mut(ply + 1);
            self.transformer.update_incremental(
                &mut split.1[0],
                &split.0[ply],
                &added_features,
                &removed_features,
                PieceColor::Black,
            );
        }
    }

    pub fn eval(&mut self, ply: usize, side: PieceColor) -> i32 {
        let mut input = [0; NET_IN];
        self.transformer.transform(&self.acc_stack[ply], &mut input, side);

        self.net.eval(&input, &mut self.net_buffer)
    }
}
