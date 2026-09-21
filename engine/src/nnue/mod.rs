use std::ops::{Deref, DerefMut};

use crate::board::piece::PieceColor;

mod feature_transformer;
pub mod half_kp;

mod layer;
mod network;

pub(crate) use feature_transformer::{Accumulator, FeatureTransformer};
pub(crate) use layer::LinearLayer;
pub(crate) use network::{net, Network};

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

macro_rules! Nnue {
    (
        $name:ident,
        $set:ty,
        ($input:expr, $($size:expr),+ $(,)?),
        $stack_size:expr $(,)?
    ) => {
        pub struct $name {
            net: $crate::nnue::net!(
                $input,
                $($size),+
            ),

            net_buffer: <
                $crate::nnue::net!(
                    $input,
                    $($size),+
                ) as $crate::nnue::Network
            >::Buffer,

            transformer: $crate::nnue::FeatureTransformer<
                i16,
                i16,
                { <$set>::HALF_SIZE },
                { $input / 2 },
            >,

            acc_stack: [
                $crate::nnue::Accumulator<
                    i16,
                    { $input / 2 }
                >;
                $stack_size
            ],
        }

        impl $name {
            pub fn new_boxed(
                r: &mut impl std::io::Read,
            ) -> Box<Self> {
                let mut boxed = unsafe {
                    Box::from_raw(
                        std::alloc::alloc(
                            std::alloc::Layout::new::<Self>()
                        ) as *mut Self
                    )
                };

                boxed.load(r);
                boxed
            }

            pub fn load(
                &mut self,
                r: &mut impl std::io::Read,
            ) {
                use byteorder::ReadBytesExt;

                _ = r
                    .read_i32::<byteorder::LittleEndian>()
                    .unwrap();

                let kp_hash =
                    <$set>::hash()
                        ^ ($input as u32);

                let net_hash = <
                    $crate::nnue::net!(
                        $input,
                        $($size),+
                    ) as $crate::nnue::Network
                >::hash(0xEC42E90Du32 ^ ($input as u32));

                let correct_hash =
                    kp_hash ^ net_hash;

                let hash = r
                    .read_u32::<byteorder::LittleEndian>()
                    .unwrap();

                if hash != correct_hash {
                    eprintln!(
                        "Incorrect hash!: expected {}, found {}",
                        correct_hash,
                        hash
                    );
                }

                let size = r
                    .read_i32::<byteorder::LittleEndian>()
                    .unwrap() as usize;

                let mut buf = vec![0u8; size];

                std::io::Read::read_exact(
                    r,
                    &mut buf,
                )
                .unwrap();

                let hash = r
                    .read_u32::<byteorder::LittleEndian>()
                    .unwrap();

                assert_eq!(
                    hash,
                    kp_hash,
                    "Incorrect feature hash! expected {}, found {}",
                    kp_hash,
                    hash
                );

                self.transformer.load(r);

                let hash = r
                    .read_u32::<byteorder::LittleEndian>()
                    .unwrap();

                assert_eq!(
                    hash,
                    net_hash,
                    "Incorrect network hash! expected {}, found {}",
                    net_hash,
                    hash
                );
                <$crate::nnue::net!($input, $($size),+)
                    as $crate::nnue::Network>::load(
                        &mut self.net,
                        r,
                    );
            }

            pub fn refresh_board(
                &mut self,
                board: &$crate::board::Board,
                ply: usize,
            ) {
                let mut features =
                    $crate::nnue::FeatureList::new();

                <$set>::active_features(
                    &mut features,
                    board,
                    $crate::board::piece::PieceColor::White,
                );

                self.transformer.refresh(
                    &mut self.acc_stack[ply],
                    &features,
                    $crate::board::piece::PieceColor::White,
                );

                let mut features =
                    $crate::nnue::FeatureList::new();

                <$set>::active_features(
                    &mut features,
                    board,
                    $crate::board::piece::PieceColor::Black,
                );

                self.transformer.refresh(
                    &mut self.acc_stack[ply],
                    &features,
                    $crate::board::piece::PieceColor::Black,
                );
            }

            pub fn make_null_move(
                &mut self,
                board: &mut $crate::board::Board,
                ply: usize,
            ) {
                board.make_null_move();

                let split =
                    self.acc_stack.split_at_mut(ply + 1);

                split.1[0]
                    .accumulators
                    .copy_from_slice(
                        &split.0[ply].accumulators
                    );
            }

            pub fn make_move(
                &mut self,
                r#move: $crate::board::r#move::Move,
                board: &mut $crate::board::Board,
                ply: usize,
            ) {
                let mut delta =
                    $crate::board::PiecesDelta::new();

                board.make_move(
                    r#move,
                    &mut delta,
                );

                let needs_refresh =
                    <$set>::needs_refresh(
                        r#move
                    );

                if needs_refresh.white {
                    let mut features =
                        $crate::nnue::FeatureList::new();

                    <$set>::active_features(
                        &mut features,
                        board,
                        $crate::board::piece::PieceColor::White,
                    );

                    self.transformer.refresh(
                        &mut self.acc_stack[ply + 1],
                        &features,
                        $crate::board::piece::PieceColor::White,
                    );
                } else {
                    let mut removed_features =
                        $crate::nnue::FeatureList::<4>::new();

                    let mut added_features =
                        $crate::nnue::FeatureList::<4>::new();

                    <$set>::features_diff(
                        &delta,
                        &mut added_features,
                        &mut removed_features,
                        board,
                        $crate::board::piece::PieceColor::White,
                    );

                    let split =
                        self.acc_stack.split_at_mut(ply + 1);

                    self.transformer.update_incremental(
                        &mut split.1[0],
                        &split.0[ply],
                        &added_features,
                        &removed_features,
                        $crate::board::piece::PieceColor::White,
                    );
                }

                if needs_refresh.black {
                    let mut features =
                        $crate::nnue::FeatureList::new();

                    <$set>::active_features(
                        &mut features,
                        board,
                        $crate::board::piece::PieceColor::Black,
                    );

                    self.transformer.refresh(
                        &mut self.acc_stack[ply + 1],
                        &features,
                        $crate::board::piece::PieceColor::Black,
                    );
                } else {
                    let mut removed_features =
                        $crate::nnue::FeatureList::<4>::new();

                    let mut added_features =
                        $crate::nnue::FeatureList::<4>::new();

                    <$set>::features_diff(
                        &delta,
                        &mut added_features,
                        &mut removed_features,
                        board,
                        $crate::board::piece::PieceColor::Black,
                    );

                    let split =
                        self.acc_stack.split_at_mut(ply + 1);

                    self.transformer.update_incremental(
                        &mut split.1[0],
                        &split.0[ply],
                        &added_features,
                        &removed_features,
                        $crate::board::piece::PieceColor::Black,
                    );
                }
            }

            pub fn eval(
                &mut self,
                ply: usize,
                side: $crate::board::piece::PieceColor,
            ) -> i32 {
                let mut input =
                    [0i8; $input];

                self.transformer.transform(
                    &self.acc_stack[ply],
                    &mut input,
                    side,
                );
                <$crate::nnue::net!($input, $($size),+)
                    as $crate::nnue::Network>::eval(
                        &self.net,
                        &input,
                        &mut self.net_buffer,
                    )
            }
        }
    };
}

pub(crate) use Nnue;
