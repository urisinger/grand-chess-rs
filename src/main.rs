#![allow(dead_code)]

use std::{
    fs::File,
    io::{self, BufReader},
};

use board::Board;
use engine::{
    nnue::{feature_transformer::Accumulator, half_kp::HalfKP},
    GrandChessEngine,
};
use uci::UciConnection;

pub fn main() {
    let mut net: Box<HalfKP<512, 32, 32>> = HalfKP::load_boxed(&mut BufReader::new(
        File::open("/home/uri_singer/Downloads/nn-97f742aaefcd.nnue").unwrap(),
    ));

    let mut acc = Accumulator::new_boxed();

    net.refresh_acc(&mut acc, &Board::default());

    dbg!(net.eval(&acc));

    let connection = UciConnection::new(
        BufReader::new(io::stdin()),
        io::stdout(),
        GrandChessEngine::new(10000000),
    );

    connection.run();
}
