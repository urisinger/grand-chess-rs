#![allow(dead_code)]
use std::io::{self, BufReader};

use engine::GrandChessEngine;
use uci::UciConnection;

pub fn main() {
    let connection = UciConnection::new(
        BufReader::new(io::stdin()),
        io::stdout(),
        GrandChessEngine::new(10000000),
    );

    connection.run();
}
