#[allow(long_running_const_eval)]
mod board;
mod common;
mod movegen;

use board::Board;

fn main() {
    let board = Board::default();

    println!("{}", board);
}
