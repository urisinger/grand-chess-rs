#![allow(dead_code)]
use std::{
    fs::File,
    io::{self, BufReader},
};

use engine::{
    board::{Board},
    nnue::{half_kp::HalfKP, network::TripleLayerNetwork, Nnue},
    GrandChessEngine,
};
use uci::UciConnection;

pub fn main() {
    let mut net = Nnue::<TripleLayerNetwork<512, 32, 32>, HalfKP, 128, 512>::new_boxed(
        &mut BufReader::new(File::open("/home/uri_singer/Downloads/nn-62ef826d1a6d.nnue").unwrap()),
    );

    let board =
        Board::from_fen("r1bqk2r/pppnbpp1/4pn1p/3p4/Q1P5/3P2PP/PP2PPB1/RNB1K1NR w KQkq - 2 7")
            .unwrap();

    net.refresh_board(&board);

    println!("{}", board);

    dbg!(net.eval(0, board.current_color));

    let connection = UciConnection::new(
        BufReader::new(io::stdin()),
        io::stdout(),
        GrandChessEngine::new(10000000),
    );

    connection.run();
}
