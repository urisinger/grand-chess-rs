use std::env;

use board::{perft, Board};

use crate::board::movegen::generate_moves;

#[allow(long_running_const_eval)]
mod board;
mod util;

fn main() {
    let args: Vec<String> = env::args().collect();
    let depth: u32 = args[1].parse().unwrap();
    let mut board = Board::from_fen(&args[2]).unwrap();
    args.get(3).map(|moves| {
        for m in moves.split_whitespace(){
            board.make_move(board.parse_move(m).unwrap());
        }
    });

    perft(board, depth);
}
