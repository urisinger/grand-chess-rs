use std::array::from_fn;

pub const KNIGHT_ATTACKS: [u64; 64] = generate_knight_attacks();

pub const KING_ATTACKS: [u64; 64] = generate_king_attacks();

#[derive(Clone, Copy)]
pub struct AttackLookup {
    mask: u64,
    magic: u64,

    table: [u64; 4096],
}

#[allow(long_running_const_eval)]
pub const BISHOP_ATTACKS: [AttackLookup; 64] = generate_bishop_attacks();

#[allow(long_running_const_eval)]
pub const ROOK_ATTACKS: [AttackLookup; 64] = generate_rook_attacks();

const fn generate_knight_attacks() -> [u64; 64] {
    #[rustfmt::skip]
    const KNIGHT_OFFSETS: [(i32, i32); 8] = [
        (-1, -2), (1, -2),
        (-2, -1), (2, -1),
        (-2, 1), (2, 1),
        (-1, 2), (1, 2),
    ];

    let mut attacks = [0u64; 64];

    let mut square = 0;
    while square < 64 {
        let mut bitmask = 0;
        let rank = square / 8;
        let file = square % 8;
        let mut i = 0;
        while i < KNIGHT_OFFSETS.len() {
            let offset = KNIGHT_OFFSETS[i];
            let target_rank = rank + offset.0;
            let target_file = file + offset.1;

            if target_rank >= 0 && target_rank < 8 && target_file >= 0 && target_file < 8 {
                let target_square = target_rank * 8 + target_file;
                bitmask |= 1u64 << target_square;
            }
            i += 1;
        }

        attacks[square as usize] = bitmask;

        square += 1;
    }

    attacks
}

const fn generate_king_attacks() -> [u64; 64] {
    #[rustfmt::skip]
    const KING_OFFSETS: [(i32, i32); 8] = [
        (-1,-1), (-1, 0), (-1, 1),
        ( 0,-1),          ( 0, 1),
        (-1, 1), ( 1, 0), ( 1, 1),
    ];

    let mut attacks = [0u64; 64];

    let mut square = 0;
    while square < 64 {
        let mut bitmask = 0;
        let rank = square / 8;
        let file = square % 8;
        let mut i = 0;
        while i < KING_OFFSETS.len() {
            let offset = KING_OFFSETS[i];
            let target_rank = rank + offset.0;
            let target_file = file + offset.1;

            if target_rank >= 0 && target_rank < 8 && target_file >= 0 && target_file < 8 {
                let target_square = target_rank * 8 + target_file;
                bitmask |= 1u64 << target_square;
            }

            i += 1;
        }

        attacks[square as usize] = bitmask;

        square += 1;
    }

    attacks
}

const fn directional_attacks(square: usize, xdir: i32, ydir: i32, occupancy: u64) -> u64 {
    let mut attack_mask = 0u64;

    let mut target_square_x = (square as i32 % 8) + xdir;
    let mut target_square_y = (square as i32 / 8) + ydir;

    while target_square_x >= 0 && target_square_x < 8 && target_square_y >= 0 && target_square_y < 8
    {
        let target_square = (target_square_y * 8 + target_square_x) as u64;
        attack_mask |= 1u64 << target_square;

        if (1u64 << target_square) & occupancy != 0 {
            break;
        }

        target_square_x += xdir;
        target_square_y += ydir;
    }

    attack_mask
}

const fn generate_bishop_attacks() -> [AttackLookup; 64] {
    let mut attacks = [AttackLookup {
        mask: 0,
        magic: 0,
        table: [0; 4096],
    }; 64];

    let mut square = 0;
    while square < 64 {
        attacks[square].mask = generate_bishop_attack(square, 0);

        let mut table = attacks[square].table;
        let mut occupancy_index = 0;
        while occupancy_index < 4096 {
            let occupancy = get_occupancy(occupancy_index, attacks[square].mask);
            table[occupancy_index as usize] = generate_rook_attack(square, occupancy);

            occupancy_index += 1;
        }
        square += 1;
    }

    attacks
}

const fn generate_bishop_attack(square: usize, occupancy: u64) -> u64 {
    directional_attacks(square, -1, -1, occupancy)
        | directional_attacks(square, 1, 1, occupancy)
        | directional_attacks(square, -1, 1, occupancy)
        | directional_attacks(square, 1, -1, occupancy)
}

const fn generate_rook_attacks() -> [AttackLookup; 64] {
    let mut attacks = [AttackLookup {
        mask: 0,
        magic: 0,
        table: [0; 4096],
    }; 64];

    let mut square = 0;
    while square < 64 {
        attacks[square].mask = generate_rook_attack(square, 0);

        let mut table = attacks[square].table;
        let mut occupancy_index = 0;
        while occupancy_index < 4096 {
            let occupancy = get_occupancy(occupancy_index, attacks[square].mask);
            table[occupancy_index as usize] = generate_rook_attack(square, occupancy);

            occupancy_index += 1;
        }
        square += 1;
    }

    attacks
}

const fn generate_rook_attack(square: usize, occupancy: u64) -> u64 {
    directional_attacks(square, 0, -1, occupancy)
        | directional_attacks(square, 0, 1, occupancy)
        | directional_attacks(square, -1, 0, occupancy)
        | directional_attacks(square, 1, 0, occupancy)
}

#[inline]
const fn get_occupancy(index: u32, mut mask: u64) -> u64 {
    let mut occupancy = 0;

    let bits = mask.count_ones();

    let mut i = 0;

    while i < bits {
        let cur = 1u64 << mask.trailing_zeros();
        mask |= cur;

        if index & 1 << i != 0 {
            occupancy |= cur;
        }
        i += 1;
    }

    occupancy
}

const fn generate_magic_number() {
    let something: [u32; 4] = from_fn(|index| 3);
}
