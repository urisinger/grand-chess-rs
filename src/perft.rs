#![allow(dead_code)]
use std::env;

use board::{perft, Board};

mod board;

fn main() {
    let args: Vec<String> = env::args().collect();
    let depth: u32 = args[1].parse().unwrap();
    let mut board =
        if let Some(fen) = args.get(2) { Board::from_fen(fen).unwrap() } else { Board::default() };
    args.get(3).map(|moves| {
        for m in moves.split_whitespace() {
            board.make_move(board.parse_move(m).unwrap());
        }
    });

    perft(&board, depth);
}
