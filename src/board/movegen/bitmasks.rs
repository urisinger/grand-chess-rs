use std::array::from_fn;

use rand::random;
use static_init::dynamic;
#[dynamic]
pub static KNIGHT_ATTACKS: [u64; 64] = generate_knight_attacks();

#[dynamic]
pub static KING_ATTACKS: [u64; 64] = generate_king_attacks();

#[dynamic]
pub static BISHOP_MASKS: [u64; 64] = generate_bishop_masks();
#[dynamic]
pub static BISHOP_ATTACKS: [[u64; 4096]; 64] =
    generate_bishop_attacks(&BISHOP_MASKS, &BISHOP_MAGICS);

#[dynamic]
pub static ROOK_MASKS: [u64; 64] = generate_rook_masks();
#[dynamic]
pub static ROOK_ATTACKS: [[u64; 4096]; 64] = generate_rook_attacks(&ROOK_MASKS, &ROOK_MAGICS);

pub static BISHOP_MAGICS: [u64; 64] = [
    0x81900116082200a0,
    0x20040102002012,
    0x410040280200080,
    0x624404081020000,
    0x5244042001004080,
    0x80c41040000294,
    0x1082021282402210,
    0x4000834310012004,
    0xc00840104c0108c0,
    0x30100a004040,
    0x5000481489020939,
    0x840400852408,
    0xc10211040601889,
    0x81013008200600,
    0x4004a02504040,
    0x11a0042023004,
    0x42c005020084908,
    0x880200040c084208,
    0x1408a210a40100,
    0x488040882004081,
    0x180c401a00200,
    0x1042000108010404,
    0x20400030c0a3241,
    0x10000c4108400,
    0x8008200009021040,
    0x4100452028804,
    0xc04440008080090,
    0x42100401404011a0,
    0x9010840000802022,
    0xa004080801018200,
    0x202d1e0c05080b00,
    0x10a00030084c0,
    0xc21206010880800,
    0x802011000151001,
    0x100c0a0500220401,
    0x6069200800090504,
    0x4440010200010084,
    0x146080110020040,
    0xce009e00010800,
    0x8048020021044500,
    0x4100c10540483,
    0x1012030120a00800,
    0x14209c0048026402,
    0x4000406031080808,
    0x80904000040,
    0x3200210c040d200,
    0x20c080801004040,
    0x2020a00300600,
    0x4c2108081404401c,
    0x8824406a00012,
    0x1108842c4100004,
    0x22000000840c0080,
    0x80002002048000,
    0x1484040c08061001,
    0x8065822040020,
    0x4082108c00808000,
    0xa081840088440210,
    0x81001c208030880,
    0x4600038401a806,
    0x8040000080420200,
    0x82200600a0486,
    0x800002020820086,
    0x2e22230404248400,
    0x114900411002200,
];

pub static ROOK_MAGICS: [u64; 64] = [
    0x408000c004322080,
    0x2040002001485001,
    0x100090012402000,
    0x880245000080080,
    0x200041002008820,
    0x100061c00150048,
    0x80060009000080,
    0x6000200810ac024,
    0x221800020804000,
    0x410c04000201000,
    0x8882801000816008,
    0x821801000800800,
    0x2001001008030004,
    0x1080800a004c00,
    0x800e00800300,
    0x6449000220904d00,
    0x208009814002,
    0x4590a0020804208,
    0x8006820020304601,
    0x40808038003001,
    0x4200808004020800,
    0x204008004801600,
    0x4040a40088093006,
    0xa20020008419104,
    0x1041400080046080,
    0x200820004000d000,
    0x200211010040a001,
    0x420200100a20,
    0x1019040080380080,
    0x6802000200043048,
    0x520400218830,
    0x811008600004104,
    0xb080400022800280,
    0x20802000804009,
    0x600280801000,
    0x508002880801000,
    0x10080280800400,
    0x500cc20080800400,
    0x4012004526002408,
    0x11008080c2000421,
    0x14400080018020,
    0x2800200150004000,
    0x10200010008080,
    0x10010108110020,
    0x400a021220120008,
    0x40002a0004008080,
    0x490011806040010,
    0x4000840040820001,
    0x824121020200,
    0x8060004000847080,
    0x1008445900200100,
    0xe402100088048080,
    0x200440800110100,
    0xc002011430080600,
    0x30002004c8100,
    0xa041000229804100,
    0x200248000110141,
    0x888627100400081,
    0xa0005018422101,
    0x81000820141001,
    0xc02000920100482,
    0x2211002204000801,
    0x481000284020001,
    0x40224408102,
];

