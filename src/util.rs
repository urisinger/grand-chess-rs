pub fn print_bitboard(bitboard: u64) {
    for i in 0..64 {
        if bitboard & 1 << i != 0 {
            print!("1|");
        } else {
            print!("0|");
        }
        if i % 8 == 7 {
            println!("");
        }
    }
}
