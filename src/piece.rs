#[derive(Debug, Clone, Copy)]
pub struct PieceColor {
    val: i8,
}

impl PieceColor {
    pub const WHITE: PieceColor = PieceColor { val: 0 };
    pub const BLACK: PieceColor = PieceColor { val: 1 };
}

impl Default for PieceColor {
    fn default() -> Self {
        PieceColor::WHITE
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PieceType {
    val: i8,
}

impl Default for PieceType {
    fn default() -> Self {
        PieceType {
            val: PieceType::EMPTY.val,
        }
    }
}

impl PieceType {
    pub const PAWN: PieceType = PieceType { val: 0 };
    pub const KNIGHT: PieceType = PieceType { val: 1 };
    pub const BISHOP: PieceType = PieceType { val: 2 };
    pub const ROOK: PieceType = PieceType { val: 3 };
    pub const QUEEN: PieceType = PieceType { val: 4 };
    pub const KING: PieceType = PieceType { val: 5 };
    pub const EMPTY: PieceType = PieceType { val: 6 };
}

#[derive(Default, Clone, Copy)]
pub struct Piece {
    pub piece_color: PieceColor,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn new(piece_type: PieceType, piece_color: PieceColor) -> Piece {
        Piece {
            piece_color,
            piece_type,
        }
    }
}

#[derive(Debug)]
pub struct NoSuchPieceError(char);

impl TryFrom<char> for Piece {
    type Error = NoSuchPieceError;

    fn try_from(c: char) -> Result<Self, NoSuchPieceError> {
        match c {
            'P' => Ok(Piece::new(PieceType::PAWN, PieceColor::WHITE)),
            'p' => Ok(Piece::new(PieceType::PAWN, PieceColor::BLACK)),
            'N' => Ok(Piece::new(PieceType::KNIGHT, PieceColor::WHITE)),
            'n' => Ok(Piece::new(PieceType::KNIGHT, PieceColor::BLACK)),
            'B' => Ok(Piece::new(PieceType::BISHOP, PieceColor::WHITE)),
            'b' => Ok(Piece::new(PieceType::BISHOP, PieceColor::BLACK)),
            'R' => Ok(Piece::new(PieceType::ROOK, PieceColor::WHITE)),
            'r' => Ok(Piece::new(PieceType::ROOK, PieceColor::BLACK)),
            'Q' => Ok(Piece::new(PieceType::QUEEN, PieceColor::WHITE)),
            'q' => Ok(Piece::new(PieceType::QUEEN, PieceColor::BLACK)),
            'K' => Ok(Piece::new(PieceType::KING, PieceColor::WHITE)),
            'k' => Ok(Piece::new(PieceType::KING, PieceColor::BLACK)),
            _ => Err(NoSuchPieceError(c)),
        }
    }
}