fn generate_knight_attacks() -> [u64; 64] {
    #[rustfmt::skip]
    const KNIGHT_OFFSETS: [(i32, i32); 8] = [
        (-1, -2), (1, -2),
        (-2, -1), (2, -1),
        (-2, 1), (2, 1),
        (-1, 2), (1, 2),
    ];

    from_fn(|square| {
        let mut bitmask = 0;
        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;

        for offset in KNIGHT_OFFSETS {
            let target_rank = rank + offset.0;
            let target_file = file + offset.1;

            if target_rank >= 0 && target_rank < 8 && target_file >= 0 && target_file < 8 {
                let target_square = target_rank * 8 + target_file;
                bitmask |= 1u64 << target_square;
            }
        }

        bitmask
    })
}

fn generate_king_attacks() -> [u64; 64] {
    #[rustfmt::skip]
    const KING_OFFSETS: [(i32, i32); 8] = [
        (-1,-1), (-1, 0), (-1, 1),
        ( 0,-1),          ( 0, 1),
        (1, -1), ( 1, 0), ( 1, 1),
    ];

    from_fn(|square| {
        let mut bitmask = 0;
        let rank = (square / 8) as i32;
        let file = (square % 8) as i32;
        for offset in KING_OFFSETS {
            let target_rank = rank + offset.0;
            let target_file = file + offset.1;

            if target_rank >= 0 && target_rank < 8 && target_file >= 0 && target_file < 8 {
                let target_square = target_rank * 8 + target_file;
                bitmask |= 1u64 << target_square;
            }
        }
        bitmask
    })
}

