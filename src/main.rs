#![allow(dead_code)]

use std::{
    fs::File,
    io::{self, BufReader},
};

use engine::{nnue::half_kp::HalfKP, GrandChessEngine};
use uci::UciConnection;

pub fn main() {
    let net: Box<HalfKP<512, 32, 32>> = HalfKP::load_boxed(
        &mut File::open("/home/uri_singer/Downloads/nn-97f742aaefcd.nnue").unwrap(),
    );

    let connection = UciConnection::new(
        BufReader::new(io::stdin()),
        io::stdout(),
        GrandChessEngine::new(10000000),
    );

    connection.run();
}
