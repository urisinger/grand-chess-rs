use rand::{rngs::StdRng, RngCore, SeedableRng};
use static_init::dynamic;

#[dynamic]
pub static PIECE_KEYS: [[u64; 64]; 12] = (|| {
    let mut rng: StdRng = StdRng::seed_from_u64(123456);
    let mut keys = [[0; 64]; 12];
    for row in keys.iter_mut() {
        for val in row.iter_mut() {
            *val = rng.next_u64();
        }
    }
    keys
})();

#[dynamic]
pub static DOUBLE_PUSH_KEYS: [u64; 64] = (|| {
    let mut rng: StdRng = StdRng::seed_from_u64(789012);
    let mut keys = [0; 64];
    for val in keys.iter_mut() {
        *val = rng.next_u64();
    }
    keys
})();

#[dynamic]
pub static CASTLE_KEYS: [u64; 16] = (|| {
    let mut rng: StdRng = StdRng::seed_from_u64(345678);
    let mut keys = [0; 16];
    for val in keys.iter_mut() {
        *val = rng.next_u64();
    }
    keys
})();

#[dynamic]
pub static SIDE_KEY: u64 = StdRng::seed_from_u64(901234).next_u64();