fn directional_attack(square: usize, xdir: i32, ydir: i32, occupancy: u64) -> u64 {
    let mut attack_mask = 0u64;

    let mut target_square_x = (square % 8) as i32 + xdir;
    let mut target_square_y = (square / 8) as i32 + ydir;

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

fn generate_bishop_masks() -> [u64; 64] {
    from_fn(|square| {
        let mut mask = 0u64;
        let target_file = square as i32 % 8;
        let target_rank = square as i32 / 8;

        let mut file = target_file + 1;
        let mut rank = target_rank + 1;

        while file < 7 && rank < 7 {
            let target_square = (rank * 8 + file) as u64;
            mask |= 1u64 << target_square;

            file += 1;
            rank += 1;
        }

        file = target_file - 1;
        rank = target_rank - 1;

        while file >= 1 && rank >= 1 {
            let target_square = (rank * 8 + file) as u64;
            mask |= 1u64 << target_square;

            file -= 1;
            rank -= 1;
        }

        file = target_file + 1;
        rank = target_rank - 1;

        while file < 7 && rank >= 1 {
            let target_square = (rank * 8 + file) as u64;
            mask |= 1u64 << target_square;

            file += 1;
            rank -= 1;
        }

        file = target_file - 1;
        rank = target_rank + 1;

        while file >= 1 && rank < 7 {
            let target_square = (rank * 8 + file) as u64;
            mask |= 1u64 << target_square;

            file -= 1;
            rank += 1;
        }
        mask
    })
}

fn generate_bishop_attacks(masks: &[u64; 64], magics: &[u64; 64]) -> [[u64; 4096]; 64] {
    let mut attacks = [[0; 4096]; 64];

    for square in 0..64 {
        let mask = masks[square];
        let magic = magics[square];
        let n = mask.count_ones();

        for occupancy_index in 0..4096 {
            let occupancy = get_occupancy(occupancy_index, mask);
            attacks[square][magic_key(magic, occupancy, n)] =
                generate_bishop_attack(square, occupancy);
        }
    }

    attacks
}

fn generate_bishop_attack(square: usize, occupancy: u64) -> u64 {
    directional_attack(square, -1, -1, occupancy)
        | directional_attack(square, 1, 1, occupancy)
        | directional_attack(square, 1, -1, occupancy)
        | directional_attack(square, -1, 1, occupancy)
}

fn generate_rook_masks() -> [u64; 64] {
    from_fn(|square| {
        let mut mask = 0u64;
        let target_file = square as i32 % 8;
        let target_rank = square as i32 / 8;

        let mut file = target_file + 1;
        let mut rank = target_rank;

        while file < 7 {
            let target_square = (rank * 8 + file) as u64;
            mask |= 1u64 << target_square;

            file += 1;
        }

        file = target_file - 1;
        rank = target_rank;

        while file >= 1 {
            let target_square = (rank * 8 + file) as u64;
            mask |= 1u64 << target_square;

            file -= 1;
        }

        file = target_file;
        rank = target_rank + 1;

        while rank < 7 {
            let target_square = (rank * 8 + file) as u64;
            mask |= 1u64 << target_square;

            rank += 1;
        }

        file = target_file;
        rank = target_rank - 1;

        while rank >= 1 {
            let target_square = (rank * 8 + file) as u64;
            mask |= 1u64 << target_square;

            rank -= 1;
        }
        mask
    })
}

fn generate_rook_attacks(masks: &[u64; 64], magics: &[u64; 64]) -> [[u64; 4096]; 64] {
    let mut attacks = [[0; 4096]; 64];

    for square in 0..64 {
        let mask = masks[square];
        let magic = magics[square];
        let n = mask.count_ones();

        for occupancy_index in 0..4096 {
            let occupancy = get_occupancy(occupancy_index, mask);
            attacks[square][magic_key(magic, occupancy, n)] =
                generate_rook_attack(square, occupancy);
        }
    }

    attacks
}

fn generate_rook_attack(square: usize, occupancy: u64) -> u64 {
    directional_attack(square, 0, -1, occupancy)
        | directional_attack(square, 0, 1, occupancy)
        | directional_attack(square, -1, 0, occupancy)
        | directional_attack(square, 1, 0, occupancy)
}

fn get_occupancy(index: u32, mut mask: u64) -> u64 {
    let mut occupancy = 0;

    let bits = mask.count_ones();

    let mut i = 0;

    while i < bits {
        let cur = 1u64 << mask.trailing_zeros();
        mask &= !cur;
        if index & 1 << i != 0 {
            occupancy |= cur;
        }
        i += 1;
    }

    occupancy
}

pub const fn magic_key(magic: u64, occupancy: u64, shift: u32) -> usize {
    ((occupancy.overflowing_mul(magic).0) >> (64 - shift)) as usize
}

pub fn bishop_attacks(square: usize, occupancy: u64) -> u64 {
    let mask = BISHOP_MASKS[square];
    BISHOP_ATTACKS[square][magic_key(BISHOP_MAGICS[square], occupancy & mask, mask.count_ones())]
}

pub fn rook_attacks(square: usize, occupancy: u64) -> u64 {
    let mask = ROOK_MASKS[square];
    ROOK_ATTACKS[square][magic_key(ROOK_MAGICS[square], occupancy & mask, mask.count_ones())]
}

#[cfg(test)]
pub mod tests {
    use super::{
        generate_bishop_attack, generate_rook_attack, get_occupancy, magic_key, BISHOP_ATTACKS,
        BISHOP_MAGICS, BISHOP_MASKS, ROOK_ATTACKS, ROOK_MAGICS, ROOK_MASKS,
    };

    #[test]
    pub fn bishop_attacks() {
        for square in 0..64 {
            let magic = BISHOP_MAGICS[square];
            let mask = BISHOP_MASKS[square];
            let n = mask.count_ones();
            for occupancy_index in 0..4096 {
                let occupancy = get_occupancy(occupancy_index, mask);
                let correct_attack = generate_bishop_attack(square, occupancy);
                let magic_attack = BISHOP_ATTACKS[square][magic_key(magic, occupancy, n)];

                assert_eq!(correct_attack, magic_attack);
            }
        }
    }

    #[test]
    pub fn rook_attacks() {
        for square in 0..64 {
            let magic = ROOK_MAGICS[square];
            let mask = ROOK_MASKS[square];
            let n = mask.count_ones();
            for occupancy_index in 0..4096 {
                let occupancy = get_occupancy(occupancy_index, mask);
                let correct_attack = generate_rook_attack(square, occupancy);
                let magic_attack = &ROOK_ATTACKS[square][magic_key(magic, occupancy, n)];

                assert_eq!(correct_attack, *magic_attack);
            }
        }
    }
}
