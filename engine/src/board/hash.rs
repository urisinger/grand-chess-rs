use std::array::from_fn;

use rand::random;
use static_init::dynamic;

#[dynamic]
pub static PIECE_KEYS: [[u64; 64]; 12] = from_fn(|_| from_fn(|_| random()));

#[dynamic]
pub static DOUBLE_PUSH_KEYS: [u64; 64] = from_fn(|_| random());

#[dynamic]
pub static CASTLE_KEYS: [u64; 16] = from_fn(|_| random());

#[dynamic]
pub static SIDE_KEY: u64 = random();
