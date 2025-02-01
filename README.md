# Grand Chess - a chess engine
Grand Chess is an (unknown) elo chess engine written in rust.

## Running
to run, you simply clone the repo and run cargo run --release, to change the net the engine uses you can use the EVALFILE env variable, and to compile with native cpu features you can add RUSTFLAGS="-Ctarget-cpu=native" before cargo run
```bash
git clone https://github.com/urisinger/grand-chess-rs.git
cd grand-chess-rs
cargo run --release
```
