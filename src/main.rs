use board::Board;

use crate::board::movegen::generate_moves;

#[allow(long_running_const_eval)]
mod board;
mod util;

fn main() {
    let board = Board::default();

    println!("{}", board);

    println!("{:?}", generate_moves(board));
}
