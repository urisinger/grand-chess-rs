#![allow(dead_code)]
use std::{
    env,
    io::{self, BufReader},
};

use engine::GrandChessEngine;
use uci::UciConnection;

pub fn main() {
    let mut args = env::args();

    args.next();
    if let Some("bench") = args.next().as_deref() {
        println!("benching!");
    } else {
        let connection = UciConnection::new(
            BufReader::new(io::stdin()),
            io::stdout(),
            GrandChessEngine::new(100000000),
        );

        connection.run();
    }
}
