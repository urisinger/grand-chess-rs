use std::io::{BufRead, Write};

use vampirc_uci::{
    parse_one, UciFen, UciMessage, UciMove, UciPiece, UciSearchControl, UciSquare, UciTimeControl,
};

pub trait Engine {
    fn id() -> (Option<String>, Option<String>) {
        (None, None)
    }

    fn best_move<W: Write>(
        &mut self,
        writer: &W,
        time_control: Option<UciTimeControl>,
        search_control: Option<UciSearchControl>,
    ) -> UciMove;

    fn new_game(&mut self);

    fn set_pos(&mut self, fen: &str, moves: Vec<UciMove>);

    fn stop(&self) -> bool {
        false
    }
}

struct UciConnection<R: BufRead, W: Write + Send + Sync, E: Engine> {
    reader: R,
    writer: W,

    engine: E,
}

impl<R: BufRead, W: Write + Send + Sync, E: Engine> UciConnection<R, W, E> {
    pub fn new(reader: R, writer: W, engine: E) -> Self {
        Self { reader, writer, engine }
    }

    pub fn run(mut self) {
        for line in self.reader.lines() {
            match parse_one(&line.unwrap()) {
                UciMessage::Uci => {
                    self.writer.write_fmt(format_args!("{}", UciMessage::UciOk)).unwrap();

                    let id = E::id();

                    self.writer
                        .write_fmt(format_args!("{}", UciMessage::Id { name: id.0, author: id.1 }))
                        .unwrap();
                }
                UciMessage::IsReady => {
                    self.engine.stop();

                    self.writer.write_fmt(format_args!("{}", UciMessage::ReadyOk)).unwrap();
                }
                UciMessage::Go { time_control, search_control } => {
                    let best_move =
                        self.engine.best_move(&self.writer, time_control, search_control);

                    self.writer
                        .write_fmt(format_args!(
                            "{}",
                            UciMessage::BestMove { best_move, ponder: None }
                        ))
                        .unwrap();
                }
                UciMessage::Position { startpos, fen, moves } => {
                    let fen = if startpos || fen.is_none() {
                        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
                    } else {
                        fen.as_ref().unwrap().0.as_str()
                    };

                    self.engine.set_pos(fen, moves);
                }
                UciMessage::UciNewGame => {
                    self.engine.new_game();
                    self.writer.write_fmt(format_args!("{}", UciMessage::ReadyOk)).unwrap();
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
