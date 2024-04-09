use std::io::{BufRead, Write};

use vampirc_uci::{
    parse_one, UciMessage, UciMove, UciPiece, UciSearchControl, UciSquare, UciTimeControl,
};

use crate::board::{
    piece::{Piece, PieceType},
    r#move::{Move, MoveType},
};

pub trait Engine {
    fn best_move<W: Write>(
        &mut self,
        writer: &W,
        time_control: Option<UciTimeControl>,
        search_control: Option<UciSearchControl>,
    ) -> Move;

    fn stop(&self) -> bool {
        false
    }
}

struct UciConnection<R: BufRead, W: Write + Send + Sync, E: Engine> {
    reader: R,
    writer: W,

    engine: E,
}

fn move_to_uci(r#move: Move) -> UciMove {
    let promotion = if r#move.move_type() == MoveType::Promote {
        Some(match r#move.piece().get_type() {
            PieceType::Pawn => UciPiece::Pawn,
            PieceType::Knight => UciPiece::Knight,
            PieceType::Bishop => UciPiece::Bishop,
            PieceType::Rook => UciPiece::Rook,
            PieceType::Queen => UciPiece::Queen,
            PieceType::King => UciPiece::King,
            PieceType::Empty => panic!("Piece should not be emptry"),
        })
    } else {
        None
    };

    let from = r#move.from() as u8;
    let to = r#move.to() as u8;
    UciMove {
        from: UciSquare::from((b'a' + (from % 8)) as char, from / 8),
        to: UciSquare::from((b'a' + (to % 8)) as char, to / 8),
        promotion,
    }
}

impl<R: BufRead, W: Write + Send + Sync, E: Engine> UciConnection<R, W, E> {
    pub fn run(mut self) {
        for line in self.reader.lines() {
            match parse_one(&line.unwrap()) {
                UciMessage::Uci => {
                    self.writer.write_fmt(format_args!("{}", UciMessage::UciOk)).unwrap();
                }
                UciMessage::IsReady => {
                    self.writer.write_fmt(format_args!("{}", UciMessage::ReadyOk)).unwrap();
                }
                UciMessage::Go { time_control, search_control } => {
                    let best_move = move_to_uci(self.engine.best_move(
                        &self.writer,
                        time_control,
                        search_control,
                    ));

                    self.writer
                        .write_fmt(format_args!(
                            "{}",
                            UciMessage::BestMove { best_move, ponder: None }
                        ))
                        .unwrap();
                }
                UciMessage::Stop => {
                    self.engine.stop();
                }
                UciMessage::Quit => {
                    return;
                }
                _ => {}
            }
        }
    }
}
