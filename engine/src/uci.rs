use std::{
    ops::Div,
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

use uci::{
    UciInfoAttribute, UciMove, UciOptionConfig, UciPiece, UciSearchControl, UciSquare,
    UciTimeControl,
};

use crate::board::{
    piece::{Piece, PieceColor, PieceType},
    r#move::{Move, MoveType},
    Board, NoDelta,
};
use uci::{Engine, RecivedMessage};

use super::{GrandChessEngine, MATE_SCORE, MATE_VALUE, MAX_PLY, MAX_SCORE, MIN_SCORE};

const VAL_WINDOW: i32 = 50;

impl Engine for GrandChessEngine {
    fn go(
        &mut self,
        reciver: &Receiver<()>,
        sender: &mut Sender<RecivedMessage>,
        time_control: Option<UciTimeControl>,
        search_control: Option<UciSearchControl>,
    ) {
        self.nnue.refresh_board(&self.board, 0);

        let mut best_move = UciMove::from_to(UciSquare::from('a', 1), UciSquare::from('a', 1));

        let depth = search_control.and_then(|s| s.depth.map(|d| d as u32)).unwrap_or(1000);
        self.stop = false;

        if let Some(t) = time_control {
            match t {
                UciTimeControl::MoveTime(t) => {
                    self.max_time = Some(Instant::now() - Duration::new(0, 8_000_000) + t);
                }
                UciTimeControl::TimeLeft { white_time, black_time, moves_to_go, .. } => {
                    let moves_to_go = if moves_to_go.unwrap_or(50) == 0 {
                        1
                    } else {
                        moves_to_go.unwrap_or(50) as i32
                    };

                    self.max_time = match self.board.current_color {
                        PieceColor::White => {
                            white_time.map(|t| Instant::now() + t.div(moves_to_go as u32))
                        }
                        PieceColor::Black => {
                            black_time.map(|t| Instant::now() + t.div(moves_to_go as u32))
                        }
                    };
                }

                _ => {}
            }
        }

        let mut alpha = MIN_SCORE;
        let mut beta = MAX_SCORE;

        let mut d = 1;

        self.dont_stop = true;
        while d <= depth {
            self.node_count = 0;

            let start = Instant::now();
            let score = self.neg_max(d as i32, 0, &self.board.clone(), alpha, beta, Some(reciver));

            let time = start.elapsed();

            self.dont_stop = false;

            if self.stop {
                break;
            };

            if score <= alpha || score >= beta {
                alpha = MIN_SCORE;
                beta = MAX_SCORE;
                continue;
            }

            let pv: Vec<UciMove> = (0..self.pv_length[0])
                .map(|i| {
                    let r#move = self.pv_table[0][i];
                    let from = UciSquare::from(
                        (((r#move.from() as u8) % 8) + b'a') as char,
                        r#move.from() as u8 / 8 + 1,
                    );
                    let to = UciSquare::from(
                        (((r#move.to() as u8) % 8) + b'a') as char,
                        r#move.to() as u8 / 8 + 1,
                    );

                    UciMove {
                        from,
                        to,
                        promotion: if r#move.move_type() == MoveType::Promote {
                            Some(match r#move.piece().get_type() {
                                PieceType::Queen => UciPiece::Queen,
                                PieceType::Rook => UciPiece::Rook,
                                PieceType::Bishop => UciPiece::Bishop,
                                PieceType::Knight => UciPiece::Knight,
                                _ => UciPiece::Queen,
                            })
                        } else {
                            None
                        },
                    }
                })
                .collect();

            best_move = pv[0];

            if score > MATE_SCORE {
                _ = sender.send(RecivedMessage::Info(vec![
                    UciInfoAttribute::Depth(d as u8),
                    UciInfoAttribute::Score {
                        cp: None,
                        mate: Some(((score + MATE_VALUE) / 2 - 1) as i8),
                        lower_bound: None,
                        upper_bound: None,
                    },
                    UciInfoAttribute::Nodes(self.node_count),
                    UciInfoAttribute::Nps((self.node_count as f64 / time.as_secs_f64()) as u64),
                    UciInfoAttribute::Time(time),
                    UciInfoAttribute::Pv(pv),
                ]));
                break;
            } else if score < -MATE_SCORE {
                _ = sender.send(RecivedMessage::Info(vec![
                    UciInfoAttribute::Depth(d as u8),
                    UciInfoAttribute::Score {
                        cp: None,
                        mate: Some(((MATE_SCORE - score) / 2 + 1) as i8),
                        lower_bound: None,
                        upper_bound: None,
                    },
                    UciInfoAttribute::Nodes(self.node_count),
                    UciInfoAttribute::Nps((self.node_count as f64 / time.as_secs_f64()) as u64),
                    UciInfoAttribute::Time(time),
                    UciInfoAttribute::Pv(pv),
                ]));
                break;
            } else {
                _ = sender.send(RecivedMessage::Info(vec![
                    UciInfoAttribute::Depth(d as u8),
                    UciInfoAttribute::Score {
                        cp: Some(score),
                        mate: None,
                        lower_bound: None,
                        upper_bound: None,
                    },
                    UciInfoAttribute::Nodes(self.node_count),
                    UciInfoAttribute::Nps((self.node_count as f64 / time.as_secs_f64()) as u64),
                    UciInfoAttribute::Time(time),
                    UciInfoAttribute::Pv(pv),
                ]));
            }
            alpha = score - VAL_WINDOW;
            beta = score + VAL_WINDOW;
            d += 1;
        }

        let _ = sender.send(RecivedMessage::BestMove(best_move));

        self.max_time = None;

        self.pv_table.fill([Move::null(); 128]);
        self.pv_length.fill(0);
        self.history_moves.fill([0; 64]);
        self.killer_moves.fill([Move::null(); MAX_PLY]);
        self.repetition_table.fill(0);
    }

    fn options() -> Vec<UciOptionConfig> {
        vec![
            UciOptionConfig::Spin {
                name: "Hash".to_owned(),
                default: Some(0x1000000),
                min: None,
                max: None,
            },
            UciOptionConfig::Spin {
                name: "Threads".to_owned(),
                default: Some(1),
                min: Some(1),
                max: Some(1),
            },
        ]
    }

    fn set_option(&mut self, name: &str, value: Option<&str>) {
        match name {
            "Hash" => self.tt.resize(match value.unwrap_or("1").parse() {
                Ok(num) => num,
                Err(e) => {
                    eprintln!("could not parse option due to error: {}", e);
                    return;
                }
            }),
            "Threads" => (),
            _ => eprintln!("Invalid option {}", name),
        }
    }

    fn set_pos(&mut self, fen: &str, moves: Vec<UciMove>) {
        self.board = Board::from_fen(fen).unwrap();

        for uci_move in moves {
            let parsed_move = parse_move(&self.board, uci_move);
            self.board.make_move(parsed_move, NoDelta);
            self.ply_offset += 1;
        }
    }

    fn new_game(&mut self) {
        self.tt.clear();
    }
}

pub fn parse_move(board: &Board, uci_move: UciMove) -> Move {
    let from = ((uci_move.from.file as u8 - b'a') + (8 * (uci_move.from.rank - 1))) as usize;
    let to = ((uci_move.to.file as u8 - b'a') + (8 * (uci_move.to.rank - 1))) as usize;

    if board.piece_at(from).get_type() == PieceType::Pawn
        && board.piece_at(to) == Piece::Empty
        && (uci_move.from.rank).abs_diff(uci_move.to.rank) == 2
    {
        return Move::new(
            from as u32,
            to as u32,
            MoveType::DoublePush,
            Piece::new(PieceType::Pawn, board.current_color),
            PieceType::Empty,
        );
    }

    // Check if the move is a castling move
    if from == 4
        && to == 6
        && board.piece_at(from).get_type() == PieceType::King
        && board.current_color == PieceColor::White
    {
        return Move::new(
            from as u32,
            to as u32,
            MoveType::KingCastle,
            Piece::new(PieceType::King, board.current_color),
            PieceType::Empty,
        );
    } else if from == 4
        && to == 2
        && board.piece_at(from).get_type() == PieceType::King
        && board.current_color == PieceColor::White
    {
        return Move::new(
            from as u32,
            to as u32,
            MoveType::QueenCastle,
            Piece::new(PieceType::King, board.current_color),
            PieceType::Empty,
        );
    } else if from == 60
        && to == 62
        && board.piece_at(from).get_type() == PieceType::King
        && board.current_color == PieceColor::Black
    {
        return Move::new(
            from as u32,
            to as u32,
            MoveType::KingCastle,
            Piece::new(PieceType::King, board.current_color),
            PieceType::Empty,
        );
    } else if from == 60
        && to == 58
        && board.piece_at(from).get_type() == PieceType::King
        && board.current_color == PieceColor::Black
    {
        return Move::new(
            from as u32,
            to as u32,
            MoveType::QueenCastle,
            Piece::new(PieceType::King, board.current_color),
            PieceType::Empty,
        );
    }

    // Check if the move is an en passant capture
    if board.piece_at(from).get_type() == PieceType::Pawn && board.piece_at(to) == Piece::Empty {
        if board.current_color == PieceColor::Black
            && uci_move.from.rank == 4
            && uci_move.to.rank == 3
            && (uci_move.to.file as u8).abs_diff(uci_move.from.file as u8) == 1
        {
            println!("got en passent");
            if board.piece_at(to + 8).get_type() == PieceType::Pawn {
                return Move::new(
                    from as u32,
                    to as u32,
                    MoveType::EnPassantCapture,
                    Piece::new(PieceType::Pawn, board.current_color),
                    PieceType::Pawn,
                );
            }
        } else if board.current_color == PieceColor::White
            && uci_move.from.rank == 5
            && uci_move.to.rank == 6
            && (uci_move.to.file as u8).abs_diff(uci_move.from.file as u8) == 1
        {
            return Move::new(
                from as u32,
                to as u32,
                MoveType::EnPassantCapture,
                Piece::new(PieceType::Pawn, board.current_color),
                PieceType::Pawn,
            );
        }
    }

    // Check if there is a captured piece on the 'to' square
    let captured_piece = board.piece_at(to).get_type();

    // Extract promotion piece if present
    let promotion_piece = match uci_move.promotion {
        Some(UciPiece::Queen) => PieceType::Queen,
        Some(UciPiece::Rook) => PieceType::Rook,
        Some(UciPiece::Bishop) => PieceType::Bishop,
        Some(UciPiece::Knight) => PieceType::Knight,
        _ => PieceType::Empty,
    };

    if promotion_piece != PieceType::Empty {
        Move::new(
            from as u32,
            to as u32,
            MoveType::Promote,
            Piece::new(promotion_piece, board.current_color),
            captured_piece,
        )
    } else {
        Move::new(from as u32, to as u32, MoveType::QuietMove, board.piece_at(from), captured_piece)
    }
}

pub fn to_uci_move(r#move: Move) -> UciMove {
    let from =
        UciSquare::from((((r#move.from() as u8) % 8) + b'a') as char, r#move.from() as u8 / 8 + 1);
    let to = UciSquare::from((((r#move.to() as u8) % 8) + b'a') as char, r#move.to() as u8 / 8 + 1);
    UciMove {
        from,
        to,
        promotion: if r#move.move_type() == MoveType::Promote {
            Some(match r#move.piece().get_type() {
                PieceType::Queen => UciPiece::Queen,
                PieceType::Rook => UciPiece::Rook,
                PieceType::Bishop => UciPiece::Bishop,
                PieceType::Knight => UciPiece::Knight,
                _ => UciPiece::Queen,
            })
        } else {
            None
        },
    }
}
