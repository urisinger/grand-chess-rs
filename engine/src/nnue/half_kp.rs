use crate::board::{
    piece::{Piece, PieceColor, PieceType},
    r#move::Move,
    Board, PiecesDelta,
};

use super::{FeatureList, FeatureSet, RefreshFlags};

fn half_kp_index(king_sq: u32, piece_sq: u32, piece: Piece, prespective: PieceColor) -> usize {
    let flipped_sq = piece_sq as usize ^ (0x3F * prespective as usize);

    let flipped_king = king_sq as usize ^ (0x3F * prespective as usize);

    (((piece as usize >> 1) << 1) + (piece.get_color() != prespective) as usize) * 64
        + flipped_sq
        + 1
        + flipped_king * 641
}

pub struct HalfKP {}

impl FeatureSet for HalfKP {
    //Num squares * (num_square * (num_pieces without king) + 1)
    const HALF_SIZE: usize = 64 * (10 * 64 + 1);
    fn needs_refresh(r#move: Move) -> RefreshFlags {
        if r#move.piece().get_type() == PieceType::King {
            RefreshFlags::from_color(r#move.piece().get_color())
        } else {
            RefreshFlags { black: false, white: false }
        }
    }

    fn active_features(features: &mut FeatureList<32>, board: &Board, prespective: PieceColor) {
        let king_sq = board.bit_boards[Piece::new(PieceType::King, prespective)].trailing_zeros();
        for i in 0..Piece::WhiteKing as usize {
            let mut pieces = board.bit_boards.pieces[i];

            while pieces != 0 {
                let sq = pieces.trailing_zeros();

                features.push(half_kp_index(king_sq, sq, Piece::from(i as u8), prespective));

                pieces &= pieces - 1;
            }
        }
    }

    fn features_diff<const N: usize>(
        delta: &PiecesDelta,
        added_features: &mut FeatureList<N>,
        removed_features: &mut FeatureList<N>,
        board: &Board,
        prespective: PieceColor,
    ) {
        let king_sq = board.bit_boards[Piece::new(PieceType::King, prespective)].trailing_zeros();
        for d in delta.into_iter() {
            if d.to != 64 {
                added_features.push(half_kp_index(king_sq, d.to, d.piece, prespective));
            }
            if d.from != 64 {
                removed_features.push(half_kp_index(king_sq, d.from, d.piece, prespective));
            }
        }
    }

    fn hash() -> u32 {
        0x5D69D5B9 ^ 1
    }
}
